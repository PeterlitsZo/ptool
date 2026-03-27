use mlua::{Lua, Table, UserData, UserDataFields, UserDataMethods, Value, Variadic};
use russh::keys::{self, PrivateKeyWithHashAlg};
use russh::{ChannelMsg, Disconnect, client};
use shlex::try_quote;
use std::borrow::Cow;
use std::cell::RefCell;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

const CONNECT_SIGNATURE: &str = "ptool.ssh.connect(target_or_options)";
const RUN_SIGNATURE: &str = "ptool.ssh.Connection:run(...)";
const CLOSE_SIGNATURE: &str = "ptool.ssh.Connection:close()";
const PATH_SIGNATURE: &str = "ptool.ssh.Connection:path(path)";
const UPLOAD_SIGNATURE: &str = "ptool.ssh.Connection:upload(local_path, remote_path[, options])";
const DOWNLOAD_SIGNATURE: &str =
    "ptool.ssh.Connection:download(remote_path, local_path[, options])";

#[derive(Clone)]
pub(crate) struct LuaSshConnection {
    runtime: Rc<Runtime>,
    state: Rc<RefCell<ConnectionState>>,
}

#[derive(Clone)]
pub(crate) struct LuaSshPath {
    connection: LuaSshConnection,
    path: String,
}

struct ConnectionState {
    session: Option<client::Handle<SshClientHandler>>,
    info: ConnectionInfo,
}

#[derive(Clone)]
struct ConnectionInfo {
    host: String,
    user: String,
    port: u16,
    target: String,
}

#[derive(Clone)]
struct SshClientHandler {
    policy: HostKeyPolicy,
    host: String,
    port: u16,
}

#[derive(Clone)]
enum HostKeyPolicy {
    KnownHosts { path: Option<PathBuf> },
    Ignore,
}

enum AuthMethod {
    PrivateKeys { keys: Vec<PrivateKeyOption> },
    Password { password: String },
}

struct PrivateKeyOption {
    path: PathBuf,
    passphrase: Option<String>,
    required: bool,
}

struct ConnectOptions {
    host: String,
    connect_host: String,
    user: String,
    port: u16,
    auth: AuthMethod,
    host_key: HostKeyPolicy,
    connect_timeout_ms: u64,
    keepalive_interval_ms: Option<u64>,
}

#[derive(Default)]
struct SshConfigOptions {
    host: Option<String>,
    hostname: Option<String>,
    user: Option<String>,
    port: Option<u16>,
    identity_files: Vec<PathBuf>,
    user_known_hosts_files: Vec<PathBuf>,
    strict_host_key_checking: Option<String>,
    connect_timeout_ms: Option<u64>,
    server_alive_interval_ms: Option<u64>,
}

struct ExecOptions {
    command: String,
    stdin: Option<Vec<u8>>,
    echo: bool,
    stdout: StreamMode,
    stderr: StreamMode,
    check: bool,
}

#[derive(Clone, Copy)]
enum StreamMode {
    Inherit,
    Capture,
    Null,
}

struct ExecResult {
    code: Option<i64>,
    stdout: Option<String>,
    stderr: Option<String>,
}

struct BinaryExecResult {
    code: Option<i64>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TransferOptions {
    pub(crate) parents: bool,
    pub(crate) overwrite: bool,
    pub(crate) echo: bool,
}

impl Default for TransferOptions {
    fn default() -> Self {
        Self {
            parents: false,
            overwrite: true,
            echo: false,
        }
    }
}

pub(crate) struct TransferResult {
    pub(crate) bytes: u64,
    pub(crate) from: String,
    pub(crate) to: String,
}

pub(crate) fn connect(
    value: Value,
    current_dir: &Path,
    runtime: Rc<Runtime>,
) -> mlua::Result<LuaSshConnection> {
    let options = parse_connect_options(value, current_dir)?;
    let info = ConnectionInfo {
        target: format!("{}@{}:{}", options.user, options.host, options.port),
        host: options.host.clone(),
        user: options.user.clone(),
        port: options.port,
    };

    let handler = SshClientHandler {
        policy: options.host_key.clone(),
        host: options.host.clone(),
        port: options.port,
    };

    let config = build_client_config(&options);
    let addr = (options.connect_host.as_str(), options.port);
    let mut session = runtime
        .block_on(async { client::connect(config, addr, handler).await })
        .map_err(|err| ssh_error(CONNECT_SIGNATURE, err))?;

    authenticate_session(&runtime, &mut session, &options)?;

    Ok(LuaSshConnection {
        runtime,
        state: Rc::new(RefCell::new(ConnectionState {
            session: Some(session),
            info,
        })),
    })
}

impl UserData for LuaSshConnection {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("host", |_, this| Ok(this.info().host.clone()));
        fields.add_field_method_get("user", |_, this| Ok(this.info().user.clone()));
        fields.add_field_method_get("port", |_, this| Ok(i64::from(this.info().port)));
        fields.add_field_method_get("target", |_, this| Ok(this.info().target.clone()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("run", |lua, this, args: Variadic<Value>| {
            this.run(lua, args)
        });
        methods.add_method("path", |_, this, path: String| this.path(path));
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
        fields.add_field_method_get("host", |_, this| Ok(this.connection.info().host.clone()));
        fields.add_field_method_get("user", |_, this| Ok(this.connection.info().user.clone()));
        fields.add_field_method_get("port", |_, this| Ok(i64::from(this.connection.info().port)));
        fields.add_field_method_get("target", |_, this| {
            Ok(this.connection.info().target.clone())
        });
        fields.add_field_method_get("path", |_, this| Ok(this.path.clone()));
    }
}

impl LuaSshConnection {
    fn info(&self) -> ConnectionInfo {
        self.state.borrow().info.clone()
    }

    fn run(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        let options = parse_run_call(args)?;
        let result = self.exec_internal(options)?;
        build_exec_result(lua, result, self.info().target)
    }

    fn path(&self, path: String) -> mlua::Result<LuaSshPath> {
        ensure_non_empty_string(&path, PATH_SIGNATURE, "path")?;
        Ok(LuaSshPath {
            connection: self.clone(),
            path,
        })
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
        let session = {
            let mut state = self.state.borrow_mut();
            state.session.take()
        };

        if let Some(session) = session {
            self.runtime
                .block_on(async {
                    session
                        .disconnect(Disconnect::ByApplication, "", "English")
                        .await
                })
                .map_err(|err| ssh_error(CLOSE_SIGNATURE, err))?;
        }

        Ok(())
    }

    fn exec_internal(&self, options: ExecOptions) -> mlua::Result<ExecResult> {
        let info = self.info();
        if options.echo {
            println!("[ssh {}] {}", info.target, options.command);
        }

        let mut state = self.state.borrow_mut();
        let session = state.session.as_mut().ok_or_else(|| {
            mlua::Error::runtime(format!("{RUN_SIGNATURE} cannot use a closed connection"))
        })?;

        let command = options.command.clone();
        self.runtime.block_on(async {
            let mut channel = session.channel_open_session().await.map_err(|err| {
                ssh_error(
                    RUN_SIGNATURE,
                    format!("failed to open session channel for `{command}`: {err}"),
                )
            })?;
            channel
                .exec(true, command.as_bytes())
                .await
                .map_err(|err| {
                    ssh_error(
                        RUN_SIGNATURE,
                        format!("failed to execute remote command `{command}`: {err}"),
                    )
                })?;

            if let Some(stdin) = options.stdin {
                let mut cursor = std::io::Cursor::new(stdin);
                channel.data(&mut cursor).await.map_err(|err| {
                    ssh_error(
                        RUN_SIGNATURE,
                        format!("failed to write stdin for `{command}`: {err}"),
                    )
                })?;
                channel.eof().await.map_err(|err| {
                    ssh_error(
                        RUN_SIGNATURE,
                        format!("failed to send EOF for `{command}`: {err}"),
                    )
                })?;
            }

            let mut stdout_bytes = Vec::new();
            let mut stderr_bytes = Vec::new();
            let mut code = None;

            while let Some(msg) = channel.wait().await {
                match msg {
                    ChannelMsg::Data { data } => {
                        handle_output(&mut stdout_bytes, options.stdout, &data)
                    }
                    ChannelMsg::ExtendedData { data, ext: 1 } => {
                        handle_output(&mut stderr_bytes, options.stderr, &data)
                    }
                    ChannelMsg::ExitStatus { exit_status } => {
                        code = Some(i64::from(exit_status));
                    }
                    ChannelMsg::ExitSignal { .. } => {
                        code = None;
                    }
                    _ => {}
                }
            }

            let stdout = bytes_to_captured_string(stdout_bytes, options.stdout);
            let stderr = bytes_to_captured_string(stderr_bytes, options.stderr);

            let result = ExecResult {
                code,
                stdout,
                stderr,
            };

            if options.check && result.code != Some(0) {
                return Err(build_exec_failed_error(&info.target, &command, &result));
            }

            Ok(result)
        })
    }

    pub(crate) fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        options: TransferOptions,
    ) -> mlua::Result<TransferResult> {
        ensure_local_file(local_path, UPLOAD_SIGNATURE, "local_path")?;
        ensure_non_empty_string(remote_path, UPLOAD_SIGNATURE, "remote_path")?;

        let content = std::fs::read(local_path).map_err(|err| {
            mlua::Error::runtime(format!(
                "{UPLOAD_SIGNATURE} failed to read `{}`: {err}",
                local_path.display()
            ))
        })?;

        if options.echo {
            println!(
                "[ssh upload {}] {} -> {}",
                self.info().target,
                local_path.display(),
                remote_path
            );
        }

        let command = build_upload_command(remote_path, options)?;
        self.exec_binary_internal(UPLOAD_SIGNATURE, &command, Some(content), false)?;

        Ok(TransferResult {
            bytes: std::fs::metadata(local_path)
                .map(|metadata| metadata.len())
                .unwrap_or(0),
            from: local_path.display().to_string(),
            to: format!("{}:{}", self.info().target, remote_path),
        })
    }

    pub(crate) fn download_file(
        &self,
        remote_path: &str,
        local_path: &Path,
        options: TransferOptions,
    ) -> mlua::Result<TransferResult> {
        ensure_non_empty_string(remote_path, DOWNLOAD_SIGNATURE, "remote_path")?;

        prepare_local_destination(local_path, options, DOWNLOAD_SIGNATURE, "local_path")?;

        if options.echo {
            println!(
                "[ssh download {}] {} -> {}",
                self.info().target,
                remote_path,
                local_path.display()
            );
        }

        let command = build_download_command(remote_path)?;
        let result = self.exec_binary_internal(DOWNLOAD_SIGNATURE, &command, None, true)?;

        write_local_file(
            local_path,
            &result.stdout,
            options,
            DOWNLOAD_SIGNATURE,
            "local_path",
        )?;

        Ok(TransferResult {
            bytes: u64::try_from(result.stdout.len()).map_err(|_| {
                mlua::Error::runtime(format!(
                    "{DOWNLOAD_SIGNATURE} downloaded file is too large to report size"
                ))
            })?,
            from: format!("{}:{}", self.info().target, remote_path),
            to: local_path.display().to_string(),
        })
    }

    fn exec_binary_internal(
        &self,
        context: &str,
        command: &str,
        stdin: Option<Vec<u8>>,
        capture_stdout: bool,
    ) -> mlua::Result<BinaryExecResult> {
        let info = self.info();

        let mut state = self.state.borrow_mut();
        let session = state.session.as_mut().ok_or_else(|| {
            mlua::Error::runtime(format!("{context} cannot use a closed connection"))
        })?;

        let command = command.to_string();
        self.runtime.block_on(async {
            let mut channel = session.channel_open_session().await.map_err(|err| {
                ssh_error(
                    context,
                    format!("failed to open session channel for `{command}`: {err}"),
                )
            })?;
            channel
                .exec(true, command.as_bytes())
                .await
                .map_err(|err| {
                    ssh_error(
                        context,
                        format!("failed to execute remote command `{command}`: {err}"),
                    )
                })?;

            if let Some(stdin) = stdin {
                let mut cursor = std::io::Cursor::new(stdin);
                channel.data(&mut cursor).await.map_err(|err| {
                    ssh_error(
                        context,
                        format!("failed to write stdin for `{command}`: {err}"),
                    )
                })?;
                channel.eof().await.map_err(|err| {
                    ssh_error(
                        context,
                        format!("failed to send EOF for `{command}`: {err}"),
                    )
                })?;
            }

            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut code = None;

            while let Some(msg) = channel.wait().await {
                match msg {
                    ChannelMsg::Data { data } => {
                        if capture_stdout {
                            stdout.extend_from_slice(&data);
                        }
                    }
                    ChannelMsg::ExtendedData { data, ext: 1 } => stderr.extend_from_slice(&data),
                    ChannelMsg::ExitStatus { exit_status } => code = Some(i64::from(exit_status)),
                    ChannelMsg::ExitSignal { .. } => code = None,
                    _ => {}
                }
            }

            let result = BinaryExecResult {
                code,
                stdout,
                stderr,
            };

            if result.code != Some(0) {
                return Err(build_binary_exec_failed_error(
                    context,
                    &info.target,
                    &command,
                    result.code,
                    &result.stderr,
                ));
            }

            Ok(result)
        })
    }
}

impl LuaSshPath {
    pub(crate) fn connection(&self) -> LuaSshConnection {
        self.connection.clone()
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
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
        Value::String(path) => {
            let path = path.to_str()?.to_string();
            ensure_non_empty_string(&path, context, field)?;
            Err(mlua::Error::runtime(format!(
                "{context} `{field}` must be a remote path from `conn:path(...)`"
            )))
        }
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

fn build_upload_command(remote_path: &str, options: TransferOptions) -> mlua::Result<String> {
    let mut prefixes = Vec::new();
    if options.parents
        && let Some(parent) = remote_parent_path(remote_path)
    {
        prefixes.push(format!(
            "mkdir -p {}",
            shell_quote(parent, UPLOAD_SIGNATURE, "remote_path")?
        ));
    }
    if !options.overwrite {
        prefixes.push(format!(
            "test ! -e {}",
            shell_quote(remote_path, UPLOAD_SIGNATURE, "remote_path")?
        ));
    }

    let mut command = prefixes.join(" && ");
    if !command.is_empty() {
        command.push_str(" && ");
    }
    command.push_str("cat > ");
    command.push_str(&shell_quote(remote_path, UPLOAD_SIGNATURE, "remote_path")?);
    Ok(command)
}

fn build_download_command(remote_path: &str) -> mlua::Result<String> {
    Ok(format!(
        "cat {}",
        shell_quote(remote_path, DOWNLOAD_SIGNATURE, "remote_path")?
    ))
}

fn shell_quote(value: &str, context: &str, field: &str) -> mlua::Result<String> {
    try_quote(value)
        .map(|value| value.into_owned())
        .map_err(|err| mlua::Error::runtime(format!("{context} invalid `{field}`: {err}")))
}

fn remote_parent_path(path: &str) -> Option<&str> {
    let parent = Path::new(path).parent()?;
    if parent.as_os_str().is_empty() {
        return None;
    }
    parent.to_str()
}

fn ensure_non_empty_string(value: &str, context: &str, field: &str) -> mlua::Result<()> {
    if value.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{context} `{field}` must not be empty"
        )));
    }
    Ok(())
}

fn ensure_local_file(path: &Path, context: &str, field: &str) -> mlua::Result<()> {
    let metadata = std::fs::metadata(path).map_err(|err| {
        mlua::Error::runtime(format!(
            "{context} failed to access `{}`: {err}",
            path.display()
        ))
    })?;
    if !metadata.is_file() {
        return Err(mlua::Error::runtime(format!(
            "{context} `{field}` must be a file: `{}`",
            path.display()
        )));
    }
    Ok(())
}

fn prepare_local_destination(
    path: &Path,
    options: TransferOptions,
    context: &str,
    field: &str,
) -> mlua::Result<()> {
    if options.parents
        && let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|err| {
            mlua::Error::runtime(format!(
                "{context} failed to create parent directory `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    if !options.overwrite && path.exists() {
        return Err(mlua::Error::runtime(format!(
            "{context} `{field}` already exists: `{}`",
            path.display()
        )));
    }

    Ok(())
}

fn write_local_file(
    path: &Path,
    content: &[u8],
    options: TransferOptions,
    context: &str,
    field: &str,
) -> mlua::Result<()> {
    let mut open_options = std::fs::OpenOptions::new();
    open_options.write(true).create(true).truncate(true);
    if !options.overwrite {
        open_options.create_new(true);
    }

    let mut file = open_options.open(path).map_err(|err| {
        mlua::Error::runtime(format!(
            "{context} failed to open `{}`: {err}",
            path.display()
        ))
    })?;

    use std::io::Write as _;
    file.write_all(content).map_err(|err| {
        mlua::Error::runtime(format!(
            "{context} failed to write `{}`: {err}",
            path.display()
        ))
    })?;
    file.flush().map_err(|err| {
        mlua::Error::runtime(format!(
            "{context} failed to flush `{}`: {err}",
            path.display()
        ))
    })?;

    if path.is_dir() {
        return Err(mlua::Error::runtime(format!(
            "{context} `{field}` must not be a directory: `{}`",
            path.display()
        )));
    }

    Ok(())
}

fn build_binary_exec_failed_error(
    context: &str,
    target: &str,
    command: &str,
    code: Option<i64>,
    stderr: &[u8],
) -> mlua::Error {
    let mut message = format!("{context} remote command `{command}` on `{target}` failed");
    if let Some(code) = code {
        message.push_str(&format!(" with status {code}"));
    }
    let stderr = String::from_utf8_lossy(stderr);
    if !stderr.trim().is_empty() {
        message.push_str(&format!(": {}", stderr.trim_end()));
    }
    mlua::Error::runtime(message)
}

impl LuaSshPath {
    fn matches_connection(&self, connection: &LuaSshConnection) -> bool {
        Rc::ptr_eq(&self.connection.state, &connection.state)
    }
}

impl client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        match &self.policy {
            HostKeyPolicy::Ignore => Ok(true),
            HostKeyPolicy::KnownHosts { path } => {
                let check = if let Some(path) = path {
                    keys::check_known_hosts_path(&self.host, self.port, server_public_key, path)
                } else {
                    keys::check_known_hosts(&self.host, self.port, server_public_key)
                };
                check.map_err(russh::Error::from)
            }
        }
    }
}

fn build_client_config(options: &ConnectOptions) -> Arc<client::Config> {
    Arc::new(client::Config {
        inactivity_timeout: Some(Duration::from_millis(options.connect_timeout_ms)),
        keepalive_interval: options.keepalive_interval_ms.map(Duration::from_millis),
        ..client::Config::default()
    })
}

fn authenticate_session(
    runtime: &Runtime,
    session: &mut client::Handle<SshClientHandler>,
    options: &ConnectOptions,
) -> mlua::Result<()> {
    let user = options.user.clone();
    let authenticated = match &options.auth {
        AuthMethod::PrivateKeys { keys: key_options } => {
            let hash_alg = runtime
                .block_on(session.best_supported_rsa_hash())
                .map_err(|err| ssh_error(CONNECT_SIGNATURE, err))?
                .flatten();
            let mut authenticated = false;
            let mut attempted = false;
            let mut last_load_error = None;

            for key_option in key_options {
                let key =
                    match keys::load_secret_key(&key_option.path, key_option.passphrase.as_deref())
                    {
                        Ok(key) => key,
                        Err(err) => {
                            let err = ssh_error(CONNECT_SIGNATURE, err);
                            if key_option.required {
                                return Err(err);
                            }
                            last_load_error = Some(err);
                            continue;
                        }
                    };
                attempted = true;
                let auth_result = runtime
                    .block_on(async {
                        session
                            .authenticate_publickey(
                                user.clone(),
                                PrivateKeyWithHashAlg::new(Arc::new(key), hash_alg),
                            )
                            .await
                    })
                    .map_err(|err| ssh_error(CONNECT_SIGNATURE, err))?;
                if auth_result.success() {
                    authenticated = true;
                    break;
                }
            }

            if attempted {
                authenticated
            } else if let Some(err) = last_load_error {
                return Err(err);
            } else {
                false
            }
        }
        AuthMethod::Password { password } => runtime
            .block_on(async { session.authenticate_password(user, password).await })
            .map_err(|err| ssh_error(CONNECT_SIGNATURE, err))?
            .success(),
    };

    if !authenticated {
        return Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} authentication failed for `{}`",
            options.user
        )));
    }

    Ok(())
}

fn parse_connect_options(value: Value, current_dir: &Path) -> mlua::Result<ConnectOptions> {
    match value {
        Value::String(target) => {
            let target = target.to_str()?.to_string();
            build_connect_options(target, None, current_dir)
        }
        Value::Table(options) => {
            let target: Option<String> = options.get("target")?;
            build_connect_options(target.unwrap_or_default(), Some(options), current_dir)
        }
        _ => Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} expects a string or an options table"
        ))),
    }
}

fn build_connect_options(
    target: String,
    table: Option<Table>,
    current_dir: &Path,
) -> mlua::Result<ConnectOptions> {
    let parsed = if target.is_empty() {
        ParsedTarget {
            host: None,
            user: None,
            port: None,
        }
    } else {
        parse_target_string(&target)?
    };

    let requested_host = table
        .as_ref()
        .and_then(|options| options.get::<Option<String>>("host").transpose())
        .transpose()?
        .or(parsed.host)
        .ok_or_else(|| mlua::Error::runtime(format!("{CONNECT_SIGNATURE} requires `host`")))?;

    let requested_user = table
        .as_ref()
        .and_then(|options| options.get::<Option<String>>("user").transpose())
        .transpose()?
        .or(parsed.user);

    let requested_port = table
        .as_ref()
        .and_then(|options| options.get::<Option<i64>>("port").transpose())
        .transpose()?
        .map(parse_port)
        .transpose()?
        .or(parsed.port);

    let ssh_config =
        resolve_ssh_config(&requested_host, requested_user.as_deref(), requested_port)?;

    let host = ssh_config
        .as_ref()
        .and_then(|config| config.host.clone())
        .unwrap_or_else(|| requested_host.clone());
    let connect_host = ssh_config
        .as_ref()
        .and_then(|config| config.hostname.clone())
        .unwrap_or_else(|| host.clone());

    let user = requested_user
        .or_else(|| ssh_config.as_ref().and_then(|config| config.user.clone()))
        .unwrap_or_else(default_ssh_user);

    let port = requested_port
        .or_else(|| ssh_config.as_ref().and_then(|config| config.port))
        .unwrap_or(22);

    let auth = parse_auth_options(table.as_ref(), current_dir, ssh_config.as_ref())?;
    let host_key = parse_host_key_options(table.as_ref(), current_dir, ssh_config.as_ref())?;
    let connect_timeout_ms = table
        .as_ref()
        .and_then(|options| options.get::<Option<i64>>("connect_timeout_ms").transpose())
        .transpose()?
        .map(|value| parse_positive_u64(value, CONNECT_SIGNATURE, "connect_timeout_ms"))
        .transpose()?
        .or_else(|| {
            ssh_config
                .as_ref()
                .and_then(|config| config.connect_timeout_ms)
        })
        .unwrap_or(10_000);
    let keepalive_interval_ms = table
        .as_ref()
        .and_then(|options| {
            options
                .get::<Option<i64>>("keepalive_interval_ms")
                .transpose()
        })
        .transpose()?
        .map(|value| parse_positive_u64(value, CONNECT_SIGNATURE, "keepalive_interval_ms"))
        .transpose()?;
    let keepalive_interval_ms = keepalive_interval_ms.or_else(|| {
        ssh_config
            .as_ref()
            .and_then(|config| config.server_alive_interval_ms)
    });

    Ok(ConnectOptions {
        host,
        connect_host,
        user,
        port,
        auth,
        host_key,
        connect_timeout_ms,
        keepalive_interval_ms,
    })
}

struct ParsedTarget {
    host: Option<String>,
    user: Option<String>,
    port: Option<u16>,
}

fn parse_target_string(target: &str) -> mlua::Result<ParsedTarget> {
    if target.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} does not accept empty target"
        )));
    }

    let (user, host_port) = match target.rsplit_once('@') {
        Some((user, host_port)) => (Some(user.to_string()), host_port),
        None => (None, target),
    };

    if host_port.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} invalid target `{target}`"
        )));
    }

    let (host, port) = if host_port.starts_with('[') {
        let Some(end) = host_port.find(']') else {
            return Err(mlua::Error::runtime(format!(
                "{CONNECT_SIGNATURE} invalid IPv6 target `{target}`"
            )));
        };
        let host = host_port[1..end].to_string();
        let port = if end + 1 == host_port.len() {
            None
        } else {
            let suffix = &host_port[end + 1..];
            let Some(port_str) = suffix.strip_prefix(':') else {
                return Err(mlua::Error::runtime(format!(
                    "{CONNECT_SIGNATURE} invalid target `{target}`"
                )));
            };
            Some(parse_port_string(port_str, CONNECT_SIGNATURE)?)
        };
        (host, port)
    } else if let Some((host, port)) = host_port.rsplit_once(':') {
        if host.contains(':') {
            (host_port.to_string(), None)
        } else {
            (
                host.to_string(),
                Some(parse_port_string(port, CONNECT_SIGNATURE)?),
            )
        }
    } else {
        (host_port.to_string(), None)
    };

    if host.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} invalid target `{target}`"
        )));
    }

    Ok(ParsedTarget {
        host: Some(host),
        user,
        port,
    })
}

fn resolve_ssh_config(
    host: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> mlua::Result<Option<SshConfigOptions>> {
    let mut command = ProcessCommand::new("ssh");
    command.arg("-G");
    if let Some(user) = user {
        command.arg("-l");
        command.arg(user);
    }
    if let Some(port) = port {
        command.arg("-p");
        command.arg(port.to_string());
    }
    command.arg(host);

    let output = match command.output() {
        Ok(output) => output,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(mlua::Error::runtime(format!(
                "{CONNECT_SIGNATURE} failed to run `ssh -G`: {err}"
            )));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let details = stderr.trim();
        return Err(mlua::Error::runtime(if details.is_empty() {
            format!("{CONNECT_SIGNATURE} `ssh -G` failed for `{host}`")
        } else {
            format!("{CONNECT_SIGNATURE} `ssh -G` failed for `{host}`: {details}")
        }));
    }

    parse_ssh_config_output(&String::from_utf8_lossy(&output.stdout)).map(Some)
}

fn parse_ssh_config_output(output: &str) -> mlua::Result<SshConfigOptions> {
    let mut config = SshConfigOptions::default();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Some(index) = line.find(char::is_whitespace) else {
            continue;
        };
        let key = &line[..index];
        let value = line[index..].trim_start();

        match key {
            "host" if !value.is_empty() => config.host = Some(value.to_string()),
            "hostname" if !value.is_empty() => config.hostname = Some(value.to_string()),
            "user" if !value.is_empty() => config.user = Some(value.to_string()),
            "port" if !value.is_empty() => {
                config.port = Some(parse_ssh_config_port(value)?);
            }
            "identityfile" if !value.eq_ignore_ascii_case("none") => {
                config.identity_files.push(resolve_ssh_config_path(value));
            }
            "userknownhostsfile" => {
                config.user_known_hosts_files = split_ssh_config_values(value)
                    .into_iter()
                    .filter(|item| !item.eq_ignore_ascii_case("none"))
                    .map(|item| resolve_ssh_config_path(&item))
                    .collect();
            }
            "stricthostkeychecking" if !value.is_empty() => {
                config.strict_host_key_checking = Some(value.to_string());
            }
            "connecttimeout" => {
                config.connect_timeout_ms = parse_ssh_config_duration_ms(value, key, false)?;
            }
            "serveraliveinterval" => {
                config.server_alive_interval_ms = parse_ssh_config_duration_ms(value, key, true)?;
            }
            _ => {}
        }
    }

    Ok(config)
}

fn split_ssh_config_values(value: &str) -> Vec<String> {
    shlex::split(value).unwrap_or_else(|| value.split_whitespace().map(str::to_string).collect())
}

fn parse_ssh_config_port(value: &str) -> mlua::Result<u16> {
    let port = value.parse::<i64>().map_err(|_| {
        mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} `ssh -G` returned invalid port `{value}`"
        ))
    })?;
    parse_port(port)
}

fn parse_ssh_config_duration_ms(
    value: &str,
    key: &str,
    zero_is_none: bool,
) -> mlua::Result<Option<u64>> {
    if value.eq_ignore_ascii_case("none") {
        return Ok(None);
    }

    let seconds = value.parse::<i64>().map_err(|_| {
        mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} `ssh -G` returned invalid `{key}` `{value}`"
        ))
    })?;
    if seconds == 0 && zero_is_none {
        return Ok(None);
    }
    let seconds = parse_positive_u64(seconds, CONNECT_SIGNATURE, key)?;
    seconds
        .checked_mul(1000)
        .ok_or_else(|| mlua::Error::runtime(format!("{CONNECT_SIGNATURE} `{key}` is too large")))
        .map(Some)
}

fn resolve_ssh_config_path(path: &str) -> PathBuf {
    PathBuf::from(expand_home(path).into_owned())
}

fn parse_auth_options(
    table: Option<&Table>,
    current_dir: &Path,
    ssh_config: Option<&SshConfigOptions>,
) -> mlua::Result<AuthMethod> {
    if let Some(table) = table
        && let Some(auth) = table.get::<Option<Table>>("auth")?
    {
        if let Some(password) = auth.get::<Option<String>>("password")? {
            return Ok(AuthMethod::Password { password });
        }

        if let Some(path) = auth.get::<Option<String>>("private_key_file")? {
            let path = resolve_local_path(&path, current_dir);
            let passphrase = auth.get::<Option<String>>("private_key_passphrase")?;
            return Ok(AuthMethod::PrivateKeys {
                keys: vec![PrivateKeyOption {
                    path,
                    passphrase,
                    required: true,
                }],
            });
        }
    }

    let keys = find_private_key_candidates(ssh_config);
    if !keys.is_empty() {
        return Ok(AuthMethod::PrivateKeys { keys });
    }

    Err(mlua::Error::runtime(format!(
        "{CONNECT_SIGNATURE} requires `auth.password` or `auth.private_key_file`"
    )))
}

fn parse_host_key_options(
    table: Option<&Table>,
    current_dir: &Path,
    ssh_config: Option<&SshConfigOptions>,
) -> mlua::Result<HostKeyPolicy> {
    let default_policy = ssh_config
        .map(|config| match config.strict_host_key_checking.as_deref() {
            Some("no") | Some("off") => HostKeyPolicy::Ignore,
            _ => HostKeyPolicy::KnownHosts {
                path: config.user_known_hosts_files.first().cloned(),
            },
        })
        .unwrap_or(HostKeyPolicy::KnownHosts { path: None });

    let Some(table) = table else {
        return Ok(default_policy);
    };
    let Some(host_key) = table.get::<Option<Table>>("host_key")? else {
        return Ok(default_policy);
    };

    let verify = host_key
        .get::<Option<String>>("verify")?
        .unwrap_or_else(|| "known_hosts".to_string());

    match verify.as_str() {
        "known_hosts" => {
            let path = host_key
                .get::<Option<String>>("known_hosts_file")?
                .map(|path| resolve_local_path(&path, current_dir));
            Ok(HostKeyPolicy::KnownHosts { path })
        }
        "ignore" => Ok(HostKeyPolicy::Ignore),
        _ => Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} `host_key.verify` must be `known_hosts` or `ignore`"
        ))),
    }
}

fn parse_run_call(args: Variadic<Value>) -> mlua::Result<ExecOptions> {
    match args.len() {
        0 => Err(mlua::Error::runtime(format!(
            "{RUN_SIGNATURE} requires arguments"
        ))),
        1 => match args.first() {
            Some(Value::String(command)) => Ok(ExecOptions {
                command: command.to_str()?.to_string(),
                stdin: None,
                echo: false,
                stdout: StreamMode::Inherit,
                stderr: StreamMode::Inherit,
                check: false,
            }),
            Some(Value::Table(options)) => parse_run_options_table(options.clone()),
            _ => Err(mlua::Error::runtime(format!(
                "{RUN_SIGNATURE} expects a command string or an options table"
            ))),
        },
        2 => match (args.first(), args.get(1)) {
            (Some(Value::String(command_or_cmdline)), Some(Value::Table(second))) => {
                if looks_like_array_table(second)? {
                    build_exec_options_from_parts(
                        command_or_cmdline.to_str()?.to_string(),
                        parse_string_list(second)?,
                        None,
                    )
                } else {
                    let options = parse_exec_overrides(second.clone(), RUN_SIGNATURE)?;
                    build_exec_options_from_cmdline(
                        command_or_cmdline.to_str()?.to_string(),
                        options,
                    )
                }
            }
            (Some(Value::String(command)), Some(Value::String(argsline))) => {
                let args = parse_argsline(&argsline.to_str()?)?;
                build_exec_options_from_parts(command.to_str()?.to_string(), args, None)
            }
            _ => Err(mlua::Error::runtime(format!(
                "{RUN_SIGNATURE} invalid arguments"
            ))),
        },
        3 => match (args.first(), args.get(1), args.get(2)) {
            (
                Some(Value::String(command)),
                Some(Value::String(argsline)),
                Some(Value::Table(options)),
            ) => {
                let args = parse_argsline(&argsline.to_str()?)?;
                let options = parse_exec_overrides(options.clone(), RUN_SIGNATURE)?;
                build_exec_options_from_parts(command.to_str()?.to_string(), args, Some(options))
            }
            (
                Some(Value::String(command)),
                Some(Value::Table(args)),
                Some(Value::Table(options)),
            ) => {
                let args = parse_string_list(args)?;
                let options = parse_exec_overrides(options.clone(), RUN_SIGNATURE)?;
                build_exec_options_from_parts(command.to_str()?.to_string(), args, Some(options))
            }
            _ => Err(mlua::Error::runtime(format!(
                "{RUN_SIGNATURE} invalid arguments"
            ))),
        },
        _ => Err(mlua::Error::runtime(format!(
            "{RUN_SIGNATURE} accepts at most 3 arguments"
        ))),
    }
}

struct ExecOverrides {
    stdin: Option<Vec<u8>>,
    echo: Option<bool>,
    stdout: Option<StreamMode>,
    stderr: Option<StreamMode>,
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
) -> mlua::Result<ExecOptions> {
    Ok(ExecOptions {
        command,
        stdin: options.stdin,
        echo: options.echo.unwrap_or(false),
        stdout: options.stdout.unwrap_or(StreamMode::Inherit),
        stderr: options.stderr.unwrap_or(StreamMode::Inherit),
        check: options.check.unwrap_or(false),
    })
}

fn build_exec_options_from_parts(
    cmd: String,
    args: Vec<String>,
    options: Option<ExecOverrides>,
) -> mlua::Result<ExecOptions> {
    let command = quote_words_for_shell(
        std::iter::once(cmd.as_str()).chain(args.iter().map(String::as_str)),
    )?;
    let options = options.unwrap_or(ExecOverrides {
        stdin: None,
        echo: None,
        stdout: None,
        stderr: None,
        check: None,
    });

    Ok(ExecOptions {
        command,
        stdin: options.stdin,
        echo: options.echo.unwrap_or(false),
        stdout: options.stdout.unwrap_or(StreamMode::Inherit),
        stderr: options.stderr.unwrap_or(StreamMode::Inherit),
        check: options.check.unwrap_or(false),
    })
}

fn parse_run_options_table(options: Table) -> mlua::Result<ExecOptions> {
    let cmd: Option<String> = options.get("cmd")?;
    let args = options.get::<Option<Table>>("args")?;
    let env = options.get::<Option<Table>>("env")?;
    let cwd = options.get::<Option<String>>("cwd")?;

    let base_command = match (cmd, args) {
        (Some(cmd), Some(args)) => quote_words_for_shell(
            std::iter::once(cmd.as_str())
                .chain(parse_string_list(&args)?.iter().map(String::as_str)),
        )?,
        (Some(cmd), None) => try_quote(&cmd)
            .map(|value| value.into_owned())
            .map_err(|err| {
                mlua::Error::runtime(format!("{RUN_SIGNATURE} invalid command: {err}"))
            })?,
        (None, _) => {
            return Err(mlua::Error::runtime(format!(
                "{RUN_SIGNATURE} options mode requires `cmd`"
            )));
        }
    };

    let command = wrap_remote_command(base_command, cwd, env)?;
    Ok(ExecOptions {
        command,
        stdin: parse_stdin(options.get::<Option<Value>>("stdin")?, RUN_SIGNATURE)?,
        echo: options.get::<Option<bool>>("echo")?.unwrap_or(false),
        stdout: parse_stream_mode(
            options.get::<Option<String>>("stdout")?,
            "stdout",
            RUN_SIGNATURE,
        )?
        .unwrap_or(StreamMode::Inherit),
        stderr: parse_stream_mode(
            options.get::<Option<String>>("stderr")?,
            "stderr",
            RUN_SIGNATURE,
        )?
        .unwrap_or(StreamMode::Inherit),
        check: options.get::<Option<bool>>("check")?.unwrap_or(false),
    })
}

fn wrap_remote_command(
    command: String,
    cwd: Option<String>,
    env: Option<Table>,
) -> mlua::Result<String> {
    let mut prefixes = Vec::new();

    if let Some(cwd) = cwd {
        let quoted = try_quote(&cwd)
            .map_err(|err| mlua::Error::runtime(format!("{RUN_SIGNATURE} invalid `cwd`: {err}")))?;
        prefixes.push(format!("cd {quoted}"));
    }

    if let Some(env) = env {
        let mut env_parts = Vec::new();
        for pair in env.pairs::<String, String>() {
            let (key, value) = pair?;
            if key.is_empty() {
                return Err(mlua::Error::runtime(format!(
                    "{RUN_SIGNATURE} `env` keys must not be empty"
                )));
            }
            let quoted = try_quote(&value).map_err(|err| {
                mlua::Error::runtime(format!(
                    "{RUN_SIGNATURE} invalid env value for `{key}`: {err}"
                ))
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

fn build_exec_result(lua: &Lua, result: ExecResult, target: String) -> mlua::Result<Table> {
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

fn handle_output(buffer: &mut Vec<u8>, mode: StreamMode, data: &[u8]) {
    match mode {
        StreamMode::Inherit => {
            use std::io::Write as _;
            let _ = std::io::stdout().write_all(data);
            let _ = std::io::stdout().flush();
        }
        StreamMode::Capture => buffer.extend_from_slice(data),
        StreamMode::Null => {}
    }
}

fn bytes_to_captured_string(bytes: Vec<u8>, mode: StreamMode) -> Option<String> {
    match mode {
        StreamMode::Capture => Some(String::from_utf8_lossy(&bytes).to_string()),
        StreamMode::Inherit | StreamMode::Null => None,
    }
}

fn build_exec_failed_error(target: &str, command: &str, result: &ExecResult) -> mlua::Error {
    let mut message = format!("ptool.ssh command `{command}` on `{target}` failed");
    if let Some(code) = result.code {
        message.push_str(&format!(" with status {code}"));
    }
    if let Some(stderr) = result
        .stderr
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        message.push_str(&format!(": {}", stderr.trim_end()));
    }
    mlua::Error::runtime(message)
}

fn ssh_error(context: &str, err: impl std::fmt::Display) -> mlua::Error {
    mlua::Error::runtime(format!("{context} failed: {err}"))
}

fn parse_port(value: i64) -> mlua::Result<u16> {
    if !(1..=65535).contains(&value) {
        return Err(mlua::Error::runtime(format!(
            "{CONNECT_SIGNATURE} `port` must be between 1 and 65535"
        )));
    }
    Ok(value as u16)
}

fn parse_port_string(value: &str, context: &str) -> mlua::Result<u16> {
    let port = value
        .parse::<i64>()
        .map_err(|_| mlua::Error::runtime(format!("{context} invalid port `{value}`")))?;
    parse_port(port)
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

fn resolve_local_path(path: &str, current_dir: &Path) -> PathBuf {
    let expanded = expand_home(path);
    let path = PathBuf::from(expanded.into_owned());
    if path.is_absolute() {
        path
    } else {
        current_dir.join(path)
    }
}

fn expand_home(path: &str) -> Cow<'_, str> {
    if path == "~"
        && let Some(home) = home_dir_string()
    {
        return Cow::Owned(home);
    }
    if let Some(suffix) = path.strip_prefix("~/")
        && let Some(home) = home_dir_string()
    {
        return Cow::Owned(format!("{home}/{suffix}"));
    }
    Cow::Borrowed(path)
}

fn home_dir_string() -> Option<String> {
    env::var("HOME").ok().filter(|value| !value.is_empty())
}

fn default_ssh_user() -> String {
    env::var("USER")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "root".to_string())
}

fn find_private_key_candidates(ssh_config: Option<&SshConfigOptions>) -> Vec<PrivateKeyOption> {
    let paths = ssh_config
        .filter(|config| !config.identity_files.is_empty())
        .map(|config| config.identity_files.clone())
        .unwrap_or_else(find_default_private_keys);

    let mut keys = Vec::new();
    for path in paths {
        if !path.is_file()
            || keys
                .iter()
                .any(|candidate: &PrivateKeyOption| candidate.path == path)
        {
            continue;
        }
        keys.push(PrivateKeyOption {
            path,
            passphrase: None,
            required: false,
        });
    }

    keys
}

fn find_default_private_keys() -> Vec<PathBuf> {
    let Some(home) = home_dir_string() else {
        return Vec::new();
    };
    let ssh_dir = PathBuf::from(home).join(".ssh");
    ["id_ed25519", "id_rsa", "id_ecdsa"]
        .iter()
        .map(|name| ssh_dir.join(name))
        .filter(|path| path.is_file())
        .collect()
}

fn looks_like_array_table(table: &Table) -> mlua::Result<bool> {
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
        let index = usize::try_from(index).map_err(|_| {
            mlua::Error::runtime(format!("{RUN_SIGNATURE} array argument is too large"))
        })?;
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

fn parse_argsline(input: &str) -> mlua::Result<Vec<String>> {
    shlex::split(input)
        .ok_or_else(|| mlua::Error::runtime(format!("{RUN_SIGNATURE} failed to parse args string")))
}

fn quote_words_for_shell<'a>(words: impl IntoIterator<Item = &'a str>) -> mlua::Result<String> {
    let mut quoted = Vec::new();
    for word in words {
        quoted.push(
            try_quote(word)
                .map(|value| value.into_owned())
                .map_err(|err| {
                    mlua::Error::runtime(format!("{RUN_SIGNATURE} invalid argument: {err}"))
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
) -> mlua::Result<Option<StreamMode>> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value.as_str() {
        "inherit" => Ok(Some(StreamMode::Inherit)),
        "capture" => Ok(Some(StreamMode::Capture)),
        "null" => Ok(Some(StreamMode::Null)),
        _ => Err(mlua::Error::runtime(format!(
            "{context} `{field}` must be `inherit`, `capture`, or `null`"
        ))),
    }
}
