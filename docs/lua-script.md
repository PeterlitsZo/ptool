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
- `op` (string, required): One of `major`, `minor`, `patch`, `release`,
  `alpha`, `beta`, or `rc`.
- Returns: `Version`.

```lua
local v = ptool.semver.bump("1.2.3", "alpha")
print(tostring(v)) -- 1.2.4-alpha.1

local stable = ptool.semver.bump("1.2.4-rc.2", "release")
print(tostring(stable)) -- 1.2.4
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
- `release` removes prerelease and build metadata while keeping the same
  `major.minor.patch` values, such as `1.2.3-rc.2 -> 1.2.3`.
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

## ptool.inspect

> `v0.1.0` - Introduced.

`ptool.inspect(value[, options])` renders a Lua value as a readable Lua-style
string. It is primarily intended for debugging and displaying table contents.

- `value` (any, required): The Lua value to inspect.
- `options` (table, optional): Rendering options. Supported fields:
  - `indent` (string, optional): Indentation used for each nesting level.
    Defaults to two spaces.
  - `multiline` (boolean, optional): Whether tables are rendered across multiple
    lines. Defaults to `true`.
  - `max_depth` (integer, optional): Maximum nesting depth to render. Deeper
    values are replaced with `<max-depth>`.
- Returns: `string`.

Behavior:

- Array-like entries (`1..n`) are rendered first.
- Remaining table fields are rendered after the array part in stable key order.
- Identifier-like string keys are rendered as `key = value`; other keys are
  rendered as `[key] = value`.
- Recursive table references are rendered as `<cycle>`.
- Functions, threads, and userdata are rendered as placeholder values such as
  `<function>` and `<userdata>`.

Example:

```lua
local value = {
  "hello",
  user = { name = "alice", tags = {"dev", "ops"} },
}
value.self = value

print(ptool.inspect(value))
print(ptool.inspect(value, { multiline = false }))
```

## ptool.ask

> `v0.1.0` - Introduced.

`ptool.ask(prompt[, options])` asks the user for a line of text and returns the
answer.

- `prompt` (string, required): The prompt shown to the user.
- `options` (table, optional): Prompt options. Supported fields:
  - `default` (string, optional): Default value used when the user submits an
    empty answer.
  - `help` (string, optional): Extra help text shown below the prompt.
  - `placeholder` (string, optional): Placeholder text shown before the user
    starts typing.
- Returns: `string`.

Behavior:

- Requires an interactive TTY. Running it in a non-interactive environment
  raises an error.
- If the user cancels the prompt, the script raises an error.
- Unknown option names or invalid option value types raise an error.

Example:

```lua
local name = ptool.ask("Your name?", {
  placeholder = "Alice",
  help = "Press Enter to confirm",
})

local city = ptool.ask("City?", {
  default = "Shanghai",
})

print(string.format("Hello, %s from %s!", name, city))
```

## ptool.ansi.style

> `v0.1.0` - Introduced.

`ptool.ansi.style(text[, options])` returns `text` wrapped in ANSI style escape
sequences.

- `text` (string, required): The text to style.
- `options` (table, optional): Style options. Supported fields:
  - `enabled` (boolean, optional): Whether ANSI escapes should be emitted.
    Defaults to whether `ptool` is writing to a terminal.
  - `fg` (string|nil, optional): The foreground color. Supported values are
    `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `purple`, `cyan`,
    `white`, `bright_black`, `bright_red`, `bright_green`,
    `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_purple`,
    `bright_cyan`, and `bright_white`.
  - `bold` (boolean, optional): Whether to apply bold text.
  - `dimmed` (boolean, optional): Whether to apply dimmed text.
  - `italic` (boolean, optional): Whether to apply italic text.
  - `underline` (boolean, optional): Whether to apply underline text.
- Returns: `string`.

Behavior:

- If `enabled = false`, the original text is returned unchanged.
- If `fg = nil` or omitted, no foreground color is applied.
- Unknown option names or invalid option value types raise an error.

Example:

```lua
print(ptool.ansi.style("warning", {
  fg = "bright_yellow",
  bold = true,
}))
```

## ptool.ansi.<color>

> `v0.1.0` - Introduced.

`ptool.ansi.black`, `ptool.ansi.red`, `ptool.ansi.green`,
`ptool.ansi.yellow`, `ptool.ansi.blue`, `ptool.ansi.magenta`,
`ptool.ansi.cyan`, and `ptool.ansi.white` are convenience helpers with the
following signature:

```lua
ptool.ansi.red(text[, options])
```

They accept the same `text` argument and the same `options` table as
`ptool.ansi.style`, except the foreground color is fixed by the helper itself.
If `options.fg` is also provided, the helper color takes precedence.

Example:

```lua
print(ptool.ansi.green("ok", { bold = true }))
print(ptool.ansi.red("failed", { enabled = true, underline = true }))
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
  - `retry` (boolean, optional): Whether to ask the user whether to retry after
    a failed execution when `check = true`. Defaults to `false`.

Example:

```lua
ptool.config({
  run = {
    echo = false,
    check = true,
    confirm = false,
    retry = false,
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
- When both `check = true` and `retry = true`, `ptool.run` asks whether the
  failed command should be retried before raising the final error.

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
- `retry` (boolean, optional): Whether to ask the user whether to retry after a
  failed execution when `check = true`. If omitted, the value from
  `ptool.config({ run = { retry = ... } })` is used; if that is also unset, the
  default is `false`.
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
- When `retry = true` and `check = true`:
  - If the command fails, `ptool.run` asks whether to retry the same command.
  - If the current environment is not interactive (no TTY), an error is raised
    immediately instead of prompting for retry.

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

## ptool.db.connect

> `v0.1.0` - Introduced.

`ptool.db.connect(url_or_options)` opens a database connection and returns a
`Connection` UserData.

Supported databases:

- SQLite
- PostgreSQL
- MySQL

Arguments:

- `url_or_options` (string|table, required):
  - When a string is provided, it is treated as the database URL.
  - When a table is provided, it currently supports:
    - `url` (string, required): The database URL.

Supported URL examples:

```lua
local sqlite_db = ptool.db.connect("sqlite:test.db")
local pg_db = ptool.db.connect("postgres://user:pass@localhost/app")
local mysql_db = ptool.db.connect("mysql://user:pass@localhost/app")
```

SQLite notes:

- `sqlite:test.db` and `sqlite://test.db` are supported.
- Relative SQLite paths are resolved from the current `ptool` runtime
  directory, so they follow `ptool.cd(...)`.
- If no `mode=` query parameter is provided, SQLite connections default to
  `mode=rwc`, which allows creating the database file automatically.

Example:

```lua
ptool.cd("workdir")
local db = ptool.db.connect({
  url = "sqlite:data/app.db",
})
```

## ptool.db.Connection

> `v0.1.0` - Introduced.

`ptool.db.connect(...)` returns a `Connection` UserData with the following
methods:

- `db:query(sql, params?)` -> `table`
- `db:query_one(sql, params?)` -> `table|nil`
- `db:scalar(sql, params?)` -> `boolean|integer|number|string|nil`
- `db:execute(sql, params?)` -> `table`
- `db:transaction(fn)` -> `any`
- `db:close()` -> `nil`

Parameter binding:

- `params` is optional.
- When `params` is an array table, it is treated as positional parameters and
  SQL placeholders should use `?`.
- When `params` is a key-value table, it is treated as named parameters and SQL
  placeholders should use `:name`.
- Positional and named parameters cannot be mixed in the same call.
- Supported parameter value types are:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil` is not supported as a bound parameter in `v0.1.0`.

Result value rules:

- Query results only guarantee these Lua value types:
  - `boolean`
  - `integer`
  - `number`
  - `string`
  - `nil` (for SQL `NULL`)
- Text columns are returned as Lua strings.
- Binary/blob columns are also returned as Lua strings.
- If a query result contains duplicate column names, an error is raised. Use SQL
  aliases such as `AS` to disambiguate them.

## ptool.db.Connection:query

> `v0.1.0` - Introduced.

`db:query(sql, params?)` executes a query and returns a table with:

- `rows` (table): An array of row tables.
- `columns` (table): An array of column names.
- `row_count` (integer): The number of rows returned.

Example:

```lua
local db = ptool.db.connect("sqlite:test.db")

db:execute("create table users (id integer primary key, name text)")
db:execute("insert into users(name) values (?)", {"alice"})
db:execute("insert into users(name) values (:name)", { name = "bob" })

local res = db:query("select id, name from users order by id")
print(res.row_count)
print(res.columns[1], res.columns[2])
print(res.rows[1].name)
print(res.rows[2].name)
```

## ptool.db.Connection:query_one

> `v0.1.0` - Introduced.

`db:query_one(sql, params?)` returns the first row as a table, or `nil` if the
query returns no rows.

Example:

```lua
local row = db:query_one("select id, name from users where name = ?", {"alice"})
if row then
  print(row.id, row.name)
end
```

## ptool.db.Connection:scalar

> `v0.1.0` - Introduced.

`db:scalar(sql, params?)` returns the first column of the first row, or `nil` if
the query returns no rows.

Example:

```lua
local count = db:scalar("select count(*) from users")
print(count)
```

## ptool.db.Connection:execute

> `v0.1.0` - Introduced.

`db:execute(sql, params?)` executes a statement and returns a table with:

- `rows_affected` (integer): The number of affected rows.

Example:

```lua
local res = db:execute("update users set name = ? where id = ?", {"alice-2", 1})
print(res.rows_affected)
```

## ptool.db.Connection:transaction

> `v0.1.0` - Introduced.

`db:transaction(fn)` runs `fn(tx)` inside a database transaction.

Behavior:

- If `fn(tx)` returns normally, the transaction is committed.
- If `fn(tx)` raises an error, the transaction is rolled back and the error is
  re-raised.
- Nested transactions are not supported.
- While the callback is active, the outer connection object must not be used;
  use the provided `tx` object instead.

The `tx` object supports the same query methods as `Connection`:

- `tx:query(sql, params?)`
- `tx:query_one(sql, params?)`
- `tx:scalar(sql, params?)`
- `tx:execute(sql, params?)`

Example:

```lua
db:transaction(function(tx)
  tx:execute("insert into users(name) values (?)", {"charlie"})
  tx:execute("insert into users(name) values (?)", {"dora"})
end)

local ok, err = pcall(function()
  db:transaction(function(tx)
    tx:execute("insert into users(name) values (?)", {"eve"})
    error("stop")
  end)
end)
print(ok) -- false
print(tostring(err))
```

## ptool.db.Connection:close

> `v0.1.0` - Introduced.

`db:close()` closes the connection.

Behavior:

- After closing, the connection can no longer be used.
- Closing during an active transaction callback raises an error.

Example:

```lua
local db = ptool.db.connect("sqlite:test.db")
db:close()
```

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

## ptool.ssh.connect

> `v0.1.0` - Introduced.

`ptool.ssh.connect(target_or_options)` opens an SSH connection and returns a
`Connection` UserData.

Arguments:

- `target_or_options` (string|table, required):
  - When a string is provided, it is treated as an SSH target.
  - When a table is provided, it currently supports:
    - `target` (string, optional): SSH target string such as
      `"deploy@example.com"` or `"deploy@example.com:2222"`.
    - `host` (string, optional): Hostname or IP address.
    - `user` (string, optional): SSH username. Defaults to `$USER`, or `"root"`
      if `$USER` is unavailable.
    - `port` (integer, optional): SSH port. Defaults to `22`.
    - `auth` (table, optional): Authentication settings.
    - `host_key` (table, optional): Host key verification settings.
    - `connect_timeout_ms` (integer, optional): Timeout in milliseconds.
      Defaults to `10000`.
    - `keepalive_interval_ms` (integer, optional): Keepalive interval in
      milliseconds.

Supported target string examples:

```lua
local a = ptool.ssh.connect("deploy@example.com")
local b = ptool.ssh.connect("deploy@example.com:2222")
local c = ptool.ssh.connect("[2001:db8::10]:2222")
```

`auth` fields:

- `private_key_file` (string, optional): Path to a private key file.
- `private_key_passphrase` (string, optional): Passphrase for the private key.
- `password` (string, optional): Password-based authentication.

Authentication behavior:

- If `auth.password` is provided, password authentication is used.
- Otherwise, if `auth.private_key_file` is provided, public-key
  authentication is used with that key.
- Otherwise, `ptool` tries these default private key files in order:
  `~/.ssh/id_ed25519`, `~/.ssh/id_rsa`, `~/.ssh/id_ecdsa`.
- Relative key paths are resolved from the current `ptool` runtime directory,
  so they follow `ptool.cd(...)`.
- `~` and `~/...` are expanded in key paths.

`host_key` fields:

- `verify` (string, optional): Host key verification mode. Supported values:
  - `"known_hosts"`: Verify against a known_hosts file (default).
  - `"ignore"`: Skip host key verification.
- `known_hosts_file` (string, optional): Path to a known_hosts file. Used only
  when `verify = "known_hosts"`.

Host key behavior:

- When `known_hosts_file` is omitted, the default known_hosts location is used.
- Relative `known_hosts_file` paths are resolved from the current `ptool`
  runtime directory.
- `~` and `~/...` are expanded in `known_hosts_file`.

Example:

```lua
local ssh = ptool.ssh.connect({
  host = "example.com",
  user = "deploy",
  port = 22,
  auth = {
    private_key_file = "~/.ssh/id_ed25519",
  },
  host_key = {
    verify = "known_hosts",
  },
})
```

## ptool.ssh.Connection

> `v0.1.0` - Introduced.

`ptool.ssh.connect(...)` returns a `Connection` UserData with the following
fields and methods:

- Fields:
  - `conn.host` (string)
  - `conn.user` (string)
  - `conn.port` (integer)
  - `conn.target` (string)
- Methods:
  - `conn:run(...)` -> `table`
  - `conn:path(path)` -> `userdata`
  - `conn:upload(local_path, remote_path[, options])` -> `table`
  - `conn:download(remote_path, local_path[, options])` -> `table`
  - `conn:close()` -> `nil`

## ptool.ssh.Connection:run

> `v0.1.0` - Introduced.

`conn:run(...)` executes a remote command through the current SSH connection.

The following call forms are supported:

```lua
conn:run("hostname")
conn:run("echo", "hello world")
conn:run("echo", {"hello", "world"})
conn:run("hostname", { stdout = "capture" })
conn:run("echo", {"hello", "world"}, { stdout = "capture" })
conn:run({ cmd = "git", args = {"rev-parse", "HEAD"} })
```

Argument rules:

- `conn:run(cmdline)`: `cmdline` is sent as the remote command string.
- `conn:run(cmd, argsline)`: `cmd` is treated as the command, and `argsline` is
  split using shell-style (`shlex`) rules.
- `conn:run(cmd, args)`: `cmd` is a string and `args` is an array of strings.
  Arguments are shell-quoted before remote execution.
- `conn:run(cmdline, options)`: `options` overrides this invocation.
- `conn:run(cmd, args, options)`: `options` overrides this invocation.
- `conn:run(options)`: `options` is a table.
- When the second argument is a table: if it is an array (consecutive integer
  keys `1..n`), it is treated as `args`; otherwise it is treated as `options`.

When `conn:run(options)` is used, `options` currently supports:

- `cmd` (string, required): The command name or executable path.
- `args` (string[], optional): The argument list.
- `cwd` (string, optional): Remote working directory. This is applied by
  prepending `cd ... &&` to the generated remote shell command.
- `env` (table, optional): Remote environment variables, where keys and values
  are strings. This is applied by prepending `export ... &&` to the generated
  remote shell command.
- `stdin` (string, optional): String sent to the remote process stdin.
- `echo` (boolean, optional): Whether to echo the remote command before
  execution. Defaults to `false`.
- `check` (boolean, optional): Whether to raise an error immediately when the
  exit status is not `0`. Defaults to `false`.
- `stdout` (string, optional): Stdout handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stdout`.
  - `"null"`: Discard the output.
- `stderr` (string, optional): Stderr handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stderr`.
  - `"null"`: Discard the output.

When the shortcut forms are used, the `options` table supports only:

- `stdin` (string, optional): String sent to the remote process stdin.
- `echo` (boolean, optional): Whether to echo the remote command before
  execution. Defaults to `false`.
- `check` (boolean, optional): Whether to raise an error immediately when the
  exit status is not `0`. Defaults to `false`.
- `stdout` (string, optional): Stdout handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stdout`.
  - `"null"`: Discard the output.
- `stderr` (string, optional): Stderr handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stderr`.
  - `"null"`: Discard the output.

Return value rules:

- A table is always returned with the following fields:
  - `ok` (boolean): Whether the remote exit status is `0`.
  - `code` (integer|nil): The remote exit status. If the remote process exits by
    signal, this is `nil`.
  - `target` (string): The SSH target string in the form `user@host:port`.
  - `stdout` (string, optional): Present when `stdout = "capture"`.
  - `stderr` (string, optional): Present when `stderr = "capture"`.
  - `assert_ok(self)` (function): Raises an error when `ok = false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:run("uname -a", { stdout = "capture" })
print(res.target)
print(res.stdout)
local res2 = ssh:run({
  cmd = "git",
  args = {"rev-parse", "HEAD"},
  cwd = "/srv/app",
  env = {
    FOO = "bar",
  },
  stdout = "capture",
  check = true,
})

print(res2.stdout)
```

## ptool.ssh.Connection:path

> `v0.1.0-alpha.4` - Introduced.

`conn:path(path)` creates a reusable remote path value bound to the current SSH
connection.

- `path` (string, required): The remote path.
- Returns: A remote path userdata that can be passed to
  `conn:upload(...)`, `conn:download(...)`, and `ptool.fs.copy(...)`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

ssh:download(remote_release, "./tmp/current.tar.gz")
```

## ptool.ssh.Connection:upload

> `v0.1.0-alpha.4` - Introduced.

`conn:upload(local_path, remote_path[, options])` uploads a local file to the
remote host.

- `local_path` (string, required): The local file to upload.
- `remote_path` (string|remote path, required): The destination path on the
  remote host. It can be a string or a value created by `conn:path(...)`.
- `options` (table, optional): Transfer options.
- Returns: A table with the following fields:
  - `bytes` (integer): The number of bytes uploaded.
  - `from` (string): The local source path.
  - `to` (string): The remote destination path.

Supported transfer options:

- `parents` (boolean, optional): Create the parent directory of `remote_path`
  before uploading. Defaults to `false`.
- `overwrite` (boolean, optional): Whether an existing destination file may be
  replaced. Defaults to `true`.
- `echo` (boolean, optional): Whether to print the transfer before executing
  it. Defaults to `false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

local res = ssh:upload("./dist/app.tar.gz", remote_tarball, {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.to)
```

## ptool.ssh.Connection:download

> `v0.1.0-alpha.4` - Introduced.

`conn:download(remote_path, local_path[, options])` downloads a remote file to a
local path.

- `remote_path` (string|remote path, required): The source path on the remote
  host. It can be a string or a value created by `conn:path(...)`.
- `local_path` (string, required): The local destination path.
- `options` (table, optional): Transfer options.
- Returns: A table with the following fields:
  - `bytes` (integer): The number of bytes downloaded.
  - `from` (string): The remote source path.
  - `to` (string): The local destination path.

Supported transfer options:

- `parents` (boolean, optional): Create the parent directory of `local_path`
  before downloading. Defaults to `false`.
- `overwrite` (boolean, optional): Whether an existing destination file may be
  replaced. Defaults to `true`.
- `echo` (boolean, optional): Whether to print the transfer before executing
  it. Defaults to `false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:download("/srv/app/logs/app.log", "./tmp/app.log", {
  parents = true,
  overwrite = false,
  echo = true,
})

print(res.bytes)
print(res.from)
```

## ptool.ssh.Connection:close

> `v0.1.0` - Introduced.

`conn:close()` closes the SSH connection.

Behavior:

- After closing, the connection can no longer be used.
- Closing an already-closed connection is allowed and has no effect.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
ssh:close()
```

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

## ptool.str.trim

> `v0.1.0` - Introduced.

`ptool.str.trim(s)` removes leading and trailing whitespace.

- `s` (string, required): The input string.
- Returns: `string`.

```lua
print(ptool.str.trim("  hello\n")) -- hello
```

## ptool.str.trim_start

> `v0.1.0` - Introduced.

`ptool.str.trim_start(s)` removes leading whitespace.

- `s` (string, required): The input string.
- Returns: `string`.

```lua
print(ptool.str.trim_start("  hello  ")) -- hello  
```

## ptool.str.trim_end

> `v0.1.0` - Introduced.

`ptool.str.trim_end(s)` removes trailing whitespace.

- `s` (string, required): The input string.
- Returns: `string`.

```lua
print(ptool.str.trim_end("  hello  ")) --   hello
```

## ptool.str.is_blank

> `v0.1.0` - Introduced.

`ptool.str.is_blank(s)` checks whether a string is empty or contains only
whitespace.

- `s` (string, required): The input string.
- Returns: `boolean`.

```lua
print(ptool.str.is_blank(" \t\n")) -- true
print(ptool.str.is_blank("x")) -- false
```

## ptool.str.starts_with

> `v0.1.0` - Introduced.

`ptool.str.starts_with(s, prefix)` checks whether `s` starts with `prefix`.

- `s` (string, required): The input string.
- `prefix` (string, required): The prefix to test.
- Returns: `boolean`.

```lua
print(ptool.str.starts_with("hello.lua", "hello")) -- true
```

## ptool.str.ends_with

> `v0.1.0` - Introduced.

`ptool.str.ends_with(s, suffix)` checks whether `s` ends with `suffix`.

- `s` (string, required): The input string.
- `suffix` (string, required): The suffix to test.
- Returns: `boolean`.

```lua
print(ptool.str.ends_with("hello.lua", ".lua")) -- true
```

## ptool.str.contains

> `v0.1.0` - Introduced.

`ptool.str.contains(s, needle)` checks whether `needle` appears in `s`.

- `s` (string, required): The input string.
- `needle` (string, required): The substring to search for.
- Returns: `boolean`.

```lua
print(ptool.str.contains("hello.lua", "lo.l")) -- true
```

## ptool.str.split

> `v0.1.0` - Introduced.

`ptool.str.split(s, sep[, options])` splits a string by a non-empty separator.

- `s` (string, required): The input string.
- `sep` (string, required): The separator. Empty strings are not allowed.
- `options` (table, optional): Split options. Supported fields:
  - `trim` (boolean, optional): Whether to trim each piece before returning it.
    Defaults to `false`.
  - `skip_empty` (boolean, optional): Whether to remove empty pieces after
    optional trimming. Defaults to `false`.
- Returns: `string[]`.

Behavior:

- Unknown option names or invalid option value types raise an error.
- `skip_empty = true` is applied after `trim`, so whitespace-only pieces can be
  removed when both are enabled.

```lua
local parts = ptool.str.split(" a, b ,, c ", ",", {
  trim = true,
  skip_empty = true,
})

print(ptool.inspect(parts)) -- { "a", "b", "c" }
```

## ptool.str.split_lines

> `v0.1.0` - Introduced.

`ptool.str.split_lines(s[, options])` splits a string into lines.

- `s` (string, required): The input string.
- `options` (table, optional): Line-splitting options. Supported fields:
  - `keep_ending` (boolean, optional): Whether to keep line endings (`\n`,
    `\r\n`, or `\r`) in returned items. Defaults to `false`.
  - `skip_empty` (boolean, optional): Whether to remove empty lines. Defaults
    to `false`.
- Returns: `string[]`.

Behavior:

- Supports Unix (`\n`) and Windows (`\r\n`) line endings, and also lone `\r`.
- When `skip_empty = true`, a line containing only a line ending is treated as
  empty and is removed.
- Unknown option names or invalid option value types raise an error.

```lua
local lines = ptool.str.split_lines("a\n\n b\r\n", {
  skip_empty = true,
})

print(ptool.inspect(lines)) -- { "a", " b" }
```

## ptool.str.join

> `v0.1.0` - Introduced.

`ptool.str.join(parts, sep)` joins a string array with a separator.

- `parts` (string[], required): The string parts to join.
- `sep` (string, required): The separator string.
- Returns: `string`.

```lua
print(ptool.str.join({"a", "b", "c"}, "/")) -- a/b/c
```

## ptool.str.replace

> `v0.1.0` - Introduced.

`ptool.str.replace(s, from, to[, n])` replaces occurrences of `from` with `to`.

- `s` (string, required): The input string.
- `from` (string, required): The substring to replace. Empty strings are not
  allowed.
- `to` (string, required): The replacement string.
- `n` (integer, optional): Maximum replacement count. Must be greater than or
  equal to `0`. If omitted, all matches are replaced.
- Returns: `string`.

```lua
print(ptool.str.replace("a-b-c", "-", "/")) -- a/b/c
print(ptool.str.replace("a-b-c", "-", "/", 1)) -- a/b-c
```

## ptool.str.repeat

> `v0.1.0` - Introduced.

`ptool.str.repeat(s, n)` repeats a string `n` times.

- `s` (string, required): The input string.
- `n` (integer, required): Repeat count. Must be greater than or equal to `0`.
- Returns: `string`.

```lua
print(ptool.str.repeat("ab", 3)) -- ababab
```

## ptool.str.cut_prefix

> `v0.1.0` - Introduced.

`ptool.str.cut_prefix(s, prefix)` removes `prefix` from the start of `s` when
it is present.

- `s` (string, required): The input string.
- `prefix` (string, required): The prefix to remove.
- Returns: `string`.

Behavior:

- If `s` does not start with `prefix`, the original string is returned
  unchanged.

```lua
print(ptool.str.cut_prefix("refs/heads/main", "refs/heads/")) -- main
```

## ptool.str.cut_suffix

> `v0.1.0` - Introduced.

`ptool.str.cut_suffix(s, suffix)` removes `suffix` from the end of `s` when it
is present.

- `s` (string, required): The input string.
- `suffix` (string, required): The suffix to remove.
- Returns: `string`.

Behavior:

- If `s` does not end with `suffix`, the original string is returned unchanged.

```lua
print(ptool.str.cut_suffix("archive.tar.gz", ".gz")) -- archive.tar
```

## ptool.str.indent

> `v0.1.0` - Introduced.

`ptool.str.indent(s, prefix[, options])` adds `prefix` to each line.

- `s` (string, required): The input string.
- `prefix` (string, required): The text inserted before each line.
- `options` (table, optional): Indent options. Supported fields:
  - `skip_first` (boolean, optional): Whether to leave the first line unchanged.
    Defaults to `false`.
- Returns: `string`.

Behavior:

- Existing line endings are preserved.
- Empty input is returned unchanged.
- Unknown option names or invalid option value types raise an error.

```lua
local text = "first\nsecond\n"
print(ptool.str.indent(text, "> "))
print(ptool.str.indent(text, "  ", { skip_first = true }))
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
- Missing values use chainable undefined semantics. This means nested lookups
  such as `foo.bar.baz` can be passed to filters like `default(...)` without
  raising an error. When rendered directly without a fallback, undefined values
  become an empty string.

```lua
local template = ptool.unindent([[
  | {{ foo.bar.baz | default("N/A") }}
]])

print(ptool.template.render(template, {})) -- N/A
```

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
