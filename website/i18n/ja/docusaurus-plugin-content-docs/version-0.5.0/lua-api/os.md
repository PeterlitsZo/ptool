# OS API

`ptool.os` は、現在のランタイム環境を読み取り、ホストプロセスに関する基本
情報を取得するためのヘルパーを公開します。

## ptool.os.getenv

> `v0.4.0` - Introduced.

`ptool.os.getenv(name)` は環境変数の現在値を返します。

- `name` (string, 必須): 環境変数名。
- 戻り値: `string|nil`。

## ptool.os.env

> `v0.4.0` - Introduced.

`ptool.os.env()` は現在のランタイム環境のスナップショットテーブルを返します。

- 戻り値: `table`。

## ptool.os.setenv

> `v0.4.0` - Introduced.

`ptool.os.setenv(name, value)` は現在の `ptool` ランタイムに環境変数を設定
します。

- `name` (string, 必須): 環境変数名。
- `value` (string, 必須): 環境変数の値。

動作:

- これは親シェルではなく、現在の `ptool` ランタイム環境を更新します。
- ここで設定した値は `ptool.os.getenv(...)`、`ptool.os.env()`、
  そして後から `ptool.run(...)` で起動した子プロセスから見えます。

## ptool.os.unsetenv

> `v0.4.0` - Introduced.

`ptool.os.unsetenv(name)` は現在の `ptool` ランタイムから環境変数を削除
します。

- `name` (string, 必須): 環境変数名。

## ptool.os.homedir

> `v0.4.0` - Introduced.

`ptool.os.homedir()` は現在のユーザーのホームディレクトリを返します。

- 戻り値: `string|nil`。

## ptool.os.tmpdir

> `v0.4.0` - Introduced.

`ptool.os.tmpdir()` はシステムの一時ディレクトリを返します。

- 戻り値: `string`。

## ptool.os.hostname

> `v0.4.0` - Introduced.

`ptool.os.hostname()` は現在のホスト名を返します。

- 戻り値: `string|nil`。

## ptool.os.username

> `v0.4.0` - Introduced.

`ptool.os.username()` は現在のユーザー名を返します。

- 戻り値: `string|nil`。

## ptool.os.pid

> `v0.4.0` - Introduced.

`ptool.os.pid()` は現在の `ptool` プロセス ID を返します。

- 戻り値: `integer`。

## ptool.os.exepath

> `v0.4.0` - Introduced.

`ptool.os.exepath()` は実行中の `ptool` 実行ファイルの解決済みパスを返します。

- 戻り値: `string|nil`。
