use ptool_engine::PtoolEngine;

const CURRENT_PTOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn ensure_min_ptool_version(
    engine: &PtoolEngine,
    required_raw: &str,
) -> mlua::Result<()> {
    let is_ok = engine
        .semver_is_min_version(CURRENT_PTOOL_VERSION, required_raw)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.use"))?;

    if !is_ok {
        let required = engine
            .semver_parse(required_raw)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.use"))?;
        let current = engine.semver_strip_prerelease(
            engine
                .semver_parse(CURRENT_PTOOL_VERSION)
                .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.use"))?,
        );
        return Err(crate::lua_error::to_mlua_error(
            crate::lua_error::LuaError::new(
                "version_too_old",
                format!(
                    "ptool is too old: need at least v{required}, current version is v{current}"
                ),
            )
            .with_op("ptool.use")
            .with_detail(format!("required: v{required}, current: v{current}")),
        ));
    }

    Ok(())
}
