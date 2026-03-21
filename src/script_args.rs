use clap::{Arg, ArgAction, Command, value_parser};
use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods, Value};
use std::collections::HashSet;
use std::process;

const ARG_FACTORY_SIGNATURE: &str = "ptool.args.arg(id, kind, options)";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptArgKind {
    Flag,
    String,
    Int,
    Positional,
}

#[derive(Clone, Debug)]
enum ScriptArgDefault {
    String(String),
    Int(i64),
}

#[derive(Clone, Debug)]
struct ScriptArgSpec {
    id: String,
    kind: ScriptArgKind,
    long: Option<String>,
    short: Option<char>,
    help: Option<String>,
    required: bool,
    multiple: bool,
    default: Option<ScriptArgDefault>,
}

#[derive(Debug)]
struct ScriptArgsSchema {
    name: String,
    about: Option<String>,
    args: Vec<ScriptArgSpec>,
}

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
    let matches = match try_parse_script_matches(&schema, script_args) {
        Ok(matches) => matches,
        Err(err) => {
            let _ = err.print();
            process::exit(err.exit_code());
        }
    };
    script_arg_matches_to_lua(lua, &schema, &matches)
}

fn parse_script_args_schema(schema: Table, default_name: &str) -> mlua::Result<ScriptArgsSchema> {
    let name = schema
        .get::<Option<String>>("name")?
        .unwrap_or_else(|| default_name.to_string());
    let about: Option<String> = schema.get("about")?;
    let Some(args_table) = schema.get::<Option<Table>>("args")? else {
        return Err(mlua::Error::runtime(
            "ptool.args.parse(schema) requires schema.args",
        ));
    };

    let mut args = Vec::new();
    let mut seen_ids = HashSet::new();
    let args_count = args_table.raw_len();
    for (index, arg_value) in args_table.sequence_values::<Value>().enumerate() {
        let arg = parse_script_arg_spec(arg_value?, index + 1, args_count)?;
        if !seen_ids.insert(arg.id.clone()) {
            return Err(mlua::Error::runtime(format!(
                "ptool.args.parse duplicate argument id `{}`",
                arg.id
            )));
        }
        args.push(arg);
    }

    Ok(ScriptArgsSchema { name, about, args })
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
    validate_script_arg_spec(&spec, &context, index, total)?;
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
    validate_script_arg_spec(&spec, &context, index, total)?;
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

fn validate_script_arg_spec(
    spec: &ScriptArgSpec,
    context: &str,
    index: usize,
    total: usize,
) -> mlua::Result<()> {
    validate_script_arg_spec_base(spec, context)?;
    if matches!(spec.kind, ScriptArgKind::Positional) && spec.multiple && index != total {
        return Err(mlua::Error::runtime(format!(
            "{context} positional argument with multiple=true must be the last entry"
        )));
    }
    Ok(())
}

fn validate_script_arg_spec_base(spec: &ScriptArgSpec, context: &str) -> mlua::Result<()> {
    if matches!(spec.kind, ScriptArgKind::Positional) {
        if spec.long.is_some() {
            return Err(mlua::Error::runtime(format!(
                "{context} positional argument cannot set `long`"
            )));
        }
        if spec.short.is_some() {
            return Err(mlua::Error::runtime(format!(
                "{context} positional argument cannot set `short`"
            )));
        }
        if spec.default.is_some() {
            return Err(mlua::Error::runtime(format!(
                "{context} positional argument cannot set `default`"
            )));
        }
    }

    if spec.multiple && !matches!(spec.kind, ScriptArgKind::String | ScriptArgKind::Positional) {
        return Err(mlua::Error::runtime(format!(
            "{context} only string/positional support multiple=true"
        )));
    }

    if spec.default.is_some() && spec.multiple {
        return Err(mlua::Error::runtime(format!(
            "{context} does not support default with multiple=true"
        )));
    }

    Ok(())
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

fn try_parse_script_matches(
    schema: &ScriptArgsSchema,
    script_args: &[String],
) -> Result<clap::ArgMatches, clap::Error> {
    let mut command = build_script_arg_command(schema);
    let mut argv = Vec::with_capacity(script_args.len() + 1);
    argv.push(schema.name.clone());
    argv.extend(script_args.iter().cloned());
    command.try_get_matches_from_mut(argv)
}

fn build_script_arg_command(schema: &ScriptArgsSchema) -> Command {
    let mut command = Command::new(schema.name.clone());
    if let Some(about) = &schema.about {
        command = command.about(about.clone());
    }

    let mut positional_index = 1;
    for arg in &schema.args {
        let mut clap_arg = Arg::new(arg.id.clone());
        if let Some(help) = &arg.help {
            clap_arg = clap_arg.help(help.clone());
        }

        match arg.kind {
            ScriptArgKind::Flag => {
                clap_arg = clap_arg.action(ArgAction::SetTrue).required(arg.required);
            }
            ScriptArgKind::String => {
                clap_arg = clap_arg
                    .value_parser(value_parser!(String))
                    .required(arg.required);
                if arg.multiple {
                    clap_arg = clap_arg.action(ArgAction::Append).num_args(1..);
                } else {
                    clap_arg = clap_arg.action(ArgAction::Set);
                }
                if let Some(ScriptArgDefault::String(default)) = &arg.default {
                    clap_arg = clap_arg.default_value(default.clone());
                }
            }
            ScriptArgKind::Int => {
                clap_arg = clap_arg
                    .action(ArgAction::Set)
                    .value_parser(value_parser!(i64))
                    .required(arg.required);
                if let Some(ScriptArgDefault::Int(default)) = arg.default.as_ref() {
                    clap_arg = clap_arg.default_value(default.to_string());
                }
            }
            ScriptArgKind::Positional => {
                clap_arg = clap_arg.index(positional_index).required(arg.required);
                positional_index += 1;
                if arg.multiple {
                    clap_arg = clap_arg.action(ArgAction::Append).num_args(0..);
                } else {
                    clap_arg = clap_arg.action(ArgAction::Set);
                }
            }
        }

        if let Some(long) = &arg.long {
            clap_arg = clap_arg.long(long.clone());
        }
        if let Some(short) = arg.short {
            clap_arg = clap_arg.short(short);
        }
        command = command.arg(clap_arg);
    }

    command
}

fn script_arg_matches_to_lua(
    lua: &Lua,
    schema: &ScriptArgsSchema,
    matches: &clap::ArgMatches,
) -> mlua::Result<Table> {
    let values = lua.create_table()?;
    for arg in &schema.args {
        match arg.kind {
            ScriptArgKind::Flag => {
                values.set(arg.id.as_str(), matches.get_flag(&arg.id))?;
            }
            ScriptArgKind::String => {
                if arg.multiple {
                    let list = matches
                        .get_many::<String>(&arg.id)
                        .map(|items| items.cloned().collect::<Vec<_>>())
                        .unwrap_or_default();
                    values.set(arg.id.as_str(), strings_to_lua_table(lua, &list)?)?;
                } else if let Some(value) = matches.get_one::<String>(&arg.id) {
                    values.set(arg.id.as_str(), value.clone())?;
                }
            }
            ScriptArgKind::Int => {
                if let Some(value) = matches.get_one::<i64>(&arg.id) {
                    values.set(arg.id.as_str(), *value)?;
                }
            }
            ScriptArgKind::Positional => {
                if arg.multiple {
                    let list = matches
                        .get_many::<String>(&arg.id)
                        .map(|items| items.cloned().collect::<Vec<_>>())
                        .unwrap_or_default();
                    values.set(arg.id.as_str(), strings_to_lua_table(lua, &list)?)?;
                } else if let Some(value) = matches.get_one::<String>(&arg.id) {
                    values.set(arg.id.as_str(), value.clone())?;
                }
            }
        }
    }
    Ok(values)
}

fn strings_to_lua_table(lua: &Lua, values: &[String]) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    for (index, value) in values.iter().enumerate() {
        table.set(index + 1, value.clone())?;
    }
    Ok(table)
}
