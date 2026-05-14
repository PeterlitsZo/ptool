use crate::{Error, ErrorKind, Result};
use shlex::try_quote;
use std::borrow::Cow;
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Output, Stdio};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tar::{Archive, Builder};
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
    pub trim: bool,
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
    closed: bool,
    info: SshConnectionInfo,
    options: ConnectOptions,
}

#[derive(Clone, Debug)]
struct ConnectOptions {
    host: String,
    requested_user: Option<String>,
    requested_port: Option<u16>,
    auth: Option<AuthMethod>,
    host_key: Option<HostKeyPolicy>,
    connect_timeout_ms: Option<u64>,
    keepalive_interval_ms: Option<u64>,
}

#[derive(Clone, Debug)]
enum AuthMethod {
    PrivateKeyFile { path: PathBuf },
}

#[derive(Clone, Debug)]
enum HostKeyPolicy {
    KnownHosts { path: Option<PathBuf> },
    Ignore,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LocalPathKind {
    File,
    Directory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RemotePathKind {
    File,
    Directory,
}

struct TempArchivePath {
    path: PathBuf,
}

enum ProcessOutputMode {
    Capture,
    Inherit,
    Null,
}

impl TempArchivePath {
    fn create(prefix: &str) -> Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let base_dir = env::temp_dir();

        for attempt in 0..1024 {
            let path = base_dir.join(format!(
                "ptool-{prefix}-{}-{nanos}-{attempt}.tar",
                std::process::id()
            ));
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(_) => return Ok(Self { path }),
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(err) => {
                    return Err(ssh_error(format!(
                        "failed to create temporary archive `{}`: {err}",
                        path.display()
                    )));
                }
            }
        }

        Err(ssh_error("failed to allocate a temporary archive path"))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempArchivePath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn connect(
    _runtime: Arc<Runtime>,
    request: SshConnectRequest,
    current_dir: &Path,
) -> Result<SshConnection> {
    ensure_ssh_available()?;

    let options = build_connect_options(request, current_dir)?;
    let info = SshConnectionInfo {
        target: format!(
            "{}@{}:{}",
            options
                .requested_user
                .clone()
                .unwrap_or_else(default_ssh_user),
            options.host,
            options.requested_port.unwrap_or(22)
        ),
        host: options.host.clone(),
        user: options
            .requested_user
            .clone()
            .unwrap_or_else(default_ssh_user),
        port: options.requested_port.unwrap_or(22),
    };

    Ok(SshConnection {
        state: Rc::new(RefCell::new(ConnectionState {
            closed: false,
            info,
            options,
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
        self.state.borrow_mut().closed = true;
        Ok(())
    }

    pub fn run(&self, options: SshExecOptions) -> Result<SshExecResult> {
        let info = self.info();
        let exec = self.exec_ssh(
            &options.command,
            options.stdin,
            process_output_mode(options.stdout),
            process_output_mode(options.stderr),
        )?;

        let result = SshExecResult {
            code: exec.code,
            stdout: bytes_to_captured_string(exec.stdout, options.stdout, options.trim),
            stderr: bytes_to_captured_string(exec.stderr, options.stderr, options.trim),
        };

        if options.check && result.code != Some(0) {
            return Err(build_exec_failed_error(
                &info.target,
                &options.command,
                &result,
            ));
        }

        Ok(result)
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

    pub fn upload_path(
        &self,
        local_path: &Path,
        remote_path: &str,
        options: SshTransferOptions,
    ) -> Result<SshTransferResult> {
        match classify_local_path(local_path, "local_path")? {
            LocalPathKind::File => self.upload_file(local_path, remote_path, options),
            LocalPathKind::Directory => self.upload_directory(local_path, remote_path, options),
        }
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

    pub fn download_path(
        &self,
        remote_path: &str,
        local_path: &Path,
        options: SshTransferOptions,
    ) -> Result<SshTransferResult> {
        match self.classify_remote_path(remote_path)? {
            RemotePathKind::File => self.download_file(remote_path, local_path, options),
            RemotePathKind::Directory => self.download_directory(remote_path, local_path, options),
        }
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

    fn ensure_open_options(&self) -> Result<ConnectOptions> {
        let state = self.state.borrow();
        if state.closed {
            return Err(ssh_error("cannot use a closed connection"));
        }
        Ok(state.options.clone())
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

    fn classify_remote_path(&self, remote_path: &str) -> Result<RemotePathKind> {
        ensure_non_empty_string(remote_path, "remote_path")?;

        if self.is_file(remote_path)? {
            return Ok(RemotePathKind::File);
        }
        if self.is_dir(remote_path)? {
            return Ok(RemotePathKind::Directory);
        }
        if self.exists(remote_path)? {
            return Err(ssh_error(format!(
                "unsupported remote path type: `{remote_path}`"
            )));
        }
        Err(ssh_error(format!(
            "remote path does not exist: `{remote_path}`"
        )))
    }

    fn upload_directory(
        &self,
        local_path: &Path,
        remote_path: &str,
        options: SshTransferOptions,
    ) -> Result<SshTransferResult> {
        ensure_local_dir(local_path, "local_path")?;
        ensure_non_empty_string(remote_path, "remote_path")?;

        let destination_root =
            self.resolve_remote_directory_destination(local_path, remote_path, options)?;
        if options.echo {
            println!(
                "[ssh upload {}] {} -> {}",
                self.info().target,
                local_path.display(),
                remote_path
            );
        }

        let archive_path = TempArchivePath::create("ssh-upload")?;
        let bytes = create_directory_archive(local_path, archive_path.path())?;
        let archive_bytes = std::fs::read(archive_path.path()).map_err(|err| {
            ssh_error(format!(
                "failed to read temporary archive `{}`: {err}",
                archive_path.path().display()
            ))
        })?;
        let command = build_upload_directory_command(
            &destination_root,
            !self.exists(&destination_root)?,
            options,
        )?;
        self.exec_binary(command.as_str(), Some(archive_bytes), false)?;

        Ok(SshTransferResult {
            bytes,
            from: local_path.display().to_string(),
            to: format!("{}:{}", self.info().target, remote_path),
        })
    }

    fn download_directory(
        &self,
        remote_path: &str,
        local_path: &Path,
        options: SshTransferOptions,
    ) -> Result<SshTransferResult> {
        ensure_non_empty_string(remote_path, "remote_path")?;

        let source_root = normalize_remote_path(remote_path);
        let destination_root =
            self.resolve_local_directory_destination(&source_root, local_path, options)?;
        if options.echo {
            println!(
                "[ssh download {}] {} -> {}",
                self.info().target,
                remote_path,
                local_path.display()
            );
        }

        let command = build_download_directory_command(&source_root)?;
        let result = self.exec_binary(command.as_str(), None, true)?;
        let archive_path = TempArchivePath::create("ssh-download")?;
        std::fs::write(archive_path.path(), &result.stdout).map_err(|err| {
            ssh_error(format!(
                "failed to write temporary archive `{}`: {err}",
                archive_path.path().display()
            ))
        })?;
        let bytes = unpack_directory_archive(archive_path.path(), &destination_root)?;

        Ok(SshTransferResult {
            bytes,
            from: format!("{}:{}", self.info().target, remote_path),
            to: local_path.display().to_string(),
        })
    }

    fn resolve_remote_directory_destination(
        &self,
        local_path: &Path,
        remote_path: &str,
        options: SshTransferOptions,
    ) -> Result<String> {
        let remote_path = normalize_remote_path(remote_path);
        let destination_root = if self.exists(&remote_path)? {
            if !self.is_dir(&remote_path)? {
                return Err(ssh_error(format!(
                    "`remote_path` must be a directory for directory upload: `{remote_path}`"
                )));
            }
            join_remote_path(&remote_path, &path_basename(local_path, "local_path")?)
        } else {
            remote_path
        };
        validate_remote_directory_destination(self, &destination_root, options)
    }

    fn resolve_local_directory_destination(
        &self,
        remote_path: &str,
        local_path: &Path,
        options: SshTransferOptions,
    ) -> Result<PathBuf> {
        let destination_root = if local_path.exists() {
            if !local_path.is_dir() {
                return Err(ssh_error(format!(
                    "`local_path` must be a directory for directory download: `{}`",
                    local_path.display()
                )));
            }
            local_path.join(remote_basename(remote_path)?)
        } else {
            local_path.to_path_buf()
        };
        prepare_local_directory_destination(&destination_root, options)?;
        Ok(destination_root)
    }

    fn exec_binary(
        &self,
        command: &str,
        stdin: Option<Vec<u8>>,
        capture_stdout: bool,
    ) -> Result<BinaryExecResult> {
        let info = self.info();
        let result = self.exec_ssh(
            command,
            stdin,
            if capture_stdout {
                ProcessOutputMode::Capture
            } else {
                ProcessOutputMode::Null
            },
            ProcessOutputMode::Capture,
        )?;

        if result.code != Some(0) {
            return Err(build_binary_exec_failed_error(
                &info.target,
                command,
                result.code,
                &result.stderr,
            ));
        }

        Ok(result)
    }

    fn exec_binary_status(&self, command: &str) -> Result<Option<i64>> {
        self.exec_ssh(
            command,
            None,
            ProcessOutputMode::Null,
            ProcessOutputMode::Null,
        )
        .map(|result| result.code)
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

    fn exec_ssh(
        &self,
        remote_command: &str,
        stdin: Option<Vec<u8>>,
        stdout_mode: ProcessOutputMode,
        stderr_mode: ProcessOutputMode,
    ) -> Result<BinaryExecResult> {
        let options = self.ensure_open_options()?;
        let mut command = build_ssh_command(&options, remote_command);
        run_ssh_command(&mut command, stdin, stdout_mode, stderr_mode)
    }
}

fn ensure_ssh_available() -> Result<()> {
    match ProcessCommand::new("ssh").arg("-V").output() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(ssh_error(format!(
            "`ssh -V` failed{}",
            format_subprocess_stderr(&output)
        ))),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Err(ssh_error("`ssh` is not available on PATH"))
        }
        Err(err) => Err(ssh_error(format!("failed to execute `ssh -V`: {err}"))),
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

    let host = request
        .host
        .or(parsed.host)
        .filter(|host| !host.is_empty())
        .ok_or_else(|| ssh_error("requires `host`"))?;

    Ok(ConnectOptions {
        host,
        requested_user: request.user.or(parsed.user),
        requested_port: request.port.or(parsed.port),
        auth: parse_auth_options(request.auth, current_dir)?,
        host_key: parse_host_key_options(request.host_key, current_dir),
        connect_timeout_ms: request.connect_timeout_ms,
        keepalive_interval_ms: request.keepalive_interval_ms,
    })
}

fn parse_auth_options(
    request: Option<SshAuthRequest>,
    current_dir: &Path,
) -> Result<Option<AuthMethod>> {
    let Some(request) = request else {
        return Ok(None);
    };

    match request {
        SshAuthRequest::PrivateKeyFile { path, passphrase } => {
            if passphrase.is_some() {
                return Err(ssh_error(
                    "`auth.private_key_passphrase` is not supported when using system `ssh`",
                ));
            }
            Ok(Some(AuthMethod::PrivateKeyFile {
                path: resolve_local_path(&path, current_dir),
            }))
        }
        SshAuthRequest::Password { .. } => Err(ssh_error(
            "`auth.password` is not supported when using system `ssh`; configure OpenSSH authentication instead",
        )),
    }
}

fn parse_host_key_options(
    request: Option<SshHostKeyRequest>,
    current_dir: &Path,
) -> Option<HostKeyPolicy> {
    match request {
        Some(SshHostKeyRequest::KnownHosts { path }) => Some(HostKeyPolicy::KnownHosts {
            path: path.map(|path| resolve_local_path(&path, current_dir)),
        }),
        Some(SshHostKeyRequest::Ignore) => Some(HostKeyPolicy::Ignore),
        None => None,
    }
}

fn build_ssh_command(options: &ConnectOptions, remote_command: &str) -> ProcessCommand {
    let mut command = ProcessCommand::new("ssh");
    command.arg("-T");

    if let Some(user) = options.requested_user.as_deref() {
        command.arg("-l");
        command.arg(user);
    }

    if let Some(port) = options.requested_port {
        command.arg("-p");
        command.arg(port.to_string());
    }

    if let Some(connect_timeout_ms) = options.connect_timeout_ms {
        command.arg("-o");
        command.arg(format!(
            "ConnectTimeout={}",
            duration_millis_to_ssh_seconds(connect_timeout_ms)
        ));
    }

    if let Some(keepalive_interval_ms) = options.keepalive_interval_ms {
        command.arg("-o");
        command.arg(format!(
            "ServerAliveInterval={}",
            duration_millis_to_ssh_seconds(keepalive_interval_ms)
        ));
    }

    if let Some(auth) = &options.auth {
        match auth {
            AuthMethod::PrivateKeyFile { path } => {
                command.arg("-i");
                command.arg(path);
                command.arg("-o");
                command.arg("IdentitiesOnly=yes");
            }
        }
    }

    if let Some(host_key) = &options.host_key {
        match host_key {
            HostKeyPolicy::KnownHosts { path } => {
                command.arg("-o");
                command.arg("StrictHostKeyChecking=yes");
                if let Some(path) = path {
                    command.arg("-o");
                    command.arg(format!("UserKnownHostsFile={}", path.display()));
                }
            }
            HostKeyPolicy::Ignore => {
                command.arg("-o");
                command.arg("StrictHostKeyChecking=no");
                command.arg("-o");
                command.arg("UserKnownHostsFile=/dev/null");
            }
        }
    }

    command.arg(&options.host);
    command.arg(remote_command);
    command
}

fn run_ssh_command(
    command: &mut ProcessCommand,
    stdin: Option<Vec<u8>>,
    stdout_mode: ProcessOutputMode,
    stderr_mode: ProcessOutputMode,
) -> Result<BinaryExecResult> {
    configure_stdio(command, stdin.is_some(), &stdout_mode, &stderr_mode);

    let mut child = command
        .spawn()
        .map_err(|err| ssh_error(format!("failed to spawn `ssh`: {err}")))?;

    let stdin_handle = stdin.and_then(|bytes| {
        child.stdin.take().map(|mut pipe| {
            thread::spawn(move || -> std::io::Result<()> {
                pipe.write_all(&bytes)?;
                pipe.flush()?;
                Ok(())
            })
        })
    });

    let stdout_handle = child.stdout.take().map(spawn_reader_thread);
    let stderr_handle = child.stderr.take().map(spawn_reader_thread);

    let status = child
        .wait()
        .map_err(|err| ssh_error(format!("failed to wait for `ssh`: {err}")))?;

    if let Some(handle) = stdin_handle {
        finish_io_thread(handle, "stdin")?;
    }

    let stdout = collect_process_output(stdout_handle, &stdout_mode, "stdout")?;
    let stderr = collect_process_output(stderr_handle, &stderr_mode, "stderr")?;

    Ok(BinaryExecResult {
        code: status.code().map(i64::from),
        stdout,
        stderr,
    })
}

fn configure_stdio(
    command: &mut ProcessCommand,
    has_stdin: bool,
    stdout_mode: &ProcessOutputMode,
    stderr_mode: &ProcessOutputMode,
) {
    command.stdin(if has_stdin {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    command.stdout(match stdout_mode {
        ProcessOutputMode::Capture => Stdio::piped(),
        ProcessOutputMode::Inherit => Stdio::inherit(),
        ProcessOutputMode::Null => Stdio::null(),
    });
    command.stderr(match stderr_mode {
        ProcessOutputMode::Capture => Stdio::piped(),
        ProcessOutputMode::Inherit => Stdio::inherit(),
        ProcessOutputMode::Null => Stdio::null(),
    });
}

fn spawn_reader_thread<R>(mut reader: R) -> thread::JoinHandle<std::io::Result<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    })
}

fn finish_io_thread(
    handle: thread::JoinHandle<std::io::Result<()>>,
    stream_name: &str,
) -> Result<()> {
    match handle.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(ssh_error(format!(
            "failed to write {stream_name} for `ssh`: {err}"
        ))),
        Err(_) => Err(ssh_error(format!(
            "failed to join {stream_name} writer thread for `ssh`"
        ))),
    }
}

fn collect_process_output(
    handle: Option<thread::JoinHandle<std::io::Result<Vec<u8>>>>,
    mode: &ProcessOutputMode,
    stream_name: &str,
) -> Result<Vec<u8>> {
    match mode {
        ProcessOutputMode::Capture => match handle {
            Some(handle) => match handle.join() {
                Ok(Ok(buffer)) => Ok(buffer),
                Ok(Err(err)) => Err(ssh_error(format!(
                    "failed to read {stream_name} from `ssh`: {err}"
                ))),
                Err(_) => Err(ssh_error(format!(
                    "failed to join {stream_name} reader thread for `ssh`"
                ))),
            },
            None => Ok(Vec::new()),
        },
        ProcessOutputMode::Inherit | ProcessOutputMode::Null => Ok(Vec::new()),
    }
}

fn format_subprocess_stderr(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        String::new()
    } else {
        format!(": {stderr}")
    }
}

fn process_output_mode(mode: SshStreamMode) -> ProcessOutputMode {
    match mode {
        SshStreamMode::Capture => ProcessOutputMode::Capture,
        SshStreamMode::Inherit => ProcessOutputMode::Inherit,
        SshStreamMode::Null => ProcessOutputMode::Null,
    }
}

fn duration_millis_to_ssh_seconds(value: u64) -> u64 {
    value.div_ceil(1000)
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

fn build_upload_directory_command(
    remote_root: &str,
    create_root: bool,
    options: SshTransferOptions,
) -> Result<String> {
    let mut prefixes = Vec::new();
    if create_root {
        let mkdir = if options.parents { "mkdir -p" } else { "mkdir" };
        prefixes.push(format!(
            "{mkdir} {}",
            shell_quote(remote_root, "remote_path")?
        ));
    }

    let mut command = prefixes.join(" && ");
    if !command.is_empty() {
        command.push_str(" && ");
    }
    command.push_str("cd ");
    command.push_str(&shell_quote(remote_root, "remote_path")?);
    command.push_str(" && tar -xf -");
    Ok(command)
}

fn build_download_directory_command(remote_path: &str) -> Result<String> {
    Ok(format!(
        "cd {} && tar -cf - .",
        shell_quote(remote_path, "remote_path")?
    ))
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

fn ensure_local_dir(path: &Path, field: &str) -> Result<()> {
    let metadata = std::fs::metadata(path)
        .map_err(|err| ssh_error(format!("failed to access `{}`: {err}", path.display())))?;
    if !metadata.is_dir() {
        return Err(ssh_error(format!(
            "`{field}` must be a directory: `{}`",
            path.display()
        )));
    }
    Ok(())
}

fn classify_local_path(path: &Path, field: &str) -> Result<LocalPathKind> {
    let metadata = std::fs::metadata(path)
        .map_err(|err| ssh_error(format!("failed to access `{}`: {err}", path.display())))?;
    if metadata.is_file() {
        return Ok(LocalPathKind::File);
    }
    if metadata.is_dir() {
        return Ok(LocalPathKind::Directory);
    }
    Err(ssh_error(format!(
        "`{field}` must be a file or directory: `{}`",
        path.display()
    )))
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

fn prepare_local_directory_destination(path: &Path, options: SshTransferOptions) -> Result<()> {
    if path.exists() {
        if !path.is_dir() {
            return Err(ssh_error(format!(
                "directory destination must not be a file: `{}`",
                path.display()
            )));
        }
        if !options.overwrite {
            return Err(ssh_error(format!(
                "directory destination already exists: `{}`",
                path.display()
            )));
        }
        return Ok(());
    }

    let create_result = if options.parents {
        std::fs::create_dir_all(path)
    } else {
        std::fs::create_dir(path)
    };
    create_result.map_err(|err| {
        ssh_error(format!(
            "failed to create destination directory `{}`: {err}",
            path.display()
        ))
    })?;
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

fn create_directory_archive(source_dir: &Path, archive_path: &Path) -> Result<u64> {
    let file = File::create(archive_path).map_err(|err| {
        ssh_error(format!(
            "failed to create archive `{}`: {err}",
            archive_path.display()
        ))
    })?;
    let mut builder = Builder::new(file);
    let bytes = append_directory_contents(&mut builder, source_dir, source_dir)?;
    builder.finish().map_err(|err| {
        ssh_error(format!(
            "failed to finalize archive `{}`: {err}",
            archive_path.display()
        ))
    })?;
    Ok(bytes)
}

fn append_directory_contents(
    builder: &mut Builder<File>,
    source_dir: &Path,
    current_dir: &Path,
) -> Result<u64> {
    let mut entries = std::fs::read_dir(current_dir)
        .map_err(|err| ssh_error(format!("failed to read `{}`: {err}", current_dir.display())))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| ssh_error(format!("failed to read `{}`: {err}", current_dir.display())))?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut total_bytes = 0u64;
    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(source_dir)
            .map_err(|err| {
                ssh_error(format!(
                    "failed to resolve archive entry `{}`: {err}",
                    path.display()
                ))
            })?
            .to_path_buf();
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|err| ssh_error(format!("failed to access `{}`: {err}", path.display())))?;

        if metadata.is_dir() {
            builder.append_dir(&relative, &path).map_err(|err| {
                ssh_error(format!(
                    "failed to append directory `{}` to archive: {err}",
                    path.display()
                ))
            })?;
            total_bytes =
                total_bytes.saturating_add(append_directory_contents(builder, source_dir, &path)?);
        } else {
            builder
                .append_path_with_name(&path, &relative)
                .map_err(|err| {
                    ssh_error(format!(
                        "failed to append `{}` to archive: {err}",
                        path.display()
                    ))
                })?;
            if metadata.is_file() {
                total_bytes = total_bytes.saturating_add(metadata.len());
            }
        }
    }

    Ok(total_bytes)
}

fn unpack_directory_archive(archive_path: &Path, destination_root: &Path) -> Result<u64> {
    let file = File::open(archive_path).map_err(|err| {
        ssh_error(format!(
            "failed to open archive `{}`: {err}",
            archive_path.display()
        ))
    })?;
    let mut archive = Archive::new(file);
    let mut bytes = 0u64;

    let entries = archive.entries().map_err(|err| {
        ssh_error(format!(
            "failed to read archive `{}`: {err}",
            archive_path.display()
        ))
    })?;

    for entry in entries {
        let mut entry = entry.map_err(|err| {
            ssh_error(format!(
                "failed to read archive entry from `{}`: {err}",
                archive_path.display()
            ))
        })?;
        let entry_type = entry.header().entry_type();
        let size = entry.header().size().map_err(|err| {
            ssh_error(format!(
                "failed to read archive entry size from `{}`: {err}",
                archive_path.display()
            ))
        })?;
        let unpacked = entry.unpack_in(destination_root).map_err(|err| {
            ssh_error(format!(
                "failed to extract archive `{}` into `{}`: {err}",
                archive_path.display(),
                destination_root.display()
            ))
        })?;
        if !unpacked {
            return Err(ssh_error(format!(
                "archive `{}` contains an entry outside `{}`",
                archive_path.display(),
                destination_root.display()
            )));
        }
        if entry_type.is_file() {
            bytes = bytes.saturating_add(size);
        }
    }

    Ok(bytes)
}

fn validate_remote_directory_destination(
    connection: &SshConnection,
    remote_root: &str,
    options: SshTransferOptions,
) -> Result<String> {
    if connection.exists(remote_root)? {
        if !connection.is_dir(remote_root)? {
            return Err(ssh_error(format!(
                "directory destination must not be a file: `{remote_root}`"
            )));
        }
        if !options.overwrite {
            return Err(ssh_error(format!(
                "directory destination already exists: `{remote_root}`"
            )));
        }
    }
    Ok(remote_root.to_string())
}

fn path_basename(path: &Path, field: &str) -> Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| {
            ssh_error(format!(
                "failed to determine `{field}` basename from `{}`",
                path.display()
            ))
        })
}

fn remote_basename(path: &str) -> Result<String> {
    Path::new(normalize_remote_path(path).as_str())
        .file_name()
        .and_then(|name| name.to_str())
        .map(std::string::ToString::to_string)
        .ok_or_else(|| {
            ssh_error(format!(
                "failed to determine remote path basename: `{path}`"
            ))
        })
}

fn normalize_remote_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

fn join_remote_path(base: &str, child: &str) -> String {
    let base = normalize_remote_path(base);
    if base == "/" {
        format!("/{child}")
    } else {
        format!("{base}/{child}")
    }
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

fn bytes_to_captured_string(bytes: Vec<u8>, mode: SshStreamMode, trim: bool) -> Option<String> {
    match mode {
        SshStreamMode::Capture => {
            let text = String::from_utf8_lossy(&bytes);
            Some(if trim {
                text.trim().to_string()
            } else {
                text.to_string()
            })
        }
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

fn ssh_error(msg: impl Into<String>) -> Error {
    Error::new(ErrorKind::Ssh, msg)
}
