use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods, Value};
use ptool_engine::{
    ParsedScriptArgs, ScriptArgDefault, ScriptArgKind, ScriptArgSpec, ScriptArgValue,
    ScriptArgValues, ScriptArgsSchema,
};
use std::collections::HashSet;
use std::process;

const ARG_FACTORY_SIGNATURE: &str = "ptool.args.arg(id, kind, options)";

#[derive(Clone, Debug)]
pub(crate) struct ScriptArgBuilder {
    spec: ScriptArgSpec,
}

impl UserData for ScriptArgBuilder {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("long", |_, this, long: Option<String>| {
            let mut next = this.spec.clone();
            next.long = if matches!(next.kind, ScriptArgKind::Positional) {
                long
            } else {
                Some(long.unwrap_or_else(|| next.id.clone()))
            };
            validate_script_arg_spec_base(&next, "ptool.args.arg(...):long(value)")?;
            this.spec = next;
            Ok(this.clone())
        });

        methods.add_method_mut("short", |_, this, short: Option<String>| {
            let mut next = this.spec.clone();
            next.short = parse_short_name(short.as_deref(), "ptool.args.arg(...):short(value)")?;
            validate_script_arg_spec_base(&next, "ptool.args.arg(...):short(value)")?;
            this.spec = next;
            Ok(this.clone())
        });

        methods.add_method_mut("help", |_, this, help: Option<String>| {
            let mut next = this.spec.clone();
            next.help = help;
            this.spec = next;
            Ok(this.clone())
        });

        methods.add_method_mut("required", |_, this, required: Option<bool>| {
            let mut next = this.spec.clone();
            next.required = required.unwrap_or(true);
            this.spec = next;
            Ok(this.clone())
        });

        methods.add_method_mut("multiple", |_, this, multiple: Option<bool>| {
            let mut next = this.spec.clone();
            next.multiple = multiple.unwrap_or(true);
            validate_script_arg_spec_base(&next, "ptool.args.arg(...):multiple(value)")?;
            this.spec = next;
            Ok(this.clone())
        });

        methods.add_method_mut("default", |_, this, value: Value| {
            let mut next = this.spec.clone();
            next.default = if matches!(value, Value::Nil) {
                None
            } else {
                Some(parse_default_value(
                    value,
                    next.kind,
                    "ptool.args.arg(...):default(value)",
                )?)
            };
            validate_script_arg_spec_base(&next, "ptool.args.arg(...):default(value)")?;
            this.spec = next;
            Ok(this.clone())
        });
    }
}

pub(crate) fn create_script_arg_builder(
    id: String,
    kind_raw: String,
    options: Option<Table>,
) -> mlua::Result<ScriptArgBuilder> {
    let kind = parse_arg_kind(&kind_raw, ARG_FACTORY_SIGNATURE)?;
    let mut spec = ScriptArgSpec {
        id: id.clone(),
        kind,
        long: if matches!(kind, ScriptArgKind::Positional) {
            None
        } else {
            Some(id)
        },
        short: None,
        help: None,
        required: false,
        multiple: false,
        default: None,
    };

    if let Some(options) = options {
        apply_builder_options(&mut spec, options)?;
    }

    validate_script_arg_spec_base(&spec, ARG_FACTORY_SIGNATURE)?;
    Ok(ScriptArgBuilder { spec })
}

pub(crate) fn parse_script_args(
    lua: &Lua,
    schema: Table,
    default_name: &str,
    script_args: &[String],
) -> mlua::Result<Table> {
    let schema = parse_script_args_schema(schema, default_name)?;
    let parsed = match ptool_engine::parse_script_args(&schema, script_args) {
        Ok(parsed) => parsed,
        Err(err) => {
            let _ = err.print();
            process::exit(err.exit_code());
        }
    };
    parsed_script_args_to_lua(lua, &parsed)
}

fn parse_script_args_schema(schema: Table, default_name: &str) -> mlua::Result<ScriptArgsSchema> {
    let name = schema
        .get::<Option<String>>("name")?
        .unwrap_or_else(|| default_name.to_string());
    parse_script_args_schema_with_context(schema, name, "ptool.args.parse(schema)", true)
}

fn parse_script_args_schema_with_context(
    schema: Table,
    name: String,
    context: &str,
    is_root: bool,
) -> mlua::Result<ScriptArgsSchema> {
    let about: Option<String> = schema.get("about")?;
    let args = parse_script_arg_specs(
        schema.get::<Option<Table>>("args")?,
        &format!("{context}.args"),
    )?;
    let subcommands = parse_script_subcommands(
        schema.get::<Option<Table>>("subcommands")?,
        &format!("{context}.subcommands"),
    )?;
    let parsed = ScriptArgsSchema {
        name,
        about,
        args,
        subcommands,
    };
    ptool_engine::validate_script_args_schema(&parsed, context, is_root).map_err(engine_error)?;
    Ok(parsed)
}

fn parse_script_arg_specs(
    args_table: Option<Table>,
    context: &str,
) -> mlua::Result<Vec<ScriptArgSpec>> {
    let Some(args_table) = args_table else {
        return Ok(Vec::new());
    };

    let mut args = Vec::new();
    let mut seen_ids = HashSet::new();
    let args_count = args_table.raw_len();
    for (index, arg_value) in args_table.sequence_values::<Value>().enumerate() {
        let arg = parse_script_arg_spec(arg_value?, index + 1, args_count)?;
        if !seen_ids.insert(arg.id.clone()) {
            return Err(mlua::Error::runtime(format!(
                "{context} duplicate argument id `{}`",
                arg.id
            )));
        }
        args.push(arg);
    }
    Ok(args)
}

fn parse_script_subcommands(
    subcommands_table: Option<Table>,
    context: &str,
) -> mlua::Result<Vec<ScriptArgsSchema>> {
    let Some(subcommands_table) = subcommands_table else {
        return Ok(Vec::new());
    };

    let mut entries = Vec::new();
    for pair in subcommands_table.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let name = match key {
            Value::String(name) => name.to_str()?.to_string(),
            _ => {
                return Err(mlua::Error::runtime(format!(
                    "{context} keys must be strings"
                )));
            }
        };
        let Value::Table(schema) = value else {
            return Err(mlua::Error::runtime(format!(
                "{context}.{name} must be a table"
            )));
        };
        entries.push((name, schema));
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut subcommands = Vec::with_capacity(entries.len());
    for (name, schema) in entries {
        let subcommand = parse_script_args_schema_with_context(
            schema,
            name.clone(),
            &format!("{context}.{name}"),
            false,
        )?;
        subcommands.push(subcommand);
    }
    Ok(subcommands)
}

fn parse_script_arg_spec(value: Value, index: usize, total: usize) -> mlua::Result<ScriptArgSpec> {
    match value {
        Value::Table(arg_table) => parse_script_arg_spec_from_table(arg_table, index, total),
        Value::UserData(userdata) => parse_script_arg_spec_from_builder(userdata, index, total),
        _ => Err(mlua::Error::runtime(format!(
            "ptool.args.parse schema.args[{index}] must be a table or ptool.args.arg(...) result"
        ))),
    }
}

fn parse_script_arg_spec_from_table(
    arg_table: Table,
    index: usize,
    total: usize,
) -> mlua::Result<ScriptArgSpec> {
    let context = format!("ptool.args.parse schema.args[{index}]");
    let Some(id) = arg_table.get::<Option<String>>("id")? else {
        return Err(mlua::Error::runtime(format!("{context} requires `id`")));
    };
    let Some(kind_raw) = arg_table.get::<Option<String>>("kind")? else {
        return Err(mlua::Error::runtime(format!("{context} requires `kind`")));
    };
    let kind = parse_arg_kind(&kind_raw, &context)?;

    let long_raw: Option<String> = arg_table.get("long")?;
    let short_raw: Option<String> = arg_table.get("short")?;
    let short = parse_short_name(short_raw.as_deref(), &context)?;
    let help: Option<String> = arg_table.get("help")?;
    let required = arg_table.get::<Option<bool>>("required")?.unwrap_or(false);
    let multiple = arg_table.get::<Option<bool>>("multiple")?.unwrap_or(false);
    let default_value = parse_default_value_from_table(&arg_table, kind, &context)?;

    let long = if matches!(kind, ScriptArgKind::Positional) {
        long_raw
    } else {
        Some(long_raw.unwrap_or_else(|| id.clone()))
    };

    let spec = ScriptArgSpec {
        id,
        kind,
        long,
        short,
        help,
        required,
        multiple,
        default: default_value,
    };
    ptool_engine::validate_script_arg_spec(&spec, &context, index, total).map_err(engine_error)?;
    Ok(spec)
}

fn parse_script_arg_spec_from_builder(
    userdata: AnyUserData,
    index: usize,
    total: usize,
) -> mlua::Result<ScriptArgSpec> {
    let context = format!("ptool.args.parse schema.args[{index}]");
    let builder = userdata.borrow::<ScriptArgBuilder>().map_err(|_| {
        mlua::Error::runtime(format!(
            "{context} must be a table or ptool.args.arg(...) result"
        ))
    })?;
    let spec = builder.spec.clone();
    ptool_engine::validate_script_arg_spec(&spec, &context, index, total).map_err(engine_error)?;
    Ok(spec)
}

fn parse_arg_kind(kind_raw: &str, context: &str) -> mlua::Result<ScriptArgKind> {
    match kind_raw {
        "flag" => Ok(ScriptArgKind::Flag),
        "string" => Ok(ScriptArgKind::String),
        "int" => Ok(ScriptArgKind::Int),
        "positional" => Ok(ScriptArgKind::Positional),
        _ => Err(mlua::Error::runtime(format!(
            "{context} has unsupported kind `{kind_raw}`"
        ))),
    }
}

fn parse_short_name(short: Option<&str>, context: &str) -> mlua::Result<Option<char>> {
    let Some(value) = short else {
        return Ok(None);
    };
    let mut chars = value.chars();
    let Some(ch) = chars.next() else {
        return Err(mlua::Error::runtime(format!(
            "{context} `short` cannot be empty"
        )));
    };
    if chars.next().is_some() {
        return Err(mlua::Error::runtime(format!(
            "{context} `short` must be exactly one character"
        )));
    }
    Ok(Some(ch))
}

fn parse_default_value_from_table(
    arg_table: &Table,
    kind: ScriptArgKind,
    context: &str,
) -> mlua::Result<Option<ScriptArgDefault>> {
    let Some(value) = arg_table.get::<Option<Value>>("default")? else {
        return Ok(None);
    };
    Ok(Some(parse_default_value(value, kind, context)?))
}

fn parse_default_value(
    value: Value,
    kind: ScriptArgKind,
    context: &str,
) -> mlua::Result<ScriptArgDefault> {
    match (kind, value) {
        (ScriptArgKind::String, Value::String(v)) => {
            Ok(ScriptArgDefault::String(v.to_str()?.to_string()))
        }
        (ScriptArgKind::Int, Value::Integer(v)) => Ok(ScriptArgDefault::Int(v)),
        (ScriptArgKind::Int, Value::Number(v)) if v.fract() == 0.0 => {
            Ok(ScriptArgDefault::Int(v as i64))
        }
        (ScriptArgKind::String, _) => Err(mlua::Error::runtime(format!(
            "{context} string default must be a string"
        ))),
        (ScriptArgKind::Int, _) => Err(mlua::Error::runtime(format!(
            "{context} int default must be an integer"
        ))),
        _ => Err(mlua::Error::runtime(format!(
            "{context} only string/int support default"
        ))),
    }
}

fn apply_builder_options(spec: &mut ScriptArgSpec, options: Table) -> mlua::Result<()> {
    if let Some(long) = options.get::<Option<String>>("long")? {
        spec.long = Some(long);
    }

    if let Some(short) = options.get::<Option<String>>("short")? {
        spec.short = parse_short_name(Some(short.as_str()), ARG_FACTORY_SIGNATURE)?;
    }

    if let Some(help) = options.get::<Option<String>>("help")? {
        spec.help = Some(help);
    }

    if let Some(required) = options.get::<Option<bool>>("required")? {
        spec.required = required;
    }

    if let Some(multiple) = options.get::<Option<bool>>("multiple")? {
        spec.multiple = multiple;
    }

    if let Some(value) = options.get::<Option<Value>>("default")? {
        spec.default = Some(parse_default_value(
            value,
            spec.kind,
            ARG_FACTORY_SIGNATURE,
        )?);
    }

    Ok(())
}

fn parsed_script_args_to_lua(lua: &Lua, parsed: &ParsedScriptArgs) -> mlua::Result<Table> {
    let values = script_arg_values_to_lua(lua, &parsed.values)?;
    if parsed.command_path.is_empty() {
        return Ok(values);
    }

    values.set(
        "command_path",
        strings_to_lua_table(lua, &parsed.command_path)?,
    )?;
    values.set("args", script_arg_values_to_lua(lua, &parsed.args)?)?;
    Ok(values)
}

fn script_arg_values_to_lua(lua: &Lua, values: &ScriptArgValues) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (key, value) in values {
        match value {
            ScriptArgValue::Flag(value) => table.set(key.as_str(), *value)?,
            ScriptArgValue::String(value) => table.set(key.as_str(), value.clone())?,
            ScriptArgValue::Strings(values) => {
                table.set(key.as_str(), strings_to_lua_table(lua, values)?)?;
            }
            ScriptArgValue::Int(value) => table.set(key.as_str(), *value)?,
        }
    }
    Ok(table)
}

fn strings_to_lua_table(lua: &Lua, values: &[String]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.iter().enumerate() {
        table.set(index + 1, value.clone())?;
    }
    Ok(table)
}

fn validate_script_arg_spec_base(spec: &ScriptArgSpec, context: &str) -> mlua::Result<()> {
    ptool_engine::validate_script_arg_spec_base(spec, context).map_err(engine_error)
}

fn engine_error(err: ptool_engine::Error) -> mlua::Error {
    mlua::Error::runtime(err.to_string())
}
