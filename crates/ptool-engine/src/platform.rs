use std::env;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OS {
    Linux,
    Macos,
    Windows,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arch {
    X86_64,
    Aarch64,
    X86,
    Arm,
    Riscv64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserHost {
    pub user: String,
    pub host: String,
}

static CURRENT_USER_HOST: OnceLock<UserHost> = OnceLock::new();

pub(crate) fn detect_current_os() -> OS {
    normalize_os(std::env::consts::OS)
        .unwrap_or_else(|other| panic!("unsupported operating system: {other}"))
}

pub(crate) fn detect_current_arch() -> Arch {
    normalize_arch(std::env::consts::ARCH)
        .unwrap_or_else(|other| panic!("unsupported architecture: {other}"))
}

pub(crate) fn detect_current_user_host() -> UserHost {
    CURRENT_USER_HOST
        .get_or_init(detect_current_user_host_uncached)
        .clone()
}

pub(crate) fn detect_current_username() -> Option<String> {
    first_non_empty_env_var(&["USER", "USERNAME"])
}

pub(crate) fn detect_current_hostname() -> Option<String> {
    detect_current_host()
}

pub(crate) fn getenv(name: &str) -> Option<String> {
    env::var(name).ok()
}

pub(crate) fn env_vars() -> Vec<(String, String)> {
    let mut vars: Vec<(String, String)> = env::vars_os()
        .filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?)))
        .collect();
    vars.sort_by(|a, b| a.0.cmp(&b.0));
    vars
}

pub(crate) fn home_dir() -> Option<String> {
    first_non_empty_env_var(&["HOME", "USERPROFILE"]).or_else(|| {
        let drive = env::var("HOMEDRIVE").ok()?;
        let path = env::var("HOMEPATH").ok()?;
        let home = format!("{drive}{path}");
        (!home.is_empty()).then_some(home)
    })
}

pub(crate) fn temp_dir() -> String {
    path_to_string(&env::temp_dir())
}

pub(crate) fn current_pid() -> u32 {
    std::process::id()
}

pub(crate) fn current_exe() -> Option<String> {
    let path = env::current_exe().ok()?;
    Some(path_to_string(&path))
}

fn normalize_os(os: &'static str) -> Result<OS, &'static str> {
    match os {
        "linux" => Ok(OS::Linux),
        "macos" => Ok(OS::Macos),
        "windows" => Ok(OS::Windows),
        other => Err(other),
    }
}

fn normalize_arch(arch: &'static str) -> Result<Arch, &'static str> {
    match arch {
        "x86_64" => Ok(Arch::X86_64),
        "aarch64" => Ok(Arch::Aarch64),
        "x86" | "i386" | "i486" | "i586" | "i686" => Ok(Arch::X86),
        "arm" | "armv6" | "armv6l" | "armv7" | "armv7l" => Ok(Arch::Arm),
        "riscv64" | "riscv64gc" => Ok(Arch::Riscv64),
        other => Err(other),
    }
}

fn detect_current_user_host_uncached() -> UserHost {
    UserHost {
        user: first_non_empty_env_var(&["USER", "USERNAME"])
            .unwrap_or_else(|| "<unknown-user>".to_string()),
        host: detect_current_host().unwrap_or_else(|| "<unknown-host>".to_string()),
    }
}

fn detect_current_host() -> Option<String> {
    first_non_empty_env_var(&["HOSTNAME", "COMPUTERNAME"]).or_else(detect_host_via_hostname_command)
}

fn first_non_empty_env_var(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| env::var(key).ok().filter(|value| !value.is_empty()))
}

fn detect_host_via_hostname_command() -> Option<String> {
    let output = ProcessCommand::new("hostname").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let host = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!host.is_empty()).then_some(host)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
