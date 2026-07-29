use crate::{Error, ErrorKind, Result};
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::operation::put_bucket_acl::builders::PutBucketAclFluentBuilder;
use aws_sdk_s3::operation::put_object_acl::builders::PutObjectAclFluentBuilder;
use aws_sdk_s3::types::{BucketCannedAcl, ObjectCannedAcl, RequestPayer};
use opendal::{
    EntryMode, Metadata, Operator, layers::HttpClientLayer, raw::HttpClient, services::S3,
};
use std::error::Error as StdError;
use std::future::IntoFuture;
use std::ops::Bound;
use std::sync::Arc;
use tokio::runtime::Runtime;

const PUT_BUCKET_ACL_OP: &str = "ptool.s3.Connection:put_bucket_acl";
const PUT_OBJECT_ACL_OP: &str = "ptool.s3.Connection:put_object_acl";

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
    sdk_client: aws_sdk_s3::Client,
    bucket: String,
    root: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum S3BucketCannedAcl {
    AuthenticatedRead,
    Private,
    PublicRead,
    PublicReadWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum S3ObjectCannedAcl {
    AuthenticatedRead,
    AwsExecRead,
    BucketOwnerFullControl,
    BucketOwnerRead,
    Private,
    PublicRead,
    PublicReadWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum S3RequestPayer {
    Requester,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3AclGrants {
    pub full_control: Option<String>,
    pub read: Option<String>,
    pub read_acp: Option<String>,
    pub write: Option<String>,
    pub write_acp: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3PutBucketAclOptions {
    pub acl: Option<S3BucketCannedAcl>,
    pub grants: S3AclGrants,
    pub expected_bucket_owner: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3PutObjectAclOptions {
    pub acl: Option<S3ObjectCannedAcl>,
    pub grants: S3AclGrants,
    pub expected_bucket_owner: Option<String>,
    pub version_id: Option<String>,
    pub request_payer: Option<S3RequestPayer>,
}

impl TryFrom<&str> for S3BucketCannedAcl {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "authenticated-read" => Ok(Self::AuthenticatedRead),
            "private" => Ok(Self::Private),
            "public-read" => Ok(Self::PublicRead),
            "public-read-write" => Ok(Self::PublicReadWrite),
            _ => Err(Error::new(
                ErrorKind::InvalidArgs,
                "`acl` must be one of `authenticated-read`, `private`, `public-read`, or `public-read-write`",
            )
            .with_op(PUT_BUCKET_ACL_OP)),
        }
    }
}

impl TryFrom<&str> for S3ObjectCannedAcl {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "authenticated-read" => Ok(Self::AuthenticatedRead),
            "aws-exec-read" => Ok(Self::AwsExecRead),
            "bucket-owner-full-control" => Ok(Self::BucketOwnerFullControl),
            "bucket-owner-read" => Ok(Self::BucketOwnerRead),
            "private" => Ok(Self::Private),
            "public-read" => Ok(Self::PublicRead),
            "public-read-write" => Ok(Self::PublicReadWrite),
            _ => Err(Error::new(
                ErrorKind::InvalidArgs,
                "`acl` must be one of `authenticated-read`, `aws-exec-read`, `bucket-owner-full-control`, `bucket-owner-read`, `private`, `public-read`, or `public-read-write`",
            )
            .with_op(PUT_OBJECT_ACL_OP)),
        }
    }
}

impl TryFrom<&str> for S3RequestPayer {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "requester" => Ok(Self::Requester),
            _ => Err(Error::new(
                ErrorKind::InvalidArgs,
                "`request_payer` must be `requester`",
            )
            .with_op(PUT_OBJECT_ACL_OP)),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3WriteOptions {
    pub content_type: Option<String>,
    pub cache_control: Option<String>,
    pub content_disposition: Option<String>,
    pub content_encoding: Option<String>,
    pub metadata: Option<Vec<(String, String)>>,
    pub if_not_exists: bool,
    pub if_match: Option<String>,
    pub if_none_match: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3ReadOptions {
    pub range: Option<S3Range>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct S3Range {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct S3Entry {
    pub path: String,
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
    pub version: Option<String>,
    pub metadata: Option<Vec<(String, String)>>,
    pub is_file: bool,
    pub is_dir: bool,
    pub mode: String,
}

pub(crate) fn connect(runtime: Arc<Runtime>, options: S3ConnectOptions) -> Result<S3Connection> {
    ensure_non_empty("bucket", &options.bucket, "ptool.s3.connect")?;

    let sdk_client = build_aws_sdk_client(&options);
    let root = normalize_root_prefix(options.root.as_deref());
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

    // ptool ships as a static musl binary, whose getaddrinfo cannot resolve some
    // internal names (e.g. es1.ft) that glibc/macOS resolve fine. Use a
    // hickory-dns (pure-Rust) HTTP client so S3 resolves /etc/resolv.conf and
    // /etc/hosts itself instead of going through the system getaddrinfo. See
    // OpenDAL's HTTP optimization guide.
    let http_client = reqwest::Client::builder()
        .hickory_dns(true)
        .build()
        .map_err(|err| {
            Error::new(ErrorKind::S3, format!("failed to build HTTP client: {err}"))
                .with_op("ptool.s3.connect")
                .with_detail(err.to_string())
        })?;

    let operator = Operator::new(builder)
        .map_err(|err| opendal_error("ptool.s3.connect", "create S3 operator", err))?
        .layer(HttpClientLayer::new(HttpClient::with(http_client)))
        .finish();

    Ok(S3Connection {
        runtime,
        operator,
        sdk_client,
        bucket: options.bucket,
        root,
    })
}

impl S3Connection {
    pub fn read(&self, path: &str, options: &S3ReadOptions) -> Result<Vec<u8>> {
        let path = normalize_object_path(path, "ptool.s3.Connection:read", false)?;
        let mut reader = self.operator.read_with(&path);
        if let Some(range) = &options.range {
            reader = reader.range(range.to_bounds());
        }
        let buffer = self
            .runtime
            .block_on(reader.into_future())
            .map_err(|err| opendal_error("ptool.s3.Connection:read", "read object", err))?;
        Ok(buffer.to_vec())
    }

    pub fn write(&self, path: &str, content: &[u8], options: &S3WriteOptions) -> Result<S3Entry> {
        let path = normalize_object_path(path, "ptool.s3.Connection:write", false)?;
        let mut writer = self.operator.write_with(&path, content.to_vec());

        if let Some(content_type) = &options.content_type {
            writer = writer.content_type(content_type);
        }
        if let Some(cache_control) = &options.cache_control {
            writer = writer.cache_control(cache_control);
        }
        if let Some(content_disposition) = &options.content_disposition {
            writer = writer.content_disposition(content_disposition);
        }
        if let Some(content_encoding) = &options.content_encoding {
            writer = writer.content_encoding(content_encoding);
        }
        if let Some(metadata) = &options.metadata {
            writer = writer.user_metadata(metadata.clone());
        }
        if options.if_not_exists {
            writer = writer.if_not_exists(true);
        }
        if let Some(if_match) = &options.if_match {
            writer = writer.if_match(if_match);
        }
        if let Some(if_none_match) = &options.if_none_match {
            writer = writer.if_none_match(if_none_match);
        }

        let metadata = self
            .runtime
            .block_on(writer.into_future())
            .map_err(|err| opendal_error("ptool.s3.Connection:write", "write object", err))?;
        Ok(write_metadata_to_entry(
            path,
            content.len() as u64,
            metadata,
        ))
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

    pub fn put_bucket_acl(&self, options: &S3PutBucketAclOptions) -> Result<()> {
        validate_bucket_acl_options(options)?;

        let mut request = self.sdk_client.put_bucket_acl().bucket(self.bucket.clone());
        if let Some(acl) = options.acl {
            request = request.acl(bucket_canned_acl(acl));
        }
        if let Some(owner) = &options.expected_bucket_owner {
            request = request.expected_bucket_owner(owner.clone());
        }
        request = apply_bucket_grants(request, &options.grants);

        self.runtime
            .block_on(request.send())
            .map_err(|err| aws_sdk_error(PUT_BUCKET_ACL_OP, "put bucket ACL", err))?;
        Ok(())
    }

    pub fn put_object_acl(&self, path: &str, options: &S3PutObjectAclOptions) -> Result<()> {
        validate_object_acl_options(options)?;
        let key = object_key(&self.root, path, PUT_OBJECT_ACL_OP)?;

        let mut request = self
            .sdk_client
            .put_object_acl()
            .bucket(self.bucket.clone())
            .key(key);
        if let Some(acl) = options.acl {
            request = request.acl(object_canned_acl(acl));
        }
        if let Some(owner) = &options.expected_bucket_owner {
            request = request.expected_bucket_owner(owner.clone());
        }
        if let Some(version_id) = &options.version_id {
            request = request.version_id(version_id.clone());
        }
        if let Some(request_payer) = options.request_payer {
            request = request.request_payer(request_payer_value(request_payer));
        }
        request = apply_object_grants(request, &options.grants);

        self.runtime
            .block_on(request.send())
            .map_err(|err| aws_sdk_error(PUT_OBJECT_ACL_OP, "put object ACL", err))?;
        Ok(())
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

fn build_aws_sdk_client(options: &S3ConnectOptions) -> aws_sdk_s3::Client {
    let region = options
        .region
        .clone()
        .unwrap_or_else(|| "us-east-1".to_string());
    let mut builder = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(region));

    if let Some(endpoint) = &options.endpoint {
        builder = builder.endpoint_url(endpoint).force_path_style(true);
    }

    if let (Some(access_key_id), Some(secret_access_key)) =
        (&options.access_key_id, &options.secret_access_key)
    {
        builder = builder.credentials_provider(Credentials::new(
            access_key_id.clone(),
            secret_access_key.clone(),
            options.session_token.clone(),
            None,
            "ptool.s3.connect",
        ));
    }

    aws_sdk_s3::Client::from_conf(builder.build())
}

impl S3AclGrants {
    fn is_empty(&self) -> bool {
        self.full_control.is_none()
            && self.read.is_none()
            && self.read_acp.is_none()
            && self.write.is_none()
            && self.write_acp.is_none()
    }
}

fn validate_acl_selection(has_acl: bool, grants: &S3AclGrants, op: &str) -> Result<()> {
    if !has_acl && grants.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "requires `acl` or at least one `grant_*` option",
        )
        .with_op(op));
    }
    if has_acl && !grants.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            "`acl` cannot be combined with `grant_*` options",
        )
        .with_op(op));
    }
    Ok(())
}

fn validate_optional_non_empty(value: Option<&str>, field: &str, op: &str) -> Result<()> {
    if matches!(value, Some("")) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!("`{field}` must not be empty"),
        )
        .with_op(op));
    }
    Ok(())
}

fn validate_grants(grants: &S3AclGrants, op: &str) -> Result<()> {
    validate_optional_non_empty(grants.full_control.as_deref(), "grant_full_control", op)?;
    validate_optional_non_empty(grants.read.as_deref(), "grant_read", op)?;
    validate_optional_non_empty(grants.read_acp.as_deref(), "grant_read_acp", op)?;
    validate_optional_non_empty(grants.write.as_deref(), "grant_write", op)?;
    validate_optional_non_empty(grants.write_acp.as_deref(), "grant_write_acp", op)
}

fn validate_bucket_acl_options(options: &S3PutBucketAclOptions) -> Result<()> {
    validate_acl_selection(options.acl.is_some(), &options.grants, PUT_BUCKET_ACL_OP)?;
    validate_grants(&options.grants, PUT_BUCKET_ACL_OP)?;
    validate_optional_non_empty(
        options.expected_bucket_owner.as_deref(),
        "expected_bucket_owner",
        PUT_BUCKET_ACL_OP,
    )
}

fn validate_object_acl_options(options: &S3PutObjectAclOptions) -> Result<()> {
    validate_acl_selection(options.acl.is_some(), &options.grants, PUT_OBJECT_ACL_OP)?;
    validate_grants(&options.grants, PUT_OBJECT_ACL_OP)?;
    validate_optional_non_empty(
        options.expected_bucket_owner.as_deref(),
        "expected_bucket_owner",
        PUT_OBJECT_ACL_OP,
    )?;
    validate_optional_non_empty(
        options.version_id.as_deref(),
        "version_id",
        PUT_OBJECT_ACL_OP,
    )
}

fn bucket_canned_acl(value: S3BucketCannedAcl) -> BucketCannedAcl {
    match value {
        S3BucketCannedAcl::AuthenticatedRead => BucketCannedAcl::AuthenticatedRead,
        S3BucketCannedAcl::Private => BucketCannedAcl::Private,
        S3BucketCannedAcl::PublicRead => BucketCannedAcl::PublicRead,
        S3BucketCannedAcl::PublicReadWrite => BucketCannedAcl::PublicReadWrite,
    }
}

fn object_canned_acl(value: S3ObjectCannedAcl) -> ObjectCannedAcl {
    match value {
        S3ObjectCannedAcl::AuthenticatedRead => ObjectCannedAcl::AuthenticatedRead,
        S3ObjectCannedAcl::AwsExecRead => ObjectCannedAcl::AwsExecRead,
        S3ObjectCannedAcl::BucketOwnerFullControl => ObjectCannedAcl::BucketOwnerFullControl,
        S3ObjectCannedAcl::BucketOwnerRead => ObjectCannedAcl::BucketOwnerRead,
        S3ObjectCannedAcl::Private => ObjectCannedAcl::Private,
        S3ObjectCannedAcl::PublicRead => ObjectCannedAcl::PublicRead,
        S3ObjectCannedAcl::PublicReadWrite => ObjectCannedAcl::PublicReadWrite,
    }
}

fn request_payer_value(value: S3RequestPayer) -> RequestPayer {
    match value {
        S3RequestPayer::Requester => RequestPayer::Requester,
    }
}

fn apply_bucket_grants(
    mut request: PutBucketAclFluentBuilder,
    grants: &S3AclGrants,
) -> PutBucketAclFluentBuilder {
    if let Some(value) = &grants.full_control {
        request = request.grant_full_control(value.clone());
    }
    if let Some(value) = &grants.read {
        request = request.grant_read(value.clone());
    }
    if let Some(value) = &grants.read_acp {
        request = request.grant_read_acp(value.clone());
    }
    if let Some(value) = &grants.write {
        request = request.grant_write(value.clone());
    }
    if let Some(value) = &grants.write_acp {
        request = request.grant_write_acp(value.clone());
    }
    request
}

fn apply_object_grants(
    mut request: PutObjectAclFluentBuilder,
    grants: &S3AclGrants,
) -> PutObjectAclFluentBuilder {
    if let Some(value) = &grants.full_control {
        request = request.grant_full_control(value.clone());
    }
    if let Some(value) = &grants.read {
        request = request.grant_read(value.clone());
    }
    if let Some(value) = &grants.read_acp {
        request = request.grant_read_acp(value.clone());
    }
    if let Some(value) = &grants.write {
        request = request.grant_write(value.clone());
    }
    if let Some(value) = &grants.write_acp {
        request = request.grant_write_acp(value.clone());
    }
    request
}

fn normalize_root_prefix(root: Option<&str>) -> String {
    let root = root
        .unwrap_or_default()
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    if root.is_empty() {
        String::new()
    } else {
        format!("{root}/")
    }
}

fn object_key(root: &str, path: &str, op: &str) -> Result<String> {
    let path = path.trim().trim_start_matches('/');
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "path must not be empty").with_op(op));
    }

    let has_trailing_slash = path.ends_with('/');
    let mut path = path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    if has_trailing_slash {
        path.push('/');
    }
    Ok(format!("{root}{path}"))
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
        version: metadata.version().map(ToOwned::to_owned),
        metadata: metadata.user_metadata().map(|metadata| {
            metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        mode: entry_mode_name(mode).to_string(),
    })
}

fn write_metadata_to_entry(path: String, size: u64, metadata: Metadata) -> S3Entry {
    S3Entry {
        path,
        size,
        etag: metadata.etag().map(ToOwned::to_owned),
        last_modified: metadata.last_modified().map(|value| value.to_string()),
        content_type: metadata.content_type().map(ToOwned::to_owned),
        version: metadata.version().map(ToOwned::to_owned),
        metadata: metadata.user_metadata().map(|metadata| {
            metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        }),
        is_file: true,
        is_dir: false,
        mode: "file".to_string(),
    }
}

impl S3Range {
    fn to_bounds(&self) -> (Bound<u64>, Bound<u64>) {
        let start = self.start.map(Bound::Included).unwrap_or(Bound::Unbounded);
        let end = self.end.map(Bound::Excluded).unwrap_or(Bound::Unbounded);
        (start, end)
    }
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

fn aws_sdk_error(op: &str, action: &str, err: impl StdError + 'static) -> Error {
    let mut detail = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        let message = cause.to_string();
        if !message.is_empty() {
            detail.push_str(": ");
            detail.push_str(&message);
        }
        source = cause.source();
    }

    Error::new(ErrorKind::S3, format!("failed to {action}: {detail}"))
        .with_op(op)
        .with_detail(detail)
}
