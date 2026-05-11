# 正規表現 API

正規表現ヘルパーは `ptool.re` と `p.re` にあります。

## ptool.re.compile

> `v0.1.0` - Introduced.

`ptool.re.compile(pattern[, opts])` は正規表現をコンパイルし、 `Regex` オブジェクトを返します。

- `pattern` (string, 必須): 正規表現パターン。
- `opts` (table, 任意): コンパイルオプション。現在サポートされるもの:
  - `case_insensitive` (boolean, 任意): 大文字小文字を区別しないかどうか。 デフォルトは `false`。

例:

```lua
local re = ptool.re.compile("(?P<name>\\w+)", { case_insensitive = true })
print(re:is_match("Alice")) -- true
```

## ptool.re.escape

> `v0.1.0` - Introduced.

`ptool.re.escape(text)` はプレーンテキストを正規表現リテラル文字列として エスケープします。

- `text` (string, 必須): エスケープするテキスト。
- 戻り値: エスケープ後の文字列。

例:

```lua
local keyword = "a+b?"
local re = ptool.re.compile("^" .. ptool.re.escape(keyword) .. "$")
print(re:is_match("a+b?")) -- true
```

## Regex

> `v0.1.0` - Introduced.

`Regex` は `ptool.re.compile(...)` が返すコンパイル済み正規表現を 表します。

これは Lua userdata として実装されています。

メソッド:

- `re:is_match(input)` -> `boolean`
- `re:find(input[, init])` -> `Match|nil`
- `re:find_all(input)` -> `Match[]`
- `re:captures(input)` -> `Captures|nil`
- `re:captures_all(input)` -> `Captures[]`
- `re:replace(input, replacement)` -> `string`
- `re:replace_all(input, replacement)` -> `string`
- `re:split(input[, limit])` -> `string[]`

### is_match

Canonical API name: `ptool.re.Regex:is_match`.

`re:is_match(input)` は、その正規表現が `input` に一致するか確認します。

- `input` (string, 必須): 入力テキスト。
- 戻り値: `boolean`。

### find

Canonical API name: `ptool.re.Regex:find`.

`re:find(input[, init])` は `input` 内の最初の一致を返します。一致しない 場合は `nil` を返します。

- `input` (string, 必須): 入力テキスト。

パラメータに関する注意:

- `init` は 1 始まりの開始位置で、デフォルトは `1` です。
- `limit` は `0` より大きくなければなりません。

戻り値の構造:

- `Match`:
  - `start` (integer): 1 始まりの開始インデックス。
  - `finish` (integer): `string.sub` にそのまま渡せる終了インデックス。
  - `text` (string): 一致したテキスト。
- `Captures`:
  - `full` (string): 完全一致したテキスト。
  - `groups` (table): キャプチャ順のキャプチャグループ配列。一致しない グループは `nil`。
  - `named` (table): グループ名をキーにした名前付きキャプチャのマップ。

### find_all

Canonical API name: `ptool.re.Regex:find_all`.

`re:find_all(input)` は `input` 内のすべての一致を `Match[]` として返します。

### captures

Canonical API name: `ptool.re.Regex:captures`.

`re:captures(input)` は `input` 内の最初のキャプチャセットを返します。 一致しない場合は `nil` を返します。

### captures_all

Canonical API name: `ptool.re.Regex:captures_all`.

`re:captures_all(input)` は `input` 内のすべてのキャプチャセットを `Captures[]` として返します。

### replace

Canonical API name: `ptool.re.Regex:replace`.

`re:replace(input, replacement)` は `input` 内の最初の一致を置換します。

### replace_all

Canonical API name: `ptool.re.Regex:replace_all`.

`re:replace_all(input, replacement)` は `input` 内のすべての一致を 置換します。

### split

Canonical API name: `ptool.re.Regex:split`.

`re:split(input[, limit])` は、その正規表現を区切り文字として `input` を 分割します。

例:

```lua
local re = ptool.re.compile("(?P<word>\\w+)")
local cap = re:captures("hello world")
print(cap.full)         -- hello
print(cap.named.word)   -- hello
print(re:replace_all("a b c", "_")) -- _ _ _
```
