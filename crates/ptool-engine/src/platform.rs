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
}

pub(crate) fn detect_current_os() -> OS {
    match std::env::consts::OS {
        "linux" => OS::Linux,
        "macos" => OS::Macos,
        "windows" => OS::Windows,
        other => panic!("unsupported operating system: {other}"),
    }
}

pub(crate) fn detect_current_arch() -> Arch {
    match std::env::consts::ARCH {
        "x86_64" => Arch::X86_64,
        "aarch64" => Arch::Aarch64,
        other => panic!("unsupported architecture: {other}"),
    }
}
