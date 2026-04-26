use mlua::{Lua, Table, UserData, UserDataFields, UserDataMethods, Value};
use ptool_engine::{
    Error as EngineError, HttpRequestOptions, HttpResponse as EngineHttpResponse, PtoolEngine,
};
use std::collections::BTreeMap;

const REQUEST_SIGNATURE: &str = "ptool.http.request(options)";

pub(crate) struct HttpResponse {
    inner: EngineHttpResponse,
}

pub(crate) fn request(
    lua: &Lua,
    engine: &PtoolEngine,
    options: Table,
) -> mlua::Result<HttpResponse> {
    let options = parse_request_options(lua, options)?;
    let response = engine.http_request(options).map_err(to_lua_request_error)?;
    Ok(HttpResponse { inner: response })
}

fn parse_request_options(lua: &Lua, options: Table) -> mlua::Result<HttpRequestOptions> {
    validate_request_option_keys(&options)?;
    let Some(url) = options.get::<Option<String>>("url")? else {
        return Err(crate::lua_error::invalid_argument(
            REQUEST_SIGNATURE,
            "requires `url`",
        ));
    };

    let method = options.get::<Option<String>>("method")?;
    let headers = parse_headers(options.get::<Option<Table>>("headers")?)?;
    let body = parse_body(options.get::<Option<Value>>("body")?)?;
    let query = parse_string_pairs(options.get::<Option<Table>>("query")?, "query")?;
    let json = parse_json_body(lua, options.get::<Option<Value>>("json")?)?;
    let form = parse_string_pairs(options.get::<Option<Table>>("form")?, "form")?;
    let timeout_ms = options.get::<Option<i64>>("timeout_ms")?;
    let connect_timeout_ms = options.get::<Option<i64>>("connect_timeout_ms")?;
    let follow_redirects = options.get::<Option<bool>>("follow_redirects")?;
    let max_redirects = options.get::<Option<i64>>("max_redirects")?;
    let user_agent = options.get::<Option<String>>("user_agent")?;
    let basic_auth = parse_basic_auth(options.get::<Option<Table>>("basic_auth")?)?;
    let bearer_token = options.get::<Option<String>>("bearer_token")?;
    let fail_on_http_error = options
        .get::<Option<bool>>("fail_on_http_error")?
        .unwrap_or(false);

    Ok(HttpRequestOptions {
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
    })
}

fn validate_request_option_keys(options: &Table) -> mlua::Result<()> {
    for pair in options.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = match key {
            Value::String(value) => value.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REQUEST_SIGNATURE,
                    "option keys must be strings",
                ));
            }
        };

        match key.as_str() {
            "url" | "method" | "headers" | "body" | "query" | "json" | "form" | "timeout_ms"
            | "connect_timeout_ms" | "follow_redirects" | "max_redirects" | "user_agent"
            | "basic_auth" | "bearer_token" | "fail_on_http_error" => {}
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REQUEST_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(())
}

fn parse_headers(headers: Option<Table>) -> mlua::Result<Vec<(String, String)>> {
    let Some(headers) = headers else {
        return Ok(Vec::new());
    };

    let mut pairs = Vec::new();
    for pair in headers.pairs::<Value, Value>() {
        let (name, value) = pair?;
        let name = parse_string_key(name, "headers")?;
        let value = parse_string_value(value, "headers")?;
        pairs.push((name, value));
    }

    Ok(pairs)
}

fn parse_string_pairs(table: Option<Table>, field: &str) -> mlua::Result<Vec<(String, String)>> {
    let Some(table) = table else {
        return Ok(Vec::new());
    };

    let mut pairs = Vec::new();
    for pair in table.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_string_key(key, field)?;
        let value = parse_string_like_value(value, field)?;
        pairs.push((key, value));
    }

    Ok(pairs)
}

fn parse_body(body: Option<Value>) -> mlua::Result<Option<Vec<u8>>> {
    let Some(body) = body else {
        return Ok(None);
    };

    match body {
        Value::String(value) => Ok(Some(value.as_bytes().to_vec())),
        _ => Err(crate::lua_error::invalid_argument(
            REQUEST_SIGNATURE,
            "`body` must be a string",
        )),
    }
}

fn parse_json_body(lua: &Lua, value: Option<Value>) -> mlua::Result<Option<serde_json::Value>> {
    let Some(value) = value else {
        return Ok(None);
    };

    crate::json::lua_value_to_json(
        lua,
        value,
        "ptool.http.request(options) `json` has invalid value",
    )
    .map(Some)
}

fn parse_basic_auth(value: Option<Table>) -> mlua::Result<Option<(String, String)>> {
    let Some(value) = value else {
        return Ok(None);
    };

    for pair in value.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = parse_string_key(key, "basic_auth")?;
        match key.as_str() {
            "username" | "password" => {}
            _ => {
                return Err(crate::lua_error::invalid_option(
                    REQUEST_SIGNATURE,
                    format!("`basic_auth` unknown option `{key}`"),
                ));
            }
        }
    }

    let Some(username) = value.get::<Option<String>>("username")? else {
        return Err(crate::lua_error::invalid_argument(
            REQUEST_SIGNATURE,
            "`basic_auth.username` is required",
        ));
    };
    let Some(password) = value.get::<Option<String>>("password")? else {
        return Err(crate::lua_error::invalid_argument(
            REQUEST_SIGNATURE,
            "`basic_auth.password` is required",
        ));
    };

    Ok(Some((username, password)))
}

fn parse_string_key(value: Value, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_argument(
            REQUEST_SIGNATURE,
            format!("`{field}` keys must be strings"),
        )),
    }
}

fn parse_string_value(value: Value, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_argument(
            REQUEST_SIGNATURE,
            format!("`{field}` values must be strings"),
        )),
    }
}

fn parse_string_like_value(value: Value, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        Value::Integer(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Boolean(value) => Ok(value.to_string()),
        _ => Err(crate::lua_error::invalid_argument(
            REQUEST_SIGNATURE,
            format!("`{field}` values must be strings, numbers, or booleans"),
        )),
    }
}

impl HttpResponse {
    fn headers_as_lua_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        let mut merged = BTreeMap::new();
        for (key, value) in &self.inner.headers {
            merged
                .entry(key.clone())
                .and_modify(|existing: &mut String| {
                    existing.push_str(", ");
                    existing.push_str(value);
                })
                .or_insert_with(|| value.clone());
        }
        for (key, value) in merged {
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
            let json = this.inner.json().map_err(to_lua_response_error)?;
            crate::json::json_value_to_lua(
                lua,
                &json,
                "ptool.http response json has unsupported number",
            )
        });

        methods.add_method_mut("bytes", |lua, this, ()| {
            let bytes = this.inner.bytes().map_err(to_lua_response_error)?;
            lua.create_string(&bytes)
        });

        methods.add_method("header", |lua, this, name: String| {
            match this.inner.header(&name) {
                Some(value) => Ok(Value::String(lua.create_string(value)?)),
                None => Ok(Value::Nil),
            }
        });

        methods.add_method("header_values", |lua, this, name: String| {
            let values = this.inner.header_values(&name);
            lua.create_sequence_from(values)
        });

        methods.add_method("raise_for_status", |_, this, ()| {
            this.inner.raise_for_status().map_err(to_lua_response_error)
        });
    }
}

fn to_lua_request_error(err: EngineError) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, REQUEST_SIGNATURE)
}

fn to_lua_response_error(err: EngineError) -> mlua::Error {
    crate::lua_error::lua_error_from_engine(err, "ptool.http")
}
