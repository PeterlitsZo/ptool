# プロセス API

`ptool.proc` と `p.proc` でローカルプロセス用の補助機能を利用できます。

このモジュールは、すでにローカルで実行中のプロセスを調べたり管理したりするためのものです。新しいコマンドを起動したい場合は `ptool.run(...)` を使ってください。

## ptool.proc.self

> `Unreleased` - 追加。

`ptool.proc.self()` は現在の `ptool` プロセスのスナップショット table を返します。

- 戻り値：`table`。

返される table は `ptool.proc.get(...)` と `ptool.proc.find(...)` と同じ構造です。

## ptool.proc.get

> `Unreleased` - 追加。

`ptool.proc.get(pid)` は指定したプロセス ID のスナップショット table を返します。プロセスが存在しない場合は `nil` を返します。

- `pid` (integer, 必須): プロセス ID。
- 戻り値：`table|nil`。

## ptool.proc.exists

> `Unreleased` - 追加。

`ptool.proc.exists(pid)` は、そのプロセス ID が現在存在するかどうかを返します。

- `pid` (integer, 必須): プロセス ID。
- 戻り値：`boolean`。

## ptool.proc.find

> `Unreleased` - 追加。

`ptool.proc.find([options])` はローカルプロセスを列挙し、スナップショット table の配列を返します。

- `options` (table, 任意): フィルターとソートのオプション。
- 戻り値：`table`。

サポートされる `options` フィールド:

- `pid` (integer, 任意): 1 つの正確なプロセス ID に一致します。
- `pids` (integer[], 任意): 複数のプロセス ID の集合に一致します。
- `ppid` (integer, 任意): 正確な親プロセス ID に一致します。
- `name` (string, 任意): 正確なプロセス名に一致します。
- `name_contains` (string, 任意): プロセス名に含まれる部分文字列に一致します。
- `exe` (string, 任意): 正確な実行ファイルパスに一致します。
- `exe_contains` (string, 任意): 実行ファイルパスに含まれる部分文字列に一致します。
- `cmdline_contains` (string, 任意): 連結されたコマンドラインに含まれる部分文字列に一致します。
- `user` (string, 任意): 正確なユーザー名に一致します。
- `cwd` (string, 任意): 正確な現在の作業ディレクトリに一致します。
- `include_self` (boolean, 任意): 現在の `ptool` プロセスを含めるかどうか。デフォルトは `false` です。
- `limit` (integer, 任意): フィルターとソートの後に返されるエントリーの最大数。
- `sort_by` (string, 任意): ソートキー。サポートされる値:
  - `"pid"` (デフォルト)
  - `"start_time"`
- `reverse` (boolean, 任意): 最終的な並び順を反転するかどうか。デフォルトは `false` です。

返される各プロセススナップショットには次の項目が含まれることがあります:

- `pid` (integer): プロセス ID。
- `ppid` (integer|nil): 親プロセス ID。
- `name` (string): プロセス名。
- `exe` (string|nil): 実行ファイルパス。利用できる場合のみ設定されます。
- `cwd` (string|nil): 現在の作業ディレクトリ。利用できる場合のみ設定されます。
- `user` (string|nil): 所有ユーザー名。利用できる場合のみ設定されます。
- `cmdline` (string|nil): 連結されたコマンドライン。利用できる場合のみ設定されます。
- `argv` (string[]): コマンドライン引数の配列。
- `state` (string): `"running"` や `"sleeping"` のようなプロセス状態ラベル。
- `start_time_unix_ms` (integer): Unix ミリ秒で表したプロセス開始時刻。

注意:

- 現在のプラットフォームや権限レベルで公開されていない場合、一部のフィールドは `nil` になることがあります。
- プロセススナップショットはある時点の値です。自動では更新されません。

例:

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

> `Unreleased` - 追加。

`ptool.proc.kill(targets[, options])` は 1 つ以上のローカルプロセスにシグナルを送り、構造化された結果 table を返します。

- `targets` (integer|table, 必須): 1 つの pid、1 つのプロセススナップショット table、またはそれらの配列。
- `options` (table, 任意): シグナル送信オプション。
- 戻り値：`table`。

サポートされる `options` フィールド:

- `signal` (string, 任意): シグナル名。サポートされる値:
  - `"hup"`
  - `"term"` (デフォルト)
  - `"kill"`
  - `"int"`
  - `"quit"`
  - `"stop"`
  - `"cont"`
  - `"user1"`
  - `"user2"`
- `missing_ok` (boolean, 任意): 存在しないプロセスも成功として扱うかどうか。デフォルトは `true` です。
- `allow_self` (boolean, 任意): 現在の `ptool` プロセスにシグナルを送ってよいかどうか。デフォルトは `false` です。
- `check` (boolean, 任意): 最終結果が ok でない場合に直ちにエラーを送出するかどうか。デフォルトは `false` です。
- `confirm` (boolean, 任意): シグナルを送る前に確認を求めるかどうか。デフォルトは `false` です。

返される結果 table には次の項目が含まれます:

- `ok` (boolean): 現在の options の下で操作全体が成功したかどうか。
- `signal` (string): 要求されたシグナルラベル。
- `total` (integer): 正規化された対象の総数。
- `sent` (integer): シグナルが送られた対象の数。
- `missing` (integer): すでに存在しなかった対象の数。
- `failed` (integer): 最終的に失敗した対象の数。
- `entries` (table): 対象ごとの結果エントリー。
- `assert_ok(self)` (function): `ok = false` のときに構造化された Lua エラーを送出します。

各 `entries[i]` table には次の項目が含まれます:

- `pid` (integer): 対象プロセス ID。
- `ok` (boolean): この対象が成功したかどうか。
- `existed` (boolean): 対象プロセスが存在し、かつ引き続き一致していたかどうか。
- `signal` (string): 要求されたシグナルラベル。
- `message` (string|nil): 追加の状態詳細。

例:

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

> `Unreleased` - 追加。

`ptool.proc.wait_gone(targets[, options])` は 1 つ以上の対象プロセスが存在しなくなるまで待機し、その後で構造化された結果 table を返します。

- `targets` (integer|table, 必須): 1 つの pid、1 つのプロセススナップショット table、またはそれらの配列。
- `options` (table, 任意): 待機オプション。
- 戻り値：`table`。

サポートされる `options` フィールド:

- `timeout_ms` (integer, 任意): 最大待機時間をミリ秒で指定します。省略した場合は無期限に待機します。
- `interval_ms` (integer, 任意): ポーリング間隔をミリ秒で指定します。デフォルトは `100` です。
- `check` (boolean, 任意): 待機がタイムアウトしたときに直ちにエラーを送出するかどうか。デフォルトは `false` です。

返される結果 table には次の項目が含まれます:

- `ok` (boolean): すべての対象プロセスがタイムアウト前に消えたかどうか。
- `timed_out` (boolean): タイムアウトに達したかどうか。
- `total` (integer): 正規化された対象の総数。
- `gone` (integer[]): 待機終了時点までに消えていたプロセス ID。
- `remaining` (integer[]): 待機終了時点でまだ存在していたプロセス ID。
- `elapsed_ms` (integer): 待機にかかった合計時間（ミリ秒）。
- `assert_ok(self)` (function): `ok = false` のときに構造化された Lua エラーを送出します。

例:

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
