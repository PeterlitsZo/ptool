use crate::{Error, ErrorKind, Result};
use reqwest::Method;
use reqwest::blocking::{Client, Response as BlockingResponse};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 30_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpRequestOptions {
    pub url: String,
    pub method: Option<String>,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: Option<i64>,
}

enum BodyState {
    Unread(BlockingResponse),
    Consumed,
}

pub struct HttpResponse {
    pub status: i64,
    pub ok: bool,
    pub url: String,
    pub headers: Vec<(String, String)>,
    body: BodyState,
}

pub fn request(options: HttpRequestOptions) -> Result<HttpResponse> {
    let client = Client::builder()
        .timeout(Duration::from_millis(parse_timeout_ms(options.timeout_ms)?))
        .build()
        .map_err(|err| http_error(format!("request build failed: {err:?}")))?;

    let mut request = client.request(parse_method(options.method)?, &options.url);
    let headers = parse_headers(options.headers)?;
    if !headers.is_empty() {
        request = request.headers(headers);
    }
    if let Some(body) = options.body {
        request = request.body(body);
    }

    let response = request
        .send()
        .map_err(|err| http_error(format!("request failed: {err:?}")))?;

    let status = i64::from(response.status().as_u16());
    let ok = response.status().is_success();
    let url = response.url().to_string();
    let headers = collect_response_headers(response.headers());

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
        let bytes = self.consume_body_bytes()?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }

    pub fn json(&mut self) -> Result<JsonValue> {
        let bytes = self.consume_body_bytes()?;
        serde_json::from_slice(&bytes)
            .map_err(|err| http_error(format!("response json parse failed: {err}")))
    }

    pub fn bytes(&mut self) -> Result<Vec<u8>> {
        self.consume_body_bytes()
    }

    fn consume_body_bytes(&mut self) -> Result<Vec<u8>> {
        let state = std::mem::replace(&mut self.body, BodyState::Consumed);
        match state {
            BodyState::Unread(response) => response
                .bytes()
                .map(|body| body.to_vec())
                .map_err(|err| http_error(format!("response read failed: {err}"))),
            BodyState::Consumed => Err(http_error("response body already consumed")),
        }
    }
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

fn parse_timeout_ms(timeout_ms: Option<i64>) -> Result<u64> {
    let timeout_ms = timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS as i64);
    if timeout_ms <= 0 {
        return Err(Error::new(
            ErrorKind::InvalidHttpTimeout,
            "`timeout_ms` must be > 0",
        ));
    }

    u64::try_from(timeout_ms)
        .map_err(|_| Error::new(ErrorKind::InvalidHttpTimeout, "`timeout_ms` is too large"))
}

fn collect_response_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    let mut merged = BTreeMap::new();
    for (name, value) in headers {
        let key = name.as_str().to_string();
        let value = match value.to_str() {
            Ok(value) => value.to_string(),
            Err(_) => String::from_utf8_lossy(value.as_bytes()).to_string(),
        };

        merged
            .entry(key)
            .and_modify(|existing: &mut String| {
                existing.push_str(", ");
                existing.push_str(&value);
            })
            .or_insert(value);
    }

    merged.into_iter().collect()
}

fn http_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::Http, msg)
}
