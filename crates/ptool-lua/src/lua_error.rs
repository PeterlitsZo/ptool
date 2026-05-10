use mlua::{Lua, Table};
use ptool_engine::{Error as EngineError, ErrorKind as EngineErrorKind};
use std::error::Error as StdError;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LuaError {
    pub kind: String,
    pub message: String,
    pub op: Option<String>,
    pub detail: Option<String>,
    pub retryable: bool,
    pub path: Option<String>,
    pub cmd: Option<String>,
    pub status: Option<i32>,
    pub stderr: Option<String>,
    pub url: Option<String>,
    pub input: Option<String>,
    pub cwd: Option<String>,
    pub target: Option<String>,
}

impl LuaError {
    pub(crate) fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            op: None,
            detail: None,
            retryable: false,
            path: None,
            cmd: None,
            status: None,
            stderr: None,
            url: None,
            input: None,
            cwd: None,
            target: None,
        }
    }

    pub(crate) fn from_engine(err: EngineError, default_op: &str) -> Self {
        let op = err.op().map(str::to_string);
        let detail = err.detail().map(str::to_string);
        let retryable = err.retryable();
        let path = err.path().map(str::to_string);
        let cmd = err.cmd().map(str::to_string);
        let status = err.status();
        let stderr = err.stderr().map(str::to_string);
        let url = err.url().map(str::to_string);
        let input = err.input().map(str::to_string);

        Self {
            kind: engine_kind_name(err.kind).to_string(),
            message: err.msg,
            op: op.or_else(|| Some(default_op.to_string())),
            detail,
            retryable,
            path,
            cmd,
            status,
            stderr,
            url,
            input,
            cwd: None,
            target: None,
        }
    }

    pub(crate) fn invalid_argument(op: &str, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Self::new("invalid_argument", format!("{op} {detail}"))
            .with_op(op)
            .with_detail(detail)
    }

    pub(crate) fn invalid_option(op: &str, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Self::new("invalid_option", format!("{op} {detail}"))
            .with_op(op)
            .with_detail(detail)
    }

    pub(crate) fn cancelled(op: &str, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Self::new("cancelled", format!("{op} {detail}"))
            .with_op(op)
            .with_detail(detail)
    }

    pub(crate) fn prompt_failed(op: &str, detail: impl Into<String>) -> Self {
        let detail = detail.into();
        Self::new("prompt_failed", format!("{op} {detail}"))
            .with_op(op)
            .with_detail(detail)
    }

    pub(crate) fn command_failed(
        op: &str,
        cmd: &str,
        status: Option<i32>,
        stderr: Option<&str>,
    ) -> Self {
        let code = status
            .map(|value| value.to_string())
            .unwrap_or_else(|| "terminated by signal".to_string());
        let mut message = format!("{op} command `{cmd}` failed with status {code}");
        let trimmed_stderr = stderr.map(str::trim).filter(|value| !value.is_empty());
        if let Some(stderr) = trimmed_stderr {
            message.push_str(": ");
            message.push_str(stderr);
        }

        let mut error = Self::new("command_failed", message)
            .with_op(op)
            .with_cmd(cmd)
            .with_detail(format!("status: {code}"));
        if let Some(status) = status {
            error = error.with_status(status);
        }
        if let Some(stderr) = trimmed_stderr {
            error = error.with_stderr(stderr);
        }
        error
    }

    pub(crate) fn with_op(mut self, op: impl Into<String>) -> Self {
        self.op = Some(op.into());
        self
    }

    pub(crate) fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub(crate) fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub(crate) fn with_cmd(mut self, cmd: impl Into<String>) -> Self {
        self.cmd = Some(cmd.into());
        self
    }

    pub(crate) fn with_status(mut self, status: i32) -> Self {
        self.status = Some(status);
        self
    }

    pub(crate) fn with_stderr(mut self, stderr: impl Into<String>) -> Self {
        self.stderr = Some(stderr.into());
        self
    }

    pub(crate) fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub(crate) fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub(crate) fn into_mlua_error(self) -> mlua::Error {
        mlua::Error::external(self)
    }

    pub(crate) fn to_table(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        table.set("kind", self.kind.clone())?;
        table.set("message", self.message.clone())?;
        table.set("retryable", self.retryable)?;
        table.set("op", self.op.clone())?;
        table.set("detail", self.detail.clone())?;
        table.set("path", self.path.clone())?;
        table.set("cmd", self.cmd.clone())?;
        table.set("status", self.status.map(i64::from))?;
        table.set("stderr", self.stderr.clone())?;
        table.set("url", self.url.clone())?;
        table.set("input", self.input.clone())?;
        table.set("cwd", self.cwd.clone())?;
        table.set("target", self.target.clone())?;
        Ok(table)
    }

    pub(crate) fn render_report(&self) -> String {
        let mut lines = vec![self.message.clone()];
        push_report_field(&mut lines, "kind", Some(self.kind.as_str()));
        push_report_field(&mut lines, "op", self.op.as_deref());
        push_report_field(&mut lines, "detail", self.detail.as_deref());
        push_report_field(&mut lines, "path", self.path.as_deref());
        push_report_field(&mut lines, "cmd", self.cmd.as_deref());
        push_report_field(
            &mut lines,
            "status",
            self.status.as_ref().map(ToString::to_string).as_deref(),
        );
        push_report_field(&mut lines, "url", self.url.as_deref());
        push_report_field(&mut lines, "input", self.input.as_deref());
        push_report_field(&mut lines, "cwd", self.cwd.as_deref());
        push_report_field(&mut lines, "target", self.target.as_deref());
        if let Some(stderr) = self.stderr.as_deref().filter(|value| !value.is_empty()) {
            lines.push("stderr:".to_string());
            for line in stderr.lines() {
                lines.push(format!("  {line}"));
            }
        }
        lines.join("\n")
    }
}

impl fmt::Display for LuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for LuaError {}

pub(crate) fn to_mlua_error(err: LuaError) -> mlua::Error {
    err.into_mlua_error()
}

pub(crate) fn lua_error_from_engine(err: EngineError, default_op: &str) -> mlua::Error {
    to_mlua_error(LuaError::from_engine(err, default_op))
}

pub(crate) fn invalid_argument(op: &str, detail: impl Into<String>) -> mlua::Error {
    to_mlua_error(LuaError::invalid_argument(op, detail))
}

pub(crate) fn invalid_option(op: &str, detail: impl Into<String>) -> mlua::Error {
    to_mlua_error(LuaError::invalid_option(op, detail))
}

pub(crate) fn prompt_failed(op: &str, detail: impl Into<String>) -> mlua::Error {
    to_mlua_error(LuaError::prompt_failed(op, detail))
}

pub(crate) fn caught_error_to_table(lua: &Lua, err: &mlua::Error) -> mlua::Result<Table> {
    match extract_lua_error(err) {
        Some(lua_err) => lua_err.to_table(lua),
        None => LuaError::new("lua_error", err.to_string()).to_table(lua),
    }
}

pub(crate) fn render_error_report(err: &mlua::Error) -> String {
    match extract_lua_error(err) {
        Some(lua_err) => lua_err.render_report(),
        None => err.to_string(),
    }
}

pub(crate) fn render_any_error(err: &(dyn StdError + 'static)) -> String {
    if let Some(lua_err) = err.downcast_ref::<LuaError>() {
        return lua_err.render_report();
    }
    if let Some(lua_err) = err.downcast_ref::<mlua::Error>() {
        return render_error_report(lua_err);
    }
    err.to_string()
}

fn extract_lua_error(err: &mlua::Error) -> Option<&LuaError> {
    if let Some(lua_err) = err.downcast_ref::<LuaError>() {
        return Some(lua_err);
    }

    for source in err.chain() {
        if let Some(lua_err) = source.downcast_ref::<LuaError>() {
            return Some(lua_err);
        }
    }

    None
}

fn engine_kind_name(kind: EngineErrorKind) -> &'static str {
    match kind {
        EngineErrorKind::EmptyInput
        | EngineErrorKind::EmptyPath
        | EngineErrorKind::InvalidGlob
        | EngineErrorKind::InvalidArgs
        | EngineErrorKind::InvalidUrl
        | EngineErrorKind::InvalidIp
        | EngineErrorKind::InvalidHttpMethod
        | EngineErrorKind::InvalidHttpHeader
        | EngineErrorKind::InvalidHttpTimeout
        | EngineErrorKind::InvalidHttpOptions
        | EngineErrorKind::InvalidPromptOptions
        | EngineErrorKind::InvalidToml
        | EngineErrorKind::InvalidSemver
        | EngineErrorKind::InvalidSemverOperation
        | EngineErrorKind::MissingPort
        | EngineErrorKind::InvalidHost
        | EngineErrorKind::InvalidPort
        | EngineErrorKind::InvalidHostPort
        | EngineErrorKind::InvalidFsOption => "invalid_argument",
        EngineErrorKind::AlreadyExists => "already_exists",
        EngineErrorKind::NotAFile => "not_a_file",
        EngineErrorKind::NotInteractive => "not_interactive",
        EngineErrorKind::Cancelled => "cancelled",
        EngineErrorKind::Prompt => "prompt_failed",
        EngineErrorKind::Http => "http_error",
        EngineErrorKind::Io => "io_error",
        EngineErrorKind::Db => "db_error",
        EngineErrorKind::SemverOverflow => "overflow",
        EngineErrorKind::Ssh => "ssh_error",
    }
}

fn push_report_field(lines: &mut Vec<String>, name: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        lines.push(format!("{name}: {value}"));
    }
}
