mod ansi;
mod db;
mod error;
mod exec;
mod fs;
mod hash;
mod http;
mod json;
mod net;
mod path;
mod platform;
mod re;
mod script_args;
mod semver;
mod ssh;
mod strings;
mod template;
mod text;
mod toml;

pub use ansi::{Color, StyleOptions};
pub use db::{DbBindValue, DbConnection, DbExecuteResult, DbParams, DbQueryResult, DbRow, DbValue};
pub use error::{Error, ErrorKind, Result};
pub use exec::{
    RunOptions, RunResult, RunStreamMode, format_command_for_display, format_run_failed_message,
    resolve_run_cwd, run_command,
};
pub use fs::{FsCopyOptions, FsCopyResult, FsMkdirOptions};
pub use http::{HttpRequestOptions, HttpResponse};
pub use json::{JsonStringifyOptions, JsonValue};
pub use net::{HostKind, HostPortParts, IpParts, UrlParts};
pub use platform::{Arch, OS, UserHost};
pub use re::{RegexCaptures, RegexMatch, RegexOptions, RegexPattern};
pub use script_args::{
    ParsedScriptArgs, ScriptArgDefault, ScriptArgKind, ScriptArgSpec, ScriptArgValue,
    ScriptArgValues, ScriptArgsParseError, ScriptArgsSchema, parse_script_args,
    validate_script_arg_spec, validate_script_arg_spec_base, validate_script_args_schema,
};
pub use semver::{SemverBuildMetadata, SemverPrerelease, SemverVersion};
pub use ssh::{
    SshAuthRequest, SshConnectRequest, SshConnection, SshConnectionInfo, SshExecOptions,
    SshExecResult, SshHostKeyRequest, SshStreamMode, SshTransferOptions, SshTransferResult,
};
use std::path::Path;
use std::sync::Arc;
pub use strings::{IndentOptions, SplitLinesOptions, SplitOptions};
use tokio::runtime::{Builder, Runtime};
pub use toml::{TomlPathSegment, TomlValue};

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

    pub fn current_user_host(&self) -> UserHost {
        platform::detect_current_user_host()
    }

    pub fn shell_split(&self, input: &str) -> Result<Vec<String>> {
        shlex::split(input).ok_or_else(|| {
            Error::new(ErrorKind::InvalidArgs, "failed to parse shell words")
                .with_op("ptool.sh.split")
                .with_input(input.to_string())
        })
    }

    pub fn path_join<I, S>(&self, segments: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        path::join(segments)
    }

    pub fn path_normalize(&self, path: &str) -> Result<String> {
        path::normalize(path)
    }

    pub fn path_abspath(
        &self,
        path: &str,
        base: Option<&str>,
        current_dir: &Path,
    ) -> Result<String> {
        path::abspath(path, base, current_dir)
    }

    pub fn path_relpath(
        &self,
        path: &str,
        base: Option<&str>,
        current_dir: &Path,
    ) -> Result<String> {
        path::relpath(path, base, current_dir)
    }

    pub fn path_isabs(&self, path: &str) -> Result<bool> {
        path::isabs(path)
    }

    pub fn path_runtime_abspath(&self, path: &str) -> Result<String> {
        path::runtime_abspath(path)
    }

    pub fn path_dirname(&self, path: &str) -> Result<String> {
        path::dirname(path)
    }

    pub fn path_basename(&self, path: &str) -> Result<String> {
        path::basename(path)
    }

    pub fn path_extname(&self, path: &str) -> Result<String> {
        path::extname(path)
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

    pub fn fs_read(&self, path: &str) -> Result<Vec<u8>> {
        fs::read(path)
    }

    pub fn fs_write(&self, path: &str, content: &[u8]) -> Result<()> {
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

    pub fn json_parse(&self, input: &str) -> Result<JsonValue> {
        json::parse(input)
    }

    pub fn json_stringify(
        &self,
        value: &JsonValue,
        options: JsonStringifyOptions,
    ) -> Result<String> {
        json::stringify(value, options)
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

    pub fn semver_is_min_version(&self, current: &str, required: &str) -> Result<bool> {
        let current = semver::strip_prerelease(semver::parse(current)?);
        let required = semver::parse(required)?;
        Ok(required <= current)
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

    pub fn toml_parse(&self, input: &str) -> Result<TomlValue> {
        toml::parse(input)
    }

    pub fn toml_get(&self, input: &str, path: &[TomlPathSegment]) -> Result<Option<TomlValue>> {
        toml::get(input, path)
    }

    pub fn toml_set(
        &self,
        input: &str,
        path: &[TomlPathSegment],
        value: &TomlValue,
    ) -> Result<String> {
        toml::set(input, path, value)
    }

    pub fn toml_remove(&self, input: &str, path: &[TomlPathSegment]) -> Result<String> {
        toml::remove(input, path)
    }

    pub fn toml_stringify(&self, value: &TomlValue) -> Result<String> {
        toml::stringify(value)
    }

    pub fn template_render(&self, template: &str, context: &JsonValue) -> Result<String> {
        template::render(template, context)
    }

    pub fn text_unindent(&self, input: &str) -> String {
        text::unindent(input)
    }

    pub fn str_trim(&self, input: &str) -> String {
        strings::trim(input)
    }

    pub fn str_trim_start(&self, input: &str) -> String {
        strings::trim_start(input)
    }

    pub fn str_trim_end(&self, input: &str) -> String {
        strings::trim_end(input)
    }

    pub fn str_is_blank(&self, input: &str) -> bool {
        strings::is_blank(input)
    }

    pub fn str_starts_with(&self, input: &str, prefix: &str) -> bool {
        strings::starts_with(input, prefix)
    }

    pub fn str_ends_with(&self, input: &str, suffix: &str) -> bool {
        strings::ends_with(input, suffix)
    }

    pub fn str_contains(&self, input: &str, needle: &str) -> bool {
        strings::contains(input, needle)
    }

    pub fn str_split(
        &self,
        input: &str,
        separator: &str,
        options: SplitOptions,
    ) -> Result<Vec<String>> {
        strings::split(input, separator, options)
    }

    pub fn str_split_lines(&self, input: &str, options: SplitLinesOptions) -> Vec<String> {
        strings::split_lines(input, options)
    }

    pub fn str_join(&self, parts: &[String], separator: &str) -> String {
        strings::join(parts, separator)
    }

    pub fn str_replace(
        &self,
        input: &str,
        from: &str,
        to: &str,
        limit: Option<usize>,
    ) -> Result<String> {
        strings::replace(input, from, to, limit)
    }

    pub fn str_repeat(&self, input: &str, count: i64) -> Result<String> {
        strings::repeat(input, count)
    }

    pub fn str_cut_prefix(&self, input: &str, prefix: &str) -> String {
        strings::cut_prefix(input, prefix)
    }

    pub fn str_cut_suffix(&self, input: &str, suffix: &str) -> String {
        strings::cut_suffix(input, suffix)
    }

    pub fn str_indent(&self, input: &str, prefix: &str, options: IndentOptions) -> String {
        strings::indent(input, prefix, options)
    }

    pub fn re_compile(&self, pattern: &str, options: RegexOptions) -> Result<RegexPattern> {
        re::compile(pattern, options)
    }

    pub fn re_escape(&self, text: &str) -> String {
        re::escape(text)
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
