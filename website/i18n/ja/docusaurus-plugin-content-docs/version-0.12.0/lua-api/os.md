# OS API

`ptool.os` は、現在のランタイム環境を読み取り、ホストプロセスの基本情報を取得するためのヘルパーを提供します。

## ptool.os.getenv

> `v0.4.0` - 導入。

`ptool.os.getenv(name)` は環境変数の現在値を返します。

- `name` (string, 必須): 環境変数名。
- 戻り値: `string|nil`。

挙動:

- 変数が設定されていない場合は `nil` を返します。
- `ptool.os.setenv(...)` および `ptool.os.unsetenv(...)` によって変更された値を含む、現在の `ptool` ランタイム環境を読み取ります。
- `name` が空の場合、または `=` などの無効な文字が含まれている場合は、エラーが発生します。

例:

```lua
local home = p.os.getenv("HOME")
print(home)
```

## ptool.os.env

> `v0.4.0` - 導入。

`ptool.os.env()` は現在のランタイム環境のスナップショットテーブルを返します。

- 戻り値: `table`。

挙動:

- 返されたテーブルは、変数名を文字列値にマップします。
- `ptool.os.setenv(...)` および `ptool.os.unsetenv(...)` によって変更された値はスナップショットに反映されます。

例:

```lua
local env = p.os.env()
print(env.HOME)
```

## ptool.os.setenv

> `v0.4.0` - 導入。

`ptool.os.setenv(name, value)` は現在の `ptool` ランタイムに環境変数を設定します。

- `name` (string, 必須): 環境変数名。
- `value` (string, 必須): 環境変数の値。

挙動:

- これは親シェルではなく、現在の `ptool` ランタイム環境を更新します。
- ここで設定した値は、`ptool.os.getenv(...)`、`ptool.os.env()`、および後で `ptool.run(...)` を通じて起動される子プロセスから参照できます。
- `name` が空、`=` を含む、または `value` に NUL が含まれる場合はエラーを発生させます。

例:

```lua
p.os.setenv("APP_ENV", "dev")
print(p.os.getenv("APP_ENV"))
```

## ptool.os.unsetenv

> `v0.4.0` - 導入。

`ptool.os.unsetenv(name)` は現在の `ptool` ランタイムから環境変数を削除します。

- `name` (string, 必須): 環境変数名。

挙動:

- これは、その後の `ptool.os.getenv(...)`、`ptool.os.env()`、および `ptool.run(...)` によって起動される子プロセスに影響します。
- `name` が空の場合、または `=` などの無効な文字が含まれている場合は、エラーが発生します。

例:

```lua
p.os.unsetenv("APP_ENV")
assert(p.os.getenv("APP_ENV") == nil)
```

## ptool.os.homedir

> `v0.4.0` - 導入。

`ptool.os.homedir()` は現在のユーザーのホームディレクトリを返します。

- 戻り値: `string|nil`。

例:

```lua
local home = p.os.homedir()
```

## ptool.os.tmpdir

> `v0.4.0` - 導入。

`ptool.os.tmpdir()` はシステムの一時ディレクトリを返します。

- 戻り値: `string`。

例:

```lua
local tmp = p.os.tmpdir()
```

## ptool.os.hostname

> `v0.4.0` - 導入。

`ptool.os.hostname()` は現在のホスト名を返します。

- 戻り値: `string|nil`。

## ptool.os.username

> `v0.4.0` - 導入。

`ptool.os.username()` は現在のユーザー名を返します。

- 戻り値: `string|nil`。

## ptool.os.pid

> `v0.4.0` - 導入。

`ptool.os.pid()` は現在の `ptool` プロセス ID を返します。

- 戻り値: `integer`。

## ptool.os.exepath

> `v0.4.0` - 導入。

`ptool.os.exepath()` は実行中の `ptool` 実行ファイルの解決済みパスを返します。

- 戻り値: `string|nil`。

例:

```lua
print(p.os.hostname(), p.os.username(), p.os.pid())
print(p.os.exepath())
```
