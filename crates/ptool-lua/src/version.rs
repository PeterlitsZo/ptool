use ptool_engine::PtoolEngine;

const CURRENT_PTOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn ensure_min_ptool_version(
    engine: &PtoolEngine,
    required_raw: &str,
) -> mlua::Result<()> {
    let required = engine.semver_parse(required_raw).map_err(|err| {
        mlua::Error::runtime(format!(
            "ptool.use invalid version `{required_raw}`: {}",
            err.msg
        ))
    })?;
    let current_with_prerelease = engine.semver_parse(CURRENT_PTOOL_VERSION).map_err(|err| {
        mlua::Error::runtime(format!(
            "ptool internal version `{CURRENT_PTOOL_VERSION}` is invalid: {}",
            err.msg
        ))
    })?;
    let current = engine.semver_strip_prerelease(current_with_prerelease);

    if required > current {
        return Err(mlua::Error::runtime(format!(
            "ptool is too old: need at least v{required}, current version is v{current}"
        )));
    }

    Ok(())
}
