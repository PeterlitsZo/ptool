use mlua::{Lua, Table, UserData, UserDataFields, UserDataMethods, Value};
use reqwest::Method;
use reqwest::blocking::{Client, Response as BlockingResponse};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 30_000;

struct HttpRequestOptions {
    url: String,
    method: Method,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    timeout_ms: u64,
}

enum BodyState {
    Unread(BlockingResponse),
    Consumed,
}

pub(crate) struct HttpResponse {
    status: i64,
    ok: bool,
    url: String,
    headers: Vec<(String, String)>,
    body: BodyState,
}

pub(crate) fn request(options: Table) -> mlua::Result<HttpResponse> {
    let options = parse_request_options(options)?;

    let client = Client::builder()
        .timeout(Duration::from_millis(options.timeout_ms))
        .build()
        .map_err(|err| mlua::Error::runtime(format!("ptool.http request build failed: {err:?}")))?;

    let mut request = client.request(options.method, &options.url);
    if !options.headers.is_empty() {
        request = request.headers(options.headers);
    }
    if let Some(body) = options.body {
        request = request.body(body);
    }

    let response = request
        .send()
        .map_err(|err| mlua::Error::runtime(format!("ptool.http request failed: {err:?}")))?;

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

fn parse_request_options(options: Table) -> mlua::Result<HttpRequestOptions> {
    let Some(url) = options.get::<Option<String>>("url")? else {
        return Err(mlua::Error::runtime(
            "ptool.http.request(options) requires `url`",
        ));
    };

    let method = parse_method(options.get::<Option<String>>("method")?)?;
    let headers = parse_headers(options.get::<Option<Table>>("headers")?)?;
    let body = parse_body(options.get::<Option<Value>>("body")?)?;
    let timeout_ms = parse_timeout_ms(options.get::<Option<i64>>("timeout_ms")?)?;

    Ok(HttpRequestOptions {
        url,
        method,
        headers,
        body,
        timeout_ms,
    })
}

fn parse_method(method: Option<String>) -> mlua::Result<Method> {
    let method = method.unwrap_or_else(|| "GET".to_string());
    Method::from_bytes(method.as_bytes())
        .map_err(|err| mlua::Error::runtime(format!("ptool.http invalid method `{method}`: {err}")))
}

fn parse_headers(headers: Option<Table>) -> mlua::Result<HeaderMap> {
    let Some(headers) = headers else {
        return Ok(HeaderMap::new());
    };

    let mut header_map = HeaderMap::new();
    for pair in headers.pairs::<String, String>() {
        let (name, value) = pair?;
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|err| {
            mlua::Error::runtime(format!("ptool.http invalid header name `{name}`: {err}"))
        })?;
        let header_value = HeaderValue::from_str(&value).map_err(|err| {
            mlua::Error::runtime(format!(
                "ptool.http invalid header value for `{name}`: {err}"
            ))
        })?;
        header_map.append(header_name, header_value);
    }

    Ok(header_map)
}

fn parse_body(body: Option<Value>) -> mlua::Result<Option<Vec<u8>>> {
    let Some(body) = body else {
        return Ok(None);
    };

    match body {
        Value::String(value) => Ok(Some(value.as_bytes().to_vec())),
        _ => Err(mlua::Error::runtime(
            "ptool.http.request(options) `body` must be a string",
        )),
    }
}

fn parse_timeout_ms(timeout_ms: Option<i64>) -> mlua::Result<u64> {
    let timeout_ms = timeout_ms.unwrap_or(DEFAULT_TIMEOUT_MS as i64);
    if timeout_ms <= 0 {
        return Err(mlua::Error::runtime(
            "ptool.http.request(options) `timeout_ms` must be > 0",
        ));
    }

    u64::try_from(timeout_ms)
        .map_err(|_| mlua::Error::runtime("ptool.http.request(options) `timeout_ms` is too large"))
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

impl HttpResponse {
    fn consume_body_bytes(&mut self) -> mlua::Result<Vec<u8>> {
        let state = std::mem::replace(&mut self.body, BodyState::Consumed);
        match state {
            BodyState::Unread(response) => {
                response.bytes().map(|body| body.to_vec()).map_err(|err| {
                    mlua::Error::runtime(format!("ptool.http response read failed: {err}"))
                })
            }
            BodyState::Consumed => Err(mlua::Error::runtime(
                "ptool.http response body already consumed",
            )),
        }
    }

    fn headers_as_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        for (key, value) in &self.headers {
            table.raw_set(key.as_str(), value.as_str())?;
        }
        Ok(table)
    }
}

impl UserData for HttpResponse {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("status", |_, this| Ok(this.status));
        fields.add_field_method_get("ok", |_, this| Ok(this.ok));
        fields.add_field_method_get("url", |_, this| Ok(this.url.clone()));
        fields.add_field_method_get("headers", |lua, this| this.headers_as_lua_table(lua));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("text", |_, this, ()| {
            let bytes = this.consume_body_bytes()?;
            Ok(String::from_utf8_lossy(&bytes).to_string())
        });

        methods.add_method_mut("json", |lua, this, ()| {
            let bytes = this.consume_body_bytes()?;
            let json: JsonValue = serde_json::from_slice(&bytes).map_err(|err| {
                mlua::Error::runtime(format!("ptool.http response json parse failed: {err}"))
            })?;
            json_value_to_lua(lua, &json)
        });

        methods.add_method_mut("bytes", |lua, this, ()| {
            let bytes = this.consume_body_bytes()?;
            lua.create_string(&bytes)
        });
    }
}

fn json_value_to_lua(lua: &Lua, value: &JsonValue) -> mlua::Result<Value> {
    match value {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(value) => Ok(Value::Boolean(*value)),
        JsonValue::Number(value) => json_number_to_lua(value),
        JsonValue::String(value) => Ok(Value::String(lua.create_string(value)?)),
        JsonValue::Array(values) => {
            let table = lua.create_table()?;
            for (index, item) in values.iter().enumerate() {
                table.raw_set(index + 1, json_value_to_lua(lua, item)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(values) => {
            let table = lua.create_table()?;
            for (key, item) in values {
                table.raw_set(key.as_str(), json_value_to_lua(lua, item)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

fn json_number_to_lua(value: &serde_json::Number) -> mlua::Result<Value> {
    if let Some(number) = value.as_i64() {
        return Ok(Value::Integer(number));
    }
    if let Some(number) = value.as_u64() {
        if let Ok(number) = i64::try_from(number) {
            return Ok(Value::Integer(number));
        }
        return Ok(Value::Number(number as f64));
    }
    if let Some(number) = value.as_f64() {
        return Ok(Value::Number(number));
    }
    Err(mlua::Error::runtime(
        "ptool.http response json has unsupported number",
    ))
}
