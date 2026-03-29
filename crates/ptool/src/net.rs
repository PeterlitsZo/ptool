use mlua::{Lua, Table};
use ptool_engine::{Error as EngineError, HostKind, PtoolEngine};

const PARSE_URL_SIGNATURE: &str = "ptool.net.parse_url(input)";
const PARSE_IP_SIGNATURE: &str = "ptool.net.parse_ip(input)";
const PARSE_HOST_PORT_SIGNATURE: &str = "ptool.net.parse_host_port(input)";

pub(crate) fn parse_url(lua: &Lua, engine: &PtoolEngine, input: String) -> mlua::Result<Table> {
    let parsed = engine
        .parse_url(&input)
        .map_err(|err| to_lua_error(PARSE_URL_SIGNATURE, err))?;

    let table = lua.create_table()?;
    table.set("kind", "url")?;
    table.set("input", input)?;
    table.set("normalized", parsed.normalized)?;
    table.set("scheme", parsed.scheme)?;
    table.set("username", parsed.username)?;
    table.set("password", parsed.password)?;
    table.set("host", parsed.host)?;
    set_optional_host_kind(&table, parsed.host_kind)?;
    table.set("port", parsed.port.map(i64::from))?;
    table.set("path", parsed.path)?;
    table.set("query", parsed.query)?;
    table.set("fragment", parsed.fragment)?;
    Ok(table)
}

pub(crate) fn parse_ip(lua: &Lua, engine: &PtoolEngine, input: String) -> mlua::Result<Table> {
    let parsed = engine
        .parse_ip(&input)
        .map_err(|err| to_lua_error(PARSE_IP_SIGNATURE, err))?;

    let table = lua.create_table()?;
    table.set("kind", "ip")?;
    table.set("input", input)?;
    table.set("normalized", parsed.normalized)?;
    table.set("version", i64::from(parsed.version))?;
    Ok(table)
}

pub(crate) fn parse_host_port(
    lua: &Lua,
    engine: &PtoolEngine,
    input: String,
) -> mlua::Result<Table> {
    let parsed = engine
        .parse_host_port(&input)
        .map_err(|err| to_lua_error(PARSE_HOST_PORT_SIGNATURE, err))?;

    let table = lua.create_table()?;
    table.set("kind", "host_port")?;
    table.set("input", input)?;
    table.set("normalized", parsed.normalized)?;
    table.set("host", parsed.host)?;
    table.set("host_kind", host_kind_to_lua(parsed.host_kind))?;
    table.set("port", i64::from(parsed.port))?;
    Ok(table)
}

fn set_optional_host_kind(table: &Table, host_kind: Option<HostKind>) -> mlua::Result<()> {
    table.set("host_kind", host_kind.map(host_kind_to_lua))
}

fn to_lua_error(context: &str, err: EngineError) -> mlua::Error {
    mlua::Error::runtime(format!("{context}: {}", err.msg))
}

fn host_kind_to_lua(host_kind: HostKind) -> &'static str {
    match host_kind {
        HostKind::Domain => "domain",
        HostKind::Ipv4 => "ipv4",
        HostKind::Ipv6 => "ipv6",
    }
}
