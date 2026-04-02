use crate::{Error, ErrorKind, Result};
pub use semver::{
    BuildMetadata as SemverBuildMetadata, Prerelease as SemverPrerelease, Version as SemverVersion,
};
use std::cmp::Ordering;

pub(crate) fn parse(input: &str) -> Result<SemverVersion> {
    let trimmed = input.trim();
    let normalized = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);

    SemverVersion::parse(normalized)
        .map_err(|err| Error::new(ErrorKind::InvalidSemver, err.to_string()))
}

pub(crate) fn is_valid(input: &str) -> bool {
    parse(input).is_ok()
}

pub(crate) fn compare(a: &SemverVersion, b: &SemverVersion) -> Ordering {
    a.cmp(b)
}

pub(crate) fn strip_prerelease(version: SemverVersion) -> SemverVersion {
    SemverVersion::new(version.major, version.minor, version.patch)
}

pub(crate) fn bump(version: SemverVersion, op: &str) -> Result<SemverVersion> {
    match op {
        "major" => bump_major(version),
        "minor" => bump_minor(version),
        "patch" => bump_patch(version),
        "release" => Ok(bump_release(version)),
        "alpha" => bump_prerelease(version, PreChannel::Alpha),
        "beta" => bump_prerelease(version, PreChannel::Beta),
        "rc" => bump_prerelease(version, PreChannel::Rc),
        _ => Err(Error::new(
            ErrorKind::InvalidSemverOperation,
            "`op` must be one of `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`",
        )),
    }
}

fn bump_major(mut version: SemverVersion) -> Result<SemverVersion> {
    version.major = version
        .major
        .checked_add(1)
        .ok_or_else(|| Error::new(ErrorKind::SemverOverflow, "major overflow"))?;
    version.minor = 0;
    version.patch = 0;
    version.pre = SemverPrerelease::EMPTY;
    version.build = SemverBuildMetadata::EMPTY;
    Ok(version)
}

fn bump_minor(mut version: SemverVersion) -> Result<SemverVersion> {
    version.minor = version
        .minor
        .checked_add(1)
        .ok_or_else(|| Error::new(ErrorKind::SemverOverflow, "minor overflow"))?;
    version.patch = 0;
    version.pre = SemverPrerelease::EMPTY;
    version.build = SemverBuildMetadata::EMPTY;
    Ok(version)
}

fn bump_patch(mut version: SemverVersion) -> Result<SemverVersion> {
    version.patch = version
        .patch
        .checked_add(1)
        .ok_or_else(|| Error::new(ErrorKind::SemverOverflow, "patch overflow"))?;
    version.pre = SemverPrerelease::EMPTY;
    version.build = SemverBuildMetadata::EMPTY;
    Ok(version)
}

fn bump_release(mut version: SemverVersion) -> SemverVersion {
    version.pre = SemverPrerelease::EMPTY;
    version.build = SemverBuildMetadata::EMPTY;
    version
}

fn bump_prerelease(version: SemverVersion, target: PreChannel) -> Result<SemverVersion> {
    let mut version = version;
    version.build = SemverBuildMetadata::EMPTY;

    let pre = version.pre.as_str();
    if pre.is_empty() {
        version.patch = version
            .patch
            .checked_add(1)
            .ok_or_else(|| Error::new(ErrorKind::SemverOverflow, "patch overflow"))?;
        version.pre = parse_prerelease(&format!("{}.1", target.as_str()))?;
        return Ok(version);
    }

    let (current_channel, current_number) = parse_channel_and_number(pre)?;
    if current_channel.rank() > target.rank() {
        return Err(Error::new(
            ErrorKind::InvalidSemverOperation,
            format!(
                "does not allow prerelease downgrade from `{}` to `{}`",
                current_channel.as_str(),
                target.as_str()
            ),
        ));
    }

    let next_number = if current_channel.rank() < target.rank() {
        1
    } else {
        current_number.unwrap_or(0).saturating_add(1)
    };

    version.pre = parse_prerelease(&format!("{}.{}", target.as_str(), next_number))?;
    Ok(version)
}

fn parse_prerelease(value: &str) -> Result<SemverPrerelease> {
    SemverPrerelease::new(value).map_err(|err| {
        Error::new(
            ErrorKind::InvalidSemver,
            format!("invalid prerelease value `{value}`: {err}"),
        )
    })
}

fn parse_channel_and_number(pre: &str) -> Result<(PreChannel, Option<u64>)> {
    let parts: Vec<&str> = pre.split('.').collect();
    let channel =
        PreChannel::parse(parts.first().copied().unwrap_or_default()).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidSemverOperation,
                format!("only supports prerelease channels `alpha`, `beta`, `rc`, got `{pre}`"),
            )
        })?;

    if parts.len() == 1 {
        return Ok((channel, None));
    }
    if parts.len() > 2 {
        return Err(Error::new(
            ErrorKind::InvalidSemverOperation,
            format!(
                "prerelease `{pre}` is unsupported; expected `<channel>` or `<channel>.<number>`"
            ),
        ));
    }

    let number = parts[1].parse::<u64>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidSemverOperation,
            format!("prerelease `{pre}` is unsupported; expected numeric suffix"),
        )
    })?;
    Ok((channel, Some(number)))
}

#[derive(Clone, Copy)]
enum PreChannel {
    Alpha,
    Beta,
    Rc,
}

impl PreChannel {
    fn parse(input: &str) -> Option<Self> {
        match input {
            "alpha" => Some(Self::Alpha),
            "beta" => Some(Self::Beta),
            "rc" => Some(Self::Rc),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Rc => "rc",
        }
    }

    fn rank(self) -> u8 {
        match self {
            Self::Alpha => 1,
            Self::Beta => 2,
            Self::Rc => 3,
        }
    }
}
