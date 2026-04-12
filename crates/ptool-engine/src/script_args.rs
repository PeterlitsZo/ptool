use crate::{Error, ErrorKind, Result};
use clap::{Arg, ArgAction, Command, value_parser};
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptArgKind {
    Flag,
    String,
    Int,
    Positional,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptArgDefault {
    String(String),
    Int(i64),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptArgSpec {
    pub id: String,
    pub kind: ScriptArgKind,
    pub long: Option<String>,
    pub short: Option<char>,
    pub help: Option<String>,
    pub required: bool,
    pub multiple: bool,
    pub default: Option<ScriptArgDefault>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptArgsSchema {
    pub name: String,
    pub about: Option<String>,
    pub args: Vec<ScriptArgSpec>,
    pub subcommands: Vec<ScriptArgsSchema>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptArgValue {
    Flag(bool),
    String(String),
    Strings(Vec<String>),
    Int(i64),
}

pub type ScriptArgValues = BTreeMap<String, ScriptArgValue>;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParsedScriptArgs {
    pub values: ScriptArgValues,
    pub command_path: Vec<String>,
    pub args: ScriptArgValues,
}

#[derive(Debug)]
pub struct ScriptArgsParseError {
    inner: clap::Error,
}

impl ScriptArgsParseError {
    pub fn print(&self) -> std::io::Result<()> {
        self.inner.print()
    }

    pub fn exit_code(&self) -> i32 {
        self.inner.exit_code()
    }
}

impl std::fmt::Display for ScriptArgsParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for ScriptArgsParseError {}

pub fn validate_script_arg_spec_base(spec: &ScriptArgSpec, context: &str) -> Result<()> {
    if matches!(spec.kind, ScriptArgKind::Positional) {
        if spec.long.is_some() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                format!("{context} positional argument cannot set `long`"),
            ));
        }
        if spec.short.is_some() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                format!("{context} positional argument cannot set `short`"),
            ));
        }
        if spec.default.is_some() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                format!("{context} positional argument cannot set `default`"),
            ));
        }
    }

    if spec.multiple && !matches!(spec.kind, ScriptArgKind::String | ScriptArgKind::Positional) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{context} only string/positional support multiple=true"),
        ));
    }

    if spec.default.is_some() && spec.multiple {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{context} does not support default with multiple=true"),
        ));
    }

    Ok(())
}

pub fn validate_script_arg_spec(
    spec: &ScriptArgSpec,
    context: &str,
    index: usize,
    total: usize,
) -> Result<()> {
    validate_script_arg_spec_base(spec, context)?;
    if matches!(spec.kind, ScriptArgKind::Positional) && spec.multiple && index != total {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{context} positional argument with multiple=true must be the last entry"),
        ));
    }
    Ok(())
}

pub fn validate_script_args_schema(
    schema: &ScriptArgsSchema,
    context: &str,
    is_root: bool,
) -> Result<()> {
    if schema.args.is_empty() && schema.subcommands.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{context} requires `args` or `subcommands`"),
        ));
    }

    if !schema.subcommands.is_empty() {
        for arg in &schema.args {
            if matches!(arg.kind, ScriptArgKind::Positional) {
                return Err(Error::new(
                    ErrorKind::InvalidArgs,
                    format!(
                        "{context} cannot define positional arguments when `subcommands` are present"
                    ),
                ));
            }
        }
    }

    if is_root && !schema.subcommands.is_empty() {
        for arg in &schema.args {
            if matches!(arg.id.as_str(), "command_path" | "args") {
                return Err(Error::new(
                    ErrorKind::InvalidArgs,
                    format!(
                        "{context} argument id `{}` is reserved when `subcommands` are present",
                        arg.id
                    ),
                ));
            }
        }
    }

    validate_subcommand_arg_paths(schema, context, &HashSet::new(), is_root)
}

pub fn parse_script_args(
    schema: &ScriptArgsSchema,
    script_args: &[String],
) -> std::result::Result<ParsedScriptArgs, ScriptArgsParseError> {
    let matches = try_parse_script_matches(schema, script_args)
        .map_err(|inner| ScriptArgsParseError { inner })?;
    Ok(script_arg_matches_to_values(schema, &matches))
}

fn validate_subcommand_arg_paths(
    schema: &ScriptArgsSchema,
    context: &str,
    ancestor_arg_ids: &HashSet<String>,
    is_root: bool,
) -> Result<()> {
    let mut next_ancestor_arg_ids = ancestor_arg_ids.clone();
    if !is_root {
        for arg in &schema.args {
            if ancestor_arg_ids.contains(&arg.id) {
                return Err(Error::new(
                    ErrorKind::InvalidArgs,
                    format!(
                        "{context} argument id `{}` conflicts with an ancestor subcommand argument id",
                        arg.id
                    ),
                ));
            }
            next_ancestor_arg_ids.insert(arg.id.clone());
        }
    }

    for subcommand in &schema.subcommands {
        validate_subcommand_arg_paths(
            subcommand,
            &format!("{context}.subcommands.{}", subcommand.name),
            &next_ancestor_arg_ids,
            false,
        )?;
    }

    Ok(())
}

fn try_parse_script_matches(
    schema: &ScriptArgsSchema,
    script_args: &[String],
) -> std::result::Result<clap::ArgMatches, clap::Error> {
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
    let arg_is_global = !schema.subcommands.is_empty();
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
        if arg_is_global {
            clap_arg = clap_arg.global(true);
        }
        command = command.arg(clap_arg);
    }

    for subcommand in &schema.subcommands {
        command = command.subcommand(build_script_arg_command(subcommand));
    }
    if !schema.subcommands.is_empty() {
        command = command.subcommand_required(true);
    }

    command
}

fn script_arg_matches_to_values(
    schema: &ScriptArgsSchema,
    matches: &clap::ArgMatches,
) -> ParsedScriptArgs {
    let values = script_arg_values_from_matches(&schema.args, matches);

    if schema.subcommands.is_empty() {
        return ParsedScriptArgs {
            values,
            command_path: Vec::new(),
            args: BTreeMap::new(),
        };
    }

    let mut command_path = Vec::new();
    let mut args = BTreeMap::new();
    collect_subcommand_matches(&mut args, &mut command_path, schema, matches);
    ParsedScriptArgs {
        values,
        command_path,
        args,
    }
}

fn script_arg_values_from_matches(
    args: &[ScriptArgSpec],
    matches: &clap::ArgMatches,
) -> ScriptArgValues {
    let mut values = BTreeMap::new();
    for arg in args {
        match arg.kind {
            ScriptArgKind::Flag => {
                values.insert(
                    arg.id.clone(),
                    ScriptArgValue::Flag(matches.get_flag(&arg.id)),
                );
            }
            ScriptArgKind::String => {
                if arg.multiple {
                    let list = matches
                        .get_many::<String>(&arg.id)
                        .map(|items| items.cloned().collect::<Vec<_>>())
                        .unwrap_or_default();
                    values.insert(arg.id.clone(), ScriptArgValue::Strings(list));
                } else if let Some(value) = matches.get_one::<String>(&arg.id) {
                    values.insert(arg.id.clone(), ScriptArgValue::String(value.clone()));
                }
            }
            ScriptArgKind::Int => {
                if let Some(value) = matches.get_one::<i64>(&arg.id) {
                    values.insert(arg.id.clone(), ScriptArgValue::Int(*value));
                }
            }
            ScriptArgKind::Positional => {
                if arg.multiple {
                    let list = matches
                        .get_many::<String>(&arg.id)
                        .map(|items| items.cloned().collect::<Vec<_>>())
                        .unwrap_or_default();
                    values.insert(arg.id.clone(), ScriptArgValue::Strings(list));
                } else if let Some(value) = matches.get_one::<String>(&arg.id) {
                    values.insert(arg.id.clone(), ScriptArgValue::String(value.clone()));
                }
            }
        }
    }
    values
}

fn collect_subcommand_matches(
    target: &mut ScriptArgValues,
    command_path: &mut Vec<String>,
    schema: &ScriptArgsSchema,
    matches: &clap::ArgMatches,
) {
    let Some((name, sub_matches)) = matches.subcommand() else {
        return;
    };
    let subcommand = schema
        .subcommands
        .iter()
        .find(|item| item.name == name)
        .expect("matched subcommand should exist in schema");

    command_path.push(name.to_string());
    target.extend(script_arg_values_from_matches(
        &subcommand.args,
        sub_matches,
    ));
    collect_subcommand_matches(target, command_path, subcommand, sub_matches);
}
