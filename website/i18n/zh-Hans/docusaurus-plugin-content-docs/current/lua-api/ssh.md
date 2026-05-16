# SSH API

SSH 连接、远程执行和文件传输辅助能力位于 `ptool.ssh` 和 `p.ssh` 下。

## ptool.ssh.connect

> `v0.1.0` - 引入。

`ptool.ssh.connect(target_or_options)` 会准备一个由系统 `ssh` 命令驱动的 SSH 连接句柄，并返回一个 `Connection` 对象。

运行环境中必须能在 `PATH` 中找到 `ssh`。

参数：

- `target_or_options`（string|table，必填）：
  - 传入字符串时，会被视为 SSH target。
  - 传入 table 时，目前支持：
    - `target`（string，可选）：SSH target 字符串，例如 `"deploy@example.com"` 或 `"deploy@example.com:2222"`。
    - `host`（string，可选）：主机名或 IP 地址。
    - `user`（string，可选）：SSH 用户名。默认取 `$USER`；如果 `$USER` 不可用，则默认为 `"root"`。
    - `port`（integer，可选）：SSH 端口。默认值为 `22`。
    - `auth`（table，可选）：认证设置。
    - `host_key`（table，可选）：主机密钥校验设置。
    - `connect_timeout_ms`（integer，可选）：超时时间（毫秒）。默认值为 `10000`。
    - `keepalive_interval_ms`（integer，可选）：keepalive 间隔（毫秒）。

支持的 target 字符串示例：

```lua
local a = ptool.ssh.connect("deploy@example.com")
local b = ptool.ssh.connect("deploy@example.com:2222")
local c = ptool.ssh.connect("[2001:db8::10]:2222")
```

`auth` 字段：

- `private_key_file`（string，可选）：私钥文件路径。
- `private_key_passphrase`（string，可选）：私钥口令。目前不支持。
- `password`（string，可选）：密码认证所用的密码。目前不支持。

认证行为：

- 如果提供了 `auth.private_key_file`，`ptool` 会通过 `-i` 将该私钥传给 `ssh`，并同时设置 `IdentitiesOnly=yes`。
- 如果提供了 `auth.private_key_passphrase` 或 `auth.password`， `ptool.ssh.connect(...)` 会失败，因为该 API 不会把这些密钥材料直接传给 系统 `ssh` 命令。
- 否则，认证完全交给本机 OpenSSH 配置处理，包括 `IdentityFile`、 `ProxyJump`、`ProxyCommand`、`ssh-agent` 和证书等机制。
- 相对私钥路径会从当前 `ptool` 运行时目录解析，因此会受到 `ptool.cd(...)` 的影响。
- 私钥路径中的 `~` 和 `~/...` 会被展开。

`host_key` 字段：

- `verify`（string，可选）：主机密钥校验模式。支持：
  - `"known_hosts"`：根据 known_hosts 文件校验（默认）。
  - `"ignore"`：跳过主机密钥校验。
- `known_hosts_file`（string，可选）：known_hosts 文件路径。仅在 `verify = "known_hosts"` 时使用。

主机密钥行为：

- 如果 `verify = "ignore"`，`ptool` 会给 `ssh` 传入 `StrictHostKeyChecking=no` 和 `UserKnownHostsFile=/dev/null`。
- 如果 `verify = "known_hosts"` 且提供了 `known_hosts_file`，`ptool` 会给 `ssh` 传入 `StrictHostKeyChecking=yes`，并设置对应的 `UserKnownHostsFile`。
- 如果 `verify = "known_hosts"` 但省略了 `known_hosts_file`，或者整个 `host_key` 都被省略，则主机密钥处理交给本机 OpenSSH 配置和默认行为。
- 相对 `known_hosts_file` 路径会从当前 `ptool` 运行时目录解析。
- `known_hosts_file` 中的 `~` 和 `~/...` 会被展开。
- 当显式提供 `known_hosts_file` 时，它会覆盖本次连接中本机 `ssh` 命令默认 使用的 `UserKnownHostsFile`。

示例：

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

> `v0.1.0` - 引入。

`Connection` 表示由 `ptool.ssh.connect()` 返回的、基于 OpenSSH 的连接句柄。

它实现为 Lua 用户数据（userdata）。

字段和方法：

- 字段：
  - `conn.host`（string）
  - `conn.user`（string）
  - `conn.port`（integer）
  - `conn.target`（string）
- 方法：
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

> `v0.1.0` - 引入。

规范 API 名称：`ptool.ssh.Connection:run`。

`conn:run(...)` 通过当前 SSH 连接执行远程命令。

支持以下调用形式：

```lua
conn:run("hostname")
conn:run("echo", "hello world")
conn:run("echo", {"hello", "world"})
conn:run("hostname", { stdout = "capture" })
conn:run("echo", {"hello", "world"}, { stdout = "capture" })
conn:run({ cmd = "git", args = {"rev-parse", "HEAD"} })
```

参数规则：

- `conn:run(cmdline)`：`cmdline` 会作为远程命令字符串直接发送。
- `conn:run(cmd, argsline)`：`cmd` 会被视为命令，`argsline` 会按 shell 风格 （`shlex`）规则拆分。
- `conn:run(cmd, args)`：`cmd` 是字符串，`args` 是字符串数组。参数会在远程执行前 做 shell quoting。
- `conn:run(cmdline, options)`：`options` 会覆盖本次调用的行为。
- `conn:run(cmd, args, options)`：`options` 会覆盖本次调用的行为。
- `conn:run(options)`：`options` 是一个 table。
- 当第二个参数是 table 时：如果它是数组（连续整数键 `1..n`），则视为 `args`； 否则视为 `options`。

使用 `conn:run(options)` 时，`options` 目前支持：

- `cmd`（string，必填）：命令名或可执行文件路径。
- `args`（string[]，可选）：参数列表。
- `cwd`（string，可选）：远程工作目录。会通过在生成的远程 shell 命令前追加 `cd ... &&` 实现。
- `env`（table，可选）：远程环境变量，键和值都必须是字符串。会通过在生成的 远程 shell 命令前追加 `export ... &&` 实现。
- `stdin`（string，可选）：发送给远程进程 stdin 的字符串。
- `trim`（boolean，可选）：返回前是否裁掉已捕获 `stdout` 和已捕获 `stderr` 两端的空白字符。只影响设置为 `"capture"` 的流。默认值为 `false`。
- `echo`（boolean，可选）：执行前是否回显远程命令。默认值为 `true`。
- `check`（boolean，可选）：退出状态不为 `0` 时是否立即抛错。默认值为 `false`。
- `stdout`（string，可选）：stdout 处理策略。支持：
  - `"inherit"`：继承到当前终端（默认）。
  - `"capture"`：捕获到 `res.stdout`。
  - `"null"`：丢弃输出。
- `stderr`（string，可选）：stderr 处理策略。支持：
  - `"inherit"`：继承到当前终端（默认）。
  - `"capture"`：捕获到 `res.stderr`。
  - `"null"`：丢弃输出。

使用快捷调用形式时，`options` 仅支持：

- `stdin`（string，可选）：发送给远程进程 stdin 的字符串。
- `trim`（boolean，可选）：返回前是否裁掉已捕获 `stdout` 和已捕获 `stderr` 两端的空白字符。只影响设置为 `"capture"` 的流。默认值为 `false`。
- `echo`（boolean，可选）：执行前是否回显远程命令。默认值为 `true`。
- `check`（boolean，可选）：退出状态不为 `0` 时是否立即抛错。默认值为 `false`。
- `stdout`（string，可选）：stdout 处理策略。支持：
  - `"inherit"`：继承到当前终端（默认）。
  - `"capture"`：捕获到 `res.stdout`。
  - `"null"`：丢弃输出。
- `stderr`（string，可选）：stderr 处理策略。支持：
  - `"inherit"`：继承到当前终端（默认）。
  - `"capture"`：捕获到 `res.stderr`。
  - `"null"`：丢弃输出。

返回值规则：

- 总是返回一个 table，包含以下字段：
  - `ok`（boolean）：远程退出状态是否为 `0`。
  - `code`（integer|nil）：远程退出状态。如果远程进程因信号退出，则为 `nil`。
  - `target`（string）：形如 `user@host:port` 的 SSH target 字符串。
  - `stdout`（string，可选）：当 `stdout = "capture"` 时提供。
  - `stderr`（string，可选）：当 `stderr = "capture"` 时提供。
  - `assert_ok(self)`（function）：当 `ok = false` 时抛出错误。

示例：

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

> `Unreleased` - 引入。

规范 API 名称：`ptool.ssh.Connection:run_capture`。

`conn:run_capture(...)` 通过当前 SSH 连接执行远程命令。

它接受与 `conn:run(...)` 相同的调用形式、参数规则、返回值规则和选项。

唯一差异是默认流处理方式：

- `stdout` 默认是 `"capture"`。
- `stderr` 默认是 `"capture"`。

`trim` 仍然默认是 `false`，并且这些字段都仍然可以在 `options` 中显式覆盖。

示例：

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

> `Unreleased` - 引入。

规范 API 名称：`ptool.ssh.Connection:http_request`。

`conn:http_request(options)` 从远程 SSH 主机发出 HTTP 请求，并返回与 `ptool.http.request(...)` 相同形状的 `Response` 对象。

`options` 支持与 `ptool.http.request(options)` 相同的字段和校验规则。

当目标端点只能从远程主机访问时，这很有用，例如绑定到 `127.0.0.1` 的服务、私有 VPC 地址或元数据端点。

说明：

- 请求是在远程主机上执行的，因此 DNS 解析、出站网络访问、代理设置、TLS 信任链和防火墙规则都来自该主机，而不是本地机器。
- 远程主机必须能在 `PATH` 中找到 `curl`。
- 请求体会通过 SSH 发送给远程 `curl` 进程。
- 响应头和响应体会通过 SSH 流式传回，然后通过 HTTP API 文档中说明的常规 `Response` 方法来读取。
- `basic_auth` 和 `bearer_token` 仍然互斥。
- `fail_on_http_error`、重定向处理、超时处理和响应体缓存的行为，与 `ptool.http.request(...)` 保持一致。

示例：

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

> `v0.1.0` - 引入。

规范 API 名称：`ptool.ssh.Connection:path`。

`conn:path(path)` 创建一个绑定到当前 SSH 连接的可复用 `RemotePath` 对象。

- `path`（string，必填）：远程路径。
- 返回：一个 `RemotePath` 对象，可传给 `conn:upload(...)`、`conn:download(...)` 和 `ptool.fs.copy(...)`。

示例：

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

ssh:download(remote_release, "./tmp/current.tar.gz")
```

### exists

> `v0.2.0` - 引入。

规范 API 名称：`ptool.ssh.Connection:exists`。

`conn:exists(path)` 检查远程路径是否存在。

- `path`（string|remote path，必填）：要检查的远程路径。可以是字符串，也可以是 `conn:path(...)` 创建的值。
- 返回：如果远程路径存在，则为 `true`；否则为 `false`。

示例：

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

print(ssh:exists("/srv/app"))
print(ssh:path("/srv/app/releases/current.tar.gz"):exists())
```

### is_file

> `v0.2.0` - 引入。

规范 API 名称：`ptool.ssh.Connection:is_file`。

`conn:is_file(path)` 检查远程路径是否存在且为普通文件。

- `path`（string|remote path，必填）：要检查的远程路径。可以是字符串，也可以是 `conn:path(...)` 创建的值。
- 返回：如果远程路径是文件，则为 `true`；否则为 `false`。

示例：

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if ssh:is_file(remote_tarball) then
  print("release tarball exists")
end
```

### is_dir

> `v0.2.0` - 引入。

规范 API 名称：`ptool.ssh.Connection:is_dir`。

`conn:is_dir(path)` 检查远程路径是否存在且为目录。

- `path`（string|remote path，必填）：要检查的远程路径。可以是字符串，也可以是 `conn:path(...)` 创建的值。
- 返回：如果远程路径是目录，则为 `true`；否则为 `false`。

示例：

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```

### upload

> `v0.1.0` - 引入。

规范 API 名称：`ptool.ssh.Connection:upload`。

`conn:upload(local_path, remote_path[, options])` 将本地文件或目录上传到远程主机。

- `local_path`（string，必填）：要上传的本地文件或目录。
- `remote_path`（string|remote path，必填）：远程目标路径。可以是字符串，也可以是 `conn:path(...)` 创建的值。
- `options`（table，可选）：传输选项。
- 返回：一个包含以下字段的 table。
  - `bytes`（integer）：已上传的普通文件字节数。上传目录时，它等于目录内所有已上传 文件大小之和。
  - `from`（string）：本地源路径。
  - `to`（string）：远程目标路径。

支持的传输选项：

- `parents`（boolean，可选）：上传前是否创建 `remote_path` 的父目录。默认值为 `false`。
- `overwrite`（boolean，可选）：是否允许覆盖已有目标文件。默认值为 `true`。
- `echo`（boolean，可选）：执行前是否打印传输信息。默认值为 `false`。

目标路径行为：

- 当 `local_path` 是文件且 `remote_path` 是文件路径时，文件会上传到该精确路径。
- 当 `local_path` 是文件且 `remote_path` 已存在并且是目录时，文件会在该目录下按本地文件的 basename 上传。
- 当 `local_path` 是文件且 `remote_path` 以 `/` 结尾时，`remote_path` 会被视为目标目录路径，上传后的文件会保留本地文件的 basename。如果该目录尚不存在，可通过 `parents = true` 创建。
- 当 `local_path` 是目录且 `remote_path` 不存在时，`remote_path` 会成为目标目录根。
- 当 `local_path` 是目录且 `remote_path` 已存在并且是目录时，会在其下按源目录的 basename 创建目标目录。
- `overwrite = false` 时，如果最终目标目录已存在，则会报错。
- 上传目录时，远程主机需要提供 `tar`。

示例：

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

目录示例：

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

> `v0.1.0` - 引入。

规范 API 名称：`ptool.ssh.Connection:download`。

`conn:download(remote_path, local_path[, options])` 将远程文件或目录下载到本地路径。

- `remote_path`（string|remote path，必填）：远程源路径。可以是字符串，也可以是 `conn:path(...)` 创建的值。
- `local_path`（string，必填）：本地目标路径。
- `options`（table，可选）：传输选项。
- 返回：一个包含以下字段的 table。
  - `bytes`（integer）：已下载的普通文件字节数。下载目录时，它等于目录内所有已下载 文件大小之和。
  - `from`（string）：远程源路径。
  - `to`（string）：本地目标路径。

支持的传输选项：

- `parents`（boolean，可选）：下载前是否创建 `local_path` 的父目录。默认值为 `false`。
- `overwrite`（boolean，可选）：是否允许覆盖已有目标文件。默认值为 `true`。
- `echo`（boolean，可选）：执行前是否打印传输信息。默认值为 `false`。

目录行为：

- 当 `remote_path` 是文件时，行为保持不变。
- 当 `remote_path` 是目录且 `local_path` 不存在时，`local_path` 会成为目标目录根。
- 当 `remote_path` 是目录且 `local_path` 已存在并且是目录时，会在其下按远程源目录的 basename 创建目标目录。
- `overwrite = false` 时，如果最终目标目录已存在，则会报错。
- 下载目录时，远程主机需要提供 `tar`。

示例：

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

目录示例：

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

> `v0.1.0` - 引入。

规范 API 名称：`ptool.ssh.Connection:close`。

`conn:close()` 关闭 SSH 连接句柄。

行为说明：

- 关闭后，连接不能再继续使用。
- 对已经关闭的连接再次关闭是允许的，并且不会产生效果。

示例：

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
ssh:close()
```

## RemotePath

> `v0.1.0` - 引入。

`RemotePath` 表示一个绑定到 `Connection` 的远程路径，由 `conn:path(path)` 返回。

它实现为 Lua 用户数据（userdata）。

方法：

- `remote:exists()` -> `boolean`
- `remote:is_file()` -> `boolean`
- `remote:is_dir()` -> `boolean`

### exists

`remote:exists()` 检查远程路径是否存在。

- 返回：如果远程路径存在，则为 `true`；否则为 `false`。

示例：

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

print(remote_release:exists())
```

### is_file

`remote:is_file()` 检查远程路径是否存在且为普通文件。

- 返回：如果远程路径是文件，则为 `true`；否则为 `false`。

示例：

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if remote_tarball:is_file() then
  print("release tarball exists")
end
```

### is_dir

`remote:is_dir()` 检查远程路径是否存在且为目录。

- 返回：如果远程路径是目录，则为 `true`；否则为 `false`。

示例：

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```
