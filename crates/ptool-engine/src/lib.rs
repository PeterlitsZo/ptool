mod ansi;
mod db;
mod error;
mod exec;
mod fs;
mod hash;
mod http;
mod net;
mod platform;
mod semver;
mod ssh;

pub use ansi::{Color, StyleOptions};
pub use db::{DbBindValue, DbConnection, DbExecuteResult, DbParams, DbQueryResult, DbRow, DbValue};
pub use error::{Error, ErrorKind, Result};
pub use exec::{
    RunOptions, RunResult, RunStreamMode, format_command_for_display, format_run_failed_message,
    resolve_run_cwd, run_command,
};
pub use fs::{FsCopyOptions, FsCopyResult, FsMkdirOptions};
pub use http::{HttpRequestOptions, HttpResponse};
pub use net::{HostKind, HostPortParts, IpParts, UrlParts};
pub use platform::{Arch, OS};
pub use semver::{SemverBuildMetadata, SemverPrerelease, SemverVersion};
pub use ssh::{
    SshAuthRequest, SshConnectRequest, SshConnection, SshConnectionInfo, SshExecOptions,
    SshExecResult, SshHostKeyRequest, SshStreamMode, SshTransferOptions, SshTransferResult,
};
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::{Builder, Runtime};

#[derive(Clone, Debug)]
pub struct PtoolEngine {
    runtime: Arc<Runtime>,
}

impl Default for PtoolEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PtoolEngine {
    pub fn new() -> Self {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build ptool-engine tokio runtime");

        Self {
            runtime: Arc::new(runtime),
        }
    }

    pub fn ansi_style(&self, text: String, options: StyleOptions) -> String {
        ansi::style(text, options)
    }

    pub fn current_os(&self) -> OS {
        platform::detect_current_os()
    }

    pub fn current_arch(&self) -> Arch {
        platform::detect_current_arch()
    }

    pub fn hash_sha256_hex(&self, bytes: &[u8]) -> String {
        hash::sha256_hex(bytes)
    }

    pub fn hash_sha1_hex(&self, bytes: &[u8]) -> String {
        hash::sha1_hex(bytes)
    }

    pub fn hash_md5_hex(&self, bytes: &[u8]) -> String {
        hash::md5_hex(bytes)
    }

    pub fn fs_read(&self, path: &str) -> Result<String> {
        fs::read(path)
    }

    pub fn fs_write(&self, path: &str, content: &str) -> Result<()> {
        fs::write(path, content)
    }

    pub fn fs_mkdir(&self, path: &str, options: FsMkdirOptions) -> Result<()> {
        fs::mkdir(path, options)
    }

    pub fn fs_exists(&self, path: &str) -> bool {
        fs::exists(path)
    }

    pub fn fs_glob(&self, pattern: &str, base_dir: &Path) -> Result<Vec<String>> {
        fs::glob(pattern, base_dir)
    }

    pub fn fs_copy_local(
        &self,
        src: &str,
        dst: &str,
        options: FsCopyOptions,
    ) -> Result<FsCopyResult> {
        fs::copy_local(src, dst, options)
    }

    pub fn parse_url(&self, input: &str) -> Result<UrlParts> {
        net::parse_url(input)
    }

    pub fn parse_ip(&self, input: &str) -> Result<IpParts> {
        net::parse_ip(input)
    }

    pub fn parse_host_port(&self, input: &str) -> Result<HostPortParts> {
        net::parse_host_port(input)
    }

    pub fn http_request(&self, options: HttpRequestOptions) -> Result<HttpResponse> {
        http::request(options)
    }

    pub fn semver_parse(&self, input: &str) -> Result<SemverVersion> {
        semver::parse(input)
    }

    pub fn semver_is_valid(&self, input: &str) -> bool {
        semver::is_valid(input)
    }

    pub fn semver_compare(&self, a: &SemverVersion, b: &SemverVersion) -> std::cmp::Ordering {
        semver::compare(a, b)
    }

    pub fn semver_strip_prerelease(&self, version: SemverVersion) -> SemverVersion {
        semver::strip_prerelease(version)
    }

    pub fn semver_bump(
        &self,
        version: SemverVersion,
        op: &str,
        channel: Option<&str>,
    ) -> Result<SemverVersion> {
        semver::bump(version, op, channel)
    }

    pub fn db_connect(&self, url: &str, current_dir: &Path) -> Result<DbConnection> {
        db::connect(Arc::clone(&self.runtime), url, current_dir)
    }

    pub fn run_command(&self, options: &RunOptions, current_dir: &Path) -> Result<RunResult> {
        exec::run_command(options, current_dir)
    }

    pub fn ssh_connect(
        &self,
        request: SshConnectRequest,
        current_dir: &Path,
    ) -> Result<SshConnection> {
        ssh::connect(Arc::clone(&self.runtime), request, current_dir)
    }
}
