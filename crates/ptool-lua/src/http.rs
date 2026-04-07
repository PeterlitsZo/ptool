use mlua::{Lua, Table, UserData, UserDataFields, UserDataMethods, Value};
use ptool_engine::{
    Error as EngineError, HttpRequestOptions, HttpResponse as EngineHttpResponse, PtoolEngine,
};
use serde_json::Value as JsonValue;

const REQUEST_SIGNATURE: &str = "ptool.http.request(options)";

pub(crate) struct HttpResponse {
    inner: EngineHttpResponse,
}

pub(crate) fn request(engine: &PtoolEngine, options: Table) -> mlua::Result<HttpResponse> {
    let options = parse_request_options(options)?;
    let response = engine.http_request(options).map_err(to_lua_request_error)?;
    Ok(HttpResponse { inner: response })
}

fn parse_request_options(options: Table) -> mlua::Result<HttpRequestOptions> {
    let Some(url) = options.get::<Option<String>>("url")? else {
        return Err(mlua::Error::runtime(
            "ptool.http.request(options) requires `url`",
        ));
    };

    let method = options.get::<Option<String>>("method")?;
    let headers = parse_headers(options.get::<Option<Table>>("headers")?)?;
    let body = parse_body(options.get::<Option<Value>>("body")?)?;
    let timeout_ms = options.get::<Option<i64>>("timeout_ms")?;

    Ok(HttpRequestOptions {
        url,
        method,
        headers,
        body,
        timeout_ms,
    })
}

fn parse_headers(headers: Option<Table>) -> mlua::Result<Vec<(String, String)>> {
    let Some(headers) = headers else {
        return Ok(Vec::new());
    };

    let mut pairs = Vec::new();
    for pair in headers.pairs::<String, String>() {
        let (name, value) = pair?;
        pairs.push((name, value));
    }

    Ok(pairs)
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

impl HttpResponse {
    fn headers_as_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        for (key, value) in &self.inner.headers {
            table.raw_set(key.as_str(), value.as_str())?;
        }
        Ok(table)
    }
}

impl UserData for HttpResponse {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("status", |_, this| Ok(this.inner.status));
        fields.add_field_method_get("ok", |_, this| Ok(this.inner.ok));
        fields.add_field_method_get("url", |_, this| Ok(this.inner.url.clone()));
        fields.add_field_method_get("headers", |lua, this| this.headers_as_lua_table(lua));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("text", |_, this, ()| {
            this.inner.text().map_err(to_lua_response_error)
        });

        methods.add_method_mut("json", |lua, this, ()| {
            let json: JsonValue = this.inner.json().map_err(to_lua_response_error)?;
            json_value_to_lua(lua, &json)
        });

        methods.add_method_mut("bytes", |lua, this, ()| {
            let bytes = this.inner.bytes().map_err(to_lua_response_error)?;
            lua.create_string(&bytes)
        });
    }
}

fn to_lua_request_error(err: EngineError) -> mlua::Error {
    mlua::Error::runtime(format!("{REQUEST_SIGNATURE} {}", err.msg))
}

fn to_lua_response_error(err: EngineError) -> mlua::Error {
    mlua::Error::runtime(format!("ptool.http {}", err.msg))
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
