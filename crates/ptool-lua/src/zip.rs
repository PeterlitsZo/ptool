use mlua::{Lua, String as LuaString};
use ptool_engine::{PtoolEngine, ZipFormat};

pub(crate) fn compress(
    lua: &Lua,
    engine: &PtoolEngine,
    format: String,
    input: LuaString,
    op: &str,
) -> mlua::Result<LuaString> {
    let format = ZipFormat::parse(&format, op)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, op))?;
    let output = engine
        .zip_compress(input.as_bytes().as_ref(), format, op)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, op))?;
    lua.create_string(&output)
}

pub(crate) fn decompress(
    lua: &Lua,
    engine: &PtoolEngine,
    format: String,
    input: LuaString,
    op: &str,
) -> mlua::Result<LuaString> {
    let format = ZipFormat::parse(&format, op)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, op))?;
    let output = engine
        .zip_decompress(input.as_bytes().as_ref(), format, op)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, op))?;
    lua.create_string(&output)
}
