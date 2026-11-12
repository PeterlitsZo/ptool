# Filesystem API

Filesystem helpers are available under `ptool.fs` and `p.fs`.

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` reads a file as raw bytes and returns a Lua string.

- `path` (string, required): The file path.
- Returns: `string`.

Notes:

- The returned Lua string contains the file bytes exactly as stored on disk.
- Text files continue to work as before, but binary files are also supported.

Example:

```lua
local content = ptool.fs.read("README.md")
print(content)

local png = ptool.fs.read("logo.png")
print(#png)
```

## ptool.fs.write

> `v0.1.0` - Introduced.

`ptool.fs.write(path, content)` writes a Lua string to a file as raw bytes,
overwriting existing contents.

- `path` (string, required): The file path.
- `content` (string, required): The content to write.

Notes:

- `content` is written byte-for-byte.
- Embedded NUL bytes and non-UTF-8 bytes are preserved.

Example:

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
ptool.fs.write("tmp/blob.bin", "\x00\xffABC")
```

## ptool.fs.append

> `v0.8.0` - Introduced.

`ptool.fs.append(path, content)` appends a Lua string to a file as raw bytes.
If the file does not exist, it is created.

- `path` (string, required): The file path.
- `content` (string, required): The content to append.

Notes:

- `content` is written byte-for-byte at the end of the file.
- Embedded NUL bytes and non-UTF-8 bytes are preserved.

Example:

```lua
ptool.fs.append("tmp/log.txt", "first line\n")
ptool.fs.append("tmp/log.txt", "second line\n")
```

## ptool.fs.open

> `v0.8.0` - Introduced.

`ptool.fs.open(path[, mode])` opens a local file and returns a `File` object.

Arguments:

- `path` (string, required): The file path.
- `mode` (string, optional): The file mode. Defaults to `"r"`.

Supported modes:

- `"r"`: Open for reading.
- `"w"`: Open for writing, truncating existing contents and creating the file
  when needed.
- `"a"`: Open for appending, creating the file when needed.
- `"r+"`: Open for reading and writing without truncation.
- `"w+"`: Open for reading and writing, truncating existing contents and
  creating the file when needed.
- `"a+"`: Open for reading and appending, creating the file when needed.

Notes:

- Modes may include `b`, such as `"rb"` or `"w+b"`.
- `a` and `a+` writes always go to the end of the file.

Example:

```lua
local file = ptool.fs.open("tmp/log.txt", "a+")
file:write("hello\n")
file:flush()
file:close()
```

## File

> `v0.8.0` - Introduced.

`File` represents an open local file handle returned by `ptool.fs.open()`.

It is implemented as a Lua userdata.

Methods:

- `file:read([n])` -> `string`
- `file:write(content)` -> `nil`
- `file:flush()` -> `nil`
- `file:seek([whence[, offset]])` -> `integer`
- `file:close()` -> `nil`

### read

> `v0.8.0` - Introduced.

Canonical API name: `ptool.fs.File:read`.

`file:read([n])` reads bytes from the current file position and returns them as
a Lua string.

- `n` (integer, optional): The maximum number of bytes to read. If omitted,
  reads from the current file position to EOF.
- Returns: `string`.

Behavior:

- Returns an empty string at EOF.
- Reads raw bytes, so binary data is preserved exactly.

Example:

```lua
local file = ptool.fs.open("README.md")
local prefix = file:read(16)
local rest = file:read()
file:close()
```

### write

> `v0.8.0` - Introduced.

Canonical API name: `ptool.fs.File:write`.

`file:write(content)` writes a Lua string at the current file position.

- `content` (string, required): The bytes to write.

Behavior:

- Writes raw bytes exactly as provided.
- On append-mode handles, writes are appended to the end of the file.

### flush

> `v0.8.0` - Introduced.

Canonical API name: `ptool.fs.File:flush`.

`file:flush()` flushes buffered file writes to the OS.

### seek

> `v0.8.0` - Introduced.

Canonical API name: `ptool.fs.File:seek`.

`file:seek([whence[, offset]])` moves the current file position.

- `whence` (string, optional): One of `"set"`, `"cur"`, or `"end"`. Defaults
  to `"cur"`.
- `offset` (integer, optional): The byte offset relative to `whence`. Defaults
  to `0`.
- Returns: `integer`.

Behavior:

- Returns the new absolute file position.
- `"set"` requires a non-negative `offset`.

Example:

```lua
local file = ptool.fs.open("tmp/data.bin", "w+")
file:write("abcdef")
file:seek("set", 2)
print(file:read(2)) -- cd
file:close()
```

### close

> `v0.8.0` - Introduced.

Canonical API name: `ptool.fs.File:close`.

`file:close()` closes the file handle.

Behavior:

- After closing, the file handle can no longer be used.

## ptool.fs.mkdir

> `v0.1.0` - Introduced.

`ptool.fs.mkdir(path)` creates a directory. If parent directories do not exist,
they are created recursively.

- `path` (string, required): The directory path.

Example:

```lua
ptool.fs.mkdir("tmp/a/b")
```

## ptool.fs.exists

> `v0.1.0` - Introduced.

`ptool.fs.exists(path)` checks whether a path exists.

- `path` (string, required): A file or directory path.
- Returns: `boolean`.

Example:

```lua
if ptool.fs.exists("tmp/hello.txt") then
  print("exists")
end
```

## ptool.fs.is_file

> `v0.6.0` - Introduced.

`ptool.fs.is_file(path)` checks whether a path exists and is a regular file.

- `path` (string, required): The path to check.
- Returns: `boolean`.

Example:

```lua
if ptool.fs.is_file("tmp/hello.txt") then
  print("file")
end
```

## ptool.fs.is_dir

> `v0.6.0` - Introduced.

`ptool.fs.is_dir(path)` checks whether a path exists and is a directory.

- `path` (string, required): The path to check.
- Returns: `boolean`.

Example:

```lua
if ptool.fs.is_dir("tmp") then
  print("dir")
end
```

## ptool.fs.remove

> `v0.6.0` - Introduced.

`ptool.fs.remove(path[, options])` removes a file, symlink, or directory.

- `path` (string, required): The path to remove.
- `options` (table, optional): Remove options. Supported fields:
  - `recursive` (boolean, optional): Whether to remove directories
    recursively. Defaults to `false`.
  - `missing_ok` (boolean, optional): Whether to ignore missing paths.
    Defaults to `false`.

Behavior:

- Files and symlinks can be removed without `recursive`.
- Directories require `recursive = true` when they are not empty.
- Unknown option names or invalid option value types raise an error.

Example:

```lua
ptool.fs.remove("tmp/hello.txt")
ptool.fs.remove("tmp/cache", { recursive = true })
ptool.fs.remove("tmp/missing.txt", { missing_ok = true })
```

## ptool.fs.copy

> `v0.1.0-alpha.4` - Introduced.
> `v0.9.0` - Local-to-local copies now support directories and
> destination-directory behavior for files.

`ptool.fs.copy(src, dst[, options])` copies files or directories between local
paths, or between a local path and an SSH remote path.

- `src` (string|remote path, required): The source path. Local paths use
  strings. Remote paths use values created by `conn:path(...)`.
- `dst` (string|remote path, required): The destination path. Local paths use
  strings. Remote paths use values created by `conn:path(...)`.
- `options` (table, optional): Transfer options.
- Returns: A table with the following fields:
  - `bytes` (integer): The number of regular-file bytes copied. When a
    directory is copied, this is the sum of the copied file sizes.
  - `from` (string): The source path.
  - `to` (string): The destination path.

Supported transfer options:

- `parents` (boolean, optional): Create parent directories for the final local
  or remote destination path when needed. Defaults to `false`.
- `overwrite` (boolean, optional): Whether an existing destination file or
  final destination directory may be replaced or reused. Defaults to `true`.
- `echo` (boolean, optional): Whether to print the transfer before executing
  it. Defaults to `false`.

Behavior:

- Local-to-local copies support both files and directories.
- When `src` is a file and `dst` is a file path, the file is copied to that
  exact path.
- When `src` is a file and `dst` already exists as a directory, the file is
  copied under that directory using the source file basename.
- When `src` is a file and `dst` ends with `/` or `\\`, `dst` is treated as a
  destination directory path and the copied file keeps the source file
  basename. If that directory does not exist yet, `parents = true` can create
  it.
- When `src` is a directory and `dst` does not exist, `dst` becomes the
  destination directory root.
- When `src` is a directory and `dst` already exists as a directory, the
  source directory is created under it using the source directory basename.
- `overwrite = false` rejects an already-existing destination file or final
  destination directory.
- Local directory copies reject destinations inside the source directory.
- Local-to-remote copies follow the same destination rules as
  `conn:upload(...)`.
- Remote-to-local copies follow the same destination rules as
  `conn:download(...)`.
- Remote-to-remote copies are not supported.

Example:

```lua
local res = ptool.fs.copy("./dist/app.tar.gz", "./tmp/releases/", {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.to)
```

Directory example:

```lua
local res = ptool.fs.copy("./dist/assets", "./tmp/releases", {
  parents = true,
  overwrite = true,
})

print(res.bytes)
print(res.to)
```

Remote example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ptool.fs.copy("./dist/assets", ssh:path("/srv/app/releases"), {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.to)
```

## ptool.fs.glob

> `v0.2.0` - Introduced.
> `v0.5.0` - Added the `working_dir` option.

`ptool.fs.glob(pattern[, options])` matches filesystem paths using Unix-style
glob syntax and returns a string array of matched paths sorted
lexicographically.

- `pattern` (string, required): A glob pattern. Relative patterns are resolved
  from the current `ptool` runtime directory, so they follow `ptool.cd(...)`.
- `options` (table, optional): Glob options. Supported fields:
  - `working_dir` (string, optional): Override the base directory used to
    resolve relative patterns. Relative `working_dir` values are resolved from
    the current `ptool` runtime directory.
- Returns: `string[]`.
- Hidden files and directories are matched only when the corresponding pattern
  component explicitly starts with `.`.

Example:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
local lua_scripts = ptool.fs.glob("**/*.lua", {
  working_dir = "../scripts",
})
```
