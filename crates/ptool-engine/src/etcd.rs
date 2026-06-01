use crate::http::{self, HttpRequestOptions, HttpResponse};
use crate::{Error, ErrorKind, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use url::Url;

const DEFAULT_API_PREFIX: &str = "v3";
const CONNECT_OP: &str = "ptool.etcd.connect";
const AUTH_OP: &str = "ptool.etcd.authenticate";
const GET_OP: &str = "ptool.etcd.Connection:get";
const PUT_OP: &str = "ptool.etcd.Connection:put";
const DELETE_OP: &str = "ptool.etcd.Connection:delete";
const LIST_OP: &str = "ptool.etcd.Connection:list";
const REQUEST_OP: &str = "ptool.etcd.Connection:request";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EtcdConnectOptions {
    pub address: String,
    pub token: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub api_prefix: Option<String>,
    pub timeout_ms: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EtcdGetOptions {
    pub revision: Option<i64>,
    pub serializable: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EtcdListOptions {
    pub limit: Option<i64>,
    pub revision: Option<i64>,
    pub serializable: bool,
    pub keys_only: bool,
    pub count_only: bool,
    pub min_mod_revision: Option<i64>,
    pub max_mod_revision: Option<i64>,
    pub min_create_revision: Option<i64>,
    pub max_create_revision: Option<i64>,
    pub sort_order: Option<String>,
    pub sort_target: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EtcdPutOptions {
    pub lease: Option<i64>,
    pub prev_kv: bool,
    pub ignore_value: bool,
    pub ignore_lease: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EtcdDeleteOptions {
    pub prefix: bool,
    pub prev_kv: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtcdResponseHeader {
    pub cluster_id: String,
    pub member_id: String,
    pub revision: i64,
    pub raft_term: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtcdKvEntry {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub create_revision: i64,
    pub mod_revision: i64,
    pub version: i64,
    pub lease: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtcdPutResponse {
    pub header: Option<EtcdResponseHeader>,
    pub prev_kv: Option<EtcdKvEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtcdDeleteResponse {
    pub header: Option<EtcdResponseHeader>,
    pub deleted: i64,
    pub prev_kvs: Vec<EtcdKvEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EtcdListResponse {
    pub header: Option<EtcdResponseHeader>,
    pub kvs: Vec<EtcdKvEntry>,
    pub count: i64,
    pub more: bool,
}

#[derive(Clone, Debug)]
pub struct EtcdConnection {
    base_url: Url,
    api_prefix: String,
    token: Option<String>,
    timeout_ms: Option<i64>,
}

pub(crate) fn connect(options: EtcdConnectOptions) -> Result<EtcdConnection> {
    let address = if options.address.is_empty() {
        return Err(
            Error::new(ErrorKind::EmptyInput, "address must not be empty").with_op(CONNECT_OP),
        );
    } else {
        normalize_address(&options.address)?
    };

    let api_prefix = normalize_api_prefix(options.api_prefix.as_deref())?;
    let auth = match (options.username, options.password) {
        (None, None) => None,
        (Some(username), Some(password)) => Some((username, password)),
        (Some(_), None) => {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "`username` requires `password`")
                    .with_op(CONNECT_OP),
            );
        }
        (None, Some(_)) => {
            return Err(
                Error::new(ErrorKind::InvalidArgs, "`password` requires `username`")
                    .with_op(CONNECT_OP),
            );
        }
    };

    if options.token.is_some() && auth.is_some() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "`token` cannot be combined with `username`/`password`",
        )
        .with_op(CONNECT_OP));
    }

    let token = match (options.token, auth) {
        (Some(token), None) => Some(token),
        (None, Some((username, password))) => Some(authenticate(
            &address,
            &api_prefix,
            &username,
            &password,
            options.timeout_ms,
        )?),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!("validated above"),
    };

    Ok(EtcdConnection {
        base_url: address,
        api_prefix,
        token,
        timeout_ms: options.timeout_ms,
    })
}

impl EtcdConnection {
    pub fn get(&self, key: &[u8], options: &EtcdGetOptions) -> Result<Option<EtcdKvEntry>> {
        let key = normalize_key(key, GET_OP)?;
        let body = json!({
            "key": encode_bytes(key),
        });
        let body = apply_get_options(body, options);
        let mut response = self.post_json(GET_OP, "kv/range", body)?;
        response.raise_for_status()?;
        let value = response.json()?;
        let response = parse_list_response(&value, GET_OP)?;
        Ok(response.kvs.into_iter().next())
    }

    pub fn put(
        &self,
        key: &[u8],
        value: &[u8],
        options: &EtcdPutOptions,
    ) -> Result<EtcdPutResponse> {
        let key = normalize_key(key, PUT_OP)?;
        let mut body = json!({
            "key": encode_bytes(key),
            "value": encode_bytes(value),
        });
        let object = body
            .as_object_mut()
            .expect("etcd put request body must be an object");
        if let Some(lease) = options.lease {
            object.insert("lease".to_string(), JsonValue::String(lease.to_string()));
        }
        insert_bool_field(object, "prev_kv", options.prev_kv);
        insert_bool_field(object, "ignore_value", options.ignore_value);
        insert_bool_field(object, "ignore_lease", options.ignore_lease);

        let mut response = self.post_json(PUT_OP, "kv/put", body)?;
        response.raise_for_status()?;
        let value = response.json()?;
        parse_put_response(&value, PUT_OP)
    }

    pub fn delete(&self, key: &[u8], options: &EtcdDeleteOptions) -> Result<EtcdDeleteResponse> {
        let key = normalize_delete_key(key, options.prefix, DELETE_OP)?;
        let (range_key, range_end) = delete_range(key, options.prefix);
        let mut body = json!({
            "key": encode_bytes(&range_key),
        });
        let object = body
            .as_object_mut()
            .expect("etcd delete request body must be an object");
        if let Some(range_end) = range_end {
            object.insert(
                "range_end".to_string(),
                JsonValue::String(encode_bytes(&range_end)),
            );
        }
        insert_bool_field(object, "prev_kv", options.prev_kv);

        let mut response = self.post_json(DELETE_OP, "kv/deleterange", body)?;
        response.raise_for_status()?;
        let value = response.json()?;
        parse_delete_response(&value, DELETE_OP)
    }

    pub fn list(&self, prefix: &[u8], options: &EtcdListOptions) -> Result<EtcdListResponse> {
        let (key, range_end) = normalize_list_range(prefix);
        let mut body = json!({
            "key": encode_bytes(&key),
            "range_end": encode_bytes(&range_end),
        });
        let object = body
            .as_object_mut()
            .expect("etcd list request body must be an object");
        apply_list_options(object, options)?;

        let mut response = self.post_json(LIST_OP, "kv/range", body)?;
        response.raise_for_status()?;
        let value = response.json()?;
        parse_list_response(&value, LIST_OP)
    }

    pub fn request(&self, options: HttpRequestOptions) -> Result<HttpResponse> {
        let options = self.apply_request_defaults(options)?;
        http::request_with_op(REQUEST_OP, options)
    }

    pub fn build_url(&self, path: &str) -> Result<String> {
        build_api_url(&self.base_url, &self.api_prefix, path, REQUEST_OP)
    }

    fn post_json(&self, op: &'static str, path: &str, body: JsonValue) -> Result<HttpResponse> {
        let options = HttpRequestOptions {
            url: build_api_url(&self.base_url, &self.api_prefix, path, op)?,
            method: Some("POST".to_string()),
            headers: Vec::new(),
            body: None,
            query: Vec::new(),
            json: Some(body),
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
            let has_auth_header = options
                .headers
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("authorization"));
            if !has_auth_header {
                options
                    .headers
                    .push(("authorization".to_string(), token.clone()));
            }
        }
        Ok(options)
    }
}

fn authenticate(
    address: &Url,
    api_prefix: &str,
    username: &str,
    password: &str,
    timeout_ms: Option<i64>,
) -> Result<String> {
    let options = HttpRequestOptions {
        url: build_api_url(address, api_prefix, "auth/authenticate", AUTH_OP)?,
        method: Some("POST".to_string()),
        headers: Vec::new(),
        body: None,
        query: Vec::new(),
        json: Some(json!({
            "name": username,
            "password": password,
        })),
        form: Vec::new(),
        timeout_ms,
        connect_timeout_ms: None,
        follow_redirects: None,
        max_redirects: None,
        user_agent: None,
        basic_auth: None,
        bearer_token: None,
        fail_on_http_error: false,
    };
    let mut response = http::request_with_op(AUTH_OP, options)?;
    response.raise_for_status()?;
    let value = response.json()?;
    let object = expect_object(&value, AUTH_OP)?;
    let token = required_string_field(object, "token", AUTH_OP)?;
    if token.is_empty() {
        return Err(etcd_error(
            AUTH_OP,
            "authentication response returned an empty token",
        ));
    }
    Ok(token)
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
            format!("invalid etcd address `{address}`: {err}"),
        )
        .with_op(CONNECT_OP)
        .with_url(address.clone())
    })?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err(Error::new(
            ErrorKind::InvalidUrl,
            "etcd address must not contain a query string or fragment",
        )
        .with_op(CONNECT_OP)
        .with_url(address));
    }

    let mut path = url.path().trim_end_matches('/').to_string();
    path.push('/');
    url.set_path(&path);
    Ok(url)
}

fn normalize_api_prefix(api_prefix: Option<&str>) -> Result<String> {
    let prefix = api_prefix.unwrap_or(DEFAULT_API_PREFIX).trim();
    if prefix.is_empty() {
        return Err(
            Error::new(ErrorKind::EmptyInput, "api_prefix must not be empty").with_op(CONNECT_OP),
        );
    }
    let prefix = prefix.trim_matches('/');
    if prefix.is_empty() {
        return Err(
            Error::new(ErrorKind::EmptyInput, "api_prefix must not be empty").with_op(CONNECT_OP),
        );
    }
    if prefix.contains('?') || prefix.contains('#') {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "api_prefix must not contain a query string or fragment",
        )
        .with_op(CONNECT_OP));
    }
    Ok(prefix.to_string())
}

fn normalize_key<'a>(key: &'a [u8], op: &'static str) -> Result<&'a [u8]> {
    if key.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "key must not be empty").with_op(op));
    }
    Ok(key)
}

fn normalize_delete_key<'a>(key: &'a [u8], prefix: bool, op: &'static str) -> Result<&'a [u8]> {
    if key.is_empty() && !prefix {
        return Err(Error::new(ErrorKind::EmptyPath, "key must not be empty").with_op(op));
    }
    Ok(key)
}

fn normalize_list_range(prefix: &[u8]) -> (Vec<u8>, Vec<u8>) {
    if prefix.is_empty() {
        return (vec![0], vec![0]);
    }
    (prefix.to_vec(), prefix_range_end(prefix))
}

fn delete_range(key: &[u8], prefix: bool) -> (Vec<u8>, Option<Vec<u8>>) {
    if !prefix {
        return (key.to_vec(), None);
    }
    if key.is_empty() {
        return (vec![0], Some(vec![0]));
    }
    (key.to_vec(), Some(prefix_range_end(key)))
}

fn prefix_range_end(prefix: &[u8]) -> Vec<u8> {
    let mut end = prefix.to_vec();
    for index in (0..end.len()).rev() {
        if end[index] == 0xff {
            continue;
        }
        end[index] += 1;
        end.truncate(index + 1);
        return end;
    }
    vec![0]
}

fn build_api_url(base_url: &Url, api_prefix: &str, path: &str, op: &'static str) -> Result<String> {
    let normalized = normalize_api_path(api_prefix, path, op)?;
    let mut url = base_url.clone();
    let segments = normalized.split('/').filter(|segment| !segment.is_empty());
    {
        let mut path_segments = url.path_segments_mut().map_err(|_| {
            Error::new(ErrorKind::InvalidUrl, "address cannot be a base URL").with_op(op)
        })?;
        for segment in segments {
            path_segments.push(segment);
        }
    }
    Ok(url.to_string())
}

fn normalize_api_path(api_prefix: &str, path: &str, op: &'static str) -> Result<String> {
    let path = path.trim();
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "path must not be empty").with_op(op));
    }
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "path must not be empty").with_op(op));
    }
    if path == api_prefix || path.starts_with(&format!("{api_prefix}/")) {
        return Ok(path.to_string());
    }
    Ok(format!("{api_prefix}/{path}"))
}

fn apply_get_options(mut body: JsonValue, options: &EtcdGetOptions) -> JsonValue {
    let object = body
        .as_object_mut()
        .expect("etcd get request body must be an object");
    if let Some(revision) = options.revision {
        object.insert(
            "revision".to_string(),
            JsonValue::String(revision.to_string()),
        );
    }
    insert_bool_field(object, "serializable", options.serializable);
    body
}

fn apply_list_options(
    object: &mut JsonMap<String, JsonValue>,
    options: &EtcdListOptions,
) -> Result<()> {
    if let Some(limit) = options.limit {
        object.insert("limit".to_string(), JsonValue::String(limit.to_string()));
    }
    if let Some(revision) = options.revision {
        object.insert(
            "revision".to_string(),
            JsonValue::String(revision.to_string()),
        );
    }
    insert_bool_field(object, "serializable", options.serializable);
    insert_bool_field(object, "keys_only", options.keys_only);
    insert_bool_field(object, "count_only", options.count_only);
    insert_optional_i64_string(object, "min_mod_revision", options.min_mod_revision);
    insert_optional_i64_string(object, "max_mod_revision", options.max_mod_revision);
    insert_optional_i64_string(object, "min_create_revision", options.min_create_revision);
    insert_optional_i64_string(object, "max_create_revision", options.max_create_revision);

    if let Some(sort_order) = options.sort_order.as_deref() {
        object.insert(
            "sort_order".to_string(),
            JsonValue::String(normalize_sort_order(sort_order)?),
        );
    }
    if let Some(sort_target) = options.sort_target.as_deref() {
        object.insert(
            "sort_target".to_string(),
            JsonValue::String(normalize_sort_target(sort_target)?),
        );
    }

    Ok(())
}

fn normalize_sort_order(sort_order: &str) -> Result<String> {
    match sort_order.to_ascii_lowercase().as_str() {
        "none" => Ok("NONE".to_string()),
        "ascend" => Ok("ASCEND".to_string()),
        "descend" => Ok("DESCEND".to_string()),
        _ => Err(Error::new(
            ErrorKind::InvalidArgs,
            "sort_order must be one of `none`, `ascend`, or `descend`",
        )
        .with_op(LIST_OP)),
    }
}

fn normalize_sort_target(sort_target: &str) -> Result<String> {
    match sort_target.to_ascii_lowercase().as_str() {
        "key" => Ok("KEY".to_string()),
        "version" => Ok("VERSION".to_string()),
        "create" => Ok("CREATE".to_string()),
        "mod" => Ok("MOD".to_string()),
        "value" => Ok("VALUE".to_string()),
        _ => Err(Error::new(
            ErrorKind::InvalidArgs,
            "sort_target must be one of `key`, `version`, `create`, `mod`, or `value`",
        )
        .with_op(LIST_OP)),
    }
}

fn insert_bool_field(object: &mut JsonMap<String, JsonValue>, name: &str, value: bool) {
    if value {
        object.insert(name.to_string(), JsonValue::Bool(true));
    }
}

fn insert_optional_i64_string(
    object: &mut JsonMap<String, JsonValue>,
    name: &str,
    value: Option<i64>,
) {
    if let Some(value) = value {
        object.insert(name.to_string(), JsonValue::String(value.to_string()));
    }
}

fn parse_put_response(value: &JsonValue, op: &'static str) -> Result<EtcdPutResponse> {
    let object = expect_object(value, op)?;
    Ok(EtcdPutResponse {
        header: parse_response_header(object.get("header"), op)?,
        prev_kv: parse_optional_kv_entry(object.get("prev_kv"), op)?,
    })
}

fn parse_delete_response(value: &JsonValue, op: &'static str) -> Result<EtcdDeleteResponse> {
    let object = expect_object(value, op)?;
    Ok(EtcdDeleteResponse {
        header: parse_response_header(object.get("header"), op)?,
        deleted: optional_i64_field(object, "deleted").unwrap_or(0),
        prev_kvs: parse_kv_entries(object.get("prev_kvs"), op)?,
    })
}

fn parse_list_response(value: &JsonValue, op: &'static str) -> Result<EtcdListResponse> {
    let object = expect_object(value, op)?;
    Ok(EtcdListResponse {
        header: parse_response_header(object.get("header"), op)?,
        kvs: parse_kv_entries(object.get("kvs"), op)?,
        count: optional_i64_field(object, "count").unwrap_or(0),
        more: object
            .get("more")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false),
    })
}

fn parse_response_header(
    value: Option<&JsonValue>,
    op: &'static str,
) -> Result<Option<EtcdResponseHeader>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let object = expect_object(value, op)?;
    Ok(Some(EtcdResponseHeader {
        cluster_id: required_u64_string_field(object, "cluster_id", op)?,
        member_id: required_u64_string_field(object, "member_id", op)?,
        revision: required_i64_field(object, "revision", op)?,
        raft_term: required_u64_string_field(object, "raft_term", op)?,
    }))
}

fn parse_kv_entries(value: Option<&JsonValue>, op: &'static str) -> Result<Vec<EtcdKvEntry>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if value.is_null() {
        return Ok(Vec::new());
    }
    let items = value
        .as_array()
        .ok_or_else(|| etcd_error(op, "expected `kvs` to be an array"))?;
    items.iter().map(|item| parse_kv_entry(item, op)).collect()
}

fn parse_optional_kv_entry(
    value: Option<&JsonValue>,
    op: &'static str,
) -> Result<Option<EtcdKvEntry>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(parse_kv_entry(value, op)?))
}

fn parse_kv_entry(value: &JsonValue, op: &'static str) -> Result<EtcdKvEntry> {
    let object = expect_object(value, op)?;
    Ok(EtcdKvEntry {
        key: required_base64_field(object, "key", op)?,
        value: optional_base64_field(object, "value", op)?.unwrap_or_default(),
        create_revision: required_i64_field(object, "create_revision", op)?,
        mod_revision: required_i64_field(object, "mod_revision", op)?,
        version: required_i64_field(object, "version", op)?,
        lease: optional_i64_field(object, "lease").unwrap_or(0),
    })
}

fn expect_object<'a>(
    value: &'a JsonValue,
    op: &'static str,
) -> Result<&'a JsonMap<String, JsonValue>> {
    value
        .as_object()
        .ok_or_else(|| etcd_error(op, "expected a JSON object response body"))
}

fn required_string_field(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    op: &'static str,
) -> Result<String> {
    match object.get(key) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(_) => Err(etcd_error(op, format!("expected `{key}` to be a string"))),
        None => Err(etcd_error(op, format!("missing `{key}` field in response"))),
    }
}

fn required_base64_field(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    op: &'static str,
) -> Result<Vec<u8>> {
    let encoded = required_string_field(object, key, op)?;
    BASE64_STANDARD
        .decode(encoded)
        .map_err(|err| etcd_error(op, format!("failed to decode `{key}`: {err}")))
}

fn optional_base64_field(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    op: &'static str,
) -> Result<Option<Vec<u8>>> {
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    match value {
        JsonValue::Null => Ok(None),
        JsonValue::String(value) => BASE64_STANDARD
            .decode(value)
            .map(Some)
            .map_err(|err| etcd_error(op, format!("failed to decode `{key}`: {err}"))),
        _ => Err(etcd_error(op, format!("expected `{key}` to be a string"))),
    }
}

fn required_i64_field(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    op: &'static str,
) -> Result<i64> {
    let Some(value) = object.get(key) else {
        return Err(etcd_error(op, format!("missing `{key}` field in response")));
    };
    parse_json_i64(value)
        .ok_or_else(|| etcd_error(op, format!("missing or invalid `{key}` field in response")))
}

fn optional_i64_field(object: &JsonMap<String, JsonValue>, key: &str) -> Option<i64> {
    object.get(key).and_then(parse_json_i64)
}

fn required_u64_string_field(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    op: &'static str,
) -> Result<String> {
    let Some(value) = object.get(key) else {
        return Err(etcd_error(op, format!("missing `{key}` field in response")));
    };
    parse_json_u64_string(value)
        .ok_or_else(|| etcd_error(op, format!("missing or invalid `{key}` field in response")))
}

fn parse_json_i64(value: &JsonValue) -> Option<i64> {
    match value {
        JsonValue::Number(value) => value.as_i64(),
        JsonValue::String(value) => value.parse().ok(),
        _ => None,
    }
}

fn parse_json_u64_string(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Number(value) => value.as_u64().map(|value| value.to_string()),
        JsonValue::String(value) => value.parse::<u64>().ok().map(|_| value.clone()),
        _ => None,
    }
}

fn encode_bytes(bytes: &[u8]) -> String {
    BASE64_STANDARD.encode(bytes)
}

fn etcd_error(op: &'static str, msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::Etcd, msg).with_op(op)
}
