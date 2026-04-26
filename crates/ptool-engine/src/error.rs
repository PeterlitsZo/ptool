#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    EmptyInput,
    EmptyPath,
    InvalidGlob,
    InvalidArgs,
    InvalidUrl,
    InvalidIp,
    InvalidHttpMethod,
    InvalidHttpHeader,
    InvalidHttpTimeout,
    InvalidHttpOptions,
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ErrorMeta {
    pub op: Option<String>,
    pub detail: Option<String>,
    pub retryable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandErrorPayload {
    pub cmd: String,
    pub status: Option<i32>,
    pub stderr: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpErrorPayload {
    pub url: String,
    pub status: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorPayload {
    Path { path: String },
    Input { input: String },
    Command(CommandErrorPayload),
    Http(HttpErrorPayload),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: String,
    pub meta: ErrorMeta,
    pub payload: Option<Box<ErrorPayload>>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(kind: ErrorKind, msg: impl Into<String>) -> Self {
        Self {
            kind,
            msg: msg.into(),
            meta: ErrorMeta::default(),
            payload: None,
        }
    }

    pub fn with_op(mut self, op: impl Into<String>) -> Self {
        self.meta.op = Some(op.into());
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.meta.detail = Some(detail.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.payload = Some(Box::new(ErrorPayload::Path { path: path.into() }));
        self
    }

    pub fn with_cmd(mut self, cmd: impl Into<String>) -> Self {
        let cmd = cmd.into();
        match self.payload.as_deref_mut() {
            Some(ErrorPayload::Command(payload)) => payload.cmd = cmd,
            _ => {
                self.payload = Some(Box::new(ErrorPayload::Command(CommandErrorPayload {
                    cmd,
                    status: None,
                    stderr: None,
                })));
            }
        }
        self
    }

    pub fn with_status(mut self, status: i32) -> Self {
        match self.payload.as_deref_mut() {
            Some(ErrorPayload::Command(payload)) => payload.status = Some(status),
            Some(ErrorPayload::Http(payload)) => payload.status = Some(status),
            _ => {}
        }
        self
    }

    pub fn with_stderr(mut self, stderr: impl Into<String>) -> Self {
        let stderr = stderr.into();
        if let Some(ErrorPayload::Command(payload)) = self.payload.as_deref_mut() {
            payload.stderr = Some(stderr);
        }
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        match self.payload.as_deref_mut() {
            Some(ErrorPayload::Http(payload)) => payload.url = url,
            _ => {
                self.payload = Some(Box::new(ErrorPayload::Http(HttpErrorPayload {
                    url,
                    status: None,
                })));
            }
        }
        self
    }

    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.payload = Some(Box::new(ErrorPayload::Input {
            input: input.into(),
        }));
        self
    }

    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.meta.retryable = retryable;
        self
    }

    pub fn op(&self) -> Option<&str> {
        self.meta.op.as_deref()
    }

    pub fn detail(&self) -> Option<&str> {
        self.meta.detail.as_deref()
    }

    pub fn path(&self) -> Option<&str> {
        match self.payload.as_deref() {
            Some(ErrorPayload::Path { path }) => Some(path.as_str()),
            _ => None,
        }
    }

    pub fn cmd(&self) -> Option<&str> {
        match self.payload.as_deref() {
            Some(ErrorPayload::Command(payload)) => Some(payload.cmd.as_str()),
            _ => None,
        }
    }

    pub fn status(&self) -> Option<i32> {
        match self.payload.as_deref() {
            Some(ErrorPayload::Command(payload)) => payload.status,
            Some(ErrorPayload::Http(payload)) => payload.status,
            _ => None,
        }
    }

    pub fn stderr(&self) -> Option<&str> {
        match self.payload.as_deref() {
            Some(ErrorPayload::Command(payload)) => payload.stderr.as_deref(),
            _ => None,
        }
    }

    pub fn url(&self) -> Option<&str> {
        match self.payload.as_deref() {
            Some(ErrorPayload::Http(payload)) => Some(payload.url.as_str()),
            _ => None,
        }
    }

    pub fn input(&self) -> Option<&str> {
        match self.payload.as_deref() {
            Some(ErrorPayload::Input { input }) => Some(input.as_str()),
            _ => None,
        }
    }

    pub fn retryable(&self) -> bool {
        self.meta.retryable
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.msg)
    }
}

impl std::error::Error for Error {}
