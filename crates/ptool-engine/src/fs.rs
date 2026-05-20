use crate::{Console, Error, ErrorKind, Result};
use glob::MatchOptions;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FsMkdirOptions {
    pub exist_ok: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FsRemoveOptions {
    pub recursive: bool,
    pub missing_ok: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FsOpenOptions {
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub truncate: bool,
    pub create: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FsSeekWhence {
    Set,
    Cur,
    End,
}

#[derive(Debug)]
pub struct FsFileHandle {
    path: String,
    readable: bool,
    writable: bool,
    file: Mutex<Option<fs::File>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FsGlobOptions {
    pub working_dir: Option<String>,
}

impl Default for FsMkdirOptions {
    fn default() -> Self {
        Self { exist_ok: true }
    }
}

impl Default for FsOpenOptions {
    fn default() -> Self {
        Self {
            read: true,
            write: false,
            append: false,
            truncate: false,
            create: false,
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FsCopySourceKind {
    File,
    Directory,
}

pub fn read(path: &str) -> Result<Vec<u8>> {
    ensure_non_empty_path(path)?;
    fs::read(path).map_err(|err| io_error(err).with_op("ptool.fs.read").with_path(path))
}

pub fn write(path: &str, content: &[u8]) -> Result<()> {
    ensure_non_empty_path(path)?;
    fs::write(path, content).map_err(|err| io_error(err).with_op("ptool.fs.write").with_path(path))
}

pub fn append(path: &str, content: &[u8]) -> Result<()> {
    ensure_non_empty_path(path)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| io_error(err).with_op("ptool.fs.append").with_path(path))?;
    file.write_all(content)
        .map_err(|err| io_error(err).with_op("ptool.fs.append").with_path(path))
}

pub fn open(path: &str, options: FsOpenOptions) -> Result<FsFileHandle> {
    ensure_non_empty_path(path)?;

    let file = fs::OpenOptions::new()
        .read(options.read)
        .write(options.write)
        .append(options.append)
        .truncate(options.truncate)
        .create(options.create)
        .open(path)
        .map_err(|err| io_error(err).with_op("ptool.fs.open").with_path(path))?;

    Ok(FsFileHandle {
        path: path.to_string(),
        readable: options.read,
        writable: options.write || options.append,
        file: Mutex::new(Some(file)),
    })
}

pub fn mkdir(path: &str, options: FsMkdirOptions) -> Result<()> {
    ensure_non_empty_path(path)?;

    if !options.exist_ok && Path::new(path).is_dir() {
        return Err(
            Error::new(ErrorKind::AlreadyExists, format!("`{path}` already exists"))
                .with_op("ptool.fs.mkdir")
                .with_path(path),
        );
    }

    fs::create_dir_all(path).map_err(|err| io_error(err).with_op("ptool.fs.mkdir").with_path(path))
}

pub fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn is_file(path: &str) -> bool {
    Path::new(path).is_file()
}

pub fn is_dir(path: &str) -> bool {
    Path::new(path).is_dir()
}

pub fn remove(path: &str, options: FsRemoveOptions) -> Result<()> {
    ensure_non_empty_path(path)?;

    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound && options.missing_ok => {
            return Ok(());
        }
        Err(err) => {
            return Err(io_error(err).with_op("ptool.fs.remove").with_path(path));
        }
    };

    let file_type = metadata.file_type();
    let result = if file_type.is_symlink() || file_type.is_file() {
        fs::remove_file(path)
    } else if file_type.is_dir() {
        if options.recursive {
            fs::remove_dir_all(path)
        } else {
            fs::remove_dir(path)
        }
    } else {
        fs::remove_file(path)
    };

    result.map_err(|err| io_error(err).with_op("ptool.fs.remove").with_path(path))
}

pub fn glob(pattern: &str, current_dir: &Path, options: FsGlobOptions) -> Result<Vec<String>> {
    ensure_non_empty_path(pattern)?;

    let base_dir = resolve_glob_base_dir(current_dir, options.working_dir.as_deref());
    let original_pattern = pattern;
    let pattern = resolve_glob_pattern(pattern, &base_dir);
    let entries = glob::glob_with(
        &pattern,
        MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        },
    )
    .map_err(|err| {
        Error::new(ErrorKind::InvalidGlob, err.to_string())
            .with_op("ptool.fs.glob")
            .with_input(original_pattern)
    })?;

    let mut matches = Vec::new();
    for entry in entries {
        let path = entry.map_err(|err| {
            Error::new(ErrorKind::Io, err.to_string())
                .with_op("ptool.fs.glob")
                .with_input(original_pattern)
        })?;
        if allows_hidden_match(original_pattern, &base_dir, &path)? {
            matches.push(path_to_string(path));
        }
    }

    matches.sort();
    Ok(matches)
}

pub fn copy_local(
    src: &str,
    dst: &str,
    options: FsCopyOptions,
    console: &Console,
) -> Result<FsCopyResult> {
    ensure_non_empty_named_path(src, "src")?;
    ensure_non_empty_named_path(dst, "dst")?;

    let src_path = Path::new(src);
    let bytes = match classify_copy_source(src_path, src)? {
        FsCopySourceKind::File => {
            let dst_path = resolve_file_copy_destination(src_path, dst)?;
            prepare_file_copy_destination(&dst_path, options)?;
            let resolved_dst = dst_path.display().to_string();

            if options.echo {
                console
                    .write_stdout_line(&format!("[copy] {src} -> {dst}"))
                    .map_err(|err| io_error(err).with_op("ptool.fs.copy"))?;
            }

            copy_file(src_path, &dst_path, src, &resolved_dst)?
        }
        FsCopySourceKind::Directory => {
            let destination_root = resolve_directory_copy_destination(src_path, dst)?;
            ensure_copy_destination_is_outside_source(src_path, &destination_root)?;
            prepare_directory_copy_destination(&destination_root, options)?;

            if options.echo {
                console
                    .write_stdout_line(&format!("[copy] {src} -> {dst}"))
                    .map_err(|err| io_error(err).with_op("ptool.fs.copy"))?;
            }

            copy_directory(src_path, &destination_root, options)?
        }
    };

    Ok(FsCopyResult {
        bytes,
        from: src.to_string(),
        to: dst.to_string(),
    })
}

impl FsOpenOptions {
    pub fn parse(mode: &str) -> Result<Self> {
        let normalized: String = mode.chars().filter(|ch| *ch != 'b').collect();
        match normalized.as_str() {
            "r" => Ok(Self::default()),
            "w" => Ok(Self {
                read: false,
                write: true,
                append: false,
                truncate: true,
                create: true,
            }),
            "a" => Ok(Self {
                read: false,
                write: false,
                append: true,
                truncate: false,
                create: true,
            }),
            "r+" => Ok(Self {
                read: true,
                write: true,
                append: false,
                truncate: false,
                create: false,
            }),
            "w+" => Ok(Self {
                read: true,
                write: true,
                append: false,
                truncate: true,
                create: true,
            }),
            "a+" => Ok(Self {
                read: true,
                write: false,
                append: true,
                truncate: false,
                create: true,
            }),
            _ => Err(Error::new(
                ErrorKind::InvalidFsOption,
                format!("invalid file open mode `{mode}`"),
            )
            .with_op("ptool.fs.open")),
        }
    }
}

impl FsFileHandle {
    pub fn read(&self, len: Option<usize>) -> Result<Vec<u8>> {
        if !self.readable {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "file handle was not opened for reading",
            )
            .with_op("ptool.fs.File:read()")
            .with_path(&self.path));
        }

        self.with_file("ptool.fs.File:read()", |file| match len {
            Some(len) => {
                let mut buffer = vec![0; len];
                let read = file.read(&mut buffer)?;
                buffer.truncate(read);
                Ok(buffer)
            }
            None => {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                Ok(buffer)
            }
        })
    }

    pub fn write(&self, content: &[u8]) -> Result<()> {
        if !self.writable {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "file handle was not opened for writing",
            )
            .with_op("ptool.fs.File:write()")
            .with_path(&self.path));
        }

        self.with_file("ptool.fs.File:write()", |file| file.write_all(content))
    }

    pub fn flush(&self) -> Result<()> {
        self.with_file("ptool.fs.File:flush()", |file| file.flush())
    }

    pub fn seek(&self, whence: FsSeekWhence, offset: i64) -> Result<u64> {
        let whence = match whence {
            FsSeekWhence::Set => {
                let offset = u64::try_from(offset).map_err(|_| {
                    Error::new(
                        ErrorKind::InvalidArgs,
                        "`offset` must be >= 0 when `whence` is `set`",
                    )
                    .with_op("ptool.fs.File:seek()")
                    .with_path(&self.path)
                })?;
                SeekFrom::Start(offset)
            }
            FsSeekWhence::Cur => SeekFrom::Current(offset),
            FsSeekWhence::End => SeekFrom::End(offset),
        };

        self.with_file("ptool.fs.File:seek()", |file| file.seek(whence))
    }

    pub fn close(&self) -> Result<()> {
        let mut file = self
            .file
            .lock()
            .expect("ptool-engine fs file handle lock poisoned");
        file.take();
        Ok(())
    }

    fn with_file<T>(
        &self,
        op: &str,
        f: impl FnOnce(&mut fs::File) -> std::io::Result<T>,
    ) -> Result<T> {
        let mut file = self
            .file
            .lock()
            .expect("ptool-engine fs file handle lock poisoned");
        let Some(file) = file.as_mut() else {
            return Err(Error::new(ErrorKind::Io, "file handle is closed")
                .with_op(op)
                .with_path(&self.path));
        };

        f(file).map_err(|err| io_error(err).with_op(op).with_path(&self.path))
    }
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

fn classify_copy_source(path: &Path, original: &str) -> Result<FsCopySourceKind> {
    let metadata = fs::metadata(path).map_err(|err| {
        Error::new(
            ErrorKind::Io,
            format!("failed to access `{original}`: {err}"),
        )
        .with_op("ptool.fs.copy")
        .with_path(original)
    })?;
    if metadata.is_file() {
        return Ok(FsCopySourceKind::File);
    }
    if metadata.is_dir() {
        return Ok(FsCopySourceKind::Directory);
    }
    Err(Error::new(
        ErrorKind::Unsupported,
        format!("`src` must be a file or directory: `{original}`"),
    )
    .with_op("ptool.fs.copy")
    .with_path(original))
}

fn resolve_file_copy_destination(src_path: &Path, dst: &str) -> Result<PathBuf> {
    let is_directory_hint = ends_with_path_separator(dst);
    let dst_path = Path::new(dst);

    if dst_path.exists() {
        if dst_path.is_dir() {
            return Ok(dst_path.join(path_basename(src_path, "src")?));
        }
        if is_directory_hint {
            return Err(Error::new(
                ErrorKind::Io,
                format!("`dst` must be a directory for file copy: `{dst}`"),
            )
            .with_op("ptool.fs.copy")
            .with_path(dst));
        }
        return Ok(dst_path.to_path_buf());
    }

    if is_directory_hint {
        return Ok(dst_path.join(path_basename(src_path, "src")?));
    }

    Ok(dst_path.to_path_buf())
}

fn resolve_directory_copy_destination(src_path: &Path, dst: &str) -> Result<PathBuf> {
    let dst_path = Path::new(dst);
    if dst_path.exists() {
        if !dst_path.is_dir() {
            return Err(Error::new(
                ErrorKind::Io,
                format!("`dst` must be a directory for directory copy: `{dst}`"),
            )
            .with_op("ptool.fs.copy")
            .with_path(dst));
        }
        return Ok(dst_path.join(path_basename(src_path, "src")?));
    }
    Ok(dst_path.to_path_buf())
}

fn prepare_file_copy_destination(path: &Path, options: FsCopyOptions) -> Result<()> {
    if options.parents
        && let Some(parent) = path
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
            .with_op("ptool.fs.copy")
            .with_path(parent.display().to_string())
        })?;
    }

    if !options.overwrite && path.exists() {
        return Err(Error::new(
            ErrorKind::AlreadyExists,
            format!("destination already exists: `{}`", path.display()),
        )
        .with_op("ptool.fs.copy")
        .with_path(path.display().to_string()));
    }

    Ok(())
}

fn prepare_directory_copy_destination(path: &Path, options: FsCopyOptions) -> Result<()> {
    if path.exists() {
        if !path.is_dir() {
            return Err(Error::new(
                ErrorKind::Io,
                format!(
                    "directory destination must not be a file: `{}`",
                    path.display()
                ),
            )
            .with_op("ptool.fs.copy")
            .with_path(path.display().to_string()));
        }
        if !options.overwrite {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                format!("directory destination already exists: `{}`", path.display()),
            )
            .with_op("ptool.fs.copy")
            .with_path(path.display().to_string()));
        }
        return Ok(());
    }

    let create_result = if options.parents {
        fs::create_dir_all(path)
    } else {
        fs::create_dir(path)
    };
    create_result.map_err(|err| {
        Error::new(
            ErrorKind::Io,
            format!(
                "failed to create destination directory `{}`: {err}",
                path.display()
            ),
        )
        .with_op("ptool.fs.copy")
        .with_path(path.display().to_string())
    })?;
    Ok(())
}

fn copy_file(src_path: &Path, dst_path: &Path, src: &str, dst: &str) -> Result<u64> {
    fs::copy(src_path, dst_path).map_err(|err| {
        Error::new(
            ErrorKind::Io,
            format!("failed to copy `{src}` to `{dst}`: {err}"),
        )
        .with_op("ptool.fs.copy")
        .with_path(dst)
    })
}

fn copy_directory(src_dir: &Path, dst_dir: &Path, options: FsCopyOptions) -> Result<u64> {
    let mut entries = fs::read_dir(src_dir)
        .map_err(|err| {
            Error::new(
                ErrorKind::Io,
                format!("failed to read directory `{}`: {err}", src_dir.display()),
            )
            .with_op("ptool.fs.copy")
            .with_path(src_dir.display().to_string())
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| {
            Error::new(
                ErrorKind::Io,
                format!("failed to read directory `{}`: {err}", src_dir.display()),
            )
            .with_op("ptool.fs.copy")
            .with_path(src_dir.display().to_string())
        })?;
    entries.sort_by_key(|entry| entry.file_name());

    let mut total_bytes = 0u64;
    for entry in entries {
        let entry_path = entry.path();
        let destination_path = dst_dir.join(entry.file_name());
        let source_display = entry_path.display().to_string();
        match classify_copy_source(&entry_path, &source_display)? {
            FsCopySourceKind::File => {
                prepare_file_copy_destination(&destination_path, options)?;
                let destination_display = destination_path.display().to_string();
                total_bytes = total_bytes.saturating_add(copy_file(
                    &entry_path,
                    &destination_path,
                    &source_display,
                    &destination_display,
                )?);
            }
            FsCopySourceKind::Directory => {
                prepare_directory_copy_destination(&destination_path, options)?;
                total_bytes = total_bytes.saturating_add(copy_directory(
                    &entry_path,
                    &destination_path,
                    options,
                )?);
            }
        }
    }

    Ok(total_bytes)
}

fn ensure_copy_destination_is_outside_source(src_dir: &Path, dst_dir: &Path) -> Result<()> {
    let canonical_src = fs::canonicalize(src_dir).map_err(|err| {
        Error::new(
            ErrorKind::Io,
            format!("failed to access `{}`: {err}", src_dir.display()),
        )
        .with_op("ptool.fs.copy")
        .with_path(src_dir.display().to_string())
    })?;
    let canonical_dst = canonicalize_destination_path(dst_dir)?;

    if canonical_dst == canonical_src || canonical_dst.starts_with(&canonical_src) {
        return Err(Error::new(
            ErrorKind::InvalidArgs,
            format!(
                "directory destination must not be inside source: `{}`",
                dst_dir.display()
            ),
        )
        .with_op("ptool.fs.copy")
        .with_path(dst_dir.display().to_string()));
    }

    Ok(())
}

fn canonicalize_destination_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return fs::canonicalize(path).map_err(|err| {
            Error::new(
                ErrorKind::Io,
                format!("failed to access `{}`: {err}", path.display()),
            )
            .with_op("ptool.fs.copy")
            .with_path(path.display().to_string())
        });
    }

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let canonical_parent = canonicalize_destination_path(parent)?;
    let Some(name) = path.file_name() else {
        return Ok(canonical_parent);
    };
    Ok(canonical_parent.join(name))
}

fn path_basename<'a>(path: &'a Path, field: &str) -> Result<&'a std::ffi::OsStr> {
    path.file_name().ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidArgs,
            format!(
                "`{field}` must have a final path component: `{}`",
                path.display()
            ),
        )
        .with_op("ptool.fs.copy")
        .with_path(path.display().to_string())
    })
}

fn ends_with_path_separator(path: &str) -> bool {
    path.ends_with('/') || path.ends_with('\\')
}

fn resolve_glob_pattern(pattern: &str, base_dir: &Path) -> String {
    let pattern_path = Path::new(pattern);
    if pattern_path.is_absolute() {
        pattern.to_string()
    } else {
        path_to_string(base_dir.join(pattern_path))
    }
}

fn resolve_glob_base_dir(current_dir: &Path, working_dir: Option<&str>) -> PathBuf {
    let Some(working_dir) = working_dir else {
        return current_dir.to_path_buf();
    };

    let working_dir = Path::new(working_dir);
    if working_dir.is_absolute() {
        working_dir.to_path_buf()
    } else {
        current_dir.join(working_dir)
    }
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

fn allows_hidden_match(pattern: &str, base_dir: &Path, path: &Path) -> Result<bool> {
    let candidate = if Path::new(pattern).is_absolute() {
        path.to_path_buf()
    } else {
        path.strip_prefix(base_dir)
            .map(Path::to_path_buf)
            .map_err(|err| {
                Error::new(
                    ErrorKind::Io,
                    format!(
                        "failed to resolve glob match `{}` relative to `{}`: {err}",
                        path.display(),
                        base_dir.display()
                    ),
                )
            })?
    };

    let pattern_components = collect_components(Path::new(pattern));
    let path_components = collect_components(&candidate);
    Ok(match_components_allowing_hidden(
        &pattern_components,
        &path_components,
    ))
}

fn collect_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            std::path::Component::CurDir => Some(".".to_string()),
            std::path::Component::ParentDir => Some("..".to_string()),
            std::path::Component::Prefix(_) | std::path::Component::RootDir => None,
        })
        .collect()
}

fn match_components_allowing_hidden(
    pattern_components: &[String],
    path_components: &[String],
) -> bool {
    let mut memo = vec![vec![None; path_components.len() + 1]; pattern_components.len() + 1];
    match_components_allowing_hidden_impl(pattern_components, path_components, 0, 0, &mut memo)
}

fn match_components_allowing_hidden_impl(
    pattern_components: &[String],
    path_components: &[String],
    pattern_index: usize,
    path_index: usize,
    memo: &mut [Vec<Option<bool>>],
) -> bool {
    if let Some(result) = memo[pattern_index][path_index] {
        return result;
    }

    let result = if pattern_index == pattern_components.len() {
        path_index == path_components.len()
    } else if pattern_components[pattern_index] == "**" {
        match_components_allowing_hidden_impl(
            pattern_components,
            path_components,
            pattern_index + 1,
            path_index,
            memo,
        ) || (path_index < path_components.len()
            && !is_hidden_component(&path_components[path_index])
            && match_components_allowing_hidden_impl(
                pattern_components,
                path_components,
                pattern_index,
                path_index + 1,
                memo,
            ))
    } else if path_index == path_components.len() {
        false
    } else {
        component_matches(
            &pattern_components[pattern_index],
            &path_components[path_index],
        ) && (!is_hidden_component(&path_components[path_index])
            || pattern_explicitly_matches_hidden(&pattern_components[pattern_index]))
            && match_components_allowing_hidden_impl(
                pattern_components,
                path_components,
                pattern_index + 1,
                path_index + 1,
                memo,
            )
    };

    memo[pattern_index][path_index] = Some(result);
    result
}

fn component_matches(pattern_component: &str, path_component: &str) -> bool {
    glob::Pattern::new(pattern_component)
        .map(|pattern| {
            pattern.matches_with(
                path_component,
                MatchOptions {
                    case_sensitive: true,
                    require_literal_separator: false,
                    require_literal_leading_dot: false,
                },
            )
        })
        .unwrap_or(false)
}

fn is_hidden_component(component: &str) -> bool {
    component.starts_with('.') && component != "." && component != ".."
}

fn pattern_explicitly_matches_hidden(pattern_component: &str) -> bool {
    let mut chars = pattern_component.chars();
    match chars.next() {
        Some('.') => true,
        Some('\\') => matches!(chars.next(), Some('.')),
        Some('[') => class_explicitly_matches_dot(chars),
        _ => false,
    }
}

fn class_explicitly_matches_dot(mut chars: std::str::Chars<'_>) -> bool {
    let mut negate = false;
    let mut first = true;
    let mut matches_dot = false;

    while let Some(ch) = chars.next() {
        if first {
            first = false;
            if ch == '!' || ch == '^' {
                negate = true;
                continue;
            }
        }

        if ch == ']' {
            break;
        }

        if ch == '\\' {
            if let Some(escaped) = chars.next() {
                matches_dot |= escaped == '.';
            }
            continue;
        }

        matches_dot |= ch == '.';
    }

    if negate { !matches_dot } else { matches_dot }
}

fn io_error(err: std::io::Error) -> Error {
    Error::new(ErrorKind::Io, err.to_string())
}
