use mlua::{AnyUserData, MetaMethod, Table, UserData, UserDataFields, UserDataMethods, Value};
use ptool_engine::{
    DateTimeFromUnixOptions, DateTimeParseOptions, DateTimeUnixUnit, DateTimeValue, PtoolEngine,
};
use std::cmp::Ordering;

const NOW_SIGNATURE: &str = "ptool.datetime.now([tz])";
const PARSE_SIGNATURE: &str = "ptool.datetime.parse(input[, options])";
const FROM_UNIX_SIGNATURE: &str = "ptool.datetime.from_unix(value[, options])";
const COMPARE_SIGNATURE: &str = "ptool.datetime.compare(a, b)";
const IS_VALID_SIGNATURE: &str = "ptool.datetime.is_valid(input[, options])";

#[derive(Clone)]
pub(crate) struct LuaDateTime {
    engine: PtoolEngine,
    value: DateTimeValue,
}

pub(crate) fn now(engine: &PtoolEngine, timezone: Option<String>) -> mlua::Result<LuaDateTime> {
    let value = engine
        .datetime_now(timezone.as_deref())
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, NOW_SIGNATURE))?;
    Ok(LuaDateTime {
        engine: engine.clone(),
        value,
    })
}

pub(crate) fn parse(
    engine: &PtoolEngine,
    input: String,
    options: Option<Table>,
) -> mlua::Result<LuaDateTime> {
    let options = parse_parse_options(options, PARSE_SIGNATURE)?;
    let value = engine
        .datetime_parse(&input, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, PARSE_SIGNATURE))?;
    Ok(LuaDateTime {
        engine: engine.clone(),
        value,
    })
}

pub(crate) fn from_unix(
    engine: &PtoolEngine,
    value: i64,
    options: Option<Table>,
) -> mlua::Result<LuaDateTime> {
    let options = parse_from_unix_options(options, FROM_UNIX_SIGNATURE)?;
    let value = engine
        .datetime_from_unix(value, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, FROM_UNIX_SIGNATURE))?;
    Ok(LuaDateTime {
        engine: engine.clone(),
        value,
    })
}

pub(crate) fn compare(engine: &PtoolEngine, a: Value, b: Value) -> mlua::Result<i64> {
    let left = parse_datetime_arg(engine, a, COMPARE_SIGNATURE, "a")?;
    let right = parse_datetime_arg(engine, b, COMPARE_SIGNATURE, "b")?;
    Ok(ordering_to_i64(engine.datetime_compare(&left, &right)))
}

pub(crate) fn is_valid(
    engine: &PtoolEngine,
    input: String,
    options: Option<Table>,
) -> mlua::Result<bool> {
    let options = parse_parse_options(options, IS_VALID_SIGNATURE)?;
    Ok(engine.datetime_parse(&input, options).is_ok())
}

impl UserData for LuaDateTime {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("year", |_, this| Ok(i64::from(this.value.year())));
        fields.add_field_method_get("month", |_, this| Ok(i64::from(this.value.month())));
        fields.add_field_method_get("day", |_, this| Ok(i64::from(this.value.day())));
        fields.add_field_method_get("hour", |_, this| Ok(i64::from(this.value.hour())));
        fields.add_field_method_get("minute", |_, this| Ok(i64::from(this.value.minute())));
        fields.add_field_method_get("second", |_, this| Ok(i64::from(this.value.second())));
        fields.add_field_method_get("nanosecond", |_, this| {
            Ok(i64::from(this.value.nanosecond()))
        });
        fields.add_field_method_get("offset", |_, this| Ok(this.value.offset()));
        fields.add_field_method_get("timezone", |_, this| Ok(this.value.timezone()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("format", |_, this, format: String| {
            this.value.format(&format).map_err(|err| {
                crate::lua_error::lua_error_from_engine(err, "ptool.datetime.DateTime:format")
            })
        });

        methods.add_method("to_string", |_, this, ()| Ok(this.value.to_string()));

        methods.add_method("unix", |_, this, unit: Option<String>| {
            let unit = parse_unix_unit_option(
                unit.as_deref(),
                "ptool.datetime.DateTime:unix([unit])",
                "unit",
            )?;
            i128_to_i64(
                this.value.unix(unit),
                "ptool.datetime.DateTime:unix([unit])",
            )
        });

        methods.add_method("in_tz", |_, this, timezone: String| {
            let value = this.value.in_tz(&timezone).map_err(|err| {
                crate::lua_error::lua_error_from_engine(err, "ptool.datetime.DateTime:in_tz")
            })?;
            Ok(LuaDateTime {
                engine: this.engine.clone(),
                value,
            })
        });

        methods.add_method("compare", |_, this, other: Value| {
            let other = parse_datetime_arg(
                &this.engine,
                other,
                "ptool.datetime.DateTime:compare(other)",
                "other",
            )?;
            Ok(ordering_to_i64(
                this.engine.datetime_compare(&this.value, &other),
            ))
        });

        methods.add_meta_method(MetaMethod::ToString, |_, this, ()| {
            Ok(this.value.to_string())
        });
        methods.add_meta_method(MetaMethod::Eq, |_, this, other: AnyUserData| {
            let other = borrow_datetime_userdata(other, "ptool.datetime DateTime __eq")?;
            Ok(this.engine.datetime_compare(&this.value, &other.value) == Ordering::Equal)
        });
        methods.add_meta_method(MetaMethod::Lt, |_, this, other: AnyUserData| {
            let other = borrow_datetime_userdata(other, "ptool.datetime DateTime __lt")?;
            Ok(this.engine.datetime_compare(&this.value, &other.value) == Ordering::Less)
        });
        methods.add_meta_method(MetaMethod::Le, |_, this, other: AnyUserData| {
            let other = borrow_datetime_userdata(other, "ptool.datetime DateTime __le")?;
            Ok(this.engine.datetime_compare(&this.value, &other.value) != Ordering::Greater)
        });
    }
}

fn parse_datetime_arg(
    engine: &PtoolEngine,
    value: Value,
    signature: &str,
    arg_name: &str,
) -> mlua::Result<DateTimeValue> {
    match value {
        Value::String(value) => engine
            .datetime_parse(value.to_str()?.as_ref(), DateTimeParseOptions::default())
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, signature)),
        Value::UserData(userdata) => {
            let datetime = userdata.borrow::<LuaDateTime>().map_err(|_| {
                mlua::Error::runtime(format!(
                    "{signature} `{arg_name}` must be a datetime string or ptool.datetime.DateTime"
                ))
            })?;
            Ok(datetime.value.clone())
        }
        _ => Err(mlua::Error::runtime(format!(
            "{signature} `{arg_name}` must be a datetime string or ptool.datetime.DateTime"
        ))),
    }
}

fn parse_parse_options(
    options: Option<Table>,
    signature: &str,
) -> mlua::Result<DateTimeParseOptions> {
    let mut parsed = DateTimeParseOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = table_key_to_string(key, signature)?;
        match key.as_str() {
            "timezone" => {
                parsed.timezone = Some(string_option_value(value, signature, "timezone")?);
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_from_unix_options(
    options: Option<Table>,
    signature: &str,
) -> mlua::Result<DateTimeFromUnixOptions> {
    let mut parsed = DateTimeFromUnixOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = table_key_to_string(key, signature)?;
        match key.as_str() {
            "unit" => {
                let unit = string_option_value(value, signature, "unit")?;
                parsed.unit = parse_unix_unit_option(Some(&unit), signature, "unit")?;
            }
            "timezone" => {
                parsed.timezone = Some(string_option_value(value, signature, "timezone")?);
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    signature,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(parsed)
}

fn parse_unix_unit_option(
    unit: Option<&str>,
    signature: &str,
    field: &str,
) -> mlua::Result<DateTimeUnixUnit> {
    match unit.unwrap_or("s") {
        "s" => Ok(DateTimeUnixUnit::Second),
        "ms" => Ok(DateTimeUnixUnit::Millisecond),
        "ns" => Ok(DateTimeUnixUnit::Nanosecond),
        other => Err(crate::lua_error::invalid_argument(
            signature,
            format!("`{field}` must be one of `s`, `ms`, or `ns`; got `{other}`"),
        )),
    }
}

fn string_option_value(value: Value, signature: &str, field: &str) -> mlua::Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            signature,
            format!("`{field}` must be a string"),
        )),
    }
}

fn table_key_to_string(key: Value, signature: &str) -> mlua::Result<String> {
    match key {
        Value::String(key) => Ok(key.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            signature,
            "option keys must be strings",
        )),
    }
}

fn borrow_datetime_userdata(userdata: AnyUserData, context: &str) -> mlua::Result<LuaDateTime> {
    let datetime = userdata.borrow::<LuaDateTime>().map_err(|_| {
        mlua::Error::runtime(format!(
            "{context} expects `ptool.datetime.parse(...)` return value"
        ))
    })?;
    Ok(datetime.clone())
}

fn ordering_to_i64(ordering: Ordering) -> i64 {
    match ordering {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn i128_to_i64(value: i128, signature: &str) -> mlua::Result<i64> {
    i64::try_from(value).map_err(|_| {
        mlua::Error::runtime(format!(
            "{signature} result is too large to fit in Lua integer"
        ))
    })
}
