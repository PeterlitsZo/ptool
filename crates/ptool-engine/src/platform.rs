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

pub(crate) fn detect_current_os() -> OS {
    normalize_os(std::env::consts::OS)
        .unwrap_or_else(|other| panic!("unsupported operating system: {other}"))
}

pub(crate) fn detect_current_arch() -> Arch {
    normalize_arch(std::env::consts::ARCH)
        .unwrap_or_else(|other| panic!("unsupported architecture: {other}"))
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
