# SSH API

SSH connection, remote execution, and file transfer helpers are available under `ptool.ssh` and `p.ssh`.

## ptool.ssh.connect

> `v0.1.0` - Introduced.

`ptool.ssh.connect(target_or_options)` prepares an SSH connection handle backed
by the system `ssh` command and returns a `Connection` object.

`ssh` must be available on `PATH`.

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
  This is currently not supported.
- `password` (string, optional): Password-based authentication. This is
  currently not supported.

Authentication behavior:

- If `auth.private_key_file` is provided, `ptool` invokes `ssh` with that key
  via `-i` and also sets `IdentitiesOnly=yes`.
- If `auth.private_key_passphrase` or `auth.password` is provided,
  `ptool.ssh.connect(...)` fails because this API does not pass those secrets
  to the system `ssh` command.
- Otherwise, authentication is delegated to the local OpenSSH setup, including
  settings and mechanisms such as `IdentityFile`, `ProxyJump`, `ProxyCommand`,
  `ssh-agent`, and certificates.
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

- If `verify = "ignore"`, `ptool` invokes `ssh` with
  `StrictHostKeyChecking=no` and `UserKnownHostsFile=/dev/null`.
- If `verify = "known_hosts"` and `known_hosts_file` is provided, `ptool`
  invokes `ssh` with `StrictHostKeyChecking=yes` and that
  `UserKnownHostsFile`.
- If `verify = "known_hosts"` and `known_hosts_file` is omitted, or when
  `host_key` is omitted entirely, host key handling is delegated to the local
  OpenSSH configuration and defaults.
- Relative `known_hosts_file` paths are resolved from the current `ptool`
  runtime directory.
- `~` and `~/...` are expanded in `known_hosts_file`.
- When `known_hosts_file` is provided explicitly, it overrides the default
  `UserKnownHostsFile` used by the local `ssh` command for this connection.

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

## Connection

> `v0.1.0` - Introduced.

`Connection` represents an OpenSSH-backed connection handle returned by
`ptool.ssh.connect()`.

It is implemented as a Lua userdata.

Fields and methods:

- Fields:
  - `conn.host` (string)
  - `conn.user` (string)
  - `conn.port` (integer)
  - `conn.target` (string)
- Methods:
  - `conn:run(...)` -> `table`
  - `conn:run_capture(...)` -> `table`
  - `conn:http_request(options)` -> `Response`
  - `conn:path(path)` -> `RemotePath`
  - `conn:exists(path)` -> `boolean`
  - `conn:is_file(path)` -> `boolean`
  - `conn:is_dir(path)` -> `boolean`
  - `conn:upload(local_path, remote_path[, options])` -> `table`
  - `conn:download(remote_path, local_path[, options])` -> `table`
  - `conn:close()` -> `nil`

### run

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:run`.

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
- `trim` (boolean, optional): Whether to trim leading and trailing whitespace
  from captured `stdout` and captured `stderr` before returning them. This only
  affects streams set to `"capture"`. Defaults to `false`.
- `echo` (boolean, optional): Whether to echo the remote command before
  execution. Defaults to `true`.
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
- `trim` (boolean, optional): Whether to trim leading and trailing whitespace
  from captured `stdout` and captured `stderr` before returning them. This only
  affects streams set to `"capture"`. Defaults to `false`.
- `echo` (boolean, optional): Whether to echo the remote command before
  execution. Defaults to `true`.
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
  trim = true,
  stdout = "capture",
  check = true,
})

print(res2.stdout)
```

### run_capture

> `Unreleased` - Introduced.

Canonical API name: `ptool.ssh.Connection:run_capture`.

`conn:run_capture(...)` executes a remote command through the current SSH
connection.

It accepts the same call forms, argument rules, return value rules, and options
as `conn:run(...)`.

The difference is only the default stream handling:

- `stdout` defaults to `"capture"`.
- `stderr` defaults to `"capture"`.

`trim` still defaults to `false`, and you can still override any of these
fields explicitly in `options`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:run_capture("uname -a", { trim = true })
print(res.stdout)

local res2 = ssh:run_capture({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2"},
  cwd = "/srv/app",
})
print(res2.stdout)
print(res2.stderr)

local res3 = ssh:run_capture("echo hello", {
  stderr = "inherit",
})
print(res3.stdout)
```

### http_request

> `Unreleased` - Introduced.

Canonical API name: `ptool.ssh.Connection:http_request`.

`conn:http_request(options)` sends an HTTP request from the remote SSH host and
returns the same `Response` object shape as `ptool.http.request(...)`.

`options` supports the same fields and validation rules as
`ptool.http.request(options)`.

This is useful when the target endpoint is reachable only from the remote host,
for example a service bound to `127.0.0.1`, a private VPC address, or a
metadata endpoint.

Notes:

- The request is executed on the remote host, so DNS resolution, outbound
  network access, proxy settings, TLS trust, and firewall rules come from that
  host rather than the local machine.
- The remote host must have `curl` available on `PATH`.
- Request bodies are sent to the remote `curl` process over SSH.
- Response headers and body are streamed back over SSH and then consumed
  through the normal `Response` methods documented in the HTTP API.
- `basic_auth` and `bearer_token` remain mutually exclusive.
- `fail_on_http_error`, redirect handling, timeout handling, and response body
  caching behave the same as `ptool.http.request(...)`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local resp = ssh:http_request({
  url = "http://127.0.0.1:8080/health",
  headers = {
    accept = "application/json",
  },
  timeout_ms = 5000,
  fail_on_http_error = true,
})

local data = resp:json()
print(resp.status)
print(data.status)
```

### path

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:path`.

`conn:path(path)` creates a reusable `RemotePath` object bound to the current
SSH connection.

- `path` (string, required): The remote path.
- Returns: A `RemotePath` object that can be passed to
  `conn:upload(...)`, `conn:download(...)`, and `ptool.fs.copy(...)`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

ssh:download(remote_release, "./tmp/current.tar.gz")
```

### exists

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:exists`.

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

### is_file

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:is_file`.

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

### is_dir

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:is_dir`.

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

### upload

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:upload`.

`conn:upload(local_path, remote_path[, options])` uploads a local file or
directory to the remote host.

- `local_path` (string, required): The local file or directory to upload.
- `remote_path` (string|remote path, required): The destination path on the
  remote host. It can be a string or a value created by `conn:path(...)`.
- `options` (table, optional): Transfer options.
- Returns: A table with the following fields:
  - `bytes` (integer): The number of regular-file bytes uploaded. When a
    directory is uploaded, this is the sum of the uploaded file sizes.
  - `from` (string): The local source path.
  - `to` (string): The remote destination path.

Supported transfer options:

- `parents` (boolean, optional): Create the parent directory of `remote_path`
  before uploading. Defaults to `false`.
- `overwrite` (boolean, optional): Whether an existing destination file may be
  replaced. Defaults to `true`.
- `echo` (boolean, optional): Whether to print the transfer before executing
  it. Defaults to `false`.

Directory behavior:

- When `local_path` is a file, the behavior is unchanged.
- When `local_path` is a directory and `remote_path` does not exist,
  `remote_path` becomes the destination directory root.
- When `local_path` is a directory and `remote_path` already exists as a
  directory, the source directory is created under it using the source
  directory basename.
- `overwrite = false` rejects an already-existing destination directory for the
  final directory root.
- Directory uploads require `tar` to be available on the remote host.

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

Directory example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:upload("./dist/assets", "/srv/app/releases", {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.to) -- deploy@example.com:22:/srv/app/releases
```

### download

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:download`.

`conn:download(remote_path, local_path[, options])` downloads a remote file or
directory to a local path.

- `remote_path` (string|remote path, required): The source path on the remote
  host. It can be a string or a value created by `conn:path(...)`.
- `local_path` (string, required): The local destination path.
- `options` (table, optional): Transfer options.
- Returns: A table with the following fields:
  - `bytes` (integer): The number of regular-file bytes downloaded. When a
    directory is downloaded, this is the sum of the downloaded file sizes.
  - `from` (string): The remote source path.
  - `to` (string): The local destination path.

Supported transfer options:

- `parents` (boolean, optional): Create the parent directory of `local_path`
  before downloading. Defaults to `false`.
- `overwrite` (boolean, optional): Whether an existing destination file may be
  replaced. Defaults to `true`.
- `echo` (boolean, optional): Whether to print the transfer before executing
  it. Defaults to `false`.

Directory behavior:

- When `remote_path` is a file, the behavior is unchanged.
- When `remote_path` is a directory and `local_path` does not exist,
  `local_path` becomes the destination directory root.
- When `remote_path` is a directory and `local_path` already exists as a
  directory, the remote source directory is created under it using the remote
  directory basename.
- `overwrite = false` rejects an already-existing destination directory for the
  final directory root.
- Directory downloads require `tar` to be available on the remote host.

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

Directory example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:download("/srv/app/releases/assets", "./tmp/releases", {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.from)
```

### close

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:close`.

`conn:close()` closes the SSH connection handle.

Behavior:

- After closing, the connection can no longer be used.
- Closing an already-closed connection is allowed and has no effect.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
ssh:close()
```

## RemotePath

> `v0.1.0` - Introduced.

`RemotePath` represents a remote path bound to a `Connection` and returned by
`conn:path(path)`.

It is implemented as a Lua userdata.

Methods:

- `remote:exists()` -> `boolean`
- `remote:is_file()` -> `boolean`
- `remote:is_dir()` -> `boolean`

### exists

`remote:exists()` checks whether the remote path exists.

- Returns: `true` when the remote path exists, otherwise `false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

print(remote_release:exists())
```

### is_file

`remote:is_file()` checks whether the remote path exists and is a regular file.

- Returns: `true` when the remote path is a file, otherwise `false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if remote_tarball:is_file() then
  print("release tarball exists")
end
```

### is_dir

`remote:is_dir()` checks whether the remote path exists and is a directory.

- Returns: `true` when the remote path is a directory, otherwise `false`.

Example:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```
