use crate::{Error, ErrorKind, Result};
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

#[derive(Clone, Debug, Default)]
pub struct SshConnectRequest {
    pub target: String,
    pub host: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub auth: Option<SshAuthRequest>,
    pub host_key: Option<SshHostKeyRequest>,
    pub connect_timeout_ms: Option<u64>,
    pub keepalive_interval_ms: Option<u64>,
}

#[derive(Clone, Debug)]
pub enum SshAuthRequest {
    PrivateKeyFile {
        path: String,
        passphrase: Option<String>,
    },
    Password {
        password: String,
    },
}

#[derive(Clone, Debug)]
pub enum SshHostKeyRequest {
    KnownHosts { path: Option<String> },
    Ignore,
}

#[derive(Clone)]
pub struct SshConnection {
    runtime: Arc<Runtime>,
    state: Rc<RefCell<ConnectionState>>,
}

#[derive(Clone, Debug)]
pub struct SshConnectionInfo {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub target: String,
}

#[derive(Clone, Debug)]
pub struct SshExecOptions {
    pub command: String,
    pub display_cwd: Option<String>,
    pub stdin: Option<Vec<u8>>,
    pub echo: bool,
    pub stdout: SshStreamMode,
    pub stderr: SshStreamMode,
    pub check: bool,
}

#[derive(Clone, Debug)]
pub struct SshExecResult {
    pub code: Option<i64>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SshStreamMode {
    Inherit,
    Capture,
    Null,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SshTransferOptions {
    pub parents: bool,
    pub overwrite: bool,
    pub echo: bool,
}

impl Default for SshTransferOptions {
    fn default() -> Self {
        Self {
            parents: false,
            overwrite: true,
            echo: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshTransferResult {
    pub bytes: u64,
    pub from: String,
    pub to: String,
}

struct ConnectionState {
    session: Option<client::Handle<SshClientHandler>>,
    info: SshConnectionInfo,
}

#[derive(Clone, Debug)]
struct SshClientHandler {
    policy: HostKeyPolicy,
    host: String,
    port: u16,
}

#[derive(Clone, Debug)]
enum HostKeyPolicy {
    KnownHosts { path: Option<PathBuf> },
    Ignore,
}

#[derive(Clone, Debug)]
enum AuthMethod {
    PrivateKeys { keys: Vec<PrivateKeyOption> },
    Password { password: String },
}

#[derive(Clone, Debug)]
struct PrivateKeyOption {
    path: PathBuf,
    passphrase: Option<String>,
    required: bool,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Default)]
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

struct ParsedTarget {
    host: Option<String>,
    user: Option<String>,
    port: Option<u16>,
}

struct BinaryExecResult {
    code: Option<i64>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

pub fn connect(
    runtime: Arc<Runtime>,
    request: SshConnectRequest,
    current_dir: &Path,
) -> Result<SshConnection> {
    let options = build_connect_options(request, current_dir)?;
    let info = SshConnectionInfo {
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
        .map_err(|err| ssh_error(format!("failed to connect to `{}`: {err}", info.target)))?;

    authenticate_session(&runtime, &mut session, &options)?;

    Ok(SshConnection {
        runtime,
        state: Rc::new(RefCell::new(ConnectionState {
            session: Some(session),
            info,
        })),
    })
}

impl SshConnection {
    pub fn info(&self) -> SshConnectionInfo {
        self.state.borrow().info.clone()
    }

    pub fn same_session(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    pub fn close(&self) -> Result<()> {
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
                .map_err(|err| ssh_error(format!("failed to disconnect: {err}")))?;
        }

        Ok(())
    }

    pub fn run(&self, options: SshExecOptions) -> Result<SshExecResult> {
        let info = self.info();

        let mut state = self.state.borrow_mut();
        let session = state
            .session
            .as_mut()
            .ok_or_else(|| ssh_error("cannot use a closed connection"))?;

        let command = options.command.clone();
        self.runtime.block_on(async {
            let mut channel = session.channel_open_session().await.map_err(|err| {
                ssh_error(format!(
                    "failed to open session channel for `{command}`: {err}"
                ))
            })?;
            channel
                .exec(true, command.as_bytes())
                .await
                .map_err(|err| {
                    ssh_error(format!(
                        "failed to execute remote command `{command}`: {err}"
                    ))
                })?;

            if let Some(stdin) = options.stdin {
                let mut cursor = std::io::Cursor::new(stdin);
                channel.data(&mut cursor).await.map_err(|err| {
                    ssh_error(format!("failed to write stdin for `{command}`: {err}"))
                })?;
                channel.eof().await.map_err(|err| {
                    ssh_error(format!("failed to send EOF for `{command}`: {err}"))
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

            let result = SshExecResult {
                code,
                stdout: bytes_to_captured_string(stdout_bytes, options.stdout),
                stderr: bytes_to_captured_string(stderr_bytes, options.stderr),
            };

            if options.check && result.code != Some(0) {
                return Err(build_exec_failed_error(&info.target, &command, &result));
            }

            Ok(result)
        })
    }

    pub fn resolve_display_cwd(&self, cwd: Option<&str>) -> Result<String> {
        match cwd {
            Some(cwd) if Path::new(cwd).is_absolute() => Ok(cwd.to_string()),
            Some(cwd) => self.detect_remote_cwd_in(cwd),
            None => self.detect_remote_cwd(),
        }
    }

    pub fn upload_file(
        &self,
        local_path: &Path,
        remote_path: &str,
        options: SshTransferOptions,
    ) -> Result<SshTransferResult> {
        ensure_local_file(local_path, "local_path")?;
        ensure_non_empty_string(remote_path, "remote_path")?;

        let content = std::fs::read(local_path).map_err(|err| {
            ssh_error(format!("failed to read `{}`: {err}", local_path.display()))
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
        self.exec_binary(command.as_str(), Some(content), false)?;

        Ok(SshTransferResult {
            bytes: std::fs::metadata(local_path)
                .map(|metadata| metadata.len())
                .unwrap_or(0),
            from: local_path.display().to_string(),
            to: format!("{}:{}", self.info().target, remote_path),
        })
    }

    pub fn download_file(
        &self,
        remote_path: &str,
        local_path: &Path,
        options: SshTransferOptions,
    ) -> Result<SshTransferResult> {
        ensure_non_empty_string(remote_path, "remote_path")?;
        prepare_local_destination(local_path, options, "local_path")?;

        if options.echo {
            println!(
                "[ssh download {}] {} -> {}",
                self.info().target,
                remote_path,
                local_path.display()
            );
        }

        let command = build_download_command(remote_path)?;
        let result = self.exec_binary(command.as_str(), None, true)?;

        write_local_file(local_path, &result.stdout, options, "local_path")?;

        Ok(SshTransferResult {
            bytes: u64::try_from(result.stdout.len())
                .map_err(|_| ssh_error("downloaded file is too large to report size"))?,
            from: format!("{}:{}", self.info().target, remote_path),
            to: local_path.display().to_string(),
        })
    }

    pub fn exists(&self, remote_path: &str) -> Result<bool> {
        self.check_remote_path("exists", remote_path, "-e")
    }

    pub fn is_file(&self, remote_path: &str) -> Result<bool> {
        self.check_remote_path("is_file", remote_path, "-f")
    }

    pub fn is_dir(&self, remote_path: &str) -> Result<bool> {
        self.check_remote_path("is_dir", remote_path, "-d")
    }

    fn check_remote_path(
        &self,
        operation: &str,
        remote_path: &str,
        test_flag: &str,
    ) -> Result<bool> {
        ensure_non_empty_string(remote_path, "remote_path")?;

        let command = build_remote_path_test_command(remote_path, test_flag)?;
        let code = self.exec_binary_status(command.as_str())?;
        match code {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(build_remote_path_test_failed_error(
                &self.info().target,
                operation,
                remote_path,
                code,
            )),
        }
    }

    fn exec_binary(
        &self,
        command: &str,
        stdin: Option<Vec<u8>>,
        capture_stdout: bool,
    ) -> Result<BinaryExecResult> {
        let info = self.info();

        let mut state = self.state.borrow_mut();
        let session = state
            .session
            .as_mut()
            .ok_or_else(|| ssh_error("cannot use a closed connection"))?;

        let command = command.to_string();
        self.runtime.block_on(async {
            let mut channel = session.channel_open_session().await.map_err(|err| {
                ssh_error(format!(
                    "failed to open session channel for `{command}`: {err}"
                ))
            })?;
            channel
                .exec(true, command.as_bytes())
                .await
                .map_err(|err| {
                    ssh_error(format!(
                        "failed to execute remote command `{command}`: {err}"
                    ))
                })?;

            if let Some(stdin) = stdin {
                let mut cursor = std::io::Cursor::new(stdin);
                channel.data(&mut cursor).await.map_err(|err| {
                    ssh_error(format!("failed to write stdin for `{command}`: {err}"))
                })?;
                channel.eof().await.map_err(|err| {
                    ssh_error(format!("failed to send EOF for `{command}`: {err}"))
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
                    &info.target,
                    &command,
                    result.code,
                    &result.stderr,
                ));
            }

            Ok(result)
        })
    }

    fn exec_binary_status(&self, command: &str) -> Result<Option<i64>> {
        let mut state = self.state.borrow_mut();
        let session = state
            .session
            .as_mut()
            .ok_or_else(|| ssh_error("cannot use a closed connection"))?;

        let command = command.to_string();
        self.runtime.block_on(async {
            let mut channel = session.channel_open_session().await.map_err(|err| {
                ssh_error(format!(
                    "failed to open session channel for `{command}`: {err}"
                ))
            })?;
            channel
                .exec(true, command.as_bytes())
                .await
                .map_err(|err| {
                    ssh_error(format!(
                        "failed to execute remote command `{command}`: {err}"
                    ))
                })?;

            let mut code = None;
            while let Some(msg) = channel.wait().await {
                match msg {
                    ChannelMsg::ExitStatus { exit_status } => code = Some(i64::from(exit_status)),
                    ChannelMsg::ExitSignal { .. } => code = None,
                    _ => {}
                }
            }

            Ok(code)
        })
    }

    fn detect_remote_cwd(&self) -> Result<String> {
        self.detect_remote_cwd_with_command("pwd")
    }

    fn detect_remote_cwd_in(&self, cwd: &str) -> Result<String> {
        let quoted = try_quote(cwd)
            .map_err(|err| ssh_error(format!("invalid remote cwd `{cwd}`: {err}")))?;
        self.detect_remote_cwd_with_command(format!("cd {quoted} && pwd").as_str())
    }

    fn detect_remote_cwd_with_command(&self, command: &str) -> Result<String> {
        let result = self.exec_binary(command, None, true)?;
        let cwd = String::from_utf8_lossy(&result.stdout).trim().to_string();
        if cwd.is_empty() {
            return Err(ssh_error("failed to determine remote cwd"));
        }
        Ok(cwd)
    }
}

impl client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
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

fn build_connect_options(request: SshConnectRequest, current_dir: &Path) -> Result<ConnectOptions> {
    let parsed = if request.target.is_empty() {
        ParsedTarget {
            host: None,
            user: None,
            port: None,
        }
    } else {
        parse_target_string(&request.target)?
    };

    let requested_host = request
        .host
        .or(parsed.host)
        .filter(|host| !host.is_empty())
        .ok_or_else(|| ssh_error("requires `host`"))?;
    let requested_user = request.user.or(parsed.user);
    let requested_port = request.port.or(parsed.port);

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

    let auth = parse_auth_options(request.auth, current_dir, ssh_config.as_ref())?;
    let host_key = parse_host_key_options(request.host_key, current_dir, ssh_config.as_ref())?;
    let connect_timeout_ms = request
        .connect_timeout_ms
        .or_else(|| {
            ssh_config
                .as_ref()
                .and_then(|config| config.connect_timeout_ms)
        })
        .unwrap_or(10_000);
    let keepalive_interval_ms = request.keepalive_interval_ms.or_else(|| {
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
) -> Result<()> {
    let user = options.user.clone();
    let authenticated = match &options.auth {
        AuthMethod::PrivateKeys { keys: key_options } => {
            let hash_alg = runtime
                .block_on(session.best_supported_rsa_hash())
                .map_err(|err| ssh_error(format!("failed to inspect server algorithms: {err}")))?
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
                            let err = ssh_error(err.to_string());
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
                    .map_err(|err| ssh_error(format!("authentication failed: {err}")))?;
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
            .map_err(|err| ssh_error(format!("authentication failed: {err}")))?
            .success(),
    };

    if !authenticated {
        return Err(ssh_error(format!(
            "authentication failed for `{}`",
            options.user
        )));
    }

    Ok(())
}

fn parse_target_string(target: &str) -> Result<ParsedTarget> {
    if target.is_empty() {
        return Err(ssh_error("does not accept empty target"));
    }

    let (user, host_port) = match target.rsplit_once('@') {
        Some((user, host_port)) => (Some(user.to_string()), host_port),
        None => (None, target),
    };

    if host_port.is_empty() {
        return Err(ssh_error(format!("invalid target `{target}`")));
    }

    let (host, port) = if host_port.starts_with('[') {
        let Some(end) = host_port.find(']') else {
            return Err(ssh_error(format!("invalid IPv6 target `{target}`")));
        };
        let host = host_port[1..end].to_string();
        let port = if end + 1 == host_port.len() {
            None
        } else {
            let suffix = &host_port[end + 1..];
            let Some(port_str) = suffix.strip_prefix(':') else {
                return Err(ssh_error(format!("invalid target `{target}`")));
            };
            Some(parse_port_string(port_str)?)
        };
        (host, port)
    } else if let Some((host, port)) = host_port.rsplit_once(':') {
        if host.contains(':') {
            (host_port.to_string(), None)
        } else {
            (host.to_string(), Some(parse_port_string(port)?))
        }
    } else {
        (host_port.to_string(), None)
    };

    if host.is_empty() {
        return Err(ssh_error(format!("invalid target `{target}`")));
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
) -> Result<Option<SshConfigOptions>> {
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
        Err(err) => return Err(ssh_error(format!("failed to run `ssh -G`: {err}"))),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let details = stderr.trim();
        return Err(ssh_error(if details.is_empty() {
            format!("`ssh -G` failed for `{host}`")
        } else {
            format!("`ssh -G` failed for `{host}`: {details}")
        }));
    }

    parse_ssh_config_output(&String::from_utf8_lossy(&output.stdout)).map(Some)
}

fn parse_ssh_config_output(output: &str) -> Result<SshConfigOptions> {
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

fn parse_ssh_config_port(value: &str) -> Result<u16> {
    let port = value
        .parse::<i64>()
        .map_err(|_| ssh_error(format!("`ssh -G` returned invalid port `{value}`")))?;
    parse_port(port)
}

fn parse_ssh_config_duration_ms(value: &str, key: &str, zero_is_none: bool) -> Result<Option<u64>> {
    if value.eq_ignore_ascii_case("none") {
        return Ok(None);
    }

    let seconds = value
        .parse::<i64>()
        .map_err(|_| ssh_error(format!("`ssh -G` returned invalid `{key}` `{value}`")))?;
    if seconds == 0 && zero_is_none {
        return Ok(None);
    }
    let seconds = parse_positive_u64(seconds, key)?;
    seconds
        .checked_mul(1000)
        .ok_or_else(|| ssh_error(format!("`{key}` is too large")))
        .map(Some)
}

fn parse_auth_options(
    request: Option<SshAuthRequest>,
    current_dir: &Path,
    ssh_config: Option<&SshConfigOptions>,
) -> Result<AuthMethod> {
    if let Some(request) = request {
        return Ok(match request {
            SshAuthRequest::Password { password } => AuthMethod::Password { password },
            SshAuthRequest::PrivateKeyFile { path, passphrase } => AuthMethod::PrivateKeys {
                keys: vec![PrivateKeyOption {
                    path: resolve_local_path(&path, current_dir),
                    passphrase,
                    required: true,
                }],
            },
        });
    }

    let keys = find_private_key_candidates(ssh_config);
    if !keys.is_empty() {
        return Ok(AuthMethod::PrivateKeys { keys });
    }

    Err(ssh_error(
        "requires `auth.password` or `auth.private_key_file`",
    ))
}

fn parse_host_key_options(
    request: Option<SshHostKeyRequest>,
    current_dir: &Path,
    ssh_config: Option<&SshConfigOptions>,
) -> Result<HostKeyPolicy> {
    let default_policy = ssh_config
        .map(|config| match config.strict_host_key_checking.as_deref() {
            Some("no") | Some("off") => HostKeyPolicy::Ignore,
            _ => HostKeyPolicy::KnownHosts {
                path: config.user_known_hosts_files.first().cloned(),
            },
        })
        .unwrap_or(HostKeyPolicy::KnownHosts { path: None });

    let Some(request) = request else {
        return Ok(default_policy);
    };

    match request {
        SshHostKeyRequest::KnownHosts { path } => Ok(HostKeyPolicy::KnownHosts {
            path: path.map(|path| resolve_local_path(&path, current_dir)),
        }),
        SshHostKeyRequest::Ignore => Ok(HostKeyPolicy::Ignore),
    }
}

fn build_upload_command(remote_path: &str, options: SshTransferOptions) -> Result<String> {
    let mut prefixes = Vec::new();
    if options.parents
        && let Some(parent) = remote_parent_path(remote_path)
    {
        prefixes.push(format!("mkdir -p {}", shell_quote(parent, "remote_path")?));
    }
    if !options.overwrite {
        prefixes.push(format!(
            "test ! -e {}",
            shell_quote(remote_path, "remote_path")?
        ));
    }

    let mut command = prefixes.join(" && ");
    if !command.is_empty() {
        command.push_str(" && ");
    }
    command.push_str("cat > ");
    command.push_str(&shell_quote(remote_path, "remote_path")?);
    Ok(command)
}

fn build_download_command(remote_path: &str) -> Result<String> {
    Ok(format!("cat {}", shell_quote(remote_path, "remote_path")?))
}

fn build_remote_path_test_command(remote_path: &str, test_flag: &str) -> Result<String> {
    Ok(format!(
        "test {test_flag} {}",
        shell_quote(remote_path, "remote_path")?
    ))
}

fn shell_quote(value: &str, field: &str) -> Result<String> {
    try_quote(value)
        .map(|value| value.into_owned())
        .map_err(|err| ssh_error(format!("invalid `{field}`: {err}")))
}

fn remote_parent_path(path: &str) -> Option<&str> {
    let parent = Path::new(path).parent()?;
    if parent.as_os_str().is_empty() {
        return None;
    }
    parent.to_str()
}

fn ensure_non_empty_string(value: &str, field: &str) -> Result<()> {
    if value.is_empty() {
        return Err(ssh_error(format!("`{field}` must not be empty")));
    }
    Ok(())
}

fn ensure_local_file(path: &Path, field: &str) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .map_err(|err| ssh_error(format!("failed to access `{}`: {err}", path.display())))?;
    if !metadata.is_file() {
        return Err(ssh_error(format!(
            "`{field}` must be a file: `{}`",
            path.display()
        )));
    }
    Ok(())
}

fn prepare_local_destination(path: &Path, options: SshTransferOptions, field: &str) -> Result<()> {
    if options.parents
        && let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|err| {
            ssh_error(format!(
                "failed to create parent directory `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    if !options.overwrite && path.exists() {
        return Err(ssh_error(format!(
            "`{field}` already exists: `{}`",
            path.display()
        )));
    }

    Ok(())
}

fn write_local_file(
    path: &Path,
    content: &[u8],
    options: SshTransferOptions,
    field: &str,
) -> Result<()> {
    let mut open_options = std::fs::OpenOptions::new();
    open_options.write(true).create(true).truncate(true);
    if !options.overwrite {
        open_options.create_new(true);
    }

    let mut file = open_options
        .open(path)
        .map_err(|err| ssh_error(format!("failed to open `{}`: {err}", path.display())))?;

    use std::io::Write as _;
    file.write_all(content)
        .map_err(|err| ssh_error(format!("failed to write `{}`: {err}", path.display())))?;
    file.flush()
        .map_err(|err| ssh_error(format!("failed to flush `{}`: {err}", path.display())))?;

    if path.is_dir() {
        return Err(ssh_error(format!(
            "`{field}` must not be a directory: `{}`",
            path.display()
        )));
    }

    Ok(())
}

fn build_binary_exec_failed_error(
    target: &str,
    command: &str,
    code: Option<i64>,
    stderr: &[u8],
) -> Error {
    let mut message = format!("remote command `{command}` on `{target}` failed");
    if let Some(code) = code {
        message.push_str(&format!(" with status {code}"));
    }
    let stderr = String::from_utf8_lossy(stderr);
    if !stderr.trim().is_empty() {
        message.push_str(&format!(": {}", stderr.trim_end()));
    }
    ssh_error(message)
}

fn build_remote_path_test_failed_error(
    target: &str,
    operation: &str,
    remote_path: &str,
    code: Option<i64>,
) -> Error {
    let mut message = format!("failed to {operation} remote path `{remote_path}` on `{target}`");
    if let Some(code) = code {
        message.push_str(&format!(" with status {code}"));
    }
    ssh_error(message)
}

fn handle_output(buffer: &mut Vec<u8>, mode: SshStreamMode, data: &[u8]) {
    match mode {
        SshStreamMode::Inherit => {
            use std::io::Write as _;
            let _ = std::io::stdout().write_all(data);
            let _ = std::io::stdout().flush();
        }
        SshStreamMode::Capture => buffer.extend_from_slice(data),
        SshStreamMode::Null => {}
    }
}

fn bytes_to_captured_string(bytes: Vec<u8>, mode: SshStreamMode) -> Option<String> {
    match mode {
        SshStreamMode::Capture => Some(String::from_utf8_lossy(&bytes).to_string()),
        SshStreamMode::Inherit | SshStreamMode::Null => None,
    }
}

fn build_exec_failed_error(target: &str, command: &str, result: &SshExecResult) -> Error {
    let mut message = format!("command `{command}` on `{target}` failed");
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
    ssh_error(message)
}

fn parse_port(value: i64) -> Result<u16> {
    if !(1..=65535).contains(&value) {
        return Err(ssh_error("`port` must be between 1 and 65535"));
    }
    Ok(value as u16)
}

fn parse_port_string(value: &str) -> Result<u16> {
    let port = value
        .parse::<i64>()
        .map_err(|_| ssh_error(format!("invalid port `{value}`")))?;
    parse_port(port)
}

fn parse_positive_u64(value: i64, field: &str) -> Result<u64> {
    if value <= 0 {
        return Err(ssh_error(format!("`{field}` must be > 0")));
    }
    u64::try_from(value).map_err(|_| ssh_error(format!("`{field}` is too large")))
}

fn resolve_ssh_config_path(path: &str) -> PathBuf {
    PathBuf::from(expand_home(path).into_owned())
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

fn ssh_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::Ssh, msg)
}
