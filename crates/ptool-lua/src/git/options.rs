use super::{
    ADD_SIGNATURE, CHECKOUT_SIGNATURE, CLONE_SIGNATURE, COMMIT_SIGNATURE, ConfirmableOptions,
    FETCH_SIGNATURE, INIT_SIGNATURE, PUSH_SIGNATURE, STATUS_SIGNATURE, SWITCH_SIGNATURE,
};
use mlua::{Table, Value, Variadic};
use ptool_engine::{
    GitAddOptions, GitCheckoutOptions, GitCloneOptions, GitCommitOptions, GitFetchOptions,
    GitInitOptions, GitPushOptions, GitRemoteAuth, GitSignature, GitStatusOptions,
    GitSwitchOptions, GitTagDownload,
};
use std::path::{Path, PathBuf};

pub(super) fn parse_init_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitInitOptions>> {
    let mut parsed = GitInitOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, INIT_SIGNATURE)?;
        match key.as_str() {
            "bare" => parsed.bare = parse_bool_option(value, INIT_SIGNATURE, "bare")?,
            "initial_head" => match value {
                Value::String(value) => {
                    let value = value.to_str()?.to_string();
                    if value.is_empty() {
                        return Err(crate::lua_error::invalid_option(
                            INIT_SIGNATURE,
                            "`initial_head` must not be empty",
                        ));
                    }
                    parsed.initial_head = Some(value);
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        INIT_SIGNATURE,
                        "`initial_head` must be a string",
                    ));
                }
            },
            "confirm" => confirm = parse_bool_option(value, INIT_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    INIT_SIGNATURE,
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

pub(super) fn parse_clone_options(
    options: Option<Table>,
    current_dir: &Path,
) -> mlua::Result<ConfirmableOptions<GitCloneOptions>> {
    let mut parsed = GitCloneOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, CLONE_SIGNATURE)?;
        match key.as_str() {
            "branch" => match value {
                Value::String(value) => parsed.branch = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CLONE_SIGNATURE,
                        "`branch` must be a string",
                    ));
                }
            },
            "bare" => parsed.bare = parse_bool_option(value, CLONE_SIGNATURE, "bare")?,
            "depth" => match value {
                Value::Integer(value) if value > 0 && value <= i64::from(i32::MAX) => {
                    parsed.depth = Some(value as i32);
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CLONE_SIGNATURE,
                        "`depth` must be a positive 32-bit integer",
                    ));
                }
            },
            "checkout" => parsed.checkout = parse_bool_option(value, CLONE_SIGNATURE, "checkout")?,
            "remote" => match value {
                Value::String(value) => {
                    let value = value.to_str()?.to_string();
                    if value.is_empty() {
                        return Err(crate::lua_error::invalid_option(
                            CLONE_SIGNATURE,
                            "`remote` must not be empty",
                        ));
                    }
                    parsed.remote = Some(value);
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CLONE_SIGNATURE,
                        "`remote` must be a string",
                    ));
                }
            },
            "tags" => match value {
                Value::String(value) => {
                    parsed.tags = match value.to_str()?.as_ref() {
                        "auto" => GitTagDownload::Auto,
                        "all" => GitTagDownload::All,
                        "none" => GitTagDownload::None,
                        _ => {
                            return Err(crate::lua_error::invalid_option(
                                CLONE_SIGNATURE,
                                "`tags` must be `auto`, `all`, or `none`",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CLONE_SIGNATURE,
                        "`tags` must be a string",
                    ));
                }
            },
            "confirm" => confirm = parse_bool_option(value, CLONE_SIGNATURE, "confirm")?,
            "auth" => match value {
                Value::Table(value) => {
                    parsed.auth = parse_auth_options(value, CLONE_SIGNATURE, current_dir)?
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CLONE_SIGNATURE,
                        "`auth` must be a table",
                    ));
                }
            },
            _ => {
                return Err(crate::lua_error::invalid_option(
                    CLONE_SIGNATURE,
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

pub(super) fn parse_status_options(options: Option<Table>) -> mlua::Result<GitStatusOptions> {
    let mut parsed = GitStatusOptions::default();
    let Some(options) = options else {
        return Ok(parsed);
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, STATUS_SIGNATURE)?;
        match key.as_str() {
            "include_untracked" => match value {
                Value::Boolean(value) => parsed.include_untracked = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        STATUS_SIGNATURE,
                        "`include_untracked` must be a boolean",
                    ));
                }
            },
            "include_ignored" => match value {
                Value::Boolean(value) => parsed.include_ignored = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        STATUS_SIGNATURE,
                        "`include_ignored` must be a boolean",
                    ));
                }
            },
            "recurse_untracked_dirs" => match value {
                Value::Boolean(value) => parsed.recurse_untracked_dirs = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        STATUS_SIGNATURE,
                        "`recurse_untracked_dirs` must be a boolean",
                    ));
                }
            },
            "paths" => {
                parsed.paths = parse_string_list_from_value(value, STATUS_SIGNATURE, "paths")?
            }
            _ => {
                return Err(crate::lua_error::invalid_option(
                    STATUS_SIGNATURE,
                    format!("unknown option `{key}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

pub(super) fn parse_add_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitAddOptions>> {
    let mut parsed = GitAddOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, ADD_SIGNATURE)?;
        match key.as_str() {
            "update" => match value {
                Value::Boolean(value) => parsed.update = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        ADD_SIGNATURE,
                        "`update` must be a boolean",
                    ));
                }
            },
            "confirm" => confirm = parse_bool_option(value, ADD_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    ADD_SIGNATURE,
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

pub(super) fn parse_commit_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitCommitOptions>> {
    let mut parsed = GitCommitOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, COMMIT_SIGNATURE)?;
        match key.as_str() {
            "author" => match value {
                Value::Table(value) => {
                    parsed.author = Some(parse_signature(value, COMMIT_SIGNATURE)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        COMMIT_SIGNATURE,
                        "`author` must be a table",
                    ));
                }
            },
            "committer" => match value {
                Value::Table(value) => {
                    parsed.committer = Some(parse_signature(value, COMMIT_SIGNATURE)?)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        COMMIT_SIGNATURE,
                        "`committer` must be a table",
                    ));
                }
            },
            "amend" => parsed.amend = parse_bool_option(value, COMMIT_SIGNATURE, "amend")?,
            "allow_empty" => {
                parsed.allow_empty = parse_bool_option(value, COMMIT_SIGNATURE, "allow_empty")?
            }
            "confirm" => confirm = parse_bool_option(value, COMMIT_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    COMMIT_SIGNATURE,
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

pub(super) fn parse_checkout_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitCheckoutOptions>> {
    let mut parsed = GitCheckoutOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, CHECKOUT_SIGNATURE)?;
        match key.as_str() {
            "force" => match value {
                Value::Boolean(value) => parsed.force = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        CHECKOUT_SIGNATURE,
                        "`force` must be a boolean",
                    ));
                }
            },
            "confirm" => confirm = parse_bool_option(value, CHECKOUT_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    CHECKOUT_SIGNATURE,
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

pub(super) fn parse_switch_options(
    options: Option<Table>,
) -> mlua::Result<ConfirmableOptions<GitSwitchOptions>> {
    let mut parsed = GitSwitchOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, SWITCH_SIGNATURE)?;
        match key.as_str() {
            "create" => match value {
                Value::Boolean(value) => parsed.create = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        SWITCH_SIGNATURE,
                        "`create` must be a boolean",
                    ));
                }
            },
            "force" => match value {
                Value::Boolean(value) => parsed.force = value,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        SWITCH_SIGNATURE,
                        "`force` must be a boolean",
                    ));
                }
            },
            "start_point" => match value {
                Value::String(value) => parsed.start_point = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        SWITCH_SIGNATURE,
                        "`start_point` must be a string",
                    ));
                }
            },
            "track" => match value {
                Value::String(value) => parsed.track = Some(value.to_str()?.to_string()),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        SWITCH_SIGNATURE,
                        "`track` must be a string",
                    ));
                }
            },
            "orphan" => parsed.orphan = parse_bool_option(value, SWITCH_SIGNATURE, "orphan")?,
            "confirm" => confirm = parse_bool_option(value, SWITCH_SIGNATURE, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    SWITCH_SIGNATURE,
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

pub(super) fn parse_fetch_call(
    args: Variadic<Value>,
    current_dir: &Path,
) -> mlua::Result<(Option<String>, ConfirmableOptions<GitFetchOptions>)> {
    match args.len() {
        0 => Ok((
            None,
            ConfirmableOptions {
                inner: GitFetchOptions::default(),
                confirm: false,
            },
        )),
        1 => match args.first() {
            Some(Value::String(value)) => Ok((
                Some(value.to_str()?.to_string()),
                ConfirmableOptions {
                    inner: GitFetchOptions::default(),
                    confirm: false,
                },
            )),
            Some(Value::Table(value)) => Ok((
                None,
                parse_fetch_options(Some(value.clone()), FETCH_SIGNATURE, current_dir)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                FETCH_SIGNATURE,
                "expects a string remote or an options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(remote)), Some(Value::Table(options))) => Ok((
                Some(remote.to_str()?.to_string()),
                parse_fetch_options(Some(options.clone()), FETCH_SIGNATURE, current_dir)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                FETCH_SIGNATURE,
                "expects (remote, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            FETCH_SIGNATURE,
            "accepts at most 2 arguments",
        )),
    }
}

pub(super) fn parse_fetch_options(
    options: Option<Table>,
    op: &str,
    current_dir: &Path,
) -> mlua::Result<ConfirmableOptions<GitFetchOptions>> {
    let mut parsed = GitFetchOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "refspecs" => parsed.refspecs = parse_string_list_from_value(value, op, "refspecs")?,
            "depth" => match value {
                Value::Integer(value) if value > 0 && value <= i64::from(i32::MAX) => {
                    parsed.depth = Some(value as i32)
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        op,
                        "`depth` must be a positive 32-bit integer",
                    ));
                }
            },
            "prune" => parsed.prune = parse_bool_option(value, op, "prune")?,
            "tags" => parsed.tags = parse_tag_download(value, op)?,
            "update_fetchhead" => {
                parsed.update_fetchhead = parse_bool_option(value, op, "update_fetchhead")?
            }
            "auth" => match value {
                Value::Table(value) => parsed.auth = parse_auth_options(value, op, current_dir)?,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        op,
                        "`auth` must be a table",
                    ));
                }
            },
            "confirm" => confirm = parse_bool_option(value, op, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
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

pub(super) fn parse_push_call(
    args: Variadic<Value>,
    current_dir: &Path,
) -> mlua::Result<(
    Option<String>,
    Vec<String>,
    ConfirmableOptions<GitPushOptions>,
)> {
    match args.len() {
        0 => Ok((
            None,
            Vec::new(),
            ConfirmableOptions {
                inner: GitPushOptions::default(),
                confirm: false,
            },
        )),
        1 => match args.first() {
            Some(Value::String(value)) => Ok((
                Some(value.to_str()?.to_string()),
                Vec::new(),
                ConfirmableOptions {
                    inner: GitPushOptions::default(),
                    confirm: false,
                },
            )),
            Some(Value::Table(value)) => Ok((
                None,
                Vec::new(),
                parse_push_options(Some(value.clone()), PUSH_SIGNATURE, current_dir)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                PUSH_SIGNATURE,
                "expects a string remote or an options table",
            )),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(remote)), Some(Value::Table(options))) => Ok((
                Some(remote.to_str()?.to_string()),
                Vec::new(),
                parse_push_options(Some(options.clone()), PUSH_SIGNATURE, current_dir)?,
            )),
            (Some(Value::String(remote)), Some(value)) => Ok((
                Some(remote.to_str()?.to_string()),
                parse_string_list_from_value(value.clone(), PUSH_SIGNATURE, "refspecs")?,
                ConfirmableOptions {
                    inner: GitPushOptions::default(),
                    confirm: false,
                },
            )),
            _ => Err(crate::lua_error::invalid_argument(
                PUSH_SIGNATURE,
                "expects (remote, refspecs|options)",
            )),
        },
        3 => match (args.first(), args.get(1), args.get(2)) {
            (Some(Value::String(remote)), Some(value), Some(Value::Table(options))) => Ok((
                Some(remote.to_str()?.to_string()),
                parse_string_list_from_value(value.clone(), PUSH_SIGNATURE, "refspecs")?,
                parse_push_options(Some(options.clone()), PUSH_SIGNATURE, current_dir)?,
            )),
            _ => Err(crate::lua_error::invalid_argument(
                PUSH_SIGNATURE,
                "expects (remote, refspecs, options)",
            )),
        },
        _ => Err(crate::lua_error::invalid_argument(
            PUSH_SIGNATURE,
            "accepts at most 3 arguments",
        )),
    }
}

pub(super) fn parse_push_options(
    options: Option<Table>,
    op: &str,
    current_dir: &Path,
) -> mlua::Result<ConfirmableOptions<GitPushOptions>> {
    let mut parsed = GitPushOptions::default();
    let mut confirm = false;
    let Some(options) = options else {
        return Ok(ConfirmableOptions {
            inner: parsed,
            confirm,
        });
    };

    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "auth" => match value {
                Value::Table(value) => parsed.auth = parse_auth_options(value, op, current_dir)?,
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        op,
                        "`auth` must be a table",
                    ));
                }
            },
            "force" => parsed.force = parse_bool_option(value, op, "force")?,
            "set_upstream" => parsed.set_upstream = parse_bool_option(value, op, "set_upstream")?,
            "confirm" => confirm = parse_bool_option(value, op, "confirm")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
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

pub(super) fn parse_tag_download(value: Value, op: &str) -> mlua::Result<GitTagDownload> {
    match value {
        Value::String(value) => match value.to_str()?.as_ref() {
            "auto" => Ok(GitTagDownload::Auto),
            "all" => Ok(GitTagDownload::All),
            "none" => Ok(GitTagDownload::None),
            _ => Err(crate::lua_error::invalid_option(
                op,
                "`tags` must be `auto`, `all`, or `none`",
            )),
        },
        _ => Err(crate::lua_error::invalid_option(
            op,
            "`tags` must be a string",
        )),
    }
}

pub(super) fn parse_auth_options(
    options: Table,
    op: &str,
    current_dir: &Path,
) -> mlua::Result<GitRemoteAuth> {
    let allowed = [
        "kind",
        "username",
        "password",
        "private_key",
        "public_key",
        "passphrase",
    ];
    for pair in options.clone().pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = parse_option_key(key, op)?;
        if !allowed.contains(&key.as_str()) {
            return Err(crate::lua_error::invalid_option(
                op,
                format!("unknown auth field `{key}`"),
            ));
        }
    }
    let Some(kind) = options.get::<Option<String>>("kind")? else {
        return Err(crate::lua_error::invalid_argument(
            op,
            "`auth.kind` is required",
        ));
    };
    match kind.as_str() {
        "default" => Ok(GitRemoteAuth::Default),
        "ssh_agent" => Ok(GitRemoteAuth::SshAgent {
            username: options.get::<Option<String>>("username")?,
        }),
        "ssh_key" => {
            let Some(username) = options.get::<Option<String>>("username")? else {
                return Err(crate::lua_error::invalid_argument(
                    op,
                    "`auth.username` is required for `ssh_key`",
                ));
            };
            let Some(private_key) = options.get::<Option<String>>("private_key")? else {
                return Err(crate::lua_error::invalid_argument(
                    op,
                    "`auth.private_key` is required for `ssh_key`",
                ));
            };
            if username.is_empty() || private_key.is_empty() {
                return Err(crate::lua_error::invalid_argument(
                    op,
                    "`auth.username` and `auth.private_key` must not be empty",
                ));
            }
            Ok(GitRemoteAuth::SshKey {
                username,
                public_key: options
                    .get::<Option<String>>("public_key")?
                    .map(|path| resolve_auth_path(current_dir, &path)),
                private_key: resolve_auth_path(current_dir, &private_key),
                passphrase: options.get::<Option<String>>("passphrase")?,
            })
        }
        "userpass" => {
            let Some(username) = options.get::<Option<String>>("username")? else {
                return Err(crate::lua_error::invalid_argument(
                    op,
                    "`auth.username` is required for `userpass`",
                ));
            };
            let Some(password) = options.get::<Option<String>>("password")? else {
                return Err(crate::lua_error::invalid_argument(
                    op,
                    "`auth.password` is required for `userpass`",
                ));
            };
            Ok(GitRemoteAuth::UserPass { username, password })
        }
        "credential_helper" => Ok(GitRemoteAuth::CredentialHelper {
            username: options.get::<Option<String>>("username")?,
        }),
        _ => Err(crate::lua_error::invalid_argument(
            op,
            "`auth.kind` must be `default`, `ssh_agent`, `ssh_key`, `userpass`, or `credential_helper`",
        )),
    }
}

fn resolve_auth_path(current_dir: &Path, path: &str) -> String {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path.to_string_lossy().to_string()
    } else {
        current_dir.join(path).to_string_lossy().to_string()
    }
}

pub(super) fn parse_signature(options: Table, op: &str) -> mlua::Result<GitSignature> {
    for pair in options.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let key = parse_option_key(key, op)?;
        match key.as_str() {
            "name" | "email" | "time_seconds" | "offset_minutes" => {}
            _ => {
                return Err(crate::lua_error::invalid_option(
                    op,
                    format!("unknown signature field `{key}`"),
                ));
            }
        }
    }

    let Some(name) = options.get::<Option<String>>("name")? else {
        return Err(crate::lua_error::invalid_argument(op, "`name` is required"));
    };
    let Some(email) = options.get::<Option<String>>("email")? else {
        return Err(crate::lua_error::invalid_argument(
            op,
            "`email` is required",
        ));
    };

    let time_seconds = options.get::<Option<i64>>("time_seconds")?;
    let offset_minutes = options.get::<Option<i32>>("offset_minutes")?;
    if time_seconds.is_some() != offset_minutes.is_some() {
        return Err(crate::lua_error::invalid_argument(
            op,
            "`time_seconds` and `offset_minutes` must be provided together",
        ));
    }

    Ok(GitSignature {
        name,
        email,
        time_seconds,
        offset_minutes,
    })
}

pub(super) fn parse_paths(value: Value, op: &str) -> mlua::Result<Vec<String>> {
    match value {
        Value::String(value) => Ok(vec![value.to_str()?.to_string()]),
        Value::Table(value) => parse_string_list(value, op, "paths"),
        _ => Err(crate::lua_error::invalid_argument(
            op,
            "expects a string path or an array of paths",
        )),
    }
}

pub(super) fn parse_string_list_from_value(
    value: Value,
    op: &str,
    field: &str,
) -> mlua::Result<Vec<String>> {
    match value {
        Value::String(value) => Ok(vec![value.to_str()?.to_string()]),
        Value::Table(value) => parse_string_list(value, op, field),
        _ => Err(crate::lua_error::invalid_argument(
            op,
            format!("`{field}` must be a string or an array of strings"),
        )),
    }
}

pub(super) fn parse_string_list(table: Table, op: &str, field: &str) -> mlua::Result<Vec<String>> {
    let mut values = Vec::new();
    for value in table.sequence_values::<String>() {
        values.push(value.map_err(|_| {
            crate::lua_error::invalid_argument(op, format!("`{field}` must be an array of strings"))
        })?);
    }
    Ok(values)
}

pub(super) fn parse_option_key(key: Value, op: &str) -> mlua::Result<String> {
    match key {
        Value::String(key) => Ok(key.to_str()?.to_string()),
        _ => Err(crate::lua_error::invalid_option(
            op,
            "option keys must be strings",
        )),
    }
}

pub(super) fn parse_bool_option(value: Value, op: &str, field: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(crate::lua_error::invalid_option(
            op,
            format!("`{field}` must be a boolean"),
        )),
    }
}
