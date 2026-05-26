use crate::{Error, ErrorKind, Result};
use opendal::{EntryMode, Metadata, Operator, services::S3};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3ConnectOptions {
    pub bucket: String,
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
    pub root: Option<String>,
    pub allow_anonymous: bool,
}

#[derive(Clone)]
pub struct S3Connection {
    runtime: Arc<Runtime>,
    operator: Operator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct S3Entry {
    pub path: String,
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
    pub is_file: bool,
    pub is_dir: bool,
    pub mode: String,
}

pub(crate) fn connect(runtime: Arc<Runtime>, options: S3ConnectOptions) -> Result<S3Connection> {
    ensure_non_empty("bucket", &options.bucket, "ptool.s3.connect")?;

    let mut builder = S3::default().bucket(&options.bucket).disable_config_load();

    if let Some(root) = &options.root {
        builder = builder.root(root);
    }
    if let Some(region) = &options.region {
        builder = builder.region(region);
    }
    if let Some(endpoint) = &options.endpoint {
        builder = builder.endpoint(endpoint);
    }
    if let Some(access_key_id) = &options.access_key_id {
        builder = builder.access_key_id(access_key_id);
    }
    if let Some(secret_access_key) = &options.secret_access_key {
        builder = builder.secret_access_key(secret_access_key);
    }
    if let Some(session_token) = &options.session_token {
        builder = builder.session_token(session_token);
    }
    if options.allow_anonymous {
        builder = builder.allow_anonymous();
    }

    let operator = Operator::new(builder)
        .map_err(|err| opendal_error("ptool.s3.connect", "create S3 operator", err))?
        .finish();

    Ok(S3Connection { runtime, operator })
}

impl S3Connection {
    pub fn read(&self, path: &str) -> Result<Vec<u8>> {
        let path = normalize_object_path(path, "ptool.s3.Connection:read", false)?;
        let buffer = self
            .runtime
            .block_on(self.operator.read(&path))
            .map_err(|err| opendal_error("ptool.s3.Connection:read", "read object", err))?;
        Ok(buffer.to_vec())
    }

    pub fn write(&self, path: &str, content: &[u8]) -> Result<()> {
        let path = normalize_object_path(path, "ptool.s3.Connection:write", false)?;
        self.runtime
            .block_on(self.operator.write(&path, content.to_vec()))
            .map_err(|err| opendal_error("ptool.s3.Connection:write", "write object", err))?;
        Ok(())
    }

    pub fn delete(&self, path: &str) -> Result<()> {
        let path = normalize_object_path(path, "ptool.s3.Connection:delete", false)?;
        self.runtime
            .block_on(self.operator.delete(&path))
            .map_err(|err| opendal_error("ptool.s3.Connection:delete", "delete object", err))
    }

    pub fn exists(&self, path: &str) -> Result<bool> {
        let path = normalize_object_path(path, "ptool.s3.Connection:exists", false)?;
        self.runtime
            .block_on(self.operator.exists(&path))
            .map_err(|err| opendal_error("ptool.s3.Connection:exists", "check object", err))
    }

    pub fn list(&self, prefix: &str) -> Result<Vec<S3Entry>> {
        let prefix = normalize_object_path(prefix, "ptool.s3.Connection:list", true)?;
        let entries = self
            .runtime
            .block_on(self.operator.list(&prefix))
            .map_err(|err| opendal_error("ptool.s3.Connection:list", "list objects", err))?;
        entries
            .into_iter()
            .map(|entry| {
                let (path, metadata) = entry.into_parts();
                metadata_to_entry(path, metadata)
            })
            .collect()
    }

    pub fn stat(&self, path: &str) -> Result<S3Entry> {
        let path = normalize_object_path(path, "ptool.s3.Connection:stat", false)?;
        let metadata = self
            .runtime
            .block_on(self.operator.stat(&path))
            .map_err(|err| opendal_error("ptool.s3.Connection:stat", "stat object", err))?;
        metadata_to_entry(path, metadata)
    }
}

impl S3ConnectOptions {
    pub fn with_env_fallback(
        mut self,
        env_get: impl Fn(&str) -> Result<Option<String>>,
    ) -> Result<Self> {
        self.region = fallback_option(self.region, &env_get, &["AWS_REGION"])?;
        self.endpoint = fallback_option(
            self.endpoint,
            &env_get,
            &["AWS_ENDPOINT", "AWS_ENDPOINT_URL", "AWS_S3_ENDPOINT"],
        )?;
        self.access_key_id = fallback_option(self.access_key_id, &env_get, &["AWS_ACCESS_KEY_ID"])?;
        self.secret_access_key =
            fallback_option(self.secret_access_key, &env_get, &["AWS_SECRET_ACCESS_KEY"])?;
        self.session_token = fallback_option(self.session_token, &env_get, &["AWS_SESSION_TOKEN"])?;
        Ok(self)
    }
}

fn fallback_option(
    current: Option<String>,
    env_get: &impl Fn(&str) -> Result<Option<String>>,
    keys: &[&str],
) -> Result<Option<String>> {
    if current.is_some() {
        return Ok(current);
    }

    for key in keys {
        let Some(value) = env_get(key)? else {
            continue;
        };
        if !value.is_empty() {
            return Ok(Some(value));
        }
    }

    Ok(None)
}

fn metadata_to_entry(path: String, metadata: Metadata) -> Result<S3Entry> {
    let mode = metadata.mode();
    Ok(S3Entry {
        path,
        size: metadata.content_length(),
        etag: metadata.etag().map(ToOwned::to_owned),
        last_modified: metadata.last_modified().map(|value| value.to_string()),
        content_type: metadata.content_type().map(ToOwned::to_owned),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        mode: entry_mode_name(mode).to_string(),
    })
}

fn entry_mode_name(mode: EntryMode) -> &'static str {
    match mode {
        EntryMode::FILE => "file",
        EntryMode::DIR => "dir",
        EntryMode::Unknown => "unknown",
    }
}

fn normalize_object_path(path: &str, op: &str, allow_empty: bool) -> Result<String> {
    let normalized = path.trim_start_matches('/').to_string();
    if !allow_empty && normalized.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "path must not be empty").with_op(op));
    }
    Ok(normalized)
}

fn ensure_non_empty(field: &str, value: &str, op: &str) -> Result<()> {
    if value.is_empty() {
        return Err(
            Error::new(ErrorKind::EmptyInput, format!("{field} must not be empty")).with_op(op),
        );
    }
    Ok(())
}

fn opendal_error(op: &str, action: &str, err: opendal::Error) -> Error {
    Error::new(ErrorKind::S3, format!("failed to {action}: {err}"))
        .with_op(op)
        .with_detail(err.to_string())
}
