# Lua Script Documentation

`ptool run <lua_file>` runs a Lua script and injects the global variable `ptool`
(or its alias `p`; for example, `p.run` is equivalent to `ptool.run`).

If you want to pass arguments to a Lua script, you can do it like this:

```sh
ptool run script.lua --name alice -v a.txt b.txt
```

The arguments can then be parsed with `ptool.args.parse(...)`.

Here is an example script:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

Shebang is supported, so you can add this to the top of the file:

```
#!/usr/bin/env ptool run
```

## ptool.use

> `v0.1.0` - Introduced.

`ptool.use` declares the minimum required `ptool` version for a script.

```lua
ptool.use("v0.1.0")
```

- The argument is a semantic version string (SemVer) and supports an optional
  `v` prefix, such as `v0.1.0` or `0.1.0`.
- If the required version is higher than the current `ptool` version, the script
  exits immediately with an error saying that the current `ptool` version is too
  old.

## ptool.semver.parse

> `v0.1.0` - Introduced.

`ptool.semver.parse(version)` parses a version string and returns a `Version`
UserData.

- `version` (string, required): A semantic version string, optionally prefixed
  with `v`.

```lua
local v = ptool.semver.parse("v1.2.3-alpha.1+build.9")
print(v.major, v.minor, v.patch)
print(v.pre, v.build)
print(tostring(v))
```

## ptool.semver.is_valid

> `v0.1.0` - Introduced.

`ptool.semver.is_valid(version)` checks whether a version string is valid.

- `version` (string, required): A semantic version string.
- Returns: `boolean`.

```lua
print(ptool.semver.is_valid("1.2.3")) -- true
print(ptool.semver.is_valid("x.y.z")) -- false
```

## ptool.semver.compare

> `v0.1.0` - Introduced.

`ptool.semver.compare(a, b)` compares two versions.

- `a` / `b` (string|Version, required): A version string or a `Version` object.
- Returns: `-1 | 0 | 1`.

```lua
print(ptool.semver.compare("1.2.3", "1.2.4")) -- -1
```

## ptool.semver.bump

> `v0.1.0` - Introduced.

`ptool.semver.bump(v, op)` returns a new version object after applying the bump.

- `v` (string|Version, required): The original version.
- `op` (string, required): One of `major`, `minor`, `patch`, `alpha`, `beta`, or
  `rc`.
- Returns: `Version`.

```lua
local v = ptool.semver.bump("1.2.3", "alpha")
print(tostring(v)) -- 1.2.4-alpha.1
```

## ptool.semver.Version

> `v0.1.0` - Introduced.

`ptool.semver.parse(...)` and `ptool.semver.bump(...)` return a `Version`
UserData with the following fields and methods:

- Fields:
  - `major` (integer)
  - `minor` (integer)
  - `patch` (integer)
  - `pre` (string|nil)
  - `build` (string|nil)
- Methods:
  - `v:compare(other)` -> `-1|0|1`
  - `v:bump(op)` -> `Version`
  - `v:to_string()` -> `string`
- Metamethods:
  - `tostring(v)` is available.
  - `==`, `<`, and `<=` comparisons are supported.

Prerelease bump rules:

- Bumping a stable version to `alpha`, `beta`, or `rc` first increments the
  patch version, then enters the target channel starting from `.1`.
- Bumping within the same channel increments the sequence number, such as
  `alpha.1 -> alpha.2`.
- Channel promotion is allowed (`alpha -> beta -> rc`), but channel downgrade is
  not (for example, `rc -> beta` raises an error).

## ptool.unindent

> `v0.1.0` - Introduced.

`ptool.unindent` processes multi-line strings by removing the `| ` prefix after
leading indentation on each line and trimming leading and trailing blank lines.

```lua
local str = ptool.unindent([[
  | line 1
  | line 2
]])
```

This is equivalent to:

```lua
local str = [[line 1
line 2]]
```

## ptool.config

> `v0.1.0` - Introduced.

`ptool.config` sets runtime configuration for the script.

Currently supported fields:

- `run` (table, optional): Default configuration for `ptool.run`. Supported
  fields:
  - `echo` (boolean, optional): Default echo switch. Defaults to `true`.
  - `check` (boolean, optional): Whether failures raise an error by default.
    Defaults to `false`.
  - `confirm` (boolean, optional): Whether to require confirmation before
    execution by default. Defaults to `false`.

Example:

```lua
ptool.config({
  run = {
    echo = false,
    check = true,
    confirm = false,
  },
})
```

## ptool.cd

> `v0.1.0` - Introduced.

`ptool.cd(path)` updates `ptool`'s runtime current directory.

- `path` (string, required): Target directory path, absolute or relative.

Behavior:

- Relative paths are resolved from the current `ptool` runtime directory.
- The target must exist and must be a directory.
- This updates `ptool` runtime state and affects APIs that use runtime cwd
  (such as `ptool.run`, `ptool.path.abspath`, and `ptool.path.relpath`).

Example:

```lua
ptool.cd("foobar")
local res = ptool.run({ cmd = "pwd", stdout = "capture" })
print(res.stdout)
```

## ptool.run

> `v0.1.0` - Introduced.

`ptool.run` executes external commands from Rust.

The following call forms are supported:

```lua
ptool.run("echo hello world")
ptool.run("echo", "hello world")
ptool.run("echo", {"hello", "world"})
ptool.run("echo hello world", { echo = true })
ptool.run("echo", {"hello", "world"}, { echo = true })
ptool.run({ cmd = "echo", args = {"hello", "world"} })
ptool.run({ cmd = "echo", args = {"hello"}, stdout = "capture" })
```

Argument rules:

- `ptool.run(cmdline)`: `cmdline` is split using shell-style (`shlex`) rules.
  The first item is treated as the command and the rest as arguments.
- `ptool.run(cmd, argsline)`: `cmd` is used directly as the command, and
  `argsline` is split into an argument list using shell-style (`shlex`) rules.
- `ptool.run(cmd, args)`: `cmd` is a string and `args` is an array of strings.
- `ptool.run(cmdline, options)`: `options` overrides settings for this
  invocation, such as `echo`.
- `ptool.run(cmd, args, options)`: `args` can be either a string or an array of
  strings, and `options` overrides settings for this invocation, such as `echo`.
- `ptool.run(options)`: `options` is a table.
- When the second argument is a table: if it is an array (consecutive integer
  keys `1..n`), it is treated as `args`; otherwise it is treated as `options`.

Return value rules:

- A table is always returned with the following fields:
  - `ok` (boolean): Whether the exit code is `0`.
  - `code` (integer|nil): The process exit code. If the process was terminated
    by a signal, this is `nil`.
  - `stdout` (string, optional): Present when `stdout = "capture"`.
  - `stderr` (string, optional): Present when `stderr = "capture"`.
  - `assert_ok(self)` (function): Raises an error when `ok = false`. The error
    message includes the exit code and, if available, `stderr`.
- The default value of `check` comes from `ptool.config({ run = { check = ... }
  })`. If not configured, it defaults to `false`. When `check = false`, callers
  can inspect `ok` themselves or call `res:assert_ok()`.

Example:

```lua
ptool.config({ run = { echo = false } })

ptool.run("echo from ptool")
ptool.run("echo", "from ptool")
ptool.run("echo", {"from", "ptool"})
ptool.run("echo from ptool", { echo = true })
ptool.run("echo", {"from", "ptool"}, { echo = true })
ptool.run("pwd")

local res = ptool.run({
  cmd = "sh",
  args = {"-c", "echo bad >&2; exit 7"},
  stderr = "capture",
})
print(res.ok, res.code)
res:assert_ok()
```

`ptool.run(options)` is also supported, where `options` is a table with the
following fields:

- `cmd` (string, required): The command name or executable path.
- `args` (string[], optional): The argument list.
- `cwd` (string, optional): The child process working directory.
- `env` (table, optional): Additional environment variables, where keys are
  variable names and values are variable values.
- `echo` (boolean, optional): Whether to echo command information for this
  execution. If omitted, the value from `ptool.config({ run = { echo = ... } })`
  is used; if that is also unset, the default is `true`.
- `check` (boolean, optional): Whether to raise an error immediately when the
  exit code is not `0`. If omitted, the value from `ptool.config({ run = { check
  = ... } })` is used; if that is also unset, the default is `false`.
- `confirm` (boolean, optional): Whether to ask the user for confirmation before
  execution. If omitted, the value from `ptool.config({ run = { confirm = ... }
  })` is used; if that is also unset, the default is `false`.
- `stdout` (string, optional): Stdout handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stdout`.
  - `"null"`: Discard the output.
- `stderr` (string, optional): Stderr handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stderr`.
  - `"null"`: Discard the output.
- When `confirm = true`:
  - If the user refuses the execution, an error is raised immediately.
  - If the current environment is not interactive (no TTY), an error is raised
    immediately.

Example:

```lua
ptool.run({
  cmd = "echo",
  args = {"hello"},
  env = { FOO = "bar" },
})

local res = ptool.run({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2; exit 7"},
  stdout = "capture",
  stderr = "capture",
  check = false,
})
print(res.ok, res.code)
print(res.stdout)
print(res.stderr)
res:assert_ok()
```

## ptool.http.request

> `v0.1.0` - Introduced.

`ptool.http.request(options)` sends an HTTP request and returns a `Response`
object.

`options` fields:

- `url` (string, required): The request URL.
- `method` (string, optional): The HTTP method. Defaults to `"GET"`.
- `headers` (table, optional): Request headers, where both keys and values are
  strings.
- `body` (string, optional): The request body.
- `timeout_ms` (integer, optional): Timeout in milliseconds. Defaults to
  `30000`.

Example:

```lua
local resp = ptool.http.request({
  url = "https://httpbin.org/post",
  method = "POST",
  headers = {
    ["content-type"] = "application/json",
  },
  body = [[{"name":"alice"}]],
})

print(resp.status, resp.ok)
local data = resp:json()
print(data.json.name)
```

`Response` fields:

- `status` (integer): The HTTP status code.
- `ok` (boolean): Whether the status code is in the 2xx range.
- `url` (string): The final URL after redirects.
- `headers` (table): Response headers (`table<string, string>`).

`Response` methods:

- `resp:text()`: Reads and returns the response body as text.
- `resp:json()`: Reads the response body, parses it as JSON, and returns a Lua
  value.
- `resp:bytes()`: Reads and returns the raw bytes (as a Lua string).

Notes:

- Non-2xx HTTP statuses do not raise errors. Callers should check `resp.ok`
  themselves.
- The body can only be consumed once. Calling any of `text`, `json`, or `bytes`
  more than once raises an error.

## ptool.path.join

> `v0.1.0` - Introduced.

`ptool.path.join(...segments)` joins multiple path segments and returns the
normalized path.

- `segments` (string, at least one): Path segments.

```lua
print(ptool.path.join("tmp", "a", "..", "b")) -- tmp/b
```

## ptool.path.normalize

> `v0.1.0` - Introduced.

`ptool.path.normalize(path)` performs lexical path normalization (processing `.`
and `..`).

- `path` (string, required): The input path.

```lua
print(ptool.path.normalize("./a/../b")) -- b
```

## ptool.path.abspath

> `v0.1.0` - Introduced.

`ptool.path.abspath(path[, base])` computes an absolute path.

- `path` (string, required): The input path.
- `base` (string, optional): The base directory. If omitted, the current process
  working directory is used.
- Accepts only 1 or 2 string arguments.

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
- Accepts only 1 or 2 string arguments.

```lua
print(ptool.path.relpath("src/main.rs", "/tmp/project"))
```

## ptool.path.isabs

> `v0.1.0` - Introduced.

`ptool.path.isabs(path)` checks whether a path is absolute.

- `path` (string, required): The input path.
- Returns: `boolean`.

```lua
print(ptool.path.isabs("/tmp")) -- true
```

## ptool.path.dirname

> `v0.1.0` - Introduced.

`ptool.path.dirname(path)` returns the directory-name portion.

- `path` (string, required): The input path.

```lua
print(ptool.path.dirname("a/b/c.txt")) -- a/b
```

## ptool.path.basename

> `v0.1.0` - Introduced.

`ptool.path.basename(path)` returns the last path segment (the filename
portion).

- `path` (string, required): The input path.

```lua
print(ptool.path.basename("a/b/c.txt")) -- c.txt
```

## ptool.path.extname

> `v0.1.0` - Introduced.

`ptool.path.extname(path)` returns the extension (including `.`). If there is no
extension, it returns an empty string.

- `path` (string, required): The input path.

```lua
print(ptool.path.extname("a/b/c.txt")) -- .txt
```

Notes:

- Path handling in `ptool.path` is purely lexical. It does not check whether
  paths exist and does not resolve symlinks.
- None of the interfaces accept empty string arguments. Passing one raises an
  error.

## ptool.toml.parse

> `v0.1.0` - Introduced.

`ptool.toml.parse(input)` parses a TOML string into a Lua table.

- `input` (string, required): The TOML text.
- Returns: A Lua table (the root node is always a table).

Type mapping:

- TOML table / inline table -> Lua table
- TOML array -> Lua sequence table (1-based)
- TOML string -> Lua string
- TOML integer -> Lua integer
- TOML float -> Lua number
- TOML boolean -> Lua boolean
- TOML datetime/date/time -> Lua string

Error behavior:

- An error is raised if `input` is not a string.
- A TOML syntax error raises an error whose message includes line and column
  information.

Example:

```lua
local text = ptool.fs.read("ptool.toml")
local conf = ptool.toml.parse(text)

print(conf.project.name)
print(conf.build.jobs)
print(conf.release_date) -- datetime/date/time values are strings
```

## ptool.toml.get

> `v0.1.0` - Introduced.

`ptool.toml.get(input, path)` reads the value at a specified path from TOML
text.

- `input` (string, required): The TOML text.
- `path` (string[], required): A non-empty path array, such as `{"package",
  "version"}`.
- Returns: The corresponding Lua value, or `nil` if the path does not exist.

Example:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)
```

## ptool.toml.set

> `v0.1.0` - Introduced.

`ptool.toml.set(input, path, value)` sets the value at a specified path and
returns the updated TOML text.

- `input` (string, required): The TOML text.
- `path` (string[], required): A non-empty path array, such as `{"package",
  "version"}`.
- `value` (string|integer|number|boolean, required): The value to write.
- Returns: The updated TOML string.

Behavior:

- Missing intermediate paths are created automatically as tables.
- If an intermediate path exists but is not a table, an error is raised.
- Parsing and writing back are based on `toml_edit`, which preserves original
  comments and formatting as much as possible.

Example:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.set(text, {"package", "version"}, "0.2.0")
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.remove

> `v0.1.0` - Introduced.

`ptool.toml.remove(input, path)` removes the specified path and returns the
updated TOML text.

- `input` (string, required): The TOML text.
- `path` (string[], required): A non-empty path array, such as `{"package",
  "name"}`.
- Returns: The updated TOML string.

Behavior:

- If the path does not exist, no error is raised and the original text (or an
  equivalent form) is returned.
- If an intermediate path exists but is not a table, an error is raised.

Example:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.remove(text, {"package", "description"})
ptool.fs.write("Cargo.toml", updated)
```

Notes:

- The `path` argument for `ptool.toml.get/set/remove` must be a non-empty string
  array.
- `set` currently supports writing only scalar types
  (`string`/`integer`/`number`/`boolean`).

## ptool.re.compile

> `v0.1.0` - Introduced.

`ptool.re.compile(pattern[, opts])` compiles a regular expression and returns a
`Regex` object.

- `pattern` (string, required): The regex pattern.
- `opts` (table, optional): Compile options. Currently supported:
  - `case_insensitive` (boolean, optional): Whether matching is
    case-insensitive. Defaults to `false`.

Example:

```lua
local re = ptool.re.compile("(?P<name>\\w+)", { case_insensitive = true })
print(re:is_match("Alice")) -- true
```

## ptool.re.escape

> `v0.1.0` - Introduced.

`ptool.re.escape(text)` escapes plain text into a regex literal string.

- `text` (string, required): The text to escape.
- Returns: The escaped string.

Example:

```lua
local keyword = "a+b?"
local re = ptool.re.compile("^" .. ptool.re.escape(keyword) .. "$")
print(re:is_match("a+b?")) -- true
```

### Regex Methods

`ptool.re.compile(...)` returns a `Regex` UserData with the following methods:

- `re:is_match(input)` -> `boolean`
- `re:find(input[, init])` -> `Match|nil`
- `re:find_all(input)` -> `Match[]`
- `re:captures(input)` -> `Captures|nil`
- `re:captures_all(input)` -> `Captures[]`
- `re:replace(input, replacement)` -> `string`
- `re:replace_all(input, replacement)` -> `string`
- `re:split(input[, limit])` -> `string[]`

Parameter notes:

- `init` is a 1-based start position and defaults to `1`.
- `limit` must be greater than `0`.

Return structures:

- `Match`:
  - `start` (integer): The 1-based start index.
  - `finish` (integer): The end index, directly usable with `string.sub`.
  - `text` (string): The matched text.
- `Captures`:
  - `full` (string): The full matched text.
  - `groups` (table): An array of capture groups in capture order. Unmatched
    groups are `nil`.
  - `named` (table): A mapping of named capture groups, keyed by group name.

Example:

```lua
local re = ptool.re.compile("(?P<word>\\w+)")
local cap = re:captures("hello world")
print(cap.full)         -- hello
print(cap.named.word)   -- hello
print(re:replace_all("a b c", "_")) -- _ _ _
```

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` reads a UTF-8 text file and returns a string.

- `path` (string, required): The file path.

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

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
```

## ptool.fs.mkdir

> `v0.1.0` - Introduced.

`ptool.fs.mkdir(path)` creates a directory. If parent directories do not exist,
they are created recursively.

- `path` (string, required): The directory path.

```lua
ptool.fs.mkdir("tmp/a/b")
```

## ptool.fs.exists

> `v0.1.0` - Introduced.

`ptool.fs.exists(path)` checks whether a path exists.

- `path` (string, required): A file or directory path.
- Returns: `boolean`.

```lua
if ptool.fs.exists("tmp/hello.txt") then
  print("exists")
end
```

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split` parses a command string using shell-style rules and returns an
argument array.

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

The `args` above is equivalent to:

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```

## ptool.template.render

> `v0.1.0` - Introduced.

`ptool.template.render(template, context)` renders a Jinja-style template string
and returns the rendered result.

- `template` (string, required): The template source text.
- `context` (any serializable Lua value, required): The template context.
- Returns: The rendered string.

Example:

```lua
local template = ptool.unindent([[
  | {% if user.active %}
  | Hello, {{ user.name }}!
  | {% else %}
  | Inactive user: {{ user.name }}
  | {% endif %}
  | Items:
  | {% for item in items %}
  | - {{ item }}
  | {% endfor %}
]])
local result = ptool.template.render(template, {
  user = { name = "alice", active = true },
  items = { "one", "two", "three" },
})

print(result)
```

Notes:

- The context must be serializable to data values.
- Lua values such as `function`, `thread`, and unsupported `userdata` are not
  accepted as template context values.

## ptool.args.arg

> `v0.1.0` - Introduced.

`ptool.args.arg(id, kind, options)` creates an argument builder for use in
`ptool.args.parse(...).schema.args`.

- `id` (string, required): The argument identifier. It is also the key in the
  returned table.
- `kind` (string, required): The argument type. Supported values:
  - `"flag"`: A boolean flag.
  - `"string"`: A string option.
  - `"int"`: An integer option (`i64`).
  - `"positional"`: A positional argument.
- `options` (table, optional): The same optional fields supported by argument
  tables in `ptool.args.parse`, such as `long`, `short`, `help`, `required`,
  `multiple`, and `default`.

The builder supports chainable methods, all of which return itself:

- `arg:long(value)` sets the long option name. Supported only for
  non-`positional` arguments.
- `arg:short(value)` sets the short option name. Supported only for
  non-`positional` arguments.
- `arg:help(value)` sets the help text.
- `arg:required(value)` sets whether the argument is required. If `value` is
  omitted, it defaults to `true`.
- `arg:multiple(value)` sets whether the argument can be repeated. If `value` is
  omitted, it defaults to `true`.
- `arg:default(value)` sets the default value. If `value = nil`, the default is
  cleared.

Example:

```lua
local res = ptool.args.parse({
  args = {
    ptool.args.arg("name", "string"):required(),
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("paths", "positional"):multiple(),
  }
})
```

## ptool.args.parse

> `v0.1.0` - Introduced.

`ptool.args.parse(schema)` parses script arguments with `clap` and returns a
table indexed by `id`.

Script arguments come from the part after `--` in `ptool run <lua_file> -- ...`.

For example:

```lua
ptool.use("v0.1.0")

local res = ptool.args.parse({
    name = "test",
    about = "The test command",
    args = {
        { id = "name", kind = "string" }
    }
})

print("Hello, " .. res.name .. "!")
```

### Schema Structure

- `name` (string, optional): The command name, used in help output. Defaults to
  the script file name.
- `about` (string, optional): Help description.
- `args` (table, required): An array of argument definitions. Each item supports
  two forms:
  - An argument table.
  - A builder object returned by `ptool.args.arg(...)`.

Argument table fields:

- `id` (string, required): The argument identifier. It is also the key in the
  returned table.
- `kind` (string, required): The argument type. Supported values:
  - `"flag"`: A boolean flag.
  - `"string"`: A string option.
  - `"int"`: An integer option (`i64`).
  - `"positional"`: A positional argument.
- `long` (string, optional): The long option name, such as `"name"` for
  `--name`. For non-`positional` arguments, the default can be derived from
  `id`.
- `short` (string, optional): The short option name, a single character such as
  `"v"` for `-v`.
- `help` (string, optional): Help text for the argument.
- `required` (boolean, optional): Whether the argument is required. Defaults to
  `false`.
- `multiple` (boolean, optional): Whether the argument can be repeated. Defaults
  to `false`.
- `default` (string/integer, optional): The default value.

### Constraints

- The following constraints apply to both argument tables and builder syntax.
- Non-`positional` arguments may omit `long` and `short`. If `long` is omitted,
  `id` is used automatically.
- `positional` arguments cannot set `long`, `short`, or `default`.
- When `positional.multiple = true`, it must be the last argument in `args`.
- `multiple = true` is supported only for `string` and `positional`.
- `default` is supported only for `string` and `int`, and cannot be used
  together with `multiple = true`.

### Return Value

A Lua table is returned where keys are `id` and value types are as follows:

- `flag` -> `boolean`
- `string` -> `string` (or `string[]` when `multiple = true`)
- `int` -> `integer`
- `positional` -> `string` (or `string[]` when `multiple = true`)
