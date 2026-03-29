mod ansi;
mod error;
mod hash;
mod net;
mod platform;

pub use ansi::{Color, StyleOptions};
pub use error::{Error, ErrorKind, Result};
pub use net::{HostKind, HostPortParts, IpParts, UrlParts};
pub use platform::{Arch, OS};

#[derive(Debug, Default)]
pub struct PtoolEngine;

impl PtoolEngine {
    pub fn new() -> Self {
        Self
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

    pub fn parse_url(&self, input: &str) -> Result<UrlParts> {
        net::parse_url(input)
    }

    pub fn parse_ip(&self, input: &str) -> Result<IpParts> {
        net::parse_ip(input)
    }

    pub fn parse_host_port(&self, input: &str) -> Result<HostPortParts> {
        net::parse_host_port(input)
    }
}
