use crate::{Error, ErrorKind, Result};
use reqwest::blocking::{Client, ClientBuilder, Response as BlockingResponse};
use reqwest::header::{
    AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT,
};
use reqwest::redirect::Policy;
use reqwest::{Method, StatusCode};
use serde_json::Value as JsonValue;
use std::time::Duration;
use url::{Url, form_urlencoded};

const DEFAULT_TIMEOUT_MS: u64 = 30_000;

#[derive(Clone, Debug, PartialEq)]
pub struct HttpRequestOptions {
    pub url: String,
    pub method: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub query: Vec<(String, String)>,
    pub json: Option<JsonValue>,
    pub form: Vec<(String, String)>,
    pub timeout_ms: Option<i64>,
    pub connect_timeout_ms: Option<i64>,
    pub follow_redirects: Option<bool>,
    pub max_redirects: Option<i64>,
    pub user_agent: Option<String>,
    pub basic_auth: Option<(String, String)>,
    pub bearer_token: Option<String>,
    pub fail_on_http_error: bool,
}

enum BodyState {
    Unread(BlockingResponse),
    Cached(Vec<u8>),
}

pub struct HttpResponse {
    pub status: i64,
    pub ok: bool,
    pub url: String,
    pub headers: Vec<(String, String)>,
    body: BodyState,
}

pub fn request(options: HttpRequestOptions) -> Result<HttpResponse> {
    let HttpRequestOptions {
        url,
        method,
        headers,
        body,
        query,
        json,
        form,
        timeout_ms,
        connect_timeout_ms,
        follow_redirects,
        max_redirects,
        user_agent,
        basic_auth,
        bearer_token,
        fail_on_http_error,
    } = options;

    if basic_auth.is_some() && bearer_token.is_some() {
        return Err(invalid_http_options(
            "`basic_auth` and `bearer_token` are mutually exclusive",
        ));
    }

    let url = build_request_url(&url, &query)?;
    let mut client_builder = Client::builder().timeout(Duration::from_millis(parse_timeout_ms(
        timeout_ms,
        "timeout_ms",
    )?));
    if let Some(connect_timeout_ms) = connect_timeout_ms {
        client_builder = client_builder.connect_timeout(Duration::from_millis(
            parse_timeout_value(connect_timeout_ms, "connect_timeout_ms")?,
        ));
    }
    client_builder = apply_redirect_policy(client_builder, follow_redirects, max_redirects)?;

    let client = client_builder
        .build()
        .map_err(|err| http_error(format!("request build failed: {err:?}")))?;

    let mut headers = parse_headers(headers)?;
    let body = build_request_body(body, json, form, &mut headers)?;
    apply_request_header_overrides(&mut headers, user_agent.as_deref())?;
    if basic_auth.is_some() || bearer_token.is_some() {
        headers.remove(AUTHORIZATION);
    }

    let mut request = client.request(parse_method(method)?, &url);
    if !headers.is_empty() {
        request = request.headers(headers);
    }
    if let Some((username, password)) = basic_auth {
        request = request.basic_auth(username, Some(password));
    } else if let Some(bearer_token) = bearer_token {
        request = request.bearer_auth(bearer_token);
    }
    if let Some(body) = body {
        request = request.body(body);
    }

    let response = request
        .send()
        .map_err(|err| http_error(format!("request failed: {err:?}")))?;

    let status_code = response.status();
    let status = i64::from(status_code.as_u16());
    let ok = status_code.is_success();
    let url = response.url().to_string();
    let headers = collect_response_headers(response.headers());
    if fail_on_http_error && is_http_error(status_code) {
        return Err(http_status_error(status_code, &url));
    }

    Ok(HttpResponse {
        status,
        ok,
        url,
        headers,
        body: BodyState::Unread(response),
    })
}

impl HttpResponse {
    pub fn text(&mut self) -> Result<String> {
        let bytes = self.body_bytes()?;
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    pub fn json(&mut self) -> Result<JsonValue> {
        let bytes = self.body_bytes()?;
        serde_json::from_slice(bytes)
            .map_err(|err| http_error(format!("response json parse failed: {err}")))
    }

    pub fn bytes(&mut self) -> Result<Vec<u8>> {
        Ok(self.body_bytes()?.to_vec())
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn header_values(&self, name: &str) -> Vec<String> {
        self.headers
            .iter()
            .filter(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.clone())
            .collect()
    }

    pub fn raise_for_status(&self) -> Result<()> {
        let status = u16::try_from(self.status)
            .ok()
            .and_then(|status| StatusCode::from_u16(status).ok())
            .ok_or_else(|| http_error(format!("invalid response status `{}`", self.status)))?;
        if is_http_error(status) {
            return Err(http_status_error(status, &self.url));
        }
        Ok(())
    }

    fn body_bytes(&mut self) -> Result<&[u8]> {
        if matches!(&self.body, BodyState::Unread(_)) {
            let state = std::mem::replace(&mut self.body, BodyState::Cached(Vec::new()));
            let bytes = match state {
                BodyState::Unread(response) => response
                    .bytes()
                    .map(|body| body.to_vec())
                    .map_err(|err| http_error(format!("response read failed: {err}")))?,
                BodyState::Cached(bytes) => bytes,
            };
            self.body = BodyState::Cached(bytes);
        }

        match &self.body {
            BodyState::Cached(bytes) => Ok(bytes.as_slice()),
            BodyState::Unread(_) => unreachable!("response body should be cached"),
        }
    }
}

fn build_request_url(url: &str, query: &[(String, String)]) -> Result<String> {
    if query.is_empty() {
        return Ok(url.to_string());
    }

    let mut parsed = Url::parse(url)
        .map_err(|err| Error::new(ErrorKind::InvalidUrl, format!("invalid url `{url}`: {err}")))?;
    {
        let mut pairs = parsed.query_pairs_mut();
        for (key, value) in query {
            pairs.append_pair(key, value);
        }
    }

    Ok(parsed.to_string())
}

fn apply_redirect_policy(
    mut builder: ClientBuilder,
    follow_redirects: Option<bool>,
    max_redirects: Option<i64>,
) -> Result<ClientBuilder> {
    if matches!(follow_redirects, Some(false)) && max_redirects.is_some() {
        return Err(invalid_http_options(
            "`max_redirects` cannot be set when `follow_redirects` is false",
        ));
    }

    if let Some(false) = follow_redirects {
        builder = builder.redirect(Policy::none());
    } else if let Some(max_redirects) = max_redirects {
        builder = builder.redirect(Policy::limited(parse_nonnegative_usize(
            max_redirects,
            "max_redirects",
        )?));
    }

    Ok(builder)
}

fn build_request_body(
    body: Option<Vec<u8>>,
    json: Option<JsonValue>,
    form: Vec<(String, String)>,
    headers: &mut HeaderMap,
) -> Result<Option<Vec<u8>>> {
    let body_count =
        usize::from(body.is_some()) + usize::from(json.is_some()) + usize::from(!form.is_empty());
    if body_count > 1 {
        return Err(invalid_http_options(
            "`body`, `json`, and `form` are mutually exclusive",
        ));
    }

    if let Some(body) = body {
        return Ok(Some(body));
    }

    if let Some(json) = json {
        if !headers.contains_key(CONTENT_TYPE) {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }
        return serde_json::to_vec(&json)
            .map(Some)
            .map_err(|err| http_error(format!("request json encode failed: {err}")));
    }

    if !form.is_empty() {
        if !headers.contains_key(CONTENT_TYPE) {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }
        let mut serializer = form_urlencoded::Serializer::new(String::new());
        for (key, value) in form {
            serializer.append_pair(&key, &value);
        }
        return Ok(Some(serializer.finish().into_bytes()));
    }

    Ok(None)
}

fn apply_request_header_overrides(headers: &mut HeaderMap, user_agent: Option<&str>) -> Result<()> {
    if let Some(user_agent) = user_agent {
        headers.remove(USER_AGENT);
        let value = HeaderValue::from_str(user_agent).map_err(|err| {
            Error::new(
                ErrorKind::InvalidHttpHeader,
                format!("invalid header value for `user-agent`: {err}"),
            )
        })?;
        headers.insert(USER_AGENT, value);
    }

    Ok(())
}

fn parse_method(method: Option<String>) -> Result<Method> {
    let method = method.unwrap_or_else(|| "GET".to_string());
    Method::from_bytes(method.as_bytes()).map_err(|err| {
        Error::new(
            ErrorKind::InvalidHttpMethod,
            format!("invalid method `{method}`: {err}"),
        )
    })
}

fn parse_headers(headers: Vec<(String, String)>) -> Result<HeaderMap> {
    let mut header_map = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|err| {
            Error::new(
                ErrorKind::InvalidHttpHeader,
                format!("invalid header name `{name}`: {err}"),
            )
        })?;
        let header_value = HeaderValue::from_str(&value).map_err(|err| {
            Error::new(
                ErrorKind::InvalidHttpHeader,
                format!("invalid header value for `{name}`: {err}"),
            )
        })?;
        header_map.append(header_name, header_value);
    }

    Ok(header_map)
}

fn parse_timeout_ms(timeout_ms: Option<i64>, label: &str) -> Result<u64> {
    let timeout_ms = timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS as i64);
    parse_timeout_value(timeout_ms, label)
}

fn parse_timeout_value(timeout_ms: i64, label: &str) -> Result<u64> {
    if timeout_ms <= 0 {
        return Err(Error::new(
            ErrorKind::InvalidHttpTimeout,
            format!("`{label}` must be > 0"),
        ));
    }

    u64::try_from(timeout_ms).map_err(|_| {
        Error::new(
            ErrorKind::InvalidHttpTimeout,
            format!("`{label}` is too large"),
        )
    })
}

fn parse_nonnegative_usize(value: i64, label: &str) -> Result<usize> {
    if value < 0 {
        return Err(invalid_http_options(format!("`{label}` must be >= 0")));
    }

    usize::try_from(value).map_err(|_| invalid_http_options(format!("`{label}` is too large")))
}

fn collect_response_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            let value = match value.to_str() {
                Ok(value) => value.to_string(),
                Err(_) => String::from_utf8_lossy(value.as_bytes()).to_string(),
            };
            (name.as_str().to_string(), value)
        })
        .collect()
}

fn is_http_error(status: StatusCode) -> bool {
    status.is_client_error() || status.is_server_error()
}

fn http_status_error(status: StatusCode, url: &str) -> Error {
    http_error(format!(
        "request failed with HTTP status {status} for `{url}`"
    ))
}

fn invalid_http_options(msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidHttpOptions, msg)
}

fn http_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::Http, msg)
}
