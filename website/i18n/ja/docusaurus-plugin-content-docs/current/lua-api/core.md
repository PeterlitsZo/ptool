# コア Lua API

`ptool` は、これらのコアランタイムヘルパーを `ptool` と `p` の直下に
公開します。

`ptool run <lua_file>` は Lua スクリプトを実行し、グローバル変数
`ptool` (またはその別名 `p`; たとえば `p.run` は `ptool.run` と同等)
を注入します。

Lua スクリプトへ引数を渡したい場合は、次のようにできます。

```sh
ptool run script.lua --name alice -v a.txt b.txt
```

その引数は `ptool.args.parse(...)` で解析できます。

スクリプト例:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

shebang もサポートしているため、ファイルの先頭に次を追加できます。

```
#!/usr/bin/env ptool run
```

## ptool.use

> `v0.1.0` - Introduced.

`ptool.use` は、スクリプトに必要な最小 `ptool` バージョンを宣言します。

```lua
ptool.use("v0.1.0")
```

- 引数はセマンティックバージョン文字列 (SemVer) で、`v0.1.0` や
  `0.1.0` のように先頭の `v` は省略可能です。
- 要求バージョンが現在の `ptool` バージョンより高い場合、そのスクリプトは
  現在の `ptool` が古すぎるというエラーを出して即座に終了します。

## ptool.unindent

> `v0.1.0` - Introduced.

`ptool.unindent` は複数行文字列を処理し、各行の先頭インデントのあとにある
`| ` プレフィックスを除去し、先頭と末尾の空行を取り除きます。

```lua
local str = ptool.unindent([[
  | line 1
  | line 2
]])
```

これは次と等価です。

```lua
local str = [[line 1
line 2]]
```

## ptool.inspect

> `v0.1.0` - Introduced.

`ptool.inspect(value[, options])` は Lua 値を読みやすい Lua 風の文字列へ
レンダリングします。主な用途はデバッグとテーブル内容の表示です。

- `value` (any, 必須): inspect する Lua 値。
- `options` (table, 任意): レンダリングオプション。サポートされる
  フィールド:
  - `indent` (string, 任意): ネストごとに使うインデント。デフォルトは
    2 つのスペースです。
  - `multiline` (boolean, 任意): テーブルを複数行で描画するかどうか。
    デフォルトは `true`。
  - `max_depth` (integer, 任意): 描画する最大ネスト深さ。それより深い値は
    `<max-depth>` に置き換えられます。
- 戻り値: `string`。

挙動:

- 配列風の要素 (`1..n`) が最初に描画されます。
- 残りのテーブルフィールドは、そのあとに安定したキー順で描画されます。
- 識別子風の文字列キーは `key = value`、それ以外のキーは
  `[key] = value` の形式で描画されます。
- 再帰的なテーブル参照は `<cycle>` として描画されます。
- 関数、thread、userdata は `<function>` や `<userdata>` のような
  プレースホルダー値で描画されます。

例:

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

`ptool.ask(prompt[, options])` はユーザーに 1 行のテキスト入力を求め、
その回答を返します。

- `prompt` (string, 必須): ユーザーに表示するプロンプト。
- `options` (table, 任意): プロンプトオプション。サポートされる
  フィールド:
  - `default` (string, 任意): ユーザーが空の回答を送信したときに使う
    デフォルト値。
  - `help` (string, 任意): プロンプトの下に表示する補助テキスト。
  - `placeholder` (string, 任意): ユーザーが入力を始める前に表示する
    プレースホルダー。
- 戻り値: `string`。

挙動:

- 対話型 TTY が必要です。非対話環境で実行するとエラーになります。
- ユーザーがプロンプトをキャンセルすると、スクリプトはエラーになります。
- 未知のオプション名や不正な値型はエラーになります。

例:

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

`ptool.config` はスクリプトのランタイム設定を行います。

現在サポートされているフィールド:

- `run` (table, 任意): `ptool.run` のデフォルト設定。サポートされる
  フィールド:
  - `echo` (boolean, 任意): デフォルトの echo スイッチ。デフォルトは
    `true`。
  - `check` (boolean, 任意): 失敗時にデフォルトでエラーにするかどうか。
    デフォルトは `false`。
  - `confirm` (boolean, 任意): 実行前にデフォルトで確認を要求するかどうか。
    デフォルトは `false`。
  - `retry` (boolean, 任意): `check = true` のとき、実行失敗後に再試行するか
    どうかをユーザーへ尋ねるかどうか。デフォルトは `false`。

例:

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

`ptool.cd(path)` は `ptool` のランタイムカレントディレクトリを更新します。

- `path` (string, 必須): 対象ディレクトリパス。絶対でも相対でも構いません。

挙動:

- 相対パスは現在の `ptool` ランタイムディレクトリから解決されます。
- 対象は存在していて、かつディレクトリでなければなりません。
- これにより `ptool` のランタイム状態が更新され、そのランタイム cwd を
  使用する API (`ptool.run`, `ptool.path.abspath`, `ptool.path.relpath`
  など) に影響します。

例:

```lua
ptool.cd("foobar")
local res = ptool.run({ cmd = "pwd", stdout = "capture" })
print(res.stdout)
```

## ptool.run

> `v0.1.0` - Introduced.

`ptool.run` は Rust から外部コマンドを実行します。

次の呼び出し形式をサポートします。

```lua
ptool.run("echo hello world")
ptool.run("echo", "hello world")
ptool.run("echo", {"hello", "world"})
ptool.run("echo hello world", { echo = true })
ptool.run("echo", {"hello", "world"}, { echo = true })
ptool.run({ cmd = "echo", args = {"hello", "world"} })
ptool.run({ cmd = "echo", args = {"hello"}, stdout = "capture" })
```

引数ルール:

- `ptool.run(cmdline)`: `cmdline` はシェル風 (`shlex`) ルールで分割され、
  最初の項目がコマンド、残りが引数として扱われます。
- `ptool.run(cmd, argsline)`: `cmd` はそのままコマンドとして使われ、
  `argsline` はシェル風 (`shlex`) ルールで引数リストへ分割されます。
- `ptool.run(cmd, args)`: `cmd` は文字列、`args` は文字列配列です。
- `ptool.run(cmdline, options)`: `options` は `echo` など、この呼び出し用の
  設定を上書きします。
- `ptool.run(cmd, args, options)`: `args` は文字列または文字列配列で、
  `options` は `echo` など、この呼び出し用の設定を上書きします。
- `ptool.run(options)`: `options` はテーブルです。
- 第 2 引数がテーブルの場合: それが配列 (連続する整数キー `1..n`) なら
  `args` として、それ以外なら `options` として扱われます。

戻り値ルール:

- 常に次のフィールドを持つテーブルが返されます:
  - `ok` (boolean): 終了コードが `0` かどうか。
  - `code` (integer|nil): プロセス終了コード。シグナルで終了した場合は
    `nil` です。
  - `stdout` (string, 任意): `stdout = "capture"` のとき存在します。
  - `stderr` (string, 任意): `stderr = "capture"` のとき存在します。
  - `assert_ok(self)` (function): `ok = false` のときエラーにします。
    エラーメッセージには終了コードと、利用可能なら `stderr` が
    含まれます。
- `check` のデフォルト値は
  `ptool.config({ run = { check = ... } })` から取得されます。
  未設定ならデフォルトは `false` です。`check = false` のときは、
  呼び出し側が自分で `ok` を確認するか `res:assert_ok()` を呼べます。
- `check = true` かつ `retry = true` の場合、`ptool.run` は失敗した
  コマンドを再試行するかどうかを最終エラー前に尋ねます。

例:

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

`ptool.run(options)` もサポートされます。ここで `options` は次のフィールド
を持つテーブルです:

- `cmd` (string, 必須): コマンド名または実行ファイルパス。
- `args` (string[], 任意): 引数リスト。
- `cwd` (string, 任意): 子プロセスの作業ディレクトリ。
- `env` (table, 任意): 追加の環境変数。キーは変数名、値は変数値です。
- `echo` (boolean, 任意): この実行でコマンド情報を表示するかどうか。
  省略時は `ptool.config({ run = { echo = ... } })` の値が使われ、
  それも未設定ならデフォルトは `true` です。
- `check` (boolean, 任意): 終了コードが `0` 以外のときに即座にエラーに
  するかどうか。省略時は `ptool.config({ run = { check = ... } })` の
  値が使われ、それも未設定ならデフォルトは `false` です。
- `confirm` (boolean, 任意): 実行前にユーザー確認を行うかどうか。
  省略時は `ptool.config({ run = { confirm = ... } })` の値が使われ、
  それも未設定ならデフォルトは `false` です。
- `retry` (boolean, 任意): `check = true` のとき、失敗後に再試行するか
  ユーザーへ尋ねるかどうか。省略時は
  `ptool.config({ run = { retry = ... } })` の値が使われ、それも未設定なら
  デフォルトは `false` です。
- `stdout` (string, 任意): stdout の扱い。サポートされる値:
  - `"inherit"`: 現在の端末へ引き継ぐ (デフォルト)。
  - `"capture"`: `res.stdout` にキャプチャする。
  - `"null"`: 出力を破棄する。
- `stderr` (string, 任意): stderr の扱い。サポートされる値:
  - `"inherit"`: 現在の端末へ引き継ぐ (デフォルト)。
  - `"capture"`: `res.stderr` にキャプチャする。
  - `"null"`: 出力を破棄する。
- `confirm = true` の場合:
  - ユーザーが実行を拒否すると即座にエラーになります。
  - 現在の環境が非対話 (TTY なし) なら即座にエラーになります。
- `retry = true` かつ `check = true` の場合:
  - コマンドが失敗すると、`ptool.run` は同じコマンドを再試行するか
    尋ねます。
  - 現在の環境が非対話 (TTY なし) なら、再試行を尋ねる代わりに即座に
    エラーになります。

例:

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

`ptool.run_capture` は `ptool.run` と同じ呼び出し形式、引数ルール、
戻り値ルール、オプションで、Rust から外部コマンドを実行します。

違いはデフォルトのストリーム処理だけです:

- `stdout` のデフォルトは `"capture"`。
- `stderr` のデフォルトは `"capture"`。

それでも `options` で各フィールドを明示的に上書きできます。

例:

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
