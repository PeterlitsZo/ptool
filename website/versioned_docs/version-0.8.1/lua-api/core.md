# Core Lua API

`ptool` exposes these core runtime helpers directly under `ptool` and `p`.

`ptool run <lua_file>` runs a Lua script and injects the global variable `ptool`
(or its alias `p`; for example, `p.run` is equivalent to `ptool.run`). For
files ending in `.lua`, `ptool <lua_file>` is a CLI shortcut with the same
behavior.

The embedded Lua runtime keeps the base Lua globals and exposes only these
standard libraries by default:

- `table`
- `string`
- `math`
- `utf8`

Host-facing built-in modules such as `io`, `os`, and `package` are intentionally
not available. Use `ptool` APIs such as `ptool.fs`, `ptool.os`, `ptool.path`,
and `ptool.run` for filesystem, environment, process, network, and other
runtime operations instead.

If you want to pass arguments to a Lua script, you can do it like this:

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

The arguments can then be parsed with `ptool.args.parse(...)`.

Here is an example script:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

Shebang is supported, so you can add this to the top of the file:

```
#!/usr/bin/env ptool
```

## ptool.use

> `v0.1.0` - Introduced.
> `v0.7.0` - Added Cargo-style version requirement expressions.

`ptool.use` declares the required `ptool` version or version requirement for a
script.

```lua
ptool.use("v0.1.0")
ptool.use("^0.6.0")
ptool.use(">= v0.6.0, < 0.7.0")
```

- The argument accepts either a plain semantic version string or a Cargo-style
  version requirement expression.
- A plain version string supports an optional `v` prefix, such as `v0.1.0` or
  `0.1.0`, and keeps the historical behavior of declaring the minimum required
  `ptool` version.
- Requirement expressions support operators and patterns such as `^0.6.0`,
  `~0.6.0`, `>=0.6.0, <0.7.0`, `1.*`, and `1.2.*`.
- Requirement expressions also accept version components with an optional `v`
  prefix, such as `>= v0.6.0, < 0.7.0`.
- If the current `ptool` version does not satisfy the declared version or
  requirement, the script exits immediately with an error.

## ptool.version

> `v0.8.0` - Introduced.

`ptool.version()` returns the current `ptool` version string.

- Returns: `string`.
- The returned value is a semantic version string such as `0.7.2`.
- Use this when a script needs to print, record, or compare the runtime
  `ptool` version.

Example:

```lua
print(ptool.version())
print(p.version())
```

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
> `v0.5.0` - Added prompt validation options and prompt subcommands.

`ptool.ask` provides interactive prompts. You can call it directly for text
input, or use its prompt subcommands for confirmation, selection, multi-select,
and secret input.

Common behavior:

- All `ptool.ask` prompts require an interactive TTY. Running them in a
  non-interactive environment raises an error.
- If the user cancels a prompt, the script raises an error.
- Unknown option names or invalid option value types raise an error.

### ptool.ask

`ptool.ask(prompt[, options])` asks the user for a line of text and returns the
answer.

- `prompt` (string, required): The prompt shown to the user.
- `options` (table, optional): Prompt options. Supported fields:
  - `default` (string, optional): Default value used when the user submits an
    empty answer.
  - `help` (string, optional): Extra help text shown below the prompt.
  - `placeholder` (string, optional): Placeholder text shown before the user
    starts typing.
  - `required` (boolean, optional): Whether the answer must be non-empty.
  - `allow_empty` (boolean, optional): Whether to accept an empty answer.
    Defaults to `true`.
  - `trim` (boolean, optional): Whether to trim leading and trailing
    whitespace from the answer before returning it.
  - `min_length` (integer, optional): Minimum accepted character length.
  - `max_length` (integer, optional): Maximum accepted character length.
  - `pattern` (string, optional): Regular expression the answer must match.
- Returns: `string`.

Example:

```lua
local project = ptool.ask("Project name?", {
  placeholder = "my-tool",
  help = "Lowercase letters, digits, and dashes only",
  required = true,
  trim = true,
  pattern = "^[a-z0-9-]+$",
})
```

### ptool.ask.confirm

> `v0.5.0` - Introduced.

`ptool.ask.confirm(prompt[, options])` asks the user for a yes/no answer.

- `prompt` (string, required): The prompt shown to the user.
- `options` (table, optional): Prompt options. Supported fields:
  - `default` (boolean, optional): Default answer used when the user presses
    Enter without typing.
  - `help` (string, optional): Extra help text shown below the prompt.
- Returns: `boolean`.

Example:

```lua
local confirmed = ptool.ask.confirm("Continue?", {
  default = true,
})
```

### ptool.ask.select

> `v0.5.0` - Introduced.

`ptool.ask.select(prompt, items[, options])` asks the user to choose one item
from a list.

- `prompt` (string, required): The prompt shown to the user.
- `items` (table, required): Candidate items. Each entry may be:
  - A string, which is used as both the display label and the returned value.
  - A table like `{ label = "Patch", value = "patch" }`.
- `options` (table, optional): Prompt options. Supported fields:
  - `help` (string, optional): Extra help text shown below the prompt.
  - `page_size` (integer, optional): Maximum number of rows shown at once.
  - `default_index` (integer, optional): 1-based index of the initially
    selected item.
- Returns: `string`.

Example:

```lua
local bump = ptool.ask.select("Select bump type", {
  { label = "Patch", value = "patch" },
  { label = "Minor", value = "minor" },
  { label = "Major", value = "major" },
}, {
  default_index = 2,
})
```

### ptool.ask.multiselect

> `v0.5.0` - Introduced.

`ptool.ask.multiselect(prompt, items[, options])` asks the user to choose zero
or more items from a list.

- `prompt` (string, required): The prompt shown to the user.
- `items` (table, required): Candidate items. The format is the same as
  `ptool.ask.select`.
- `options` (table, optional): Prompt options. Supported fields:
  - `help` (string, optional): Extra help text shown below the prompt.
  - `page_size` (integer, optional): Maximum number of rows shown at once.
  - `default_indexes` (table, optional): 1-based indexes selected by default.
  - `min_selected` (integer, optional): Minimum number of items that must be
    selected.
  - `max_selected` (integer, optional): Maximum number of items that may be
    selected.
- Returns: `table`.

Example:

```lua
local targets = ptool.ask.multiselect("Select targets", {
  "linux",
  "macos",
  "windows",
}, {
  default_indexes = { 1, 2 },
  min_selected = 1,
})
```

### ptool.ask.secret

> `v0.5.0` - Introduced.

`ptool.ask.secret(prompt[, options])` asks the user for secret input such as a
token or password.

- `prompt` (string, required): The prompt shown to the user.
- `options` (table, optional): Prompt options. Supported fields:
  - `help` (string, optional): Extra help text shown below the prompt.
  - `required` (boolean, optional): Whether the answer must be non-empty.
  - `allow_empty` (boolean, optional): Whether to accept an empty answer.
    Defaults to `false`.
  - `confirm` (boolean, optional): Whether to ask the user to type the secret
    twice. Defaults to `false`.
  - `confirm_prompt` (string, optional): Custom prompt for the confirmation
    step.
  - `mismatch_message` (string, optional): Custom error message shown when the
    two answers do not match.
  - `display_toggle` (boolean, optional): Whether to allow temporarily showing
    the typed secret.
  - `min_length` (integer, optional): Minimum accepted character length.
  - `max_length` (integer, optional): Maximum accepted character length.
  - `pattern` (string, optional): Regular expression the answer must match.
- Returns: `string`.

Example:

```lua
local token = ptool.ask.secret("API token?", {
  confirm = true,
  min_length = 20,
})
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

## ptool.script_path

> `v0.4.0` - Introduced.

`ptool.script_path()` returns the absolute path of the current entry script.

- Returns: `string|nil`.

Behavior:

- When running through `ptool run <file>`, this returns the entry script path as
  an absolute, normalized path.
- The returned path is fixed when the runtime starts and does not change after
  `ptool.cd(...)`.
- In `ptool repl`, this returns `nil`.

Example:

```lua
local script_path = ptool.script_path()
local script_dir = ptool.path.dirname(script_path)
local project_root = ptool.path.dirname(script_dir)
```

## ptool.try

> `v0.4.0` - Introduced.

`ptool.try(fn)` runs `fn` and converts raised errors into return values.

- `fn` (function, required): Callback to execute.
- Returns: `ok, value, err`.

Return value rules:

- On success, `ok = true`, `err = nil`, and `value` contains the callback
  result.
- If the callback returns no values, `value` is `nil`.
- If the callback returns one value, `value` is that value.
- If the callback returns multiple values, `value` is an array-like table.
- On failure, `ok = false`, `value = nil`, and `err` is a table.

Structured error fields:

- `kind` (string): Stable error category such as `io_error`,
  `command_failed`, `invalid_argument`, `http_error`, or `lua_error`.
- `message` (string): Human-readable error message.
- `op` (string, optional): API or operation name such as `ptool.fs.read`.
- `detail` (string, optional): Extra detail for the failure.
- `path` (string, optional): Path involved in a filesystem failure.
- `input` (string, optional): Original input that failed to parse or validate.
- `cmd` (string, optional): Command name for command failures.
- `status` (integer, optional): Exit status or HTTP status when available.
- `stderr` (string, optional): Captured stderr for command failures.
- `url` (string, optional): URL involved in an HTTP failure.
- `cwd` (string, optional): Effective working directory for command failures.
- `target` (string, optional): SSH target for SSH-related command failures.
- `retryable` (boolean): Whether retrying may make sense. Defaults to `false`.

Behavior:

- `ptool` APIs raise structured errors. `ptool.try` converts them into the `err`
  table above so callers can branch on `err.kind` and related fields.
- Plain Lua errors are also caught. In that case, `err.kind` is `lua_error`,
  and only `message` is guaranteed.
- `ptool.try` is the recommended way to handle errors from APIs such as
  `ptool.fs.read`, `ptool.http.request`, `ptool.run(..., { check = true })`,
  and `res:assert_ok()`.

Example:

```lua
local ok, content, err = ptool.try(function()
  return ptool.fs.read("missing.txt")
end)

if not ok and err.kind == "io_error" then
  print(err.op, err.path)
end

local ok2, _, err2 = ptool.try(function()
  local res = ptool.run({
    cmd = "sh",
    args = {"-c", "echo bad >&2; exit 7"},
    stderr = "capture",
  })
  res:assert_ok()
end)

if not ok2 and err2.kind == "command_failed" then
  print(err2.cmd, err2.status, err2.stderr)
end
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
  - `cmd` (string): Command name used for the execution.
  - `cwd` (string): Effective working directory used for the execution.
  - `stdout` (string, optional): Present when `stdout = "capture"`.
  - `stderr` (string, optional): Present when `stderr = "capture"`.
  - `assert_ok(self)` (function): Raises a structured error when `ok = false`.
    The error kind is `command_failed`, and the error may include `cmd`,
    `status`, `stderr`, and `cwd`.
- The default value of `check` comes from `ptool.config({ run = { check = ... }
  })`. If not configured, it defaults to `false`. When `check = false`, callers
  can inspect `ok` themselves or call `res:assert_ok()`.
- When both `check = true` and `retry = true`, `ptool.run` asks whether the
  failed command should be retried before raising the final error.
- When `check = true`, `ptool.run` raises the same structured `command_failed`
  error that `res:assert_ok()` raises. Use `ptool.try(...)` if you want to
  catch and inspect it from Lua.

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
- `stdin` (string|table, optional): Child process stdin source.
  - A string sends that string to the child process stdin.
  - A table `{ file = "path" }` reads stdin from a file.
  - When omitted, the child process inherits the current process stdin.
- `trim` (boolean, optional): Whether to trim leading and trailing whitespace
  from captured `stdout` and captured `stderr` before returning them. This only
  affects streams set to `"capture"`. Defaults to `false`.
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
- `stdout` (string|table, optional): Stdout handling strategy.
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stdout`.
  - `"null"`: Discard the output.
  - `{ file = "path" }`: Write stdout to a file, truncating it first.
  - `{ file = "path", append = true }`: Append stdout to a file.
- `stderr` (string|table, optional): Stderr handling strategy.
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stderr`.
  - `"null"`: Discard the output.
  - `{ file = "path" }`: Write stderr to a file, truncating it first.
  - `{ file = "path", append = true }`: Append stderr to a file.
- When shortcut call forms such as `ptool.run(cmdline, options)` or
  `ptool.run(cmd, args, options)` are used, the per-call `options` table also
  accepts `stdin` and `trim` with the same meaning.
- File redirect paths are resolved relative to the effective child-process
  `cwd`, unless an absolute path is provided.
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

local res0 = ptool.run({
  cmd = "cat",
  stdin = "hello from stdin",
  trim = true,
  stdout = "capture",
})
print(res0.stdout)

ptool.run({
  cmd = "sh",
  args = {"-c", "cat; printf ' err' >&2"},
  stdin = { file = "input.txt" },
  stdout = { file = "output.log" },
  stderr = { file = "error.log", append = true },
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

## ptool.run_capture

> `v0.6.0` - Introduced.

`ptool.run_capture` executes external commands from Rust with the same call
forms, argument rules, return value rules, and options as `ptool.run`.

The difference is only the default stream handling:

- `stdout` defaults to `"capture"`.
- `stderr` defaults to `"capture"`.

`trim` still defaults to `false`, and you can still override any of these
fields explicitly in `options`.

Example:

```lua
local res = ptool.run_capture("echo hello world")
print(res.stdout)

local res2 = ptool.run_capture({
  cmd = "cat",
  stdin = "captured stdin",
  trim = true,
})
print(res2.stdout)

ptool.run_capture({
  cmd = "sh",
  args = {"-c", "printf 'captured'; printf ' problem' >&2"},
  stdout = { file = "captured.log" },
  stderr = { file = "captured.err" },
})

local res3 = ptool.run_capture("echo hello", {
  stderr = "inherit",
})
print(res3.stdout)
```

## ptool.exec

> `v0.7.0` - Introduced.

`ptool.exec` replaces the current `ptool` process with an external command.

It supports the same command argument call forms as `ptool.run`:

```lua
ptool.exec("echo hello world")
ptool.exec("echo", "hello world")
ptool.exec("echo", {"hello", "world"})
ptool.exec("echo hello world", { echo = true })
ptool.exec("echo", {"hello", "world"}, { confirm = true })
ptool.exec({ cmd = "echo", args = {"hello", "world"} })
```

Behavior:

- On success, `ptool.exec` does not return. The current `ptool` process is
  replaced by the target command.
- Any Lua code after a successful `ptool.exec(...)` call does not run.
- In `ptool repl`, a successful `ptool.exec(...)` also replaces the REPL
  process.
- On Unix platforms, this uses true process replacement semantics.
- On non-Unix platforms, `ptool.exec` currently raises an `unsupported` error.

Argument rules:

- `ptool.exec(cmdline)`: `cmdline` is split using shell-style (`shlex`) rules.
  The first item is treated as the command and the rest as arguments.
- `ptool.exec(cmd, argsline)`: `cmd` is used directly as the command, and
  `argsline` is split into an argument list using shell-style (`shlex`) rules.
- `ptool.exec(cmd, args)`: `cmd` is a string and `args` is an array of strings.
- `ptool.exec(cmdline, options)`: `options` overrides settings for this
  invocation, such as `echo`.
- `ptool.exec(cmd, args, options)`: `args` can be either a string or an array
  of strings, and `options` overrides settings for this invocation, such as
  `confirm`.
- `ptool.exec(options)`: `options` is a table.
- When the second argument is a table: if it is an array (consecutive integer
  keys `1..n`), it is treated as `args`; otherwise it is treated as `options`.

`ptool.exec(options)` is also supported, where `options` is a table with the
following fields:

- `cmd` (string, required): The command name or executable path.
- `args` (string[], optional): The argument list.
- `cwd` (string, optional): The child process working directory.
- `env` (table, optional): Additional environment variables, where keys are
  variable names and values are variable values.
- `stdin` (table, optional): Child process stdin source.
  - `{ file = "path" }`: Read stdin from a file.
- `echo` (boolean, optional): Whether to echo command information before
  replacement. If omitted, the value from `ptool.config({ run = { echo = ... }
  })` is used; if that is also unset, the default is `true`.
- `confirm` (boolean, optional): Whether to ask the user for confirmation
  before replacement. If omitted, the value from `ptool.config({ run = {
  confirm = ... } })` is used; if that is also unset, the default is `false`.
- `stdout` (string|table, optional): Stdout handling strategy.
  - `"inherit"`: Inherit to the current terminal (default).
  - `"null"`: Discard the output.
  - `{ file = "path" }`: Write stdout to a file, truncating it first.
  - `{ file = "path", append = true }`: Append stdout to a file.
- `stderr` (string|table, optional): Stderr handling strategy.
  - `"inherit"`: Inherit to the current terminal (default).
  - `"null"`: Discard the output.
  - `{ file = "path" }`: Write stderr to a file, truncating it first.
  - `{ file = "path", append = true }`: Append stderr to a file.
- Unlike `ptool.run`, `ptool.exec` does not support string `stdin`,
  `stdout = "capture"`, `stderr = "capture"`, `trim`, `check`, or `retry`.
- File redirect paths are resolved relative to the effective child-process
  `cwd`, unless an absolute path is provided.
- When `confirm = true`:
  - If the user refuses the execution, an error is raised immediately.
  - If the current environment is not interactive (no TTY), an error is raised
    immediately.

Example:

```lua
ptool.config({ run = { echo = false } })

ptool.exec("echo from ptool")
ptool.exec("echo", "from ptool")
ptool.exec("echo", {"from", "ptool"})

ptool.exec({
  cmd = "sh",
  args = {"-c", "printf '%s|%s' \"$FOO\" \"$PWD\""},
  cwd = "crates",
  env = { FOO = "bar" },
})

ptool.exec({
  cmd = "sh",
  args = {"-c", "cat; printf ' done'; printf ' warn' >&2"},
  stdin = { file = "input.txt" },
  stdout = { file = "output.log" },
  stderr = { file = "error.log", append = true },
})
```
