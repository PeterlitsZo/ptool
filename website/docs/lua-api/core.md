# Core Lua API

`ptool` exposes these core runtime helpers directly under `ptool` and `p`.

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

## ptool.run_capture

> `Unreleased` - Introduced.

`ptool.run_capture` executes external commands from Rust with the same call
forms, argument rules, return value rules, and options as `ptool.run`.

The difference is only the default stream handling:

- `stdout` defaults to `"capture"`.
- `stderr` defaults to `"capture"`.

You can still override either field explicitly in `options`.

Example:

```lua
local res = ptool.run_capture("echo hello world")
print(res.stdout)

local res2 = ptool.run_capture({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2"},
})
print(res2.stdout)
print(res2.stderr)

local res3 = ptool.run_capture("echo hello", {
  stderr = "inherit",
})
print(res3.stdout)
```
