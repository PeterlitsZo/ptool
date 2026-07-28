use super::{
    ConfirmableOptions, LuaGitRepo, TAG_CREATE_SIGNATURE, TAG_DELETE_SIGNATURE, TAGS_SIGNATURE,
    confirm_git_action,
    options::{parse_bool_option, parse_option_key, parse_signature},
    render::{git_tag_to_lua, git_tags_to_lua},
};
use mlua::{Lua, Table, Value, Variadic};
use ptool_console::GitAction;
use ptool_engine::GitTagCreateOptions;

pub(super) fn tags(repo: &LuaGitRepo, lua: &Lua, pattern: Option<String>) -> mlua::Result<Table> {
    let tags = repo
        .repo
        .tags(pattern.as_deref())
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, TAGS_SIGNATURE))?;
    git_tags_to_lua(lua, tags)
}

pub(super) fn tag_create(
    repo: &LuaGitRepo,
    lua: &Lua,
    args: Variadic<Value>,
) -> mlua::Result<Table> {
    let (name, target, options) = parse_tag_create_call(args)?;
    if options.confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            TAG_CREATE_SIGNATURE,
            GitAction::CreateTag {
                repository: &repository,
                tag: &name,
            },
        )?;
    }
    let tag = repo
        .repo
        .tag_create(&name, target.as_deref(), options.inner)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, TAG_CREATE_SIGNATURE))?;
    git_tag_to_lua(lua, tag)
}

pub(super) fn tag_delete(
    repo: &LuaGitRepo,
    name: String,
    options: Option<Table>,
) -> mlua::Result<()> {
    let confirm = parse_confirm_only(options, TAG_DELETE_SIGNATURE)?;
    if confirm {
        let repository = repo.repo_label();
        confirm_git_action(
            &repo.engine,
            TAG_DELETE_SIGNATURE,
            GitAction::DeleteTag {
                repository: &repository,
                tag: &name,
            },
        )?;
    }
    repo.repo
        .tag_delete(&name)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, TAG_DELETE_SIGNATURE))
}

fn parse_tag_create_call(
    args: Variadic<Value>,
) -> mlua::Result<(
    String,
    Option<String>,
    ConfirmableOptions<GitTagCreateOptions>,
)> {
    let Some(Value::String(name)) = args.first() else {
        return Err(crate::lua_error::invalid_argument(
            TAG_CREATE_SIGNATURE,
            "requires a tag name string",
        ));
    };
    let name = name.to_str()?.to_string();
    match args.len() {
        1 => Ok((
            name,
            None,
            ConfirmableOptions {
                inner: GitTagCreateOptions::default(),
                confirm: false,
            },
        )),
        2 => match args.get(1) {
            Some(Value::String(target)) => Ok((
                name,
                Some(target.to_str()?.to_string()),
                ConfirmableOptions {
                    inner: GitTagCreateOptions::default(),
                    confirm: false,
                },
            )),
            Some(Value::Table(options)) => {
                Ok((name, None, parse_tag_create_options(options.clone())?))
            }
            _ => Err(crate::lua_error::invalid_argument(
                TAG_CREATE_SIGNATURE,
                "expects (name, target|options)",
            )),
        },
        3 => match (args.get(1), args.get(2)) {
            (Some(Value::String(target)), Some(Value::Table(options))) => Ok((
                name,
                Some(target.to_str()?.to_string()),
                parse_tag_create_options(options.clone())?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                TAG_CREATE_SIGNATURE,
                "expects (name, target, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            TAG_CREATE_SIGNATURE,
            "accepts between 1 and 3 arguments",
        )),
    }
}

fn parse_tag_create_options(
    options: Table,
) -> mlua::Result<ConfirmableOptions<GitTagCreateOptions>> {
    let mut parsed = GitTagCreateOptions::default();
    let mut confirm = false;
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, TAG_CREATE_SIGNATURE)?;
        match key.as_str() {
            "message" => match value {
                Value::String(value) => parsed.message = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        TAG_CREATE_SIGNATURE,
                        "`message` must be a string",
                    ));
                }
            },
            "tagger" => match value {
                Value::Table(value) => {
                    parsed.tagger = Some(parse_signature(value, TAG_CREATE_SIGNATURE)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        TAG_CREATE_SIGNATURE,
                        "`tagger` must be a table",
                    ));
                }
            },
            "force" => parsed.force = parse_bool_option(value, TAG_CREATE_SIGNATURE, "force")?,
            "confirm" => confirm = parse_bool_option(value, TAG_CREATE_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    TAG_CREATE_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(ConfirmableOptions {
        inner: parsed,
        confirm,
    })
}

fn parse_confirm_only(options: Option<Table>, op: &str) -> mlua::Result<bool> {
    let Some(options) = options else {
        return Ok(false);
    };
    let mut confirm = false;
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "confirm" => confirm = parse_bool_option(value, op, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }
    Ok(confirm)
}
