# Filesystem API

Filesystem helpers are available under `ptool.fs` and `p.fs`.

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` reads a UTF-8 text file and returns a string.

- `path` (string, required): The file path.
- Returns: `string`.

Example:

```lua
local content = ptool.fs.read("README.md")
print(content)
```

## ptool.fs.write

> `v0.1.0` - Introduced.

`ptool.fs.write(path, content)` writes a string to a file, overwriting existing
contents.

- `path` (string, required): The file path.
- `content` (string, required): The content to write.

Example:

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
```

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

## ptool.fs.glob

> `v0.2.0` - Introduced.

`ptool.fs.glob(pattern)` matches filesystem paths using Unix-style glob syntax
and returns a string array of matched paths sorted lexicographically.

- `pattern` (string, required): A glob pattern. Relative patterns are resolved
  from the current `ptool` runtime directory, so they follow `ptool.cd(...)`.
- Returns: `string[]`.
- Hidden files and directories are matched only when the corresponding pattern
  component explicitly starts with `.`.

Example:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
```
