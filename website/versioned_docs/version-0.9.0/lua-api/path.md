# Path API

Lexical path helpers are available under `ptool.path` and `p.path`.

## ptool.path.join

> `v0.1.0` - Introduced.

`ptool.path.join(...segments)` joins multiple path segments and returns the
normalized path.

- `segments` (string, at least one): Path segments.
- Returns: `string`.

Example:

```lua
print(ptool.path.join("tmp", "a", "..", "b")) -- tmp/b
```

## ptool.path.normalize

> `v0.1.0` - Introduced.

`ptool.path.normalize(path)` performs lexical path normalization (processing `.`
and `..`).

- `path` (string, required): The input path.
- Returns: `string`.

Example:

```lua
print(ptool.path.normalize("./a/../b")) -- b
```

## ptool.path.abspath

> `v0.1.0` - Introduced.

`ptool.path.abspath(path[, base])` computes an absolute path.

- `path` (string, required): The input path.
- `base` (string, optional): The base directory. If omitted, the current process
  working directory is used.
- Returns: `string`.
- Accepts only 1 or 2 string arguments.

Example:

```lua
print(ptool.path.abspath("src"))
print(ptool.path.abspath("lib", "/tmp/demo"))
```

## ptool.path.relpath

> `v0.1.0` - Introduced.

`ptool.path.relpath(path[, base])` computes a relative path from `base` to
`path`.

- `path` (string, required): The target path.
- `base` (string, optional): The starting directory. If omitted, the current
  process working directory is used.
- Returns: `string`.
- Accepts only 1 or 2 string arguments.

Example:

```lua
print(ptool.path.relpath("src/main.rs", "/tmp/project"))
```

## ptool.path.isabs

> `v0.1.0` - Introduced.

`ptool.path.isabs(path)` checks whether a path is absolute.

- `path` (string, required): The input path.
- Returns: `boolean`.

Example:

```lua
print(ptool.path.isabs("/tmp")) -- true
```

## ptool.path.dirname

> `v0.1.0` - Introduced.

`ptool.path.dirname(path)` returns the directory-name portion.

- `path` (string, required): The input path.
- Returns: `string`.

Example:

```lua
print(ptool.path.dirname("a/b/c.txt")) -- a/b
```

## ptool.path.basename

> `v0.1.0` - Introduced.

`ptool.path.basename(path)` returns the last path segment (the filename
portion).

- `path` (string, required): The input path.
- Returns: `string`.

Example:

```lua
print(ptool.path.basename("a/b/c.txt")) -- c.txt
```

## ptool.path.extname

> `v0.1.0` - Introduced.

`ptool.path.extname(path)` returns the extension (including `.`). If there is no
extension, it returns an empty string.

- `path` (string, required): The input path.
- Returns: `string`.

Example:

```lua
print(ptool.path.extname("a/b/c.txt")) -- .txt
```

Notes:

- Path handling in `ptool.path` is purely lexical. It does not check whether
  paths exist and does not resolve symlinks.
- None of the interfaces accept empty string arguments. Passing one raises an
  error.
