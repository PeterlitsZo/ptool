# Process API

Local process helpers are available under `ptool.proc` and `p.proc`.

This module is for inspecting and managing already-running local processes.
Use `ptool.run(...)` when you want to launch a new command.

## ptool.proc.self

> `v0.8.0` - Introduced.

`ptool.proc.self()` returns a snapshot table for the current `ptool` process.

- Returns: `table`.

The returned table uses the same shape as `ptool.proc.get(...)` and
`ptool.proc.find(...)`.

## ptool.proc.get

> `v0.8.0` - Introduced.

`ptool.proc.get(pid)` returns a snapshot table for the given process ID, or
`nil` if the process does not exist.

- `pid` (integer, required): Process ID.
- Returns: `table|nil`.

## ptool.proc.exists

> `v0.8.0` - Introduced.

`ptool.proc.exists(pid)` reports whether a process ID currently exists.

- `pid` (integer, required): Process ID.
- Returns: `boolean`.

## ptool.proc.find

> `v0.8.0` - Introduced.

`ptool.proc.find([options])` lists local processes and returns an array of
snapshot tables.

- `options` (table, optional): Filter and sorting options.
- Returns: `table`.

Supported `options` fields:

- `pid` (integer, optional): Match one exact process ID.
- `pids` (integer[], optional): Match a set of process IDs.
- `ppid` (integer, optional): Match an exact parent process ID.
- `name` (string, optional): Match an exact process name.
- `name_contains` (string, optional): Match a substring in the process name.
- `exe` (string, optional): Match an exact executable path.
- `exe_contains` (string, optional): Match a substring in the executable path.
- `cmdline_contains` (string, optional): Match a substring in the joined command
  line.
- `user` (string, optional): Match an exact user name.
- `cwd` (string, optional): Match an exact current working directory.
- `include_self` (boolean, optional): Whether to include the current `ptool`
  process. Defaults to `false`.
- `limit` (integer, optional): Maximum number of returned entries after
  filtering and sorting.
- `sort_by` (string, optional): Sort key. Supported values:
  - `"pid"` (default)
  - `"start_time"`
- `reverse` (boolean, optional): Whether to reverse the final sort order.
  Defaults to `false`.

Each returned process snapshot may contain:

- `pid` (integer): Process ID.
- `ppid` (integer|nil): Parent process ID.
- `name` (string): Process name.
- `exe` (string|nil): Executable path, when available.
- `cwd` (string|nil): Current working directory, when available.
- `user` (string|nil): Owning user name, when available.
- `cmdline` (string|nil): Joined command line, when available.
- `argv` (string[]): Command-line argument array.
- `state` (string): Process state label such as `"running"` or `"sleeping"`.
- `start_time_unix_ms` (integer): Process start time in Unix milliseconds.

Notes:

- Some fields may be `nil` when the current platform or permission level does
  not expose them.
- Process snapshots are point-in-time values. They do not update themselves.

Example:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
  include_self = true,
  sort_by = "start_time",
})

for _, proc in ipairs(procs) do
  print(proc.pid, proc.name, proc.cmdline)
end
```

## ptool.proc.kill

> `v0.8.0` - Introduced.

`ptool.proc.kill(targets[, options])` sends a signal to one or more local
processes and returns a structured result table.

- `targets` (integer|table, required): A pid, a process snapshot table, or an
  array of them.
- `options` (table, optional): Kill options.
- Returns: `table`.

Supported `options` fields:

- `signal` (string, optional): Signal name. Supported values:
  - `"hup"`
  - `"term"` (default)
  - `"kill"`
  - `"int"`
  - `"quit"`
  - `"stop"`
  - `"cont"`
  - `"user1"`
  - `"user2"`
- `missing_ok` (boolean, optional): Whether missing processes count as success.
  Defaults to `true`.
- `allow_self` (boolean, optional): Whether the current `ptool` process may be
  signaled. Defaults to `false`.
- `check` (boolean, optional): Whether to raise an error immediately when the
  final result is not ok. Defaults to `false`.
- `confirm` (boolean, optional): Whether to ask for confirmation before sending
  the signal. Defaults to `false`.

The returned result table contains:

- `ok` (boolean): Whether the whole operation succeeded under the current
  options.
- `signal` (string): The signal label that was requested.
- `total` (integer): Total number of normalized targets.
- `sent` (integer): Number of targets for which the signal was sent.
- `missing` (integer): Number of targets that no longer existed.
- `failed` (integer): Number of targets that failed overall.
- `entries` (table): Per-target result entries.
- `assert_ok(self)` (function): Raises a structured Lua error when `ok = false`.

Each `entries[i]` table contains:

- `pid` (integer): Target process ID.
- `ok` (boolean): Whether this target succeeded.
- `existed` (boolean): Whether the target process existed and still matched.
- `signal` (string): The signal label that was requested.
- `message` (string|nil): Additional status detail.

Example:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
})

local res = p.proc.kill(procs, {
  signal = "term",
  confirm = true,
})

res:assert_ok()
```

## ptool.proc.wait_gone

> `v0.8.0` - Introduced.

`ptool.proc.wait_gone(targets[, options])` waits until one or more target
processes no longer exist, then returns a structured result table.

- `targets` (integer|table, required): A pid, a process snapshot table, or an
  array of them.
- `options` (table, optional): Wait options.
- Returns: `table`.

Supported `options` fields:

- `timeout_ms` (integer, optional): Maximum wait time in milliseconds. If
  omitted, this waits indefinitely.
- `interval_ms` (integer, optional): Polling interval in milliseconds. Defaults
  to `100`.
- `check` (boolean, optional): Whether to raise an error immediately when the
  wait times out. Defaults to `false`.

The returned result table contains:

- `ok` (boolean): Whether all target processes disappeared before timing out.
- `timed_out` (boolean): Whether the timeout was reached.
- `total` (integer): Total number of normalized targets.
- `gone` (integer[]): Process IDs that were gone by the end of the wait.
- `remaining` (integer[]): Process IDs still present when the wait finished.
- `elapsed_ms` (integer): Total elapsed wait time in milliseconds.
- `assert_ok(self)` (function): Raises a structured Lua error when `ok = false`.

Example:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
})

local wait_res = p.proc.wait_gone(procs, {
  timeout_ms = 1000,
  interval_ms = 100,
})

wait_res:assert_ok()
```
