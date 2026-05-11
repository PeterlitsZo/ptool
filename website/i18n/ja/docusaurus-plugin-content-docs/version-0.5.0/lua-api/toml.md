# TOML API

TOML の解析と編集ヘルパーは `ptool.toml` と `p.toml` にあります。

## ptool.toml.parse

> `v0.1.0` - Introduced.

`ptool.toml.parse(input)` は TOML 文字列を Lua テーブルへ解析します。

- `input` (string, 必須): TOML テキスト。
- 戻り値: Lua テーブル (ルートノードは常にテーブル)。

型対応:

- TOML table / inline table -> Lua table
- TOML array -> Lua sequence table (1-based)
- TOML string -> Lua string
- TOML integer -> Lua integer
- TOML float -> Lua number
- TOML boolean -> Lua boolean
- TOML datetime/date/time -> Lua string

エラー時の挙動:

- `input` が文字列でない場合はエラーになります。
- TOML 構文エラーでは、行番号と列番号を含むメッセージでエラーになります。

例:

```lua
local text = ptool.fs.read("ptool.toml")
local conf = ptool.toml.parse(text)

print(conf.project.name)
print(conf.build.jobs)
print(conf.release_date) -- datetime/date/time values are strings
```

## ptool.toml.get

> `v0.1.0` - Introduced.
>
> `v0.4.0` - Added numeric path segments for array indexing.

`ptool.toml.get(input, path)` は TOML テキスト内の指定パスにある値を
読み取ります。

- `input` (string, 必須): TOML テキスト。
- `path` ((string|integer)[], 必須): `{"package", "version"}` や
  `{"bin", 1, "name"}` のような空でないパス配列。
- 戻り値: 対応する Lua 値。パスが存在しない場合は `nil`。

挙動:

- 文字列のパス要素はテーブルキーを選択します。
- 整数のパス要素は Lua の 1-based インデックスで配列要素を選択します。

例:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)

local first_bin_name = ptool.toml.get(text, {"bin", 1, "name"})
print(first_bin_name)
```

## ptool.toml.set

> `v0.1.0` - Introduced.
>
> `v0.4.0` - Added composite value writes and numeric path segments.

`ptool.toml.set(input, path, value)` は指定パスに値を設定し、更新後の
TOML テキストを返します。

- `input` (string, 必須): TOML テキスト。
- `path` ((string|integer)[], 必須): `{"package", "version"}` や
  `{"bin", 1, "name"}` のような空でないパス配列。
- `value` (string|integer|number|boolean|table, 必須): 書き込む値。
- 戻り値: 更新後の TOML 文字列。

挙動:

- 足りない中間パスは自動的にテーブルとして作成されます。
- 中間パスが存在していてもテーブルでない場合はエラーになります。
- 文字列キーだけを持つ Lua テーブルは TOML テーブルとして書き込まれます。
- Lua のシーケンステーブルは TOML 配列として書き込まれます。
- 文字列キーだけを持つ Lua テーブルのシーケンスは TOML の配列テーブルとして書き込まれます。
- 空の Lua テーブルは現在 TOML テーブルとして書き込まれます。
- 解析と書き戻しは `toml_edit` に基づいており、元のコメントや書式を
  可能な限り保持します。

例:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.set(text, {"package", "version"}, "0.2.0")
ptool.fs.write("Cargo.toml", updated)

local text2 = ptool.toml.set(text, {"package", "keywords"}, {"lua", "toml"})
local text3 = ptool.toml.set(text2, {"package", "metadata"}, {
  channel = "stable",
  maintainers = {"peterlits"},
})
```

## ptool.toml.remove

> `v0.1.0` - Introduced.
>
> `v0.4.0` - Added numeric path segments for array indexing.

`ptool.toml.remove(input, path)` は指定パスを削除し、更新後の TOML
テキストを返します。

- `input` (string, 必須): TOML テキスト。
- `path` ((string|integer)[], 必須): `{"package", "name"}` や
  `{"bin", 1}` のような空でないパス配列。
- 戻り値: 更新後の TOML 文字列。

挙動:

- パスが存在しない場合でもエラーにはならず、元のテキストまたは等価な
  形式が返ります。
- 中間パスが存在していてもテーブルでない場合はエラーになります。

例:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.remove(text, {"package", "description"})
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.stringify

> `v0.4.0` - Introduced.

`ptool.toml.stringify(value)` は Lua 値を TOML テキストへ変換します。

- `value` (table, 必須): エンコードするルート TOML テーブル。
- 戻り値: エンコード後の TOML 文字列。

挙動:

- ルート値は TOML テーブルを表す Lua テーブルである必要があります。
- ネストした Lua テーブルは `ptool.toml.set` と同じ table/array ルールに従います。
- 空の Lua テーブルは現在 TOML テーブルとしてエンコードされます。

例:

```lua
local text = ptool.toml.stringify({
  package = {
    name = "ptool",
    version = "0.4.0",
    keywords = {"lua", "toml"},
  },
})

print(text)
```

注意:

- `ptool.toml.get/set/remove` の `path` 引数は文字列および/または正の整数からなる空でない配列である必要があります。
- 整数のパス要素は Lua の配列インデックスに合わせて 1-based です。
- TOML の datetime/date/time 値は現在も Lua string として読み取られます。
