use crate::http::{
    HttpRequestOptions, HttpResponse, apply_request_header_overrides, build_request_body_for,
    build_request_url, collect_response_headers, http_error_for, http_status_error_for,
    invalid_http_options_for, is_http_error, parse_headers, parse_method,
    parse_nonnegative_usize_for, parse_timeout_ms, parse_timeout_value,
};
use crate::{Error, ErrorKind, Result};
use reqwest::StatusCode;
use reqwest::header::AUTHORIZATION;
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

const SSH_HTTP_REQUEST_OP: &str = "ptool.ssh.Connection:http_request";
const SSH_HTTP_FRAME_MAGIC: &[u8] = b"PTSSHHTTP\n";

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
    remote_http_client: Option<RemoteHttpClient>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RemoteHttpClient {
    Curl,
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
            remote_http_client: None,
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

    pub fn http_request(&self, options: HttpRequestOptions) -> Result<HttpResponse> {
        match self.ensure_remote_http_client()? {
            RemoteHttpClient::Curl => self.http_request_with_curl(options),
        }
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
        let destination_path = self.resolve_remote_file_destination(local_path, remote_path)?;

        let content = std::fs::read(local_path).map_err(|err| {
            ssh_error(format!("failed to read `{}`: {err}", local_path.display()))
        })?;

        if options.echo {
            println!(
                "[ssh upload {}] {} -> {}",
                self.info().target,
                local_path.display(),
                destination_path
            );
        }

        let command = build_upload_command(&destination_path, options)?;
        self.exec_binary(command.as_str(), Some(content), false)?;

        Ok(SshTransferResult {
            bytes: std::fs::metadata(local_path)
                .map(|metadata| metadata.len())
                .unwrap_or(0),
            from: local_path.display().to_string(),
            to: format!("{}:{}", self.info().target, destination_path),
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

    fn ensure_remote_http_client(&self) -> Result<RemoteHttpClient> {
        if let Some(client) = self.state.borrow().remote_http_client {
            return Ok(client);
        }

        let result = self.exec_ssh(
            "command -v curl >/dev/null 2>&1",
            None,
            ProcessOutputMode::Null,
            ProcessOutputMode::Capture,
        )?;
        match result.code {
            Some(0) => {
                self.state.borrow_mut().remote_http_client = Some(RemoteHttpClient::Curl);
                Ok(RemoteHttpClient::Curl)
            }
            Some(1) if result.stderr.is_empty() => Err(ssh_error(
                "remote HTTP request requires `curl` on the SSH host",
            )),
            _ => Err(build_binary_exec_failed_error(
                &self.info().target,
                "command -v curl >/dev/null 2>&1",
                result.code,
                &result.stderr,
            )),
        }
    }

    fn http_request_with_curl(&self, options: HttpRequestOptions) -> Result<HttpResponse> {
        let HttpRequestOptions {
            url,
            method,
            headers,
            body,
            query,
            json,
            form,
            timeout_ms,
            connect_timeout_ms,
            follow_redirects,
            max_redirects,
            user_agent,
            basic_auth,
            bearer_token,
            fail_on_http_error,
        } = options;

        if basic_auth.is_some() && bearer_token.is_some() {
            return Err(invalid_http_options_for(
                SSH_HTTP_REQUEST_OP,
                "`basic_auth` and `bearer_token` are mutually exclusive",
            ));
        }
        if matches!(follow_redirects, Some(false)) && max_redirects.is_some() {
            return Err(invalid_http_options_for(
                SSH_HTTP_REQUEST_OP,
                "`max_redirects` cannot be set when `follow_redirects` is false",
            ));
        }

        let url = build_request_url(&url, &query)?;
        let method = parse_method(method)?;
        let mut headers = parse_headers(headers)?;
        let body = build_request_body_for(SSH_HTTP_REQUEST_OP, body, json, form, &mut headers)?;
        apply_request_header_overrides(&mut headers, user_agent.as_deref())?;
        if basic_auth.is_some() || bearer_token.is_some() {
            headers.remove(AUTHORIZATION);
        }

        let mut curl_args = vec![
            "curl".to_string(),
            "--silent".to_string(),
            "--show-error".to_string(),
            "--globoff".to_string(),
            "--request".to_string(),
            shell_quote_arg(method.as_str())?,
            "--dump-header".to_string(),
            "\"$headers\"".to_string(),
            "--output".to_string(),
            "\"$body\"".to_string(),
            "--write-out".to_string(),
            shell_quote_arg("%{http_code}\n%{url_effective}")?,
        ];

        let timeout_ms = parse_timeout_ms(timeout_ms, "timeout_ms")?;
        curl_args.push("--max-time".to_string());
        curl_args.push(format_curl_timeout(timeout_ms));

        if let Some(connect_timeout_ms) = connect_timeout_ms {
            curl_args.push("--connect-timeout".to_string());
            curl_args.push(format_curl_timeout(parse_timeout_value(
                connect_timeout_ms,
                "connect_timeout_ms",
            )?));
        }

        if follow_redirects != Some(false) {
            curl_args.push("--location".to_string());
            if let Some(max_redirects) = max_redirects {
                curl_args.push("--max-redirs".to_string());
                curl_args.push(
                    parse_nonnegative_usize_for(
                        SSH_HTTP_REQUEST_OP,
                        max_redirects,
                        "max_redirects",
                    )?
                    .to_string(),
                );
            }
        }

        for (name, value) in collect_response_headers(&headers) {
            curl_args.push("--header".to_string());
            curl_args.push(shell_quote_arg(&format!("{name}: {value}"))?);
        }

        if let Some((username, password)) = basic_auth {
            curl_args.push("--user".to_string());
            curl_args.push(shell_quote_arg(&format!("{username}:{password}"))?);
        } else if let Some(bearer_token) = bearer_token {
            curl_args.push("--header".to_string());
            curl_args.push(shell_quote_arg(&format!(
                "Authorization: Bearer {bearer_token}"
            ))?);
        }

        if body.is_some() {
            curl_args.push("--data-binary".to_string());
            curl_args.push("@-".to_string());
        }

        curl_args.push(shell_quote_arg(&url)?);

        let command = build_remote_curl_command(&curl_args);
        let result = self.exec_ssh(
            &command,
            body,
            ProcessOutputMode::Capture,
            ProcessOutputMode::Capture,
        )?;
        if result.code != Some(0) {
            return Err(build_binary_exec_failed_error(
                &self.info().target,
                &command,
                result.code,
                &result.stderr,
            ));
        }

        let frame = parse_remote_http_frame(&result.stdout)?;
        let (status, final_url) = parse_remote_http_meta(&frame.meta, &url)?;
        if frame.curl_exit != 0 {
            let stderr = String::from_utf8_lossy(&frame.stderr).trim().to_string();
            let detail = if stderr.is_empty() {
                format!("remote curl exited with status {}", frame.curl_exit)
            } else {
                stderr
            };
            return Err(
                http_error_for(SSH_HTTP_REQUEST_OP, format!("request failed: {detail}"))
                    .with_url(final_url),
            );
        }

        let headers = parse_remote_response_headers(&frame.headers)?;
        let status_code = StatusCode::from_u16(status).map_err(|_| {
            http_error_for(
                SSH_HTTP_REQUEST_OP,
                format!("invalid response status `{status}`"),
            )
        })?;
        if fail_on_http_error && is_http_error(status_code) {
            return Err(http_status_error_for(
                SSH_HTTP_REQUEST_OP,
                status_code,
                &final_url,
            ));
        }

        Ok(HttpResponse::from_parts_with_op(
            SSH_HTTP_REQUEST_OP,
            i64::from(status),
            final_url,
            headers,
            frame.body,
        ))
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

    fn resolve_remote_file_destination(
        &self,
        local_path: &Path,
        remote_path: &str,
    ) -> Result<String> {
        let is_directory_hint = remote_path.ends_with('/');
        let remote_path = normalize_remote_path(remote_path);

        if self.exists(&remote_path)? {
            if self.is_dir(&remote_path)? {
                return Ok(join_remote_path(
                    &remote_path,
                    &path_basename(local_path, "local_path")?,
                ));
            }
            if is_directory_hint {
                return Err(ssh_error(format!(
                    "`remote_path` must be a directory for file upload: `{remote_path}`"
                )));
            }
        } else if is_directory_hint {
            return Ok(join_remote_path(
                &remote_path,
                &path_basename(local_path, "local_path")?,
            ));
        }

        Ok(remote_path)
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

struct RemoteHttpFrame {
    curl_exit: i64,
    headers: Vec<u8>,
    body: Vec<u8>,
    meta: Vec<u8>,
    stderr: Vec<u8>,
}

fn build_remote_curl_command(args: &[String]) -> String {
    let curl_command = format!("{} > \"$meta\" 2> \"$stderr_file\"", args.join(" "));
    format!(
        "tmpdir=$(mktemp -d) || exit 1\n\
headers=\"$tmpdir/headers\"\n\
body=\"$tmpdir/body\"\n\
meta=\"$tmpdir/meta\"\n\
stderr_file=\"$tmpdir/stderr\"\n\
trap 'rm -rf \"$tmpdir\"' EXIT HUP INT TERM\n\
: > \"$headers\"\n\
: > \"$body\"\n\
: > \"$meta\"\n\
: > \"$stderr_file\"\n\
{}\n\
curl_exit=$?\n\
printf 'PTSSHHTTP\\n%s\\n' \"$curl_exit\"\n\
wc -c < \"$headers\"\n\
cat \"$headers\"\n\
wc -c < \"$body\"\n\
cat \"$body\"\n\
wc -c < \"$meta\"\n\
cat \"$meta\"\n\
wc -c < \"$stderr_file\"\n\
cat \"$stderr_file\"\n",
        curl_command
    )
}

fn parse_remote_http_frame(bytes: &[u8]) -> Result<RemoteHttpFrame> {
    let mut cursor = bytes;
    if !cursor.starts_with(SSH_HTTP_FRAME_MAGIC) {
        return Err(http_error_for(
            SSH_HTTP_REQUEST_OP,
            "remote HTTP response framing is invalid",
        ));
    }
    cursor = &cursor[SSH_HTTP_FRAME_MAGIC.len()..];

    let curl_exit = parse_frame_line_i64(&mut cursor, "curl exit code")?;
    let headers = parse_frame_block(&mut cursor, "response headers")?;
    let body = parse_frame_block(&mut cursor, "response body")?;
    let meta = parse_frame_block(&mut cursor, "response metadata")?;
    let stderr = parse_frame_block(&mut cursor, "response stderr")?;

    if !cursor.is_empty() {
        return Err(http_error_for(
            SSH_HTTP_REQUEST_OP,
            "remote HTTP response framing has trailing bytes",
        ));
    }

    Ok(RemoteHttpFrame {
        curl_exit,
        headers,
        body,
        meta,
        stderr,
    })
}

fn parse_frame_line_i64(cursor: &mut &[u8], label: &str) -> Result<i64> {
    let value = parse_frame_line(cursor, label)?;
    value.parse::<i64>().map_err(|_| {
        http_error_for(
            SSH_HTTP_REQUEST_OP,
            format!("remote HTTP response has invalid {label}"),
        )
    })
}

fn parse_frame_block(cursor: &mut &[u8], label: &str) -> Result<Vec<u8>> {
    let len = parse_frame_line(cursor, label)?
        .parse::<usize>()
        .map_err(|_| {
            http_error_for(
                SSH_HTTP_REQUEST_OP,
                format!("remote HTTP response has invalid {label} length"),
            )
        })?;
    if cursor.len() < len {
        return Err(http_error_for(
            SSH_HTTP_REQUEST_OP,
            format!("remote HTTP response is truncated while reading {label}"),
        ));
    }
    let (block, rest) = cursor.split_at(len);
    *cursor = rest;
    Ok(block.to_vec())
}

fn parse_frame_line<'a>(cursor: &mut &'a [u8], label: &str) -> Result<&'a str> {
    let Some(pos) = cursor.iter().position(|byte| *byte == b'\n') else {
        return Err(http_error_for(
            SSH_HTTP_REQUEST_OP,
            format!("remote HTTP response is missing {label}"),
        ));
    };
    let (line, rest) = cursor.split_at(pos);
    *cursor = &rest[1..];
    std::str::from_utf8(line).map_err(|_| {
        http_error_for(
            SSH_HTTP_REQUEST_OP,
            format!("remote HTTP response has non-UTF-8 {label}"),
        )
    })
}

fn parse_remote_http_meta(meta: &[u8], fallback_url: &str) -> Result<(u16, String)> {
    let text = String::from_utf8(meta.to_vec()).map_err(|_| {
        http_error_for(
            SSH_HTTP_REQUEST_OP,
            "remote HTTP response metadata is not valid UTF-8",
        )
    })?;
    let mut lines = text.lines();
    let status = lines
        .next()
        .ok_or_else(|| {
            http_error_for(
                SSH_HTTP_REQUEST_OP,
                "remote HTTP response metadata is missing status",
            )
        })?
        .parse::<u16>()
        .map_err(|_| {
            http_error_for(
                SSH_HTTP_REQUEST_OP,
                "remote HTTP response metadata has invalid status",
            )
        })?;
    let url = lines
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback_url)
        .to_string();
    Ok((status, url))
}

fn parse_remote_response_headers(bytes: &[u8]) -> Result<Vec<(String, String)>> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }

    let normalized = String::from_utf8_lossy(bytes).replace("\r\n", "\n");
    let mut last_block = None;
    for block in normalized.split("\n\n") {
        let trimmed = block.trim();
        if trimmed.starts_with("HTTP/") {
            last_block = Some(trimmed.to_string());
        }
    }
    let block = last_block.ok_or_else(|| {
        http_error_for(
            SSH_HTTP_REQUEST_OP,
            "remote HTTP response headers are missing the status line",
        )
    })?;

    let mut headers = Vec::new();
    for line in block.lines().skip(1) {
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(http_error_for(
                SSH_HTTP_REQUEST_OP,
                format!("remote HTTP response has invalid header line `{line}`"),
            ));
        };
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }
    Ok(headers)
}

fn format_curl_timeout(timeout_ms: u64) -> String {
    let seconds = timeout_ms / 1000;
    let millis = timeout_ms % 1000;
    if millis == 0 {
        seconds.to_string()
    } else {
        format!("{seconds}.{millis:03}")
    }
}

fn shell_quote_arg(value: &str) -> Result<String> {
    try_quote(value)
        .map(|quoted| quoted.into_owned())
        .map_err(|err| ssh_error(format!("invalid shell argument: {err}")))
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
