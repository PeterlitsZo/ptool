use super::{GitConfigEntry, GitConfigScope, GitConfigValue, GitRepository, repo_error};
use crate::{Error, ErrorKind, Result};
use git2::{Config, ConfigEntry, ConfigLevel, ErrorCode};

impl GitRepository {
    pub fn config_get(
        &self,
        name: &str,
        scope: Option<GitConfigScope>,
    ) -> Result<Option<GitConfigValue>> {
        let op = "ptool.git.Repo:config_get(name, options?)";
        validate_config_name(name, op)?;
        let config = self.repo.config().map_err(|err| repo_error(op, err))?;
        if let Some(scope) = scope {
            for_scope_entry(&config, name, scope, op)
        } else {
            match config.get_entry(name) {
                Ok(entry) => Ok(Some(config_entry_value(&entry))),
                Err(err) if err.code() == ErrorCode::NotFound => Ok(None),
                Err(err) => Err(repo_error(op, err)),
            }
        }
    }

    pub fn config_list(&self, scope: Option<GitConfigScope>) -> Result<Vec<GitConfigEntry>> {
        let op = "ptool.git.Repo:config_list(options?)";
        let config = self.repo.config().map_err(|err| repo_error(op, err))?;
        let entries = config.entries(None).map_err(|err| repo_error(op, err))?;
        let mut result = Vec::new();
        entries
            .for_each(|entry| {
                let entry_scope = scope_from_level(entry.level());
                if scope.is_none_or(|scope| scope == entry_scope)
                    && let Some(name) = entry.name()
                {
                    result.push(GitConfigEntry {
                        name: name.to_string(),
                        value: config_entry_value(entry),
                        scope: entry_scope.as_str().to_string(),
                    });
                }
            })
            .map_err(|err| repo_error(op, err))?;
        result.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.scope.cmp(&right.scope))
        });
        Ok(result)
    }

    pub fn config_set(
        &self,
        name: &str,
        value: GitConfigValue,
        scope: GitConfigScope,
    ) -> Result<()> {
        let op = "ptool.git.Repo:config_set(name, value, options?)";
        validate_config_name(name, op)?;
        if scope == GitConfigScope::System {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:config_set system scope is read-only",
            )
            .with_op(op));
        }
        let mut config = writable_config(&self.repo, scope, op)?;
        match value {
            GitConfigValue::String(value) => config
                .set_str(name, &value)
                .map_err(|err| repo_error(op, err)),
            GitConfigValue::Boolean(value) => config
                .set_bool(name, value)
                .map_err(|err| repo_error(op, err)),
            GitConfigValue::Integer(value) => config
                .set_i64(name, value)
                .map_err(|err| repo_error(op, err)),
        }
    }

    pub fn config_remove(&self, name: &str, scope: GitConfigScope) -> Result<()> {
        let op = "ptool.git.Repo:config_remove(name, options?)";
        validate_config_name(name, op)?;
        if scope == GitConfigScope::System {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.git.Repo:config_remove system scope is read-only",
            )
            .with_op(op));
        }
        let mut config = writable_config(&self.repo, scope, op)?;
        match config.remove(name) {
            Ok(()) => Ok(()),
            Err(err) if err.code() == ErrorCode::NotFound => Ok(()),
            Err(err) => Err(repo_error(op, err)),
        }
    }
}

fn validate_config_name(name: &str, op: &str) -> Result<()> {
    if name.is_empty() || !name.contains('.') {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("{op} requires a non-empty section.name key"),
        )
        .with_op(op));
    }
    Ok(())
}

fn for_scope_entry(
    config: &Config,
    name: &str,
    scope: GitConfigScope,
    op: &str,
) -> Result<Option<GitConfigValue>> {
    let entries = config
        .entries(Some(name))
        .map_err(|err| repo_error(op, err))?;
    let mut value = None;
    entries
        .for_each(|entry| {
            if scope_from_level(entry.level()) == scope {
                value = Some(config_entry_value(entry));
            }
        })
        .map_err(|err| repo_error(op, err))?;
    Ok(value)
}

fn writable_config(repo: &git2::Repository, scope: GitConfigScope, op: &str) -> Result<Config> {
    let mut config = repo.config().map_err(|err| repo_error(op, err))?;
    match scope {
        GitConfigScope::Local => config
            .open_level(ConfigLevel::Local)
            .map_err(|err| repo_error(op, err)),
        GitConfigScope::Global => config.open_global().map_err(|err| repo_error(op, err)),
        GitConfigScope::System => unreachable!("validated read-only scope"),
    }
}

fn config_entry_value(entry: &ConfigEntry<'_>) -> GitConfigValue {
    let value = entry.value().unwrap_or_default();
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" => GitConfigValue::Boolean(true),
        "false" | "no" | "off" => GitConfigValue::Boolean(false),
        _ => value
            .parse::<i64>()
            .map(GitConfigValue::Integer)
            .unwrap_or_else(|_| GitConfigValue::String(value.to_string())),
    }
}

fn scope_from_level(level: ConfigLevel) -> GitConfigScope {
    match level {
        ConfigLevel::Local | ConfigLevel::Worktree | ConfigLevel::App | ConfigLevel::Highest => {
            GitConfigScope::Local
        }
        ConfigLevel::Global | ConfigLevel::XDG => GitConfigScope::Global,
        ConfigLevel::System | ConfigLevel::ProgramData => GitConfigScope::System,
    }
}
