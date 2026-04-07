#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    EmptyInput,
    EmptyPath,
    InvalidGlob,
    InvalidUrl,
    InvalidIp,
    InvalidHttpMethod,
    InvalidHttpHeader,
    InvalidHttpTimeout,
    InvalidSemver,
    InvalidSemverOperation,
    MissingPort,
    InvalidHost,
    InvalidPort,
    InvalidHostPort,
    InvalidFsOption,
    AlreadyExists,
    NotAFile,
    Http,
    Io,
    Db,
    SemverOverflow,
    Ssh,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
        Self {
            kind,
            msg: msg.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl std::error::Error for Error {}
