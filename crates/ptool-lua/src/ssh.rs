use crate::command_echo::print_ssh_command_echo;
use mlua::{Lua, Table, UserData, UserDataFields, UserDataMethods, Value, Variadic};
use ptool_engine::{
    Error as EngineError, PtoolEngine, SshAuthRequest, SshConnectRequest, SshConnection,
    SshConnectionInfo, SshExecOptions, SshExecResult, SshHostKeyRequest, SshStreamMode,
    SshTransferOptions, SshTransferResult,
};
use shlex::try_quote;
use std::path::Path;

const CONNECT_SIGNATURE: &str = "ptool.ssh.connect(target_or_options)";
const RUN_SIGNATURE: &str = "ptool.ssh.Connection:run(...)";
const RUN_CAPTURE_SIGNATURE: &str = "ptool.ssh.Connection:run_capture(...)";
const CLOSE_SIGNATURE: &str = "ptool.ssh.Connection:close()";
const PATH_SIGNATURE: &str = "ptool.ssh.Connection:path(path)";
const EXISTS_SIGNATURE: &str = "ptool.ssh.Connection:exists(path)";
const IS_FILE_SIGNATURE: &str = "ptool.ssh.Connection:is_file(path)";
const IS_DIR_SIGNATURE: &str = "ptool.ssh.Connection:is_dir(path)";
const UPLOAD_SIGNATURE: &str = "ptool.ssh.Connection:upload(local_path, remote_path[, options])";
const DOWNLOAD_SIGNATURE: &str =
    "ptool.ssh.Connection:download(remote_path, local_path[, options])";
const REMOTE_PATH_EXISTS_SIGNATURE: &str = "ptool.ssh.RemotePath:exists()";
const REMOTE_PATH_IS_FILE_SIGNATURE: &str = "ptool.ssh.RemotePath:is_file()";
const REMOTE_PATH_IS_DIR_SIGNATURE: &str = "ptool.ssh.RemotePath:is_dir()";

pub(crate) type TransferOptions = SshTransferOptions;
pub(crate) type TransferResult = SshTransferResult;

#[derive(Clone, Copy)]
struct ExecStreamDefaults {
    stdout: SshStreamMode,
    stderr: SshStreamMode,
}

const RUN_STREAM_DEFAULTS: ExecStreamDefaults = ExecStreamDefaults {
    stdout: SshStreamMode::Inherit,
    stderr: SshStreamMode::Inherit,
};

const RUN_CAPTURE_STREAM_DEFAULTS: ExecStreamDefaults = ExecStreamDefaults {
    stdout: SshStreamMode::Capture,
    stderr: SshStreamMode::Capture,
};

#[derive(Clone)]
pub(crate) struct LuaSshConnection {
    connection: SshConnection,
}

#[derive(Clone)]
pub(crate) struct LuaSshPath {
    connection: LuaSshConnection,
    path: String,
}

pub(crate) fn connect(
    value: Value,
    current_dir: &Path,
    engine: &PtoolEngine,
) -> mlua::Result<LuaSshConnection> {
    let request = parse_connect_request(value)?;
    let connection = engine
        .ssh_connect(request, current_dir)
        .map_err(|err| ssh_error(CONNECT_SIGNATURE, err))?;
    Ok(LuaSshConnection { connection })
}

impl UserData for LuaSshConnection {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("host", |_, this| Ok(this.info().host));
        fields.add_field_method_get("user", |_, this| Ok(this.info().user));
        fields.add_field_method_get("port", |_, this| Ok(i64::from(this.info().port)));
        fields.add_field_method_get("target", |_, this| Ok(this.info().target));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("run", |lua, this, args: Variadic<Value>| {
            this.run(lua, args)
        });
        methods.add_method("run_capture", |lua, this, args: Variadic<Value>| {
            this.run_capture(lua, args)
        });
        methods.add_method("path", |_, this, path: String| this.path(path));
        methods.add_method("exists", |_, this, value: Value| this.exists(value));
        methods.add_method("is_file", |_, this, value: Value| this.is_file(value));
        methods.add_method("is_dir", |_, this, value: Value| this.is_dir(value));
        methods.add_method("upload", |lua, this, args: Variadic<Value>| {
            this.upload(lua, args)
        });
        methods.add_method("download", |lua, this, args: Variadic<Value>| {
            this.download(lua, args)
        });
        methods.add_method("close", |_, this, ()| this.close());
    }
}

impl UserData for LuaSshPath {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("host", |_, this| Ok(this.connection.info().host));
        fields.add_field_method_get("user", |_, this| Ok(this.connection.info().user));
        fields.add_field_method_get("port", |_, this| Ok(i64::from(this.connection.info().port)));
        fields.add_field_method_get("target", |_, this| Ok(this.connection.info().target));
        fields.add_field_method_get("path", |_, this| Ok(this.path.clone()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("exists", |_, this, ()| this.exists());
        methods.add_method("is_file", |_, this, ()| this.is_file());
        methods.add_method("is_dir", |_, this, ()| this.is_dir());
    }
}

impl LuaSshConnection {
    fn info(&self) -> SshConnectionInfo {
        self.connection.info()
    }

    fn matches_connection(&self, other: &LuaSshConnection) -> bool {
        self.connection.same_session(&other.connection)
    }

    fn run(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        let mut options = parse_run_call(args)?;
        if options.echo {
            let display_cwd = self
                .connection
                .resolve_display_cwd(options.display_cwd.as_deref())
                .unwrap_or_else(|_| "<unknown remote cwd>".to_string());
            print_ssh_command_echo(&self.info().target, &display_cwd, &options.command);
            options.echo = false;
        }
        let result = self
            .connection
            .run(options)
            .map_err(|err| ssh_error(RUN_SIGNATURE, err))?;
        build_exec_result(lua, result, self.info().target)
    }

    fn run_capture(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        let mut options = parse_run_capture_call(args)?;
        if options.echo {
            let display_cwd = self
                .connection
                .resolve_display_cwd(options.display_cwd.as_deref())
                .unwrap_or_else(|_| "<unknown remote cwd>".to_string());
            print_ssh_command_echo(&self.info().target, &display_cwd, &options.command);
            options.echo = false;
        }
        let result = self
            .connection
            .run(options)
            .map_err(|err| ssh_error(RUN_CAPTURE_SIGNATURE, err))?;
        build_exec_result(lua, result, self.info().target)
    }

    fn path(&self, path: String) -> mlua::Result<LuaSshPath> {
        ensure_non_empty_string(&path, PATH_SIGNATURE, "path")?;
        Ok(LuaSshPath {
            connection: self.clone(),
            path,
        })
    }

    fn exists(&self, value: Value) -> mlua::Result<bool> {
        let path = parse_connection_remote_path_arg(self, value, EXISTS_SIGNATURE, "path")?;
        self.connection
            .exists(&path)
            .map_err(|err| ssh_error(EXISTS_SIGNATURE, err))
    }

    fn is_file(&self, value: Value) -> mlua::Result<bool> {
        let path = parse_connection_remote_path_arg(self, value, IS_FILE_SIGNATURE, "path")?;
        self.connection
            .is_file(&path)
            .map_err(|err| ssh_error(IS_FILE_SIGNATURE, err))
    }

    fn is_dir(&self, value: Value) -> mlua::Result<bool> {
        let path = parse_connection_remote_path_arg(self, value, IS_DIR_SIGNATURE, "path")?;
        self.connection
            .is_dir(&path)
            .map_err(|err| ssh_error(IS_DIR_SIGNATURE, err))
    }

    fn upload(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        let (local_path, remote_path, options) = parse_upload_call(self, args)?;
        let result = self.upload_file(Path::new(&local_path), &remote_path, options)?;
        build_transfer_result(lua, result)
    }

    fn download(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        let (remote_path, local_path, options) = parse_download_call(self, args)?;
        let result = self.download_file(&remote_path, Path::new(&local_path), options)?;
        build_transfer_result(lua, result)
    }

    fn close(&self) -> mlua::Result<()> {
        self.connection
            .close()
            .map_err(|err| ssh_error(CLOSE_SIGNATURE, err))
    }

    pub(crate) fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        options: TransferOptions,
    ) -> mlua::Result<TransferResult> {
        self.connection
            .upload_file(local_path, remote_path, options)
            .map_err(|err| ssh_error(UPLOAD_SIGNATURE, err))
    }

    pub(crate) fn download_file(
        &self,
        remote_path: &str,
        local_path: &Path,
        options: TransferOptions,
    ) -> mlua::Result<TransferResult> {
        self.connection
            .download_file(remote_path, local_path, options)
            .map_err(|err| ssh_error(DOWNLOAD_SIGNATURE, err))
    }
}

impl LuaSshPath {
    pub(crate) fn connection(&self) -> LuaSshConnection {
        self.connection.clone()
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    fn matches_connection(&self, connection: &LuaSshConnection) -> bool {
        self.connection.matches_connection(connection)
    }

    fn exists(&self) -> mlua::Result<bool> {
        self.connection
            .connection
            .exists(self.path())
            .map_err(|err| ssh_error(REMOTE_PATH_EXISTS_SIGNATURE, err))
    }

    fn is_file(&self) -> mlua::Result<bool> {
        self.connection
            .connection
            .is_file(self.path())
            .map_err(|err| ssh_error(REMOTE_PATH_IS_FILE_SIGNATURE, err))
    }

    fn is_dir(&self) -> mlua::Result<bool> {
        self.connection
            .connection
            .is_dir(self.path())
            .map_err(|err| ssh_error(REMOTE_PATH_IS_DIR_SIGNATURE, err))
    }
}

fn parse_connect_request(value: Value) -> mlua::Result<SshConnectRequest> {
    match value {
        Value::String(target) => Ok(SshConnectRequest {
            target: target.to_str()?.to_string(),
            ..SshConnectRequest::default()
        }),
        Value::Table(options) => Ok(SshConnectRequest {
            target: options.get::<Option<String>>("target")?.unwrap_or_default(),
            host: options.get("host")?,
            user: options.get("user")?,
            port: options
                .get::<Option<i64>>("port")?
                .map(parse_port)
                .transpose()?,
            auth: parse_auth_request(&options)?,
            host_key: parse_host_key_request(&options)?,
            connect_timeout_ms: options
                .get::<Option<i64>>("connect_timeout_ms")?
                .map(|value| parse_positive_u64(value, CONNECT_SIGNATURE, "connect_timeout_ms"))
                .transpose()?,
            keepalive_interval_ms: options
                .get::<Option<i64>>("keepalive_interval_ms")?
                .map(|value| parse_positive_u64(value, CONNECT_SIGNATURE, "keepalive_interval_ms"))
                .transpose()?,
        }),
        _ => Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} expects a string or an options table"
        ))),
    }
}

fn parse_auth_request(table: &Table) -> mlua::Result<Option<SshAuthRequest>> {
    let Some(auth) = table.get::<Option<Table>>("auth")? else {
        return Ok(None);
    };

    if let Some(password) = auth.get::<Option<String>>("password")? {
        return Ok(Some(SshAuthRequest::Password { password }));
    }

    if let Some(path) = auth.get::<Option<String>>("private_key_file")? {
        let passphrase = auth.get::<Option<String>>("private_key_passphrase")?;
        return Ok(Some(SshAuthRequest::PrivateKeyFile { path, passphrase }));
    }

    Ok(None)
}

fn parse_host_key_request(table: &Table) -> mlua::Result<Option<SshHostKeyRequest>> {
    let Some(host_key) = table.get::<Option<Table>>("host_key")? else {
        return Ok(None);
    };

    let verify = host_key
        .get::<Option<String>>("verify")?
        .unwrap_or_else(|| "known_hosts".to_string());

    match verify.as_str() {
        "known_hosts" => Ok(Some(SshHostKeyRequest::KnownHosts {
            path: host_key.get::<Option<String>>("known_hosts_file")?,
        })),
        "ignore" => Ok(Some(SshHostKeyRequest::Ignore)),
        _ => Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} `host_key.verify` must be `known_hosts` or `ignore`"
        ))),
    }
}

fn parse_upload_call(
    connection: &LuaSshConnection,
    args: Variadic<Value>,
) -> mlua::Result<(String, String, TransferOptions)> {
    if !(2..=3).contains(&args.len()) {
        return Err(mlua::Error::runtime(format!(
            "{UPLOAD_SIGNATURE} expects 2 or 3 arguments"
        )));
    }

    let local_path = match &args[0] {
        Value::String(value) => value.to_str()?.to_string(),
        _ => {
            return Err(mlua::Error::runtime(format!(
                "{UPLOAD_SIGNATURE} `local_path` must be a string"
            )));
        }
    };
    ensure_non_empty_string(&local_path, UPLOAD_SIGNATURE, "local_path")?;

    let remote_path = parse_connection_remote_path_arg(
        connection,
        args[1].clone(),
        UPLOAD_SIGNATURE,
        "remote_path",
    )?;

    let options = parse_transfer_options(args.get(2).cloned(), UPLOAD_SIGNATURE)?;
    Ok((local_path, remote_path, options))
}

fn parse_download_call(
    connection: &LuaSshConnection,
    args: Variadic<Value>,
) -> mlua::Result<(String, String, TransferOptions)> {
    if !(2..=3).contains(&args.len()) {
        return Err(mlua::Error::runtime(format!(
            "{DOWNLOAD_SIGNATURE} expects 2 or 3 arguments"
        )));
    }

    let remote_path = parse_connection_remote_path_arg(
        connection,
        args[0].clone(),
        DOWNLOAD_SIGNATURE,
        "remote_path",
    )?;

    let local_path = match &args[1] {
        Value::String(value) => value.to_str()?.to_string(),
        _ => {
            return Err(mlua::Error::runtime(format!(
                "{DOWNLOAD_SIGNATURE} `local_path` must be a string"
            )));
        }
    };
    ensure_non_empty_string(&local_path, DOWNLOAD_SIGNATURE, "local_path")?;

    let options = parse_transfer_options(args.get(2).cloned(), DOWNLOAD_SIGNATURE)?;
    Ok((remote_path, local_path, options))
}

fn parse_connection_remote_path_arg(
    connection: &LuaSshConnection,
    value: Value,
    context: &str,
    field: &str,
) -> mlua::Result<String> {
    match value {
        Value::String(path) => {
            let path = path.to_str()?.to_string();
            ensure_non_empty_string(&path, context, field)?;
            Ok(path)
        }
        Value::UserData(_) => {
            let remote = parse_remote_path_value(value, context, field)?;
            if !remote.matches_connection(connection) {
                return Err(mlua::Error::runtime(format!(
                    "{context} `{field}` must belong to the current connection"
                )));
            }
            Ok(remote.path().to_string())
        }
        _ => Err(mlua::Error::runtime(format!(
            "{context} `{field}` must be a string or a remote path from `conn:path(...)`"
        ))),
    }
}

pub(crate) fn parse_remote_path_value(
    value: Value,
    context: &str,
    field: &str,
) -> mlua::Result<LuaSshPath> {
    match value {
        Value::String(_) => Err(mlua::Error::runtime(format!(
            "{context} `{field}` must be a remote path from `conn:path(...)`"
        ))),
        Value::UserData(userdata) => {
            if !userdata.is::<LuaSshPath>() {
                return Err(mlua::Error::runtime(format!(
                    "{context} `{field}` must be a remote path from `conn:path(...)`"
                )));
            }
            Ok(userdata.borrow::<LuaSshPath>()?.clone())
        }
        _ => Err(mlua::Error::runtime(format!(
            "{context} `{field}` must be a remote path from `conn:path(...)`"
        ))),
    }
}

pub(crate) fn parse_transfer_options(
    value: Option<Value>,
    context: &str,
) -> mlua::Result<TransferOptions> {
    let Some(value) = value else {
        return Ok(TransferOptions::default());
    };

    match value {
        Value::Nil => Ok(TransferOptions::default()),
        Value::Table(table) => Ok(TransferOptions {
            parents: table.get::<Option<bool>>("parents")?.unwrap_or(false),
            overwrite: table.get::<Option<bool>>("overwrite")?.unwrap_or(true),
            echo: table.get::<Option<bool>>("echo")?.unwrap_or(false),
        }),
        _ => Err(mlua::Error::runtime(format!(
            "{context} options must be a table"
        ))),
    }
}

pub(crate) fn build_transfer_result(lua: &Lua, result: TransferResult) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let bytes = i64::try_from(result.bytes).map_err(|_| {
        mlua::Error::runtime("ptool transfer result is too large to represent in Lua")
    })?;
    table.set("bytes", bytes)?;
    table.set("from", result.from)?;
    table.set("to", result.to)?;
    Ok(table)
}

fn parse_run_call(args: Variadic<Value>) -> mlua::Result<SshExecOptions> {
    parse_run_call_with_defaults(args, RUN_SIGNATURE, RUN_STREAM_DEFAULTS)
}

fn parse_run_capture_call(args: Variadic<Value>) -> mlua::Result<SshExecOptions> {
    parse_run_call_with_defaults(args, RUN_CAPTURE_SIGNATURE, RUN_CAPTURE_STREAM_DEFAULTS)
}

fn parse_run_call_with_defaults(
    args: Variadic<Value>,
    context: &str,
    stream_defaults: ExecStreamDefaults,
) -> mlua::Result<SshExecOptions> {
    match args.len() {
        0 => Err(mlua::Error::runtime(format!(
            "{context} requires arguments"
        ))),
        1 => match args.first() {
            Some(Value::String(command)) => Ok(SshExecOptions {
                command: command.to_str()?.to_string(),
                display_cwd: None,
                stdin: None,
                echo: true,
                stdout: stream_defaults.stdout,
                stderr: stream_defaults.stderr,
                check: false,
            }),
            Some(Value::Table(options)) => {
                parse_run_options_table(options.clone(), context, stream_defaults)
            }
            _ => Err(mlua::Error::runtime(format!(
                "{context} expects a command string or an options table"
            ))),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(command_or_cmdline)), Some(Value::Table(second))) => {
                if looks_like_array_table(second, context)? {
                    build_exec_options_from_parts(
                        command_or_cmdline.to_str()?.to_string(),
                        parse_string_list(second)?,
                        None,
                        context,
                        stream_defaults,
                    )
                } else {
                    let options = parse_exec_overrides(second.clone(), context)?;
                    build_exec_options_from_cmdline(
                        command_or_cmdline.to_str()?.to_string(),
                        options,
                        stream_defaults,
                    )
                }
            }
            (Some(Value::String(command)), Some(Value::String(argsline))) => {
                let args = parse_argsline(&argsline.to_str()?, context)?;
                build_exec_options_from_parts(
                    command.to_str()?.to_string(),
                    args,
                    None,
                    context,
                    stream_defaults,
                )
            }
            _ => Err(mlua::Error::runtime(format!("{context} invalid arguments"))),
        },
        3 => match (args.first(), args.get(1), args.get(2)) {
            (
                Some(Value::String(command)),
                Some(Value::String(argsline)),
                Some(Value::Table(options)),
            ) => {
                let args = parse_argsline(&argsline.to_str()?, context)?;
                let options = parse_exec_overrides(options.clone(), context)?;
                build_exec_options_from_parts(
                    command.to_str()?.to_string(),
                    args,
                    Some(options),
                    context,
                    stream_defaults,
                )
            }
            (
                Some(Value::String(command)),
                Some(Value::Table(args)),
                Some(Value::Table(options)),
            ) => {
                let args = parse_string_list(args)?;
                let options = parse_exec_overrides(options.clone(), context)?;
                build_exec_options_from_parts(
                    command.to_str()?.to_string(),
                    args,
                    Some(options),
                    context,
                    stream_defaults,
                )
            }
            _ => Err(mlua::Error::runtime(format!("{context} invalid arguments"))),
        },
        _ => Err(mlua::Error::runtime(format!(
            "{context} accepts at most 3 arguments"
        ))),
    }
}

struct ExecOverrides {
    stdin: Option<Vec<u8>>,
    echo: Option<bool>,
    stdout: Option<SshStreamMode>,
    stderr: Option<SshStreamMode>,
    check: Option<bool>,
}

fn parse_exec_overrides(options: Table, context: &str) -> mlua::Result<ExecOverrides> {
    if options.get::<Option<Value>>("cmd")?.is_some()
        || options.get::<Option<Value>>("args")?.is_some()
        || options.get::<Option<Value>>("env")?.is_some()
        || options.get::<Option<Value>>("cwd")?.is_some()
    {
        return Err(mlua::Error::runtime(format!(
            "{context} options table does not allow `cmd`, `args`, `cwd`, or `env`"
        )));
    }

    Ok(ExecOverrides {
        stdin: parse_stdin(options.get::<Option<Value>>("stdin")?, context)?,
        echo: options.get::<Option<bool>>("echo")?,
        stdout: parse_stream_mode(options.get::<Option<String>>("stdout")?, "stdout", context)?,
        stderr: parse_stream_mode(options.get::<Option<String>>("stderr")?, "stderr", context)?,
        check: options.get::<Option<bool>>("check")?,
    })
}

fn build_exec_options_from_cmdline(
    command: String,
    options: ExecOverrides,
    stream_defaults: ExecStreamDefaults,
) -> mlua::Result<SshExecOptions> {
    Ok(SshExecOptions {
        command,
        display_cwd: None,
        stdin: options.stdin,
        echo: options.echo.unwrap_or(true),
        stdout: options.stdout.unwrap_or(stream_defaults.stdout),
        stderr: options.stderr.unwrap_or(stream_defaults.stderr),
        check: options.check.unwrap_or(false),
    })
}

fn build_exec_options_from_parts(
    cmd: String,
    args: Vec<String>,
    options: Option<ExecOverrides>,
    context: &str,
    stream_defaults: ExecStreamDefaults,
) -> mlua::Result<SshExecOptions> {
    let command = quote_words_for_shell(
        std::iter::once(cmd.as_str()).chain(args.iter().map(String::as_str)),
        context,
    )?;
    let options = options.unwrap_or(ExecOverrides {
        stdin: None,
        echo: None,
        stdout: None,
        stderr: None,
        check: None,
    });

    Ok(SshExecOptions {
        command,
        display_cwd: None,
        stdin: options.stdin,
        echo: options.echo.unwrap_or(true),
        stdout: options.stdout.unwrap_or(stream_defaults.stdout),
        stderr: options.stderr.unwrap_or(stream_defaults.stderr),
        check: options.check.unwrap_or(false),
    })
}

fn parse_run_options_table(
    options: Table,
    context: &str,
    stream_defaults: ExecStreamDefaults,
) -> mlua::Result<SshExecOptions> {
    let cmd: Option<String> = options.get("cmd")?;
    let args = options.get::<Option<Table>>("args")?;
    let env = options.get::<Option<Table>>("env")?;
    let cwd = options.get::<Option<String>>("cwd")?;

    let base_command = match (cmd, args) {
        (Some(cmd), Some(args)) => quote_words_for_shell(
            std::iter::once(cmd.as_str())
                .chain(parse_string_list(&args)?.iter().map(String::as_str)),
            context,
        )?,
        (Some(cmd), None) => try_quote(&cmd)
            .map(|value| value.into_owned())
            .map_err(|err| mlua::Error::runtime(format!("{context} invalid command: {err}")))?,
        (None, _) => {
            return Err(mlua::Error::runtime(format!(
                "{context} options mode requires `cmd`"
            )));
        }
    };

    let display_cwd = cwd.clone();
    let command = wrap_remote_command(base_command, cwd, env, context)?;
    Ok(SshExecOptions {
        command,
        display_cwd,
        stdin: parse_stdin(options.get::<Option<Value>>("stdin")?, context)?,
        echo: options.get::<Option<bool>>("echo")?.unwrap_or(true),
        stdout: parse_stream_mode(options.get::<Option<String>>("stdout")?, "stdout", context)?
            .unwrap_or(stream_defaults.stdout),
        stderr: parse_stream_mode(options.get::<Option<String>>("stderr")?, "stderr", context)?
            .unwrap_or(stream_defaults.stderr),
        check: options.get::<Option<bool>>("check")?.unwrap_or(false),
    })
}

fn wrap_remote_command(
    command: String,
    cwd: Option<String>,
    env: Option<Table>,
    context: &str,
) -> mlua::Result<String> {
    let mut prefixes = Vec::new();

    if let Some(cwd) = cwd {
        let quoted = try_quote(&cwd)
            .map_err(|err| mlua::Error::runtime(format!("{context} invalid `cwd`: {err}")))?;
        prefixes.push(format!("cd {quoted}"));
    }

    if let Some(env) = env {
        let mut env_parts = Vec::new();
        for pair in env.pairs::<String, String>() {
            let (key, value) = pair?;
            if key.is_empty() {
                return Err(mlua::Error::runtime(format!(
                    "{context} `env` keys must not be empty"
                )));
            }
            let quoted = try_quote(&value).map_err(|err| {
                mlua::Error::runtime(format!("{context} invalid env value for `{key}`: {err}"))
            })?;
            env_parts.push(format!("{key}={quoted}"));
        }
        if !env_parts.is_empty() {
            prefixes.push(format!("export {}", env_parts.join(" ")));
        }
    }

    if prefixes.is_empty() {
        return Ok(command);
    }

    let mut wrapped = prefixes.join(" && ");
    wrapped.push_str(" && ");
    wrapped.push_str(&command);
    Ok(wrapped)
}

fn build_exec_result(lua: &Lua, result: SshExecResult, target: String) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    let ok = result.code == Some(0);
    table.set("ok", ok)?;
    table.set("code", result.code)?;
    table.set("target", target)?;
    if let Some(stdout) = result.stdout {
        table.set("stdout", stdout)?;
    }
    if let Some(stderr) = result.stderr {
        table.set("stderr", stderr)?;
    }
    table.set(
        "assert_ok",
        lua.create_function(|_, this: Table| {
            let ok = this.get::<bool>("ok")?;
            if ok {
                return Ok(());
            }
            let target: String = this.get("target")?;
            let code: Option<i64> = this.get("code")?;
            let stderr: Option<String> = this.get("stderr").ok();
            let mut message = format!("ptool.ssh command on `{target}` failed");
            if let Some(code) = code {
                message.push_str(&format!(" with status {code}"));
            }
            if let Some(stderr) = stderr.filter(|value| !value.is_empty()) {
                message.push_str(&format!(": {}", stderr.trim_end()));
            }
            Err(mlua::Error::runtime(message))
        })?,
    )?;
    Ok(table)
}

fn ensure_non_empty_string(value: &str, context: &str, field: &str) -> mlua::Result<()> {
    if value.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{context} `{field}` must not be empty"
        )));
    }
    Ok(())
}

fn parse_port(value: i64) -> mlua::Result<u16> {
    if !(1..=65535).contains(&value) {
        return Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} `port` must be between 1 and 65535"
        )));
    }
    Ok(value as u16)
}

fn parse_positive_u64(value: i64, context: &str, field: &str) -> mlua::Result<u64> {
    if value <= 0 {
        return Err(mlua::Error::runtime(format!(
            "{context} `{field}` must be > 0"
        )));
    }
    u64::try_from(value)
        .map_err(|_| mlua::Error::runtime(format!("{context} `{field}` is too large")))
}

fn looks_like_array_table(table: &Table, context: &str) -> mlua::Result<bool> {
    let mut count = 0usize;
    let mut max = 0usize;
    for pair in table.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let Value::Integer(index) = key else {
            return Ok(false);
        };
        if index <= 0 {
            return Ok(false);
        }
        let index = usize::try_from(index)
            .map_err(|_| mlua::Error::runtime(format!("{context} array argument is too large")))?;
        count += 1;
        max = max.max(index);
    }
    Ok(count == max)
}

fn parse_string_list(table: &Table) -> mlua::Result<Vec<String>> {
    let mut values = Vec::new();
    for value in table.sequence_values::<String>() {
        values.push(value?);
    }
    Ok(values)
}

fn parse_argsline(input: &str, context: &str) -> mlua::Result<Vec<String>> {
    shlex::split(input)
        .ok_or_else(|| mlua::Error::runtime(format!("{context} failed to parse args string")))
}

fn quote_words_for_shell<'a>(
    words: impl IntoIterator<Item = &'a str>,
    context: &str,
) -> mlua::Result<String> {
    let mut quoted = Vec::new();
    for word in words {
        quoted.push(
            try_quote(word)
                .map(|value| value.into_owned())
                .map_err(|err| {
                    mlua::Error::runtime(format!("{context} invalid argument: {err}"))
                })?,
        );
    }
    Ok(quoted.join(" "))
}

fn parse_stdin(value: Option<Value>, context: &str) -> mlua::Result<Option<Vec<u8>>> {
    match value {
        None | Some(Value::Nil) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.as_bytes().to_vec())),
        Some(_) => Err(mlua::Error::runtime(format!(
            "{context} `stdin` must be a string"
        ))),
    }
}

fn parse_stream_mode(
    value: Option<String>,
    field: &str,
    context: &str,
) -> mlua::Result<Option<SshStreamMode>> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value.as_str() {
        "inherit" => Ok(Some(SshStreamMode::Inherit)),
        "capture" => Ok(Some(SshStreamMode::Capture)),
        "null" => Ok(Some(SshStreamMode::Null)),
        _ => Err(mlua::Error::runtime(format!(
            "{context} `{field}` must be `inherit`, `capture`, or `null`"
        ))),
    }
}

fn ssh_error(context: &str, err: EngineError) -> mlua::Error {
    mlua::Error::runtime(format!("{context} {}", err.msg))
}
