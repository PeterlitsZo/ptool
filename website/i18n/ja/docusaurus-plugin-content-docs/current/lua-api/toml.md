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

`ptool.toml.get(input, path)` は TOML テキスト内の指定パスにある値を
読み取ります。

- `input` (string, 必須): TOML テキスト。
- `path` (string[], 必須): `{"package", "version"}` のような空でない
  パス配列。
- 戻り値: 対応する Lua 値。パスが存在しない場合は `nil`。

例:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)
```

## ptool.toml.set

> `v0.1.0` - Introduced.

`ptool.toml.set(input, path, value)` は指定パスに値を設定し、更新後の
TOML テキストを返します。

- `input` (string, 必須): TOML テキスト。
- `path` (string[], 必須): `{"package", "version"}` のような空でない
  パス配列。
- `value` (string|integer|number|boolean, 必須): 書き込む値。
- 戻り値: 更新後の TOML 文字列。

挙動:

- 足りない中間パスは自動的にテーブルとして作成されます。
- 中間パスが存在していてもテーブルでない場合はエラーになります。
- 解析と書き戻しは `toml_edit` に基づいており、元のコメントや書式を
  可能な限り保持します。

例:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.set(text, {"package", "version"}, "0.2.0")
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.remove

> `v0.1.0` - Introduced.

`ptool.toml.remove(input, path)` は指定パスを削除し、更新後の TOML
テキストを返します。

- `input` (string, 必須): TOML テキスト。
- `path` (string[], 必須): `{"package", "name"}` のような空でない
  パス配列。
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

注意:

- `ptool.toml.get/set/remove` の `path` 引数は空でない文字列配列である
  必要があります。
- 現在の `set` はスカラー型 (`string`/`integer`/`number`/`boolean`) の
  書き込みだけをサポートします。
