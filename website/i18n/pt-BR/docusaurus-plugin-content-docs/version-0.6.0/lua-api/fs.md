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

`ptool.fs.write(path, content)` writes a Lua string to a file as raw bytes, overwriting existing contents.

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

## ptool.fs.mkdir

> `v0.1.0` - Introduced.

`ptool.fs.mkdir(path)` creates a directory. If parent directories do not exist, they are created recursively.

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

> `Unreleased` - Introduced.

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

> `Unreleased` - Introduced.

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

> `Unreleased` - Introduced.

`ptool.fs.remove(path[, options])` removes a file, symlink, or directory.

- `path` (string, required): The path to remove.
- `options` (table, optional): Remove options. Supported fields:
  - `recursive` (boolean, optional): Whether to remove directories recursively. Defaults to `false`.
  - `missing_ok` (boolean, optional): Whether to ignore missing paths. Defaults to `false`.

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

## ptool.fs.glob

> `v0.2.0` - Introduced. `v0.5.0` - Added the `working_dir` option.

`ptool.fs.glob(pattern[, options])` matches filesystem paths using Unix-style glob syntax and returns a string array of matched paths sorted lexicographically.

- `pattern` (string, required): A glob pattern. Relative patterns are resolved from the current `ptool` runtime directory, so they follow `ptool.cd(...)`.
- `options` (table, optional): Glob options. Supported fields:
  - `working_dir` (string, optional): Override the base directory used to resolve relative patterns. Relative `working_dir` values are resolved from the current `ptool` runtime directory.
- Returns: `string[]`.
- Hidden files and directories are matched only when the corresponding pattern component explicitly starts with `.`.

Example:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
local lua_scripts = ptool.fs.glob("**/*.lua", {
  working_dir = "../scripts",
})
```
