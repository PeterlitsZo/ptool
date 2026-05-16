use ptool_engine::PtoolEngine;

pub(crate) fn current_ptool_version(engine: &PtoolEngine) -> &'static str {
    engine.ptool_version()
}

pub(crate) fn ensure_min_ptool_version(
    engine: &PtoolEngine,
    required_raw: &str,
) -> mlua::Result<()> {
    let current_ptool_version = current_ptool_version(engine);

    if supports_legacy_min_version(engine, required_raw) {
        let is_ok = engine
            .semver_is_min_version(current_ptool_version, required_raw)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.use"))?;

        if is_ok {
            return Ok(());
        }

        let required = engine
            .semver_parse(required_raw)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.use"))?;
        let current = engine.semver_strip_prerelease(
            engine
                .semver_parse(current_ptool_version)
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

    let requirement = engine
        .semver_req_parse(required_raw)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.use"))?;
    let current = engine
        .semver_parse(current_ptool_version)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.use"))?;

    if engine.semver_req_matches(&requirement, &current) {
        return Ok(());
    }

    Err(crate::lua_error::to_mlua_error(
        crate::lua_error::LuaError::new(
            "version_requirement_not_met",
            format!("ptool version v{current} does not satisfy requirement `{requirement}`"),
        )
        .with_op("ptool.use")
        .with_detail(format!("requirement: {requirement}, current: v{current}")),
    ))
}

fn supports_legacy_min_version(engine: &PtoolEngine, required_raw: &str) -> bool {
    if looks_like_requirement_expression(required_raw) {
        return false;
    }

    engine.semver_parse(required_raw).is_ok()
}

fn looks_like_requirement_expression(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }

    let normalized = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);

    normalized.starts_with('>')
        || normalized.starts_with('<')
        || normalized.starts_with('=')
        || normalized.starts_with('^')
        || normalized.starts_with('~')
        || normalized.contains(',')
        || normalized.contains('*')
}
