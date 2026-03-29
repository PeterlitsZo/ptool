use crate::{Error, ErrorKind, Result};
use std::net::IpAddr;
use std::str::FromStr;
use url::{Host, Url};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostKind {
    Domain,
    Ipv4,
    Ipv6,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UrlParts {
    pub normalized: String,
    pub scheme: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub host: Option<String>,
    pub host_kind: Option<HostKind>,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IpParts {
    pub normalized: String,
    pub version: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostPortParts {
    pub normalized: String,
    pub host: String,
    pub host_kind: HostKind,
    pub port: u16,
}

pub fn parse_url(input: &str) -> Result<UrlParts> {
    ensure_non_empty(input)?;

    let parsed = Url::parse(input).map_err(|err| {
        Error::new(
            ErrorKind::InvalidUrl,
            format!("invalid URL {input:?}: {err}"),
        )
    })?;

    let (host, host_kind) = match parsed.host() {
        Some(host) => {
            let (host, kind) = host_to_parts(host);
            (Some(host), Some(kind))
        }
        None => (None, None),
    };

    let username = if parsed.username().is_empty() {
        None
    } else {
        Some(parsed.username().to_string())
    };

    Ok(UrlParts {
        normalized: parsed.to_string(),
        scheme: parsed.scheme().to_string(),
        username,
        password: parsed.password().map(ToOwned::to_owned),
        host,
        host_kind,
        port: parsed.port(),
        path: parsed.path().to_string(),
        query: parsed.query().map(ToOwned::to_owned),
        fragment: parsed.fragment().map(ToOwned::to_owned),
    })
}

pub fn parse_ip(input: &str) -> Result<IpParts> {
    ensure_non_empty(input)?;

    let parsed = IpAddr::from_str(input).map_err(|err| {
        Error::new(
            ErrorKind::InvalidIp,
            format!("invalid IP address {input:?}: {err}"),
        )
    })?;

    Ok(IpParts {
        normalized: parsed.to_string(),
        version: match parsed {
            IpAddr::V4(_) => 4,
            IpAddr::V6(_) => 6,
        },
    })
}

pub fn parse_host_port(input: &str) -> Result<HostPortParts> {
    ensure_non_empty(input)?;

    let (host, port) = parse_host_port_parts(input)?;
    let (host, host_kind) = parse_host_value(&host, input)?;
    let normalized = match host_kind {
        HostKind::Ipv6 => format!("[{host}]:{port}"),
        HostKind::Domain | HostKind::Ipv4 => format!("{host}:{port}"),
    };

    Ok(HostPortParts {
        normalized,
        host,
        host_kind,
        port,
    })
}

fn ensure_non_empty(input: &str) -> Result<()> {
    if input.is_empty() {
        return Err(Error::new(ErrorKind::EmptyInput, "empty input"));
    }
    Ok(())
}

fn parse_host_port_parts(input: &str) -> Result<(String, u16)> {
    if let Some(rest) = input.strip_prefix('[') {
        let Some(end) = rest.find(']') else {
            return Err(Error::new(
                ErrorKind::InvalidHostPort,
                format!("invalid IPv6 host:port {input:?}"),
            ));
        };

        let host = &rest[..end];
        let suffix = &rest[end + 1..];
        let Some(port) = suffix.strip_prefix(':') else {
            return Err(Error::new(
                ErrorKind::InvalidHostPort,
                format!("invalid host:port {input:?}"),
            ));
        };

        if port.is_empty() || suffix.matches(':').count() != 1 {
            return Err(Error::new(
                ErrorKind::InvalidHostPort,
                format!("invalid host:port {input:?}"),
            ));
        }

        return Ok((host.to_string(), parse_port(port, input)?));
    }

    let Some((host, port)) = input.rsplit_once(':') else {
        return Err(Error::new(
            ErrorKind::MissingPort,
            format!("missing port in {input:?}"),
        ));
    };

    if host.is_empty() || port.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidHostPort,
            format!("invalid host:port {input:?}"),
        ));
    }

    if host.contains(':') {
        return Err(Error::new(
            ErrorKind::InvalidHostPort,
            format!("IPv6 host with port must use `[addr]:port` in {input:?}"),
        ));
    }

    Ok((host.to_string(), parse_port(port, input)?))
}

fn parse_port(port: &str, input: &str) -> Result<u16> {
    port.parse::<u16>().map_err(|err| {
        Error::new(
            ErrorKind::InvalidPort,
            format!("invalid port {port:?} in {input:?}: {err}"),
        )
    })
}

fn parse_host_value(host: &str, input: &str) -> Result<(String, HostKind)> {
    if let Ok(ip) = IpAddr::from_str(host) {
        return Ok(match ip {
            IpAddr::V4(ip) => (ip.to_string(), HostKind::Ipv4),
            IpAddr::V6(ip) => (ip.to_string(), HostKind::Ipv6),
        });
    }

    Host::parse(host).map(host_to_parts).map_err(|err| {
        Error::new(
            ErrorKind::InvalidHost,
            format!("invalid host {host:?} in {input:?}: {err}"),
        )
    })
}

fn host_to_parts<S: AsRef<str>>(host: Host<S>) -> (String, HostKind) {
    match host {
        Host::Domain(host) => (host.as_ref().to_string(), HostKind::Domain),
        Host::Ipv4(host) => (host.to_string(), HostKind::Ipv4),
        Host::Ipv6(host) => (host.to_string(), HostKind::Ipv6),
    }
}
