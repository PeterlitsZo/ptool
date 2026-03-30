# SSH API

SSH connection, remote execution, and file transfer helpers are available under `ptool.ssh` and `p.ssh`.

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
  - `conn:exists(path)` -> `boolean`
  - `conn:is_file(path)` -> `boolean`
  - `conn:is_dir(path)` -> `boolean`
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

> `v0.1.0` - Introduced.

`conn:path(path)` creates a reusable remote path value bound to the current SSH
connection.

- `path` (string, required): The remote path.
- Returns: A remote path userdata that can be passed to
  `conn:upload(...)`, `conn:download(...)`, and `ptool.fs.copy(...)`.
- The returned remote path also supports:
  - `remote:exists()` -> `boolean`
  - `remote:is_file()` -> `boolean`
  - `remote:is_dir()` -> `boolean`

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

ssh:download(remote_release, "./tmp/current.tar.gz")
```

## ptool.ssh.Connection:exists

> `v0.2.0` - Introduced.

`conn:exists(path)` checks whether a remote path exists.

- `path` (string|remote path, required): The remote path to check. It can be a
  string or a value created by `conn:path(...)`.
- Returns: `true` when the remote path exists, otherwise `false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

print(ssh:exists("/srv/app"))
print(ssh:path("/srv/app/releases/current.tar.gz"):exists())
```

## ptool.ssh.Connection:is_file

> `v0.2.0` - Introduced.

`conn:is_file(path)` checks whether a remote path exists and is a regular file.

- `path` (string|remote path, required): The remote path to check. It can be a
  string or a value created by `conn:path(...)`.
- Returns: `true` when the remote path is a file, otherwise `false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if ssh:is_file(remote_tarball) then
  print("release tarball exists")
end
```

## ptool.ssh.Connection:is_dir

> `v0.2.0` - Introduced.

`conn:is_dir(path)` checks whether a remote path exists and is a directory.

- `path` (string|remote path, required): The remote path to check. It can be a
  string or a value created by `conn:path(...)`.
- Returns: `true` when the remote path is a directory, otherwise `false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```

## ptool.ssh.Connection:upload

> `v0.1.0` - Introduced.

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

> `v0.1.0` - Introduced.

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
