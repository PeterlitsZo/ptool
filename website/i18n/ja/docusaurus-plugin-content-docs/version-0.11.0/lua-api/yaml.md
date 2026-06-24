# YAML API

YAML の解析とシリアライズのヘルパーは `ptool.yaml` と `p.yaml` に あります。

## ptool.yaml.parse

> `v0.4.0` - Introduced.

`ptool.yaml.parse(input)` は YAML 文字列を Lua 値へ解析します。

- `input` (string, 必須): YAML テキスト。
- 戻り値: 解析された Lua 値。ルートは対応している任意の YAML 型を使えます。

型対応:

- YAML mapping -> Lua table
- YAML sequence -> Lua sequence table (1-based)
- YAML string -> Lua string
- `i64` に収まる YAML integer -> Lua integer
- それ以外の YAML number -> Lua number
- YAML boolean -> Lua boolean
- YAML null -> Lua `nil`

エラー時の挙動:

- `input` が文字列でない場合はエラーになります。
- YAML 構文エラーでは、パーサー詳細を含むメッセージでエラーになります。
- 非文字列キーを持つ mapping や明示的な YAML tag など、`ptool` の Lua 値へ 変換できない YAML 値でもエラーになります。

例:

```lua
local data = p.yaml.parse([[
name: ptool
features:
  - yaml
  - repl
stars: 42
]])

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.yaml.get

> `v0.4.0` - Introduced.

`ptool.yaml.get(input, path)` は YAML テキストから指定パスの値を読み出します。

- `input` (string, 必須): YAML テキスト。
- `path` ((string|integer)[], 必須): `{"spec", "template", "metadata", "name"}` や `{"items", 1, "name"}` のような空でないパス配列。
- 戻り値: 対応する Lua 値。パスが存在しない場合は `nil`。

挙動:

- 文字列のパス要素は mapping key を選択します。
- 整数のパス要素は Lua の 1-based 添字で sequence 要素を選択します。

例:

```lua
local text = [[
items:
  - name: alpha
  - name: beta
]]

local first_name = p.yaml.get(text, {"items", 1, "name"})
print(first_name)
```

## ptool.yaml.stringify

> `v0.4.0` - Introduced.

`ptool.yaml.stringify(value)` は Lua 値を YAML テキストへ変換します。

- `value` (YAML 互換 Lua 値, 必須): エンコードする値。
- 戻り値: エンコードされた YAML 文字列。

挙動:

- 値は `ptool.json.stringify` と同じ Lua 値マッピングで YAML に変換可能である 必要があります。
- Lua sequence table は YAML sequence としてエンコードされます。
- 文字列キーを持つ Lua table は YAML mapping としてエンコードされます。

例:

```lua
local text = p.yaml.stringify({
  project = "ptool",
  features = {"yaml", "lua"},
  stable = true,
})

print(text)
```

注意:

- 現在は単一ドキュメントの YAML のみ対応しています。
- YAML mapping のキーは文字列である必要があります。
- 明示的な YAML tag は対応していません。
- `ptool.yaml.get` の `path` 引数は、文字列および正の整数から成る空でない配列で ある必要があります。
- 整数のパス要素は Lua の配列添字に合わせて 1-based です。
