# SSH API

SSH 接続、リモート実行、ファイル転送のヘルパーは `ptool.ssh` と `p.ssh` で利用できます。

## ptool.ssh.connect

> `v0.1.0` - Introduced.

`ptool.ssh.connect(target_or_options)` は、システムの `ssh` コマンドを 利用する SSH 接続ハンドルを準備し、`Connection` オブジェクトを返します。

`ssh` は `PATH` 上で利用可能である必要があります。

引数:

- `target_or_options` (string|table, 必須):
  - 文字列を指定した場合は SSH ターゲットとして扱われます。
  - テーブルを指定した場合、現在サポートされるフィールドは次の通りです:
    - `target` (string, 任意): `"deploy@example.com"` や `"deploy@example.com:2222"` のような SSH ターゲット文字列。
    - `host` (string, 任意): ホスト名または IP アドレス。
    - `user` (string, 任意): SSH ユーザー名。デフォルトは `$USER`、`$USER` が利用できない場合は `"root"` です。
    - `port` (integer, 任意): SSH ポート。デフォルトは `22` です。
    - `auth` (table, 任意): 認証設定。
    - `host_key` (table, 任意): ホストキー検証設定。
    - `connect_timeout_ms` (integer, 任意): ミリ秒単位のタイムアウト。 デフォルトは `10000` です。
    - `keepalive_interval_ms` (integer, 任意): ミリ秒単位の keepalive 間隔。

サポートされるターゲット文字列の例:

```lua
local a = ptool.ssh.connect("deploy@example.com")
local b = ptool.ssh.connect("deploy@example.com:2222")
local c = ptool.ssh.connect("[2001:db8::10]:2222")
```

`auth` のフィールド:

- `private_key_file` (string, 任意): 秘密鍵ファイルへのパス。
- `private_key_passphrase` (string, 任意): 秘密鍵のパスフレーズ。 現時点では未サポートです。
- `password` (string, 任意): パスワードベース認証。現時点では未サポート です。

認証の挙動:

- `auth.private_key_file` を指定すると、`ptool` は `-i` でその鍵を渡して `ssh` を呼び出し、さらに `IdentitiesOnly=yes` も設定します。
- `auth.private_key_passphrase` または `auth.password` を指定すると、 この API はそれらの秘密情報をシステムの `ssh` コマンドへ渡さないため `ptool.ssh.connect(...)` は失敗します。
- それ以外の場合、認証はローカルの OpenSSH 設定に委譲されます。 これには `IdentityFile`、`ProxyJump`、`ProxyCommand`、`ssh-agent`、 証明書などの設定や仕組みが含まれます。
- 相対鍵パスは現在の `ptool` ランタイムディレクトリから解決されるため、 `ptool.cd(...)` に従います。
- 鍵パス内の `~` と `~/...` は展開されます。

`host_key` のフィールド:

- `verify` (string, 任意): ホストキー検証モード。サポートされる値:
  - `"known_hosts"`: `known_hosts` ファイルに対して検証します (デフォルト)。
  - `"ignore"`: ホストキー検証をスキップします。
- `known_hosts_file` (string, 任意): `known_hosts` ファイルへのパス。 `verify = "known_hosts"` のときだけ使用されます。

ホストキーの挙動:

- `verify = "ignore"` の場合、`ptool` は `StrictHostKeyChecking=no` と `UserKnownHostsFile=/dev/null` を付けて `ssh` を呼び出します。
- `verify = "known_hosts"` で `known_hosts_file` も指定されている場合、 `ptool` は `StrictHostKeyChecking=yes` とその `UserKnownHostsFile` を 付けて `ssh` を呼び出します。
- `verify = "known_hosts"` で `known_hosts_file` を省略した場合、または `host_key` 自体を省略した場合、ホストキー処理はローカル OpenSSH の 設定とデフォルト値へ委譲されます。
- 相対 `known_hosts_file` パスは現在の `ptool` ランタイムディレクトリ から解決されます。
- `known_hosts_file` 内の `~` と `~/...` は展開されます。
- `known_hosts_file` を明示的に指定した場合、この接続に対してローカルの `ssh` コマンドが使うデフォルトの `UserKnownHostsFile` を上書きします。

例:

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

`Connection` は `ptool.ssh.connect()` が返す、OpenSSH ベースの接続 ハンドルを表します。

これは Lua userdata として実装されています。

フィールドとメソッド:

- フィールド:
  - `conn.host` (string)
  - `conn.user` (string)
  - `conn.port` (integer)
  - `conn.target` (string)
- メソッド:
  - `conn:run(...)` -> `table`
  - `conn:run_capture(...)` -> `table`
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

`conn:run(...)` は現在の SSH 接続を通してリモートコマンドを実行します。

次の呼び出し形式をサポートします。

```lua
conn:run("hostname")
conn:run("echo", "hello world")
conn:run("echo", {"hello", "world"})
conn:run("hostname", { stdout = "capture" })
conn:run("echo", {"hello", "world"}, { stdout = "capture" })
conn:run({ cmd = "git", args = {"rev-parse", "HEAD"} })
```

引数ルール:

- `conn:run(cmdline)`: `cmdline` はリモートコマンド文字列として送られます。
- `conn:run(cmd, argsline)`: `cmd` はコマンドとして扱われ、`argsline` は シェル風 (`shlex`) ルールで分割されます。
- `conn:run(cmd, args)`: `cmd` は文字列で、`args` は文字列配列です。 引数はリモート実行前にシェルクォートされます。
- `conn:run(cmdline, options)`: `options` がこの呼び出しを上書きします。
- `conn:run(cmd, args, options)`: `options` がこの呼び出しを上書きします。
- `conn:run(options)`: `options` はテーブルです。
- 第 2 引数がテーブルの場合: 配列 (連続する整数キー `1..n`) なら `args`、 それ以外なら `options` として扱われます。

`conn:run(options)` を使う場合、`options` は現在次をサポートします。

- `cmd` (string, 必須): コマンド名または実行ファイルパス。
- `args` (string[], 任意): 引数リスト。
- `cwd` (string, 任意): リモート作業ディレクトリ。生成されるリモート シェルコマンドの先頭に `cd ... &&` を付けて適用します。
- `env` (table, 任意): リモート環境変数。キーと値は文字列です。生成される リモートシェルコマンドの先頭に `export ... &&` を付けて適用します。
- `stdin` (string, 任意): リモートプロセスの stdin に送る文字列。
- `trim` (boolean, optional): Whether to trim leading and trailing whitespace from captured `stdout` and captured `stderr` before returning them. This only affects streams set to `"capture"`. Defaults to `false`.
- `echo` (boolean, optional): Whether to echo the remote command before execution. Defaults to `true`.
- `check` (boolean, optional): Whether to raise an error immediately when the exit status is not `0`. Defaults to `false`.
- `stdout` (string, optional): Stdout handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stdout`.
  - `"null"`: Discard the output.
- `stderr` (string, optional): Stderr handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stderr`.
  - `"null"`: Discard the output.

ショートカット形式を使う場合、`options` テーブルがサポートするのは次だけ です。

- `stdin` (string, 任意): リモートプロセスの stdin に送る文字列。
- `trim` (boolean, optional): Whether to trim leading and trailing whitespace from captured `stdout` and captured `stderr` before returning them. This only affects streams set to `"capture"`. Defaults to `false`.
- `echo` (boolean, optional): Whether to echo the remote command before execution. Defaults to `true`.
- `check` (boolean, optional): Whether to raise an error immediately when the exit status is not `0`. Defaults to `false`.
- `stdout` (string, optional): Stdout handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stdout`.
  - `"null"`: Discard the output.
- `stderr` (string, optional): Stderr handling strategy. Supported values:
  - `"inherit"`: Inherit to the current terminal (default).
  - `"capture"`: Capture into `res.stderr`.
  - `"null"`: Discard the output.

戻り値ルール:

- 常に次のフィールドを持つテーブルが返ります:
  - `ok` (boolean): リモート終了ステータスが `0` かどうか。
  - `code` (integer|nil): リモート終了ステータス。リモートプロセスが シグナルで終了した場合は `nil` です。
  - `target` (string): `user@host:port` 形式の SSH ターゲット文字列。
  - `stdout` (string, 任意): `stdout = "capture"` のとき存在します。
  - `stderr` (string, 任意): `stderr = "capture"` のとき存在します。
  - `assert_ok(self)` (function): `ok = false` のときエラーを発生させます。

例:

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

`conn:run_capture(...)` は現在の SSH 接続を通してリモートコマンドを実行 します。

呼び出し形式、引数ルール、戻り値ルール、オプションは `conn:run(...)` と 同じです。

違いはデフォルトのストリーム処理だけです。

- `stdout` のデフォルトは `"capture"` です。
- `stderr` のデフォルトは `"capture"` です。

`trim` still defaults to `false`, and you can still override any of these fields explicitly in `options`.

例:

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

### path

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:path`.

`conn:path(path)` は現在の SSH 接続に紐づいた再利用可能な `RemotePath` オブジェクトを作成します。

- `path` (string, 必須): リモートパス。
- 戻り値: `conn:upload(...)`、`conn:download(...)`、`ptool.fs.copy(...)` に渡せる `RemotePath` オブジェクト。

例:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

ssh:download(remote_release, "./tmp/current.tar.gz")
```

### exists

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:exists`.

`conn:exists(path)` はリモートパスが存在するかどうかを確認します。

- `path` (string|remote path, required): The remote path to check. It can be a string or a value created by `conn:path(...)`.
- Returns: `true` when the remote path exists, otherwise `false`.

例:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

print(ssh:exists("/srv/app"))
print(ssh:path("/srv/app/releases/current.tar.gz"):exists())
```

### is_file

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:is_file`.

`conn:is_file(path)` はリモートパスが存在し、通常ファイルであるかどうかを 確認します。

- `path` (string|remote path, required): The remote path to check. It can be a string or a value created by `conn:path(...)`.
- Returns: `true` when the remote path is a file, otherwise `false`.

例:

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

`conn:is_dir(path)` はリモートパスが存在し、ディレクトリであるかどうかを 確認します。

- `path` (string|remote path, required): The remote path to check. It can be a string or a value created by `conn:path(...)`.
- Returns: `true` when the remote path is a directory, otherwise `false`.

例:

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

`conn:upload(local_path, remote_path[, options])` はローカルのファイルまたは ディレクトリをリモートホストへアップロードします。

- `local_path` (string, 必須): アップロードするローカルファイルまたは ディレクトリ。
- `remote_path` (string|remote path, 必須): リモートホスト上の宛先パス。 文字列でも `conn:path(...)` が作成した値でも構いません。
- `options` (table, optional): Transfer options.
- Returns: A table with the following fields:
  - `bytes` (integer): アップロードされた通常ファイルのバイト数。 ディレクトリをアップロードした場合は、アップロードされた各ファイル サイズの合計です。
  - `from` (string): ローカルの送信元パス。
  - `to` (string): リモートの宛先パス。

Supported transfer options:

- `parents` (boolean, 任意): アップロード前に `remote_path` の親 ディレクトリを作成します。デフォルトは `false` です。
- `overwrite` (boolean, optional): Whether an existing destination file may be replaced. Defaults to `true`.
- `echo` (boolean, optional): Whether to print the transfer before executing it. Defaults to `false`.

Directory behavior:

- `local_path` がファイルの場合、挙動は変わりません。
- `local_path` がディレクトリで `remote_path` が存在しない場合、 `remote_path` が宛先ディレクトリのルートになります。
- `local_path` がディレクトリで `remote_path` がすでにディレクトリとして 存在する場合、送信元ディレクトリはその配下に送信元 basename を使って 作成されます。
- `overwrite = false` rejects an already-existing destination directory for the final directory root.
- ディレクトリアップロードには、リモートホストで `tar` が利用できる必要 があります。

例:

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

`conn:download(remote_path, local_path[, options])` はリモートのファイルまたは ディレクトリをローカルパスへダウンロードします。

- `remote_path` (string|remote path, 必須): リモートホスト上の送信元パス。 文字列でも `conn:path(...)` が作成した値でも構いません。
- `local_path` (string, 必須): ローカルの宛先パス。
- `options` (table, optional): Transfer options.
- Returns: A table with the following fields:
  - `bytes` (integer): ダウンロードされた通常ファイルのバイト数。 ディレクトリをダウンロードした場合は、ダウンロードされた各ファイル サイズの合計です。
  - `from` (string): リモートの送信元パス。
  - `to` (string): ローカルの宛先パス。

Supported transfer options:

- `parents` (boolean, 任意): ダウンロード前に `local_path` の親 ディレクトリを作成します。デフォルトは `false` です。
- `overwrite` (boolean, optional): Whether an existing destination file may be replaced. Defaults to `true`.
- `echo` (boolean, optional): Whether to print the transfer before executing it. Defaults to `false`.

Directory behavior:

- `remote_path` がファイルの場合、挙動は変わりません。
- `remote_path` がディレクトリで `local_path` が存在しない場合、 `local_path` が宛先ディレクトリのルートになります。
- `remote_path` がディレクトリで `local_path` がすでにディレクトリとして 存在する場合、リモート送信元ディレクトリはその配下にリモート ディレクトリ basename を使って作成されます。
- `overwrite = false` rejects an already-existing destination directory for the final directory root.
- ディレクトリダウンロードには、リモートホストで `tar` が利用できる必要 があります。

例:

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

`conn:close()` は SSH 接続ハンドルを閉じます。

挙動:

- 閉じたあとは、その接続をもう使えません。
- すでに閉じられた接続を再度閉じても問題なく、効果はありません。

例:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
ssh:close()
```

## RemotePath

> `v0.1.0` - Introduced.

`RemotePath` は `Connection` に紐づくリモートパスを表し、 `conn:path(path)` から返されます。

これは Lua userdata として実装されています。

メソッド:

- `remote:exists()` -> `boolean`
- `remote:is_file()` -> `boolean`
- `remote:is_dir()` -> `boolean`

### exists

`remote:exists()` はリモートパスが存在するかどうかを確認します。

- Returns: `true` when the remote path exists, otherwise `false`.

例:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

print(remote_release:exists())
```

### is_file

`remote:is_file()` はリモートパスが存在し、通常ファイルであるかどうかを 確認します。

- Returns: `true` when the remote path is a file, otherwise `false`.

例:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if remote_tarball:is_file() then
  print("release tarball exists")
end
```

### is_dir

`remote:is_dir()` はリモートパスが存在し、ディレクトリであるかどうかを 確認します。

- Returns: `true` when the remote path is a directory, otherwise `false`.

例:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```
