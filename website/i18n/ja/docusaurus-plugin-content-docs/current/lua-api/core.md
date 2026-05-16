# コア Lua API

`ptool` は、これらのコアランタイムヘルパーを `ptool` と `p` の直下に 公開します。

`ptool run <lua_file>` は Lua スクリプトを実行し、グローバル変数 `ptool` (またはその別名 `p`; たとえば `p.run` は `ptool.run` と同等) を注入します。`.lua` で終わるファイルでは、`ptool <lua_file>` も同じ 挙動の CLI 短縮形として使えます。

組み込み Lua ランタイムは Lua の基本グローバルを維持しつつ、標準ライブラリ としてはデフォルトで次だけを公開します。

- `table`
- `string`
- `math`
- `utf8`

`io`、`os`、`package` のようなホスト側に近い組み込みモジュールは意図的に 利用できません。ファイルシステム、環境変数、プロセス、ネットワークなどの ランタイム操作には `ptool.fs`、`ptool.os`、`ptool.path`、 `ptool.run` などの API を使ってください。

Lua スクリプトへ引数を渡したい場合は、次のようにできます。

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

その引数は `ptool.args.parse(...)` で解析できます。

スクリプト例:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

shebang もサポートしているため、ファイルの先頭に次を追加できます。

```
#!/usr/bin/env ptool
```

## ptool.use

> `v0.1.0` - Introduced. `v0.7.0` - Cargo 形式のバージョン要件式を追加。

`ptool.use` は、スクリプトに必要な `ptool` のバージョンまたは バージョン要件を宣言します。

```lua
ptool.use("v0.1.0")
ptool.use("^0.6.0")
ptool.use(">= v0.6.0, < 0.7.0")
```

- 引数には、通常のセマンティックバージョン文字列または Cargo 形式の バージョン要件式を指定できます。
- 通常のバージョン文字列は `v0.1.0` や `0.1.0` のように先頭の `v` を 省略可能で、最低限必要な `ptool` バージョンを宣言する従来の 挙動を保ちます。
- バージョン要件式は `^0.6.0`、`~0.6.0`、`>=0.6.0, <0.7.0`、 `1.*`、`1.2.*` などの演算子やパターンをサポートします。
- バージョン要件式内の各バージョン要素でも、`>= v0.6.0, < 0.7.0` の ように `v` プレフィックスを任意で使えます。
- 現在の `ptool` バージョンが宣言されたバージョンまたは要件を満たさない 場合、スクリプトは直ちにエラーで終了します。

## ptool.version

> `Unreleased` - 追加。

`ptool.version()` は現在の `ptool` バージョン文字列を返します。

- 戻り値: `string`。
- 返される値は `0.7.2` のようなセマンティックバージョン文字列です。
- スクリプトで実行中の `ptool` バージョンを表示、記録、比較したい場合に使います。

例:

```lua
print(ptool.version())
print(p.version())
```

## ptool.unindent

> `v0.1.0` - Introduced.

`ptool.unindent` は複数行文字列を処理し、各行の先頭インデントのあとにある `| ` プレフィックスを除去し、先頭と末尾の空行を取り除きます。

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

`ptool.inspect(value[, options])` は Lua 値を読みやすい Lua 風の文字列へ レンダリングします。主な用途はデバッグとテーブル内容の表示です。

- `value` (any, 必須): inspect する Lua 値。
- `options` (table, 任意): レンダリングオプション。サポートされる フィールド:
  - `indent` (string, 任意): ネストごとに使うインデント。デフォルトは 2 つのスペースです。
  - `multiline` (boolean, 任意): テーブルを複数行で描画するかどうか。 デフォルトは `true`。
  - `max_depth` (integer, 任意): 描画する最大ネスト深さ。それより深い値は `<max-depth>` に置き換えられます。
- 戻り値: `string`。

挙動:

- 配列風の要素 (`1..n`) が最初に描画されます。
- 残りのテーブルフィールドは、そのあとに安定したキー順で描画されます。
- 識別子風の文字列キーは `key = value`、それ以外のキーは `[key] = value` の形式で描画されます。
- 再帰的なテーブル参照は `<cycle>` として描画されます。
- 関数、thread、userdata は `<function>` や `<userdata>` のような プレースホルダー値で描画されます。

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

> `v0.1.0` - Introduced. `v0.5.0` - バリデーションオプションと prompt サブコマンドを追加。

`ptool.ask` は対話型プロンプトを提供します。直接呼び出してテキスト入力を 受け取ることもでき、確認、単一選択、複数選択、シークレット入力用の サブプロンプトも使えます。

共通の挙動:

- すべての `ptool.ask` プロンプトは対話型 TTY を必要とします。 非対話環境で実行するとエラーになります。
- ユーザーがプロンプトをキャンセルすると、スクリプトはエラーになります。
- 未知のオプション名や不正な値型はエラーになります。

### ptool.ask

`ptool.ask(prompt[, options])` はユーザーに 1 行のテキスト入力を求め、 その回答を返します。

- `prompt` (string, 必須): ユーザーに表示するプロンプト。
- `options` (table, 任意): プロンプトオプション。サポートされる フィールド:
  - `default` (string, 任意): ユーザーが空の回答を送信したときに使う デフォルト値。
  - `help` (string, 任意): プロンプトの下に表示する補助テキスト。
  - `placeholder` (string, 任意): ユーザーが入力を始める前に表示する プレースホルダー。
  - `required` (boolean, 任意): 回答を空にできないようにするかどうか。
  - `allow_empty` (boolean, 任意): 空の回答を許可するかどうか。 デフォルトは `true`。
  - `trim` (boolean, 任意): 返す前に前後の空白を削除するかどうか。
  - `min_length` (integer, 任意): 許可される最小文字数。
  - `max_length` (integer, 任意): 許可される最大文字数。
  - `pattern` (string, 任意): 回答が一致しなければならない正規表現。
- 戻り値: `string`。

例:

```lua
local project = ptool.ask("Project name?", {
  placeholder = "my-tool",
  help = "Lowercase letters, digits, and dashes only",
  required = true,
  trim = true,
  pattern = "^[a-z0-9-]+$",
})
```

### ptool.ask.confirm

> `v0.5.0` - Introduced.

`ptool.ask.confirm(prompt[, options])` は yes/no の回答を求めます。

- `prompt` (string, 必須): ユーザーに表示するプロンプト。
- `options` (table, 任意): プロンプトオプション。サポートされる フィールド:
  - `default` (boolean, 任意): ユーザーが Enter を押したときに使う デフォルト回答。
  - `help` (string, 任意): プロンプトの下に表示する補助テキスト。
- 戻り値: `boolean`。

例:

```lua
local confirmed = ptool.ask.confirm("Continue?", {
  default = true,
})
```

### ptool.ask.select

> `v0.5.0` - Introduced.

`ptool.ask.select(prompt, items[, options])` は一覧から 1 つ選ばせます。

- `prompt` (string, 必須): ユーザーに表示するプロンプト。
- `items` (table, 必須): 候補項目。各要素は次のいずれかです:
  - string: 表示ラベルと戻り値の両方に使われます。
  - `{ label = "Patch", value = "patch" }` のような table。
- `options` (table, 任意): プロンプトオプション。サポートされる フィールド:
  - `help` (string, 任意): プロンプトの下に表示する補助テキスト。
  - `page_size` (integer, 任意): 一度に表示する最大行数。
  - `default_index` (integer, 任意): 初期選択位置の 1-based インデックス。
- 戻り値: `string`。

例:

```lua
local bump = ptool.ask.select("Select bump type", {
  { label = "Patch", value = "patch" },
  { label = "Minor", value = "minor" },
  { label = "Major", value = "major" },
}, {
  default_index = 2,
})
```

### ptool.ask.multiselect

> `v0.5.0` - Introduced.

`ptool.ask.multiselect(prompt, items[, options])` は一覧から 0 個以上の項目を 選ばせます。

- `prompt` (string, 必須): ユーザーに表示するプロンプト。
- `items` (table, 必須): 候補項目。形式は `ptool.ask.select` と同じです。
- `options` (table, 任意): プロンプトオプション。サポートされる フィールド:
  - `help` (string, 任意): プロンプトの下に表示する補助テキスト。
  - `page_size` (integer, 任意): 一度に表示する最大行数。
  - `default_indexes` (table, 任意): デフォルトで選択される 1-based インデックス配列。
  - `min_selected` (integer, 任意): 必須の最小選択数。
  - `max_selected` (integer, 任意): 許可される最大選択数。
- 戻り値: `table`。

例:

```lua
local targets = ptool.ask.multiselect("Select targets", {
  "linux",
  "macos",
  "windows",
}, {
  default_indexes = { 1, 2 },
  min_selected = 1,
})
```

### ptool.ask.secret

> `v0.5.0` - Introduced.

`ptool.ask.secret(prompt[, options])` はトークンやパスワードのような シークレット入力を求めます。

- `prompt` (string, 必須): ユーザーに表示するプロンプト。
- `options` (table, 任意): プロンプトオプション。サポートされる フィールド:
  - `help` (string, 任意): プロンプトの下に表示する補助テキスト。
  - `required` (boolean, 任意): 回答を空にできないようにするかどうか。
  - `allow_empty` (boolean, 任意): 空の回答を許可するかどうか。 デフォルトは `false`。
  - `confirm` (boolean, 任意): 2 回入力して確認させるかどうか。 デフォルトは `false`。
  - `confirm_prompt` (string, 任意): 確認ステップ用のカスタムプロンプト。
  - `mismatch_message` (string, 任意): 2 回の入力が一致しないときの カスタムエラーメッセージ。
  - `display_toggle` (boolean, 任意): 入力した値を一時的に表示できるように するかどうか。
  - `min_length` (integer, 任意): 許可される最小文字数。
  - `max_length` (integer, 任意): 許可される最大文字数。
  - `pattern` (string, 任意): 回答が一致しなければならない正規表現。
- 戻り値: `string`。

例:

```lua
local token = ptool.ask.secret("API token?", {
  confirm = true,
  min_length = 20,
})
```

## ptool.config

> `v0.1.0` - Introduced.

`ptool.config` はスクリプトのランタイム設定を行います。

現在サポートされているフィールド:

- `run` (table, 任意): `ptool.run` のデフォルト設定。サポートされる フィールド:
  - `echo` (boolean, 任意): デフォルトの echo スイッチ。デフォルトは `true`。
  - `check` (boolean, 任意): 失敗時にデフォルトでエラーにするかどうか。 デフォルトは `false`。
  - `confirm` (boolean, 任意): 実行前にデフォルトで確認を要求するかどうか。 デフォルトは `false`。
  - `retry` (boolean, 任意): `check = true` のとき、実行失敗後に再試行するか どうかをユーザーへ尋ねるかどうか。デフォルトは `false`。

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
- これにより `ptool` のランタイム状態が更新され、そのランタイム cwd を 使用する API (`ptool.run`, `ptool.path.abspath`, `ptool.path.relpath` など) に影響します。

例:

```lua
ptool.cd("foobar")
local res = ptool.run({ cmd = "pwd", stdout = "capture" })
print(res.stdout)
```

## ptool.script_path

> `v0.4.0` - Introduced.

`ptool.script_path()` は現在のエントリスクリプトの絶対パスを返します。

- 戻り値: `string|nil`。

挙動:

- `ptool run <file>` で実行した場合、エントリスクリプトのパスを絶対かつ 正規化されたパスとして返します。
- 返されるパスはランタイム開始時に固定され、その後 `ptool.cd(...)` を 呼んでも変化しません。
- `ptool repl` では `nil` を返します。

例:

```lua
local script_path = ptool.script_path()
local script_dir = ptool.path.dirname(script_path)
local project_root = ptool.path.dirname(script_dir)
```

## ptool.try

> `v0.4.0` - Introduced.

`ptool.try(fn)` は `fn` を実行し、送出されたエラーを戻り値に変換します。

- `fn` (function, 必須): 実行するコールバック。
- 戻り値: `ok, value, err`。

戻り値ルール:

- 成功時は `ok = true`、`err = nil` で、`value` にコールバックの戻り値が 入ります。
- コールバックが値を返さない場合、`value` は `nil` です。
- コールバックが 1 つだけ値を返す場合、`value` はその値自体です。
- コールバックが複数の値を返す場合、`value` は配列風のテーブルです。
- 失敗時は `ok = false`、`value = nil` で、`err` はテーブルです。

構造化エラーフィールド:

- `kind` (string): `io_error`、`command_failed`、`invalid_argument`、 `http_error`、`lua_error` などの安定したエラーカテゴリ。
- `message` (string): 人間が読みやすいエラーメッセージ。
- `op` (string, 任意): `ptool.fs.read` のような API または操作名。
- `detail` (string, 任意): 追加の失敗詳細。
- `path` (string, 任意): ファイルシステム失敗に関係するパス。
- `input` (string, 任意): 解析や検証に失敗した元の入力。
- `cmd` (string, 任意): コマンド失敗時のコマンド名。
- `status` (integer, 任意): 利用可能な場合の終了ステータスまたは HTTP ステータス。
- `stderr` (string, 任意): コマンド失敗時にキャプチャされた stderr。
- `url` (string, 任意): HTTP 失敗に関係する URL。
- `cwd` (string, 任意): コマンド失敗時に実際に使われた作業ディレクトリ。
- `target` (string, 任意): SSH 関連のコマンド失敗時の SSH ターゲット。
- `retryable` (boolean): 再試行に意味があるかどうか。デフォルトは `false` です。

挙動:

- `ptool` の API は構造化エラーを送出します。`ptool.try` はそれらを上記の `err` テーブルへ変換するため、呼び出し側は `err.kind` や関連フィールドで 分岐できます。
- 通常の Lua エラーも捕捉されます。その場合 `err.kind` は `lua_error` で、 `message` だけが保証されます。
- `ptool.fs.read`、`ptool.http.request`、`ptool.run(..., { check = true })`、 `res:assert_ok()` のような API のエラー処理には `ptool.try` の使用を推奨 します。

例:

```lua
local ok, content, err = ptool.try(function()
  return ptool.fs.read("missing.txt")
end)

if not ok and err.kind == "io_error" then
  print(err.op, err.path)
end

local ok2, _, err2 = ptool.try(function()
  local res = ptool.run({
    cmd = "sh",
    args = {"-c", "echo bad >&2; exit 7"},
    stderr = "capture",
  })
  res:assert_ok()
end)

if not ok2 and err2.kind == "command_failed" then
  print(err2.cmd, err2.status, err2.stderr)
end
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

- `ptool.run(cmdline)`: `cmdline` はシェル風 (`shlex`) ルールで分割され、 最初の項目がコマンド、残りが引数として扱われます。
- `ptool.run(cmd, argsline)`: `cmd` はそのままコマンドとして使われ、 `argsline` はシェル風 (`shlex`) ルールで引数リストへ分割されます。
- `ptool.run(cmd, args)`: `cmd` は文字列、`args` は文字列配列です。
- `ptool.run(cmdline, options)`: `options` は `echo` など、この呼び出し用の 設定を上書きします。
- `ptool.run(cmd, args, options)`: `args` は文字列または文字列配列で、 `options` は `echo` など、この呼び出し用の設定を上書きします。
- `ptool.run(options)`: `options` はテーブルです。
- 第 2 引数がテーブルの場合: それが配列 (連続する整数キー `1..n`) なら `args` として、それ以外なら `options` として扱われます。

戻り値ルール:

- 常に次のフィールドを持つテーブルが返されます:
  - `ok` (boolean): 終了コードが `0` かどうか。
  - `code` (integer|nil): プロセス終了コード。シグナルで終了した場合は `nil` です。
  - `cmd` (string): 実行に使われたコマンド名。
  - `cwd` (string): 実行に使われた実際の作業ディレクトリ。
  - `stdout` (string, 任意): `stdout = "capture"` のとき存在します。
  - `stderr` (string, 任意): `stderr = "capture"` のとき存在します。
  - `assert_ok(self)` (function): `ok = false` のとき構造化エラーを発生 させます。エラー種別は `command_failed` で、`cmd`、`status`、 `stderr`、`cwd` を含むことがあります。
- `check` のデフォルト値は `ptool.config({ run = { check = ... } })` から取得されます。 未設定ならデフォルトは `false` です。`check = false` のときは、 呼び出し側が自分で `ok` を確認するか `res:assert_ok()` を呼べます。
- `check = true` かつ `retry = true` の場合、`ptool.run` は失敗した コマンドを再試行するかどうかを最終エラー前に尋ねます。
- `check = true` の場合、`ptool.run` は `res:assert_ok()` と同じ `command_failed` 構造化エラーを送出します。Lua 側で捕捉して調べたいなら `ptool.try(...)` を使ってください。

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

`ptool.run(options)` もサポートされます。ここで `options` は次のフィールド を持つテーブルです:

- `cmd` (string, 必須): コマンド名または実行ファイルパス。
- `args` (string[], 任意): 引数リスト。
- `cwd` (string, 任意): 子プロセスの作業ディレクトリ。
- `env` (table, 任意): 追加の環境変数。キーは変数名、値は変数値です。
- `stdin` (string|table, 任意): 子プロセスの stdin の入力元。
  - 文字列を指定すると、その文字列が子プロセスの stdin に送られます。
  - テーブル `{ file = "path" }` を指定すると、ファイルから stdin を読み込みます。
  - 省略時は、子プロセスが現在のプロセスの stdin を継承します。
- `trim` (ブール値、オプション): キャプチャされた `stdout` とキャプチャされた `stderr` を返す前に、それらから先頭と末尾の空白をトリミングするかどうか。これは、`"capture"` に設定されたストリームにのみ影響します。デフォルトは`false`です。
- `echo` (boolean, 任意): この実行でコマンド情報を表示するかどうか。 省略時は `ptool.config({ run = { echo = ... } })` の値が使われ、 それも未設定ならデフォルトは `true` です。
- `check` (boolean, 任意): 終了コードが `0` 以外のときに即座にエラーに するかどうか。省略時は `ptool.config({ run = { check = ... } })` の 値が使われ、それも未設定ならデフォルトは `false` です。
- `confirm` (boolean, 任意): 実行前にユーザー確認を行うかどうか。 省略時は `ptool.config({ run = { confirm = ... } })` の値が使われ、 それも未設定ならデフォルトは `false` です。
- `retry` (boolean, 任意): `check = true` のとき、失敗後に再試行するか ユーザーへ尋ねるかどうか。省略時は `ptool.config({ run = { retry = ... } })` の値が使われ、それも未設定なら デフォルトは `false` です。
- `stdout` (string|table, 任意): stdout の処理方法。
  - `"inherit"`: 現在の端末へ継承します (デフォルト)。
  - `"capture"`: `res.stdout` にキャプチャする。
  - `"null"`: 出力を破棄します。
  - `{ file = "path" }`: stdout をファイルへ書き込み、最初に内容を切り詰めます。
  - `{ file = "path", append = true }`: stdout をファイルへ追記します。
- `stderr` (string|table, 任意): stderr の処理方法。
  - `"inherit"`: 現在の端末へ継承します (デフォルト)。
  - `"capture"`: `res.stderr` にキャプチャする。
  - `"null"`: 出力を破棄します。
  - `{ file = "path" }`: stderr をファイルへ書き込み、最初に内容を切り詰めます。
  - `{ file = "path", append = true }`: stderr をファイルへ追記します。
- `ptool.run(cmdline, options)` や `ptool.run(cmd, args, options)` などのショートカット呼び出し形式が使用される場合、呼び出しごとの `options` テーブルは同じ意味の `stdin` および `trim` も受け入れます。
- ファイルリダイレクトのパスは、絶対パスが指定されない限り、有効な子プロセス `cwd` からの相対パスとして解決されます。
- `confirm = true` の場合:
  - ユーザーが実行を拒否すると即座にエラーになります。
  - 現在の環境が非対話 (TTY なし) なら即座にエラーになります。
- `retry = true` かつ `check = true` の場合:
  - コマンドが失敗すると、`ptool.run` は同じコマンドを再試行するか 尋ねます。
  - 現在の環境が非対話 (TTY なし) なら、再試行を尋ねる代わりに即座に エラーになります。

例:

```lua
ptool.run({
  cmd = "echo",
  args = {"hello"},
  env = { FOO = "bar" },
})

local res0 = ptool.run({
  cmd = "cat",
  stdin = "hello from stdin",
  trim = true,
  stdout = "capture",
})
print(res0.stdout)

ptool.run({
  cmd = "sh",
  args = {"-c", "cat; printf ' err' >&2"},
  stdin = { file = "input.txt" },
  stdout = { file = "output.log" },
  stderr = { file = "error.log", append = true },
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

> `Unreleased` - 追加。

`ptool.run_capture` は `ptool.run` と同じ呼び出し形式、引数ルール、 戻り値ルール、オプションで、Rust から外部コマンドを実行します。

違いはデフォルトのストリーム処理だけです:

- `stdout` のデフォルトは `"capture"`。
- `stderr` のデフォルトは `"capture"`。

`trim` のデフォルトは引き続き `false` であり、これらのフィールドのいずれかを `options` で明示的にオーバーライドできます。

例:

```lua
local res = ptool.run_capture("echo hello world")
print(res.stdout)

local res2 = ptool.run_capture({
  cmd = "cat",
  stdin = "captured stdin",
  trim = true,
})
print(res2.stdout)

ptool.run_capture({
  cmd = "sh",
  args = {"-c", "printf 'captured'; printf ' problem' >&2"},
  stdout = { file = "captured.log" },
  stderr = { file = "captured.err" },
})

local res3 = ptool.run_capture("echo hello", {
  stderr = "inherit",
})
print(res3.stdout)
```

## ptool.exec

> `Unreleased` - 追加。

`ptool.exec` は現在の `ptool` プロセスを外部コマンドで置き換えます。

`ptool.run` と同じコマンド引数の呼び出し形式をサポートします:

```lua
ptool.exec("echo hello world")
ptool.exec("echo", "hello world")
ptool.exec("echo", {"hello", "world"})
ptool.exec("echo hello world", { echo = true })
ptool.exec("echo", {"hello", "world"}, { confirm = true })
ptool.exec({ cmd = "echo", args = {"hello", "world"} })
```

挙動:

- 成功すると `ptool.exec` は戻りません。現在の `ptool` プロセスは対象コマンドで置き換えられます。
- `ptool.exec(...)` が成功した後の Lua コードは一切実行されません。
- `ptool repl` でも、成功した `ptool.exec(...)` は REPL プロセス自体を置き換えます。
- Unix プラットフォームでは、これは本物のプロセス置換セマンティクスを使います。
- 非 Unix プラットフォームでは、`ptool.exec` は現在 `unsupported` エラーを返します。

引数ルール:

- `ptool.exec(cmdline)`: `cmdline` はシェル風 (`shlex`) ルールで分割され、最初の項目がコマンド、残りが引数として扱われます。
- `ptool.exec(cmd, argsline)`: `cmd` はそのままコマンドとして使われ、`argsline` はシェル風 (`shlex`) ルールで引数リストへ分割されます。
- `ptool.exec(cmd, args)`: `cmd` は文字列、`args` は文字列配列です。
- `ptool.exec(cmdline, options)`: `options` は `echo` など、この呼び出し用の設定を上書きします。
- `ptool.exec(cmd, args, options)`: `args` は文字列または文字列配列で、`options` は `confirm` など、この呼び出し用の設定を上書きします。
- `ptool.exec(options)`: `options` はテーブルです。
- 第 2 引数がテーブルの場合: それが配列 (連続する整数キー `1..n`) なら `args` として、それ以外なら `options` として扱われます。

`ptool.exec(options)` もサポートされます。ここで `options` は次のフィールドを持つテーブルです:

- `cmd` (string, 必須): コマンド名または実行ファイルパス。
- `args` (string[], 任意): 引数リスト。
- `cwd` (string, 任意): 子プロセスの作業ディレクトリ。
- `env` (table, 任意): 追加の環境変数。キーは変数名、値は変数値です。
- `stdin` (table, 任意): 子プロセスの stdin の入力元。
  - `{ file = "path" }`: ファイルから stdin を読み込みます。
- `echo` (boolean, 任意): 置換前にコマンド情報を表示するかどうか。省略時は `ptool.config({ run = { echo = ... } })` の値が使われ、それも未設定ならデフォルトは `true` です。
- `confirm` (boolean, 任意): 置換前にユーザー確認を行うかどうか。省略時は `ptool.config({ run = { confirm = ... } })` の値が使われ、それも未設定ならデフォルトは `false` です。
- `stdout` (string|table, 任意): stdout の処理方法。
  - `"inherit"`: 現在の端末へ継承します (デフォルト)。
  - `"null"`: 出力を破棄します。
  - `{ file = "path" }`: stdout をファイルへ書き込み、最初に内容を切り詰めます。
  - `{ file = "path", append = true }`: stdout をファイルへ追記します。
- `stderr` (string|table, 任意): stderr の処理方法。
  - `"inherit"`: 現在の端末へ継承します (デフォルト)。
  - `"null"`: 出力を破棄します。
  - `{ file = "path" }`: stderr をファイルへ書き込み、最初に内容を切り詰めます。
  - `{ file = "path", append = true }`: stderr をファイルへ追記します。
- `ptool.run` と異なり、`ptool.exec` は文字列 `stdin`、`stdout = "capture"`、`stderr = "capture"`、`trim`、`check`、`retry` をサポートしません。
- ファイルリダイレクトのパスは、絶対パスが指定されない限り、有効な子プロセス `cwd` からの相対パスとして解決されます。
- `confirm = true` の場合:
  - ユーザーが実行を拒否すると即座にエラーになります。
  - 現在の環境が非対話 (TTY なし) なら即座にエラーになります。

例:

```lua
ptool.config({ run = { echo = false } })

ptool.exec("echo from ptool")
ptool.exec("echo", "from ptool")
ptool.exec("echo", {"from", "ptool"})

ptool.exec({
  cmd = "sh",
  args = {"-c", "printf '%s|%s' \"$FOO\" \"$PWD\""},
  cwd = "crates",
  env = { FOO = "bar" },
})

ptool.exec({
  cmd = "sh",
  args = {"-c", "cat; printf ' done'; printf ' warn' >&2"},
  stdin = { file = "input.txt" },
  stdout = { file = "output.log" },
  stderr = { file = "error.log", append = true },
})
```
