# OS API

`ptool.os` は、現在のランタイム環境を読み取り、ホストプロセスに関する基本 情報を取得するためのヘルパーを公開します。

## ptool.os.getenv

> `v0.4.0` - Introduced.

`ptool.os.getenv(name)` は環境変数の現在値を返します。

- `name` (string, required): Environment variable name.
- `name` (string, 必須): 環境変数名。

Behavior:

- 戻り値: `string|nil`。
- Reads the current `ptool` runtime environment, including values changed by `ptool.os.setenv(...)` and `ptool.os.unsetenv(...)`.
- Raises an error when `name` is empty or contains invalid characters such as `=`.

Example:

```lua
local home = p.os.getenv("HOME")
print(home)
```

## ptool.os.env

> `v0.4.0` - Introduced.

`ptool.os.env()` は現在のランタイム環境のスナップショットテーブルを返します。

- 戻り値: `table`。

Behavior:

- The returned table maps variable names to string values.
- Values changed through `ptool.os.setenv(...)` and `ptool.os.unsetenv(...)` are reflected in the snapshot.

Example:

```lua
local env = p.os.env()
print(env.HOME)
```

## ptool.os.setenv

> `v0.4.0` - Introduced.

`ptool.os.setenv(name, value)` は現在の `ptool` ランタイムに環境変数を設定 します。

- `name` (string, required): Environment variable name.
- `value` (string, 必須): 環境変数の値。

Behavior:

- これは親シェルではなく、現在の `ptool` ランタイム環境を更新します。
- Values set here are visible to `ptool.os.getenv(...)`, `ptool.os.env()`, and child processes launched later through `ptool.run(...)`.
- ここで設定した値は `ptool.os.getenv(...)`、`ptool.os.env()`、 そして後から `ptool.run(...)` で起動した子プロセスから見えます。

Example:

```lua
p.os.setenv("APP_ENV", "dev")
print(p.os.getenv("APP_ENV"))
```

## ptool.os.unsetenv

> `v0.4.0` - Introduced.

`ptool.os.unsetenv(name)` は現在の `ptool` ランタイムから環境変数を削除 します。

- `name` (string, required): Environment variable name.

Behavior:

- This affects later calls to `ptool.os.getenv(...)`, `ptool.os.env()`, and child processes launched by `ptool.run(...)`.
- Raises an error when `name` is empty or contains invalid characters such as `=`.

Example:

```lua
p.os.unsetenv("APP_ENV")
assert(p.os.getenv("APP_ENV") == nil)
```

## ptool.os.homedir

> `v0.4.0` - Introduced.

`ptool.os.homedir()` は現在のユーザーのホームディレクトリを返します。

- `name` (string, 必須): 環境変数名。

Example:

```lua
local home = p.os.homedir()
```

## ptool.os.tmpdir

> `v0.4.0` - Introduced.

`ptool.os.tmpdir()` はシステムの一時ディレクトリを返します。

- 戻り値: `string`。

Example:

```lua
local tmp = p.os.tmpdir()
```

## ptool.os.hostname

> `v0.4.0` - Introduced.

`ptool.os.hostname()` は現在のホスト名を返します。

- `name` (string, 必須): 環境変数名。

## ptool.os.username

> `v0.4.0` - Introduced.

`ptool.os.username()` は現在のユーザー名を返します。

- `name` (string, 必須): 環境変数名。

## ptool.os.pid

> `v0.4.0` - Introduced.

`ptool.os.pid()` は現在の `ptool` プロセス ID を返します。

- 戻り値: `integer`。

## ptool.os.exepath

> `v0.4.0` - Introduced.

`ptool.os.exepath()` は実行中の `ptool` 実行ファイルの解決済みパスを返します。

- `name` (string, 必須): 環境変数名。

Example:

```lua
print(p.os.hostname(), p.os.username(), p.os.pid())
print(p.os.exepath())
```
