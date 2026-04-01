use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use semver::{BuildMetadata, Prerelease, Version};
use std::cmp::Ordering;

const PARSE_SIGNATURE: &str = "ptool.semver.parse(version)";
const COMPARE_SIGNATURE: &str = "ptool.semver.compare(a, b)";
const BUMP_SIGNATURE: &str = "ptool.semver.bump(v, op)";

#[derive(Clone)]
pub(crate) struct LuaSemVer {
    version: Version,
}

pub(crate) fn parse(version: Value) -> mlua::Result<LuaSemVer> {
    let version = parse_version_from_string_arg(version, PARSE_SIGNATURE, "version")?;
    Ok(LuaSemVer { version })
}

pub(crate) fn is_valid(version: Value) -> bool {
    match version {
        Value::String(value) => match value.to_str() {
            Ok(text) => parse_semver(text.as_ref()).is_ok(),
            Err(_) => false,
        },
        _ => false,
    }
}

pub(crate) fn compare(a: Value, b: Value) -> mlua::Result<i64> {
    let left = parse_version_arg(a, COMPARE_SIGNATURE, "a")?;
    let right = parse_version_arg(b, COMPARE_SIGNATURE, "b")?;
    Ok(ordering_to_i64(compare_versions(&left, &right)))
}

pub(crate) fn bump(version: Value, op: String) -> mlua::Result<LuaSemVer> {
    let version = parse_version_arg(version, BUMP_SIGNATURE, "v")?;
    let bumped = bump_version(version, &op, BUMP_SIGNATURE)?;
    Ok(LuaSemVer { version: bumped })
}

impl UserData for LuaSemVer {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("major", |_, this| u64_to_i64(this.version.major));
        fields.add_field_method_get("minor", |_, this| u64_to_i64(this.version.minor));
        fields.add_field_method_get("patch", |_, this| u64_to_i64(this.version.patch));
        fields.add_field_method_get("pre", |_, this| Ok(prerelease_to_option(&this.version.pre)));
        fields.add_field_method_get("build", |_, this| Ok(build_to_option(&this.version.build)));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("compare", |_, this, other: Value| {
            let other = parse_version_arg(other, "ptool.semver.Version:compare(other)", "other")?;
            Ok(ordering_to_i64(compare_versions(&this.version, &other)))
        });

        methods.add_method("bump", |_, this, op: String| {
            let bumped = bump_version(this.version.clone(), &op, "ptool.semver.Version:bump(op)")?;
            Ok(LuaSemVer { version: bumped })
        });

        methods.add_method("to_string", |_, this, ()| Ok(this.version.to_string()));

        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(this.version.to_string())
        });
        methods.add_meta_method(MetaMethod::Eq, |_, this, other: AnyUserData| {
            let other = borrow_semver_userdata(other, "ptool.semver Version __eq")?;
            Ok(compare_versions(&this.version, &other.version) == Ordering::Equal)
        });
        methods.add_meta_method(MetaMethod::Lt, |_, this, other: AnyUserData| {
            let other = borrow_semver_userdata(other, "ptool.semver Version __lt")?;
            Ok(compare_versions(&this.version, &other.version) == Ordering::Less)
        });
        methods.add_meta_method(MetaMethod::Le, |_, this, other: AnyUserData| {
            let other = borrow_semver_userdata(other, "ptool.semver Version __le")?;
            Ok(compare_versions(&this.version, &other.version) != Ordering::Greater)
        });
    }
}

fn parse_version_arg(value: Value, signature: &str, arg_name: &str) -> mlua::Result<Version> {
    match value {
        Value::String(value) => parse_version_text(value.to_str()?.as_ref(), signature, arg_name),
        Value::UserData(userdata) => {
            let semver = userdata.borrow::<LuaSemVer>().map_err(|_| {
                mlua::Error::runtime(format!(
                    "{signature} `{arg_name}` must be a version string or ptool.semver.Version"
                ))
            })?;
            Ok(semver.version.clone())
        }
        _ => Err(mlua::Error::runtime(format!(
            "{signature} `{arg_name}` must be a version string or ptool.semver.Version"
        ))),
    }
}

fn parse_version_from_string_arg(
    value: Value,
    signature: &str,
    arg_name: &str,
) -> mlua::Result<Version> {
    match value {
        Value::String(value) => parse_version_text(value.to_str()?.as_ref(), signature, arg_name),
        _ => Err(mlua::Error::runtime(format!(
            "{signature} `{arg_name}` must be a string"
        ))),
    }
}

fn parse_version_text(text: &str, signature: &str, arg_name: &str) -> mlua::Result<Version> {
    parse_semver(text).map_err(|err| {
        mlua::Error::runtime(format!("{signature} invalid `{arg_name}` `{text}`: {err}"))
    })
}

fn parse_semver(input: &str) -> Result<Version, semver::Error> {
    let trimmed = input.trim();
    let normalized = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    Version::parse(normalized)
}

fn compare_versions(a: &Version, b: &Version) -> Ordering {
    a.cmp(b)
}

fn ordering_to_i64(ordering: Ordering) -> i64 {
    match ordering {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn bump_version(version: Version, op: &str, signature: &str) -> mlua::Result<Version> {
    match op {
        "major" => bump_major(version),
        "minor" => bump_minor(version),
        "patch" => bump_patch(version),
        "release" => bump_release(version),
        "alpha" => bump_prerelease(version, PreChannel::Alpha, signature),
        "beta" => bump_prerelease(version, PreChannel::Beta, signature),
        "rc" => bump_prerelease(version, PreChannel::Rc, signature),
        _ => Err(mlua::Error::runtime(format!(
            "{signature} `op` must be one of `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`"
        ))),
    }
}

fn bump_major(mut version: Version) -> mlua::Result<Version> {
    version.major = version
        .major
        .checked_add(1)
        .ok_or_else(|| mlua::Error::runtime("ptool.semver.bump major overflow"))?;
    version.minor = 0;
    version.patch = 0;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
    Ok(version)
}

fn bump_minor(mut version: Version) -> mlua::Result<Version> {
    version.minor = version
        .minor
        .checked_add(1)
        .ok_or_else(|| mlua::Error::runtime("ptool.semver.bump minor overflow"))?;
    version.patch = 0;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
    Ok(version)
}

fn bump_patch(mut version: Version) -> mlua::Result<Version> {
    version.patch = version
        .patch
        .checked_add(1)
        .ok_or_else(|| mlua::Error::runtime("ptool.semver.bump patch overflow"))?;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
    Ok(version)
}

fn bump_release(mut version: Version) -> mlua::Result<Version> {
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
    Ok(version)
}

fn bump_prerelease(version: Version, target: PreChannel, signature: &str) -> mlua::Result<Version> {
    let mut version = version;
    version.build = BuildMetadata::EMPTY;

    let pre = version.pre.as_str();
    if pre.is_empty() {
        version.patch = version
            .patch
            .checked_add(1)
            .ok_or_else(|| mlua::Error::runtime("ptool.semver.bump patch overflow"))?;
        version.pre = parse_prerelease(
            &format!("{}.1", target.as_str()),
            signature,
            "internal prerelease value",
        )?;
        return Ok(version);
    }

    let (current_channel, current_number) = parse_channel_and_number(pre, signature)?;
    if current_channel.rank() > target.rank() {
        return Err(mlua::Error::runtime(format!(
            "{signature} does not allow prerelease downgrade from `{}` to `{}`",
            current_channel.as_str(),
            target.as_str()
        )));
    }

    let next_number = if current_channel.rank() < target.rank() {
        1
    } else {
        current_number.unwrap_or(0).saturating_add(1)
    };

    version.pre = parse_prerelease(
        &format!("{}.{}", target.as_str(), next_number),
        signature,
        "internal prerelease value",
    )?;
    Ok(version)
}

fn parse_prerelease(value: &str, signature: &str, field_name: &str) -> mlua::Result<Prerelease> {
    Prerelease::new(value).map_err(|err| {
        mlua::Error::runtime(format!(
            "{signature} invalid `{field_name}` `{value}`: {err}"
        ))
    })
}

fn parse_channel_and_number(pre: &str, signature: &str) -> mlua::Result<(PreChannel, Option<u64>)> {
    let parts: Vec<&str> = pre.split('.').collect();
    let channel =
        PreChannel::parse(parts.first().copied().unwrap_or_default()).ok_or_else(|| {
            mlua::Error::runtime(format!(
                "{signature} only supports prerelease channels `alpha`, `beta`, `rc`, got `{pre}`"
            ))
        })?;

    if parts.len() == 1 {
        return Ok((channel, None));
    }
    if parts.len() > 2 {
        return Err(mlua::Error::runtime(format!(
            "{signature} prerelease `{pre}` is unsupported; expected `<channel>` or `<channel>.<number>`"
        )));
    }

    let number = parts[1].parse::<u64>().map_err(|_| {
        mlua::Error::runtime(format!(
            "{signature} prerelease `{pre}` is unsupported; expected numeric suffix"
        ))
    })?;
    Ok((channel, Some(number)))
}

fn prerelease_to_option(pre: &Prerelease) -> Option<String> {
    if pre.as_str().is_empty() {
        None
    } else {
        Some(pre.to_string())
    }
}

fn build_to_option(build: &BuildMetadata) -> Option<String> {
    if build.as_str().is_empty() {
        None
    } else {
        Some(build.to_string())
    }
}

fn borrow_semver_userdata(userdata: AnyUserData, context: &str) -> mlua::Result<LuaSemVer> {
    let semver = userdata.borrow::<LuaSemVer>().map_err(|_| {
        mlua::Error::runtime(format!(
            "{context} expects `ptool.semver.parse(...)` return value"
        ))
    })?;
    Ok(semver.clone())
}

fn u64_to_i64(value: u64) -> mlua::Result<i64> {
    i64::try_from(value)
        .map_err(|_| mlua::Error::runtime("ptool.semver version component is too large"))
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
