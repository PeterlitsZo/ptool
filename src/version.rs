use semver::Version;

const CURRENT_PTOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) fn ensure_min_ptool_version(required_raw: &str) -> mlua::Result<()> {
    let required = parse_semver(required_raw).map_err(|err| {
        mlua::Error::runtime(format!("ptool.use invalid version `{required_raw}`: {err}"))
    })?;
    let current_with_prerelease = parse_semver(CURRENT_PTOOL_VERSION).map_err(|err| {
        mlua::Error::runtime(format!(
            "ptool internal version `{CURRENT_PTOOL_VERSION}` is invalid: {err}"
        ))
    })?;
    let current = strip_prerelease(current_with_prerelease);

    if required > current {
        return Err(mlua::Error::runtime(format!(
            "ptool is too old: need at least v{required}, current version is v{current}"
        )));
    }

    Ok(())
}

fn parse_semver(input: &str) -> Result<Version, semver::Error> {
    let trimmed = input.trim();
    let normalized = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    Version::parse(normalized)
}

fn strip_prerelease(version: Version) -> Version {
    Version::new(version.major, version.minor, version.patch)
}
