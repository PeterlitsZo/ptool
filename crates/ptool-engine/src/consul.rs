use crate::http::{self, HttpRequestOptions, HttpResponse};
use crate::{Error, ErrorKind, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde_json::Value as JsonValue;
use url::Url;

const CONNECT_OP: &str = "ptool.consul.connect";
const GET_OP: &str = "ptool.consul.Connection:get";
const PUT_OP: &str = "ptool.consul.Connection:put";
const DELETE_OP: &str = "ptool.consul.Connection:delete";
const LIST_OP: &str = "ptool.consul.Connection:list";
const REQUEST_OP: &str = "ptool.consul.Connection:request";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsulConnectOptions {
    pub address: String,
    pub token: Option<String>,
    pub datacenter: Option<String>,
    pub timeout_ms: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsulGetOptions {
    pub consistent: bool,
    pub stale: bool,
    pub wait_index: Option<u64>,
    pub wait_time: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsulListOptions {
    pub consistent: bool,
    pub stale: bool,
    pub wait_index: Option<u64>,
    pub wait_time: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsulPutOptions {
    pub flags: Option<u64>,
    pub cas: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsulDeleteOptions {
    pub cas: Option<u64>,
    pub recurse: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsulKvEntry {
    pub key: String,
    pub value: Option<Vec<u8>>,
    pub flags: u64,
    pub create_index: u64,
    pub modify_index: u64,
    pub lock_index: u64,
    pub session: Option<String>,
    pub namespace: Option<String>,
    pub partition: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ConsulConnection {
    base_url: Url,
    token: Option<String>,
    datacenter: Option<String>,
    timeout_ms: Option<i64>,
}

pub(crate) fn connect(options: ConsulConnectOptions) -> Result<ConsulConnection> {
    let address = if options.address.is_empty() {
        return Err(
            Error::new(ErrorKind::EmptyInput, "address must not be empty").with_op(CONNECT_OP),
        );
    } else {
        normalize_address(&options.address)?
    };

    Ok(ConsulConnection {
        base_url: address,
        token: options.token,
        datacenter: options.datacenter,
        timeout_ms: options.timeout_ms,
    })
}

impl ConsulConnectOptions {
    pub fn with_env_fallback(
        mut self,
        env_get: impl Fn(&str) -> Result<Option<String>>,
    ) -> Result<Self> {
        self.address = fallback_required(self.address, &env_get, &["CONSUL_HTTP_ADDR"])?;
        self.token = fallback_option(self.token, &env_get, &["CONSUL_HTTP_TOKEN"])?;
        Ok(self)
    }
}

impl ConsulConnection {
    pub fn get(&self, key: &str, options: &ConsulGetOptions) -> Result<Option<ConsulKvEntry>> {
        let key = normalize_key(key, GET_OP)?;
        let query = build_get_query(options);
        let path = format!("v1/kv/{key}");
        let mut response = self.request_path(GET_OP, &path, Some("GET"), query, None)?;
        if response.status == 404 {
            return Ok(None);
        }
        response.raise_for_status()?;
        let value = response.json()?;
        let items = parse_kv_array(&value, GET_OP)?;
        Ok(items.into_iter().next())
    }

    pub fn put(&self, key: &str, value: &[u8], options: &ConsulPutOptions) -> Result<bool> {
        let key = normalize_key(key, PUT_OP)?;
        let mut query = Vec::new();
        if let Some(flags) = options.flags {
            query.push(("flags".to_string(), flags.to_string()));
        }
        if let Some(cas) = options.cas {
            query.push(("cas".to_string(), cas.to_string()));
        }
        let path = format!("v1/kv/{key}");
        let mut response =
            self.request_path(PUT_OP, &path, Some("PUT"), query, Some(value.to_vec()))?;
        response.raise_for_status()?;
        parse_bool_response(&mut response, PUT_OP)
    }

    pub fn delete(&self, key: &str, options: &ConsulDeleteOptions) -> Result<bool> {
        if options.recurse && options.cas.is_some() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "`cas` cannot be combined with `recurse`",
            )
            .with_op(DELETE_OP));
        }

        let key = normalize_key(key, DELETE_OP)?;
        let mut query = Vec::new();
        if let Some(cas) = options.cas {
            query.push(("cas".to_string(), cas.to_string()));
        }
        if options.recurse {
            query.push(("recurse".to_string(), "true".to_string()));
        }
        let path = format!("v1/kv/{key}");
        let mut response = self.request_path(DELETE_OP, &path, Some("DELETE"), query, None)?;
        response.raise_for_status()?;
        parse_bool_response(&mut response, DELETE_OP)
    }

    pub fn list(&self, prefix: &str, options: &ConsulListOptions) -> Result<Vec<ConsulKvEntry>> {
        let prefix = normalize_prefix(prefix);
        let mut query = build_list_query(options);
        query.push(("recurse".to_string(), "true".to_string()));
        let path = format!("v1/kv/{prefix}");
        let mut response = self.request_path(LIST_OP, &path, Some("GET"), query, None)?;
        if response.status == 404 {
            return Ok(Vec::new());
        }
        response.raise_for_status()?;
        let value = response.json()?;
        parse_kv_array(&value, LIST_OP)
    }

    pub fn request(&self, options: HttpRequestOptions) -> Result<HttpResponse> {
        let options = self.apply_request_defaults(options)?;
        http::request_with_op(REQUEST_OP, options)
    }

    fn request_path(
        &self,
        op: &'static str,
        path: &str,
        method: Option<&str>,
        query: Vec<(String, String)>,
        body: Option<Vec<u8>>,
    ) -> Result<HttpResponse> {
        let options = HttpRequestOptions {
            url: self.build_url(path)?,
            method: method.map(ToOwned::to_owned),
            headers: Vec::new(),
            body,
            query,
            json: None,
            form: Vec::new(),
            timeout_ms: self.timeout_ms,
            connect_timeout_ms: None,
            follow_redirects: None,
            max_redirects: None,
            user_agent: None,
            basic_auth: None,
            bearer_token: None,
            fail_on_http_error: false,
        };
        let options = self.apply_request_defaults(options)?;
        http::request_with_op(op, options)
    }

    fn apply_request_defaults(
        &self,
        mut options: HttpRequestOptions,
    ) -> Result<HttpRequestOptions> {
        if options.url.is_empty() {
            return Err(
                Error::new(ErrorKind::EmptyInput, "url must not be empty").with_op(REQUEST_OP)
            );
        }

        if options.timeout_ms.is_none() {
            options.timeout_ms = self.timeout_ms;
        }
        if let Some(token) = &self.token {
            let has_token = options
                .headers
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("x-consul-token"));
            if !has_token {
                options
                    .headers
                    .push(("x-consul-token".to_string(), token.clone()));
            }
        }
        if let Some(datacenter) = &self.datacenter {
            let has_dc = options.query.iter().any(|(name, _)| name == "dc");
            if !has_dc {
                options.query.push(("dc".to_string(), datacenter.clone()));
            }
        }
        Ok(options)
    }

    pub fn build_url(&self, path: &str) -> Result<String> {
        let mut url = self.base_url.clone();
        let segments = path
            .trim_start_matches('/')
            .split('/')
            .filter(|part| !part.is_empty());
        {
            let mut path_segments = url.path_segments_mut().map_err(|_| {
                Error::new(ErrorKind::InvalidUrl, "address cannot be a base URL")
                    .with_op(CONNECT_OP)
            })?;
            for segment in segments {
                path_segments.push(segment);
            }
        }
        Ok(url.to_string())
    }
}

fn normalize_address(address: &str) -> Result<Url> {
    let address = if address.contains("://") {
        address.to_string()
    } else {
        format!("http://{address}")
    };
    let mut url = Url::parse(&address).map_err(|err| {
        Error::new(
            ErrorKind::InvalidUrl,
            format!("invalid Consul address `{address}`: {err}"),
        )
        .with_op(CONNECT_OP)
        .with_url(address.clone())
    })?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err(Error::new(
            ErrorKind::InvalidUrl,
            "Consul address must not contain a query string or fragment",
        )
        .with_op(CONNECT_OP)
        .with_url(address));
    }

    let mut path = url.path().trim_end_matches('/').to_string();
    path.push('/');
    url.set_path(&path);
    Ok(url)
}

fn normalize_key(key: &str, op: &str) -> Result<String> {
    let key = key.trim_start_matches('/').to_string();
    if key.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "key must not be empty").with_op(op));
    }
    Ok(key)
}

fn normalize_prefix(prefix: &str) -> String {
    prefix.trim_start_matches('/').to_string()
}

fn build_get_query(options: &ConsulGetOptions) -> Vec<(String, String)> {
    let mut query = Vec::new();
    push_read_query(
        &mut query,
        options.consistent,
        options.stale,
        options.wait_index,
        options.wait_time.as_deref(),
    );
    query
}

fn build_list_query(options: &ConsulListOptions) -> Vec<(String, String)> {
    let mut query = Vec::new();
    push_read_query(
        &mut query,
        options.consistent,
        options.stale,
        options.wait_index,
        options.wait_time.as_deref(),
    );
    query
}

fn push_read_query(
    query: &mut Vec<(String, String)>,
    consistent: bool,
    stale: bool,
    wait_index: Option<u64>,
    wait_time: Option<&str>,
) {
    if consistent {
        query.push(("consistent".to_string(), "true".to_string()));
    }
    if stale {
        query.push(("stale".to_string(), "true".to_string()));
    }
    if let Some(wait_index) = wait_index {
        query.push(("index".to_string(), wait_index.to_string()));
    }
    if let Some(wait_time) = wait_time {
        query.push(("wait".to_string(), wait_time.to_string()));
    }
}

fn parse_bool_response(response: &mut HttpResponse, op: &'static str) -> Result<bool> {
    let value = response.json()?;
    value
        .as_bool()
        .ok_or_else(|| consul_error(op, "expected a boolean response body"))
}

fn parse_kv_array(value: &JsonValue, op: &'static str) -> Result<Vec<ConsulKvEntry>> {
    let items = value
        .as_array()
        .ok_or_else(|| consul_error(op, "expected a JSON array response body"))?;
    items
        .iter()
        .map(|value| parse_kv_entry(value, op))
        .collect()
}

fn parse_kv_entry(value: &JsonValue, op: &'static str) -> Result<ConsulKvEntry> {
    let object = value
        .as_object()
        .ok_or_else(|| consul_error(op, "expected each KV entry to be a JSON object"))?;
    let key = required_string_field(object, "Key", op)?;
    let value = match object.get("Value") {
        Some(JsonValue::String(value)) => Some(
            BASE64_STANDARD
                .decode(value)
                .map_err(|err| consul_error(op, format!("failed to decode KV value: {err}")))?,
        ),
        Some(JsonValue::Null) | None => None,
        Some(_) => {
            return Err(consul_error(
                op,
                "expected `Value` to be a base64 string or null",
            ));
        }
    };

    Ok(ConsulKvEntry {
        key,
        value,
        flags: required_u64_field(object, "Flags", op)?,
        create_index: required_u64_field(object, "CreateIndex", op)?,
        modify_index: required_u64_field(object, "ModifyIndex", op)?,
        lock_index: required_u64_field(object, "LockIndex", op)?,
        session: optional_string_field(object, "Session", op)?,
        namespace: optional_string_field(object, "Namespace", op)?,
        partition: optional_string_field(object, "Partition", op)?,
    })
}

fn required_string_field(
    object: &serde_json::Map<String, JsonValue>,
    key: &str,
    op: &'static str,
) -> Result<String> {
    match object.get(key) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(_) => Err(consul_error(op, format!("expected `{key}` to be a string"))),
        None => Err(consul_error(
            op,
            format!("missing `{key}` field in response"),
        )),
    }
}

fn optional_string_field(
    object: &serde_json::Map<String, JsonValue>,
    key: &str,
    op: &'static str,
) -> Result<Option<String>> {
    match object.get(key) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) | None => Ok(None),
        Some(_) => Err(consul_error(
            op,
            format!("expected `{key}` to be a string or null"),
        )),
    }
}

fn required_u64_field(
    object: &serde_json::Map<String, JsonValue>,
    key: &str,
    op: &'static str,
) -> Result<u64> {
    match object.get(key).and_then(JsonValue::as_u64) {
        Some(value) => Ok(value),
        None => Err(consul_error(
            op,
            format!("missing or invalid `{key}` field in response"),
        )),
    }
}

fn fallback_required(
    current: String,
    env_get: &impl Fn(&str) -> Result<Option<String>>,
    keys: &[&str],
) -> Result<String> {
    if !current.is_empty() {
        return Ok(current);
    }
    for key in keys {
        let Some(value) = env_get(key)? else {
            continue;
        };
        if !value.is_empty() {
            return Ok(value);
        }
    }
    Ok(current)
}

fn fallback_option(
    current: Option<String>,
    env_get: &impl Fn(&str) -> Result<Option<String>>,
    keys: &[&str],
) -> Result<Option<String>> {
    if current.is_some() {
        return Ok(current);
    }
    for key in keys {
        let Some(value) = env_get(key)? else {
            continue;
        };
        if !value.is_empty() {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn consul_error(op: &'static str, msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::Consul, msg).with_op(op)
}
