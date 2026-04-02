use mlua::{AnyUserData, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use ptool_engine::{
    Error as EngineError, ErrorKind as EngineErrorKind, PtoolEngine, SemverBuildMetadata,
    SemverPrerelease, SemverVersion,
};
use std::cmp::Ordering;

const PARSE_SIGNATURE: &str = "ptool.semver.parse(version)";
const COMPARE_SIGNATURE: &str = "ptool.semver.compare(a, b)";
const BUMP_SIGNATURE: &str = "ptool.semver.bump(v, op[, channel])";

#[derive(Clone)]
pub(crate) struct LuaSemVer {
    engine: PtoolEngine,
    version: SemverVersion,
}

pub(crate) fn parse(engine: &PtoolEngine, version: Value) -> mlua::Result<LuaSemVer> {
    let version = parse_version_from_string_arg(engine, version, PARSE_SIGNATURE, "version")?;
    Ok(LuaSemVer {
        engine: engine.clone(),
        version,
    })
}

pub(crate) fn is_valid(engine: &PtoolEngine, version: Value) -> bool {
    match version {
        Value::String(value) => match value.to_str() {
            Ok(text) => engine.semver_is_valid(text.as_ref()),
            Err(_) => false,
        },
        _ => false,
    }
}

pub(crate) fn compare(engine: &PtoolEngine, a: Value, b: Value) -> mlua::Result<i64> {
    let left = parse_version_arg(engine, a, COMPARE_SIGNATURE, "a")?;
    let right = parse_version_arg(engine, b, COMPARE_SIGNATURE, "b")?;
    Ok(ordering_to_i64(engine.semver_compare(&left, &right)))
}

pub(crate) fn bump(
    engine: &PtoolEngine,
    version: Value,
    op: String,
    channel: Option<String>,
) -> mlua::Result<LuaSemVer> {
    let version = parse_version_arg(engine, version, BUMP_SIGNATURE, "v")?;
    let bumped = bump_version(engine, version, &op, channel.as_deref(), BUMP_SIGNATURE)?;
    Ok(LuaSemVer {
        engine: engine.clone(),
        version: bumped,
    })
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
            let other = parse_version_arg(
                &this.engine,
                other,
                "ptool.semver.Version:compare(other)",
                "other",
            )?;
            Ok(ordering_to_i64(
                this.engine.semver_compare(&this.version, &other),
            ))
        });

        methods.add_method(
            "bump",
            |_, this, (op, channel): (String, Option<String>)| {
                let bumped = bump_version(
                    &this.engine,
                    this.version.clone(),
                    &op,
                    channel.as_deref(),
                    "ptool.semver.Version:bump(op[, channel])",
                )?;
                Ok(LuaSemVer {
                    engine: this.engine.clone(),
                    version: bumped,
                })
            },
        );

        methods.add_method("to_string", |_, this, ()| Ok(this.version.to_string()));

        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(this.version.to_string())
        });
        methods.add_meta_method(MetaMethod::Eq, |_, this, other: AnyUserData| {
            let other = borrow_semver_userdata(other, "ptool.semver Version __eq")?;
            Ok(this.engine.semver_compare(&this.version, &other.version) == Ordering::Equal)
        });
        methods.add_meta_method(MetaMethod::Lt, |_, this, other: AnyUserData| {
            let other = borrow_semver_userdata(other, "ptool.semver Version __lt")?;
            Ok(this.engine.semver_compare(&this.version, &other.version) == Ordering::Less)
        });
        methods.add_meta_method(MetaMethod::Le, |_, this, other: AnyUserData| {
            let other = borrow_semver_userdata(other, "ptool.semver Version __le")?;
            Ok(this.engine.semver_compare(&this.version, &other.version) != Ordering::Greater)
        });
    }
}

fn parse_version_arg(
    engine: &PtoolEngine,
    value: Value,
    signature: &str,
    arg_name: &str,
) -> mlua::Result<SemverVersion> {
    match value {
        Value::String(value) => {
            parse_version_text(engine, value.to_str()?.as_ref(), signature, arg_name)
        }
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
    engine: &PtoolEngine,
    value: Value,
    signature: &str,
    arg_name: &str,
) -> mlua::Result<SemverVersion> {
    match value {
        Value::String(value) => {
            parse_version_text(engine, value.to_str()?.as_ref(), signature, arg_name)
        }
        _ => Err(mlua::Error::runtime(format!(
            "{signature} `{arg_name}` must be a string"
        ))),
    }
}

fn parse_version_text(
    engine: &PtoolEngine,
    text: &str,
    signature: &str,
    arg_name: &str,
) -> mlua::Result<SemverVersion> {
    engine.semver_parse(text).map_err(|err| {
        mlua::Error::runtime(format!(
            "{signature} invalid `{arg_name}` `{text}`: {}",
            err.msg
        ))
    })
}

fn ordering_to_i64(ordering: Ordering) -> i64 {
    match ordering {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn bump_version(
    engine: &PtoolEngine,
    version: SemverVersion,
    op: &str,
    channel: Option<&str>,
    signature: &str,
) -> mlua::Result<SemverVersion> {
    engine
        .semver_bump(version, op, channel)
        .map_err(|err| to_bump_lua_error(signature, err))
}

fn to_bump_lua_error(signature: &str, err: EngineError) -> mlua::Error {
    match err.kind {
        EngineErrorKind::SemverOverflow => {
            mlua::Error::runtime(format!("ptool.semver.bump {}", err.msg))
        }
        _ => mlua::Error::runtime(format!("{signature} {}", err.msg)),
    }
}

fn prerelease_to_option(pre: &SemverPrerelease) -> Option<String> {
    if pre.as_str().is_empty() {
        None
    } else {
        Some(pre.to_string())
    }
}

fn build_to_option(build: &SemverBuildMetadata) -> Option<String> {
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
