use crate::{Error, ErrorKind, Result};
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FsMkdirOptions {
    pub exist_ok: bool,
}

impl Default for FsMkdirOptions {
    fn default() -> Self {
        Self { exist_ok: true }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FsCopyOptions {
    pub parents: bool,
    pub overwrite: bool,
    pub echo: bool,
}

impl Default for FsCopyOptions {
    fn default() -> Self {
        Self {
            parents: false,
            overwrite: true,
            echo: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsCopyResult {
    pub bytes: u64,
    pub from: String,
    pub to: String,
}

pub fn read(path: &str) -> Result<String> {
    ensure_non_empty_path(path)?;
    fs::read_to_string(path).map_err(io_error)
}

pub fn write(path: &str, content: &str) -> Result<()> {
    ensure_non_empty_path(path)?;
    fs::write(path, content).map_err(io_error)
}

pub fn mkdir(path: &str, options: FsMkdirOptions) -> Result<()> {
    ensure_non_empty_path(path)?;

    if !options.exist_ok && Path::new(path).is_dir() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("`{path}` already exists"),
        ));
    }

    fs::create_dir_all(path).map_err(io_error)
}

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn copy_local(src: &str, dst: &str, options: FsCopyOptions) -> Result<FsCopyResult> {
    ensure_non_empty_named_path(src, "src")?;
    ensure_non_empty_named_path(dst, "dst")?;

    let src_path = Path::new(src);
    let metadata = fs::metadata(src_path)
        .map_err(|err| Error::new(ErrorKind::Io, format!("failed to access `{src}`: {err}")))?;
    if !metadata.is_file() {
        return Err(Error::new(
            ErrorKind::NotAFile,
            format!("`src` must be a file: `{src}`"),
        ));
    }

    let dst_path = Path::new(dst);
    if options.parents
        && let Some(parent) = dst_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|err| {
            Error::new(
                ErrorKind::Io,
                format!(
                    "failed to create parent directory `{}`: {err}",
                    parent.display()
                ),
            )
        })?;
    }

    if !options.overwrite && dst_path.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("`dst` already exists: `{dst}`"),
        ));
    }

    if options.echo {
        println!("[copy] {src} -> {dst}");
    }

    let bytes = fs::copy(src_path, dst_path).map_err(|err| {
        Error::new(
            ErrorKind::Io,
            format!("failed to copy `{src}` to `{dst}`: {err}"),
        )
    })?;

    Ok(FsCopyResult {
        bytes,
        from: src.to_string(),
        to: dst.to_string(),
    })
}

fn ensure_non_empty_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(ErrorKind::EmptyPath, "empty path"));
    }
    Ok(())
}

fn ensure_non_empty_named_path(path: &str, field: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::new(
            ErrorKind::EmptyPath,
            format!("`{field}` must not be empty"),
        ));
    }
    Ok(())
}

fn io_error(err: std::io::Error) -> Error {
    Error::new(ErrorKind::Io, err.to_string())
}
