# JSON API

JSON の解析とシリアライズのヘルパーは `ptool.json` と `p.json` に
あります。

## ptool.json.parse

> `v0.3.0` - Introduced.

`ptool.json.parse(input)` は JSON 文字列を Lua 値へ解析します。

- `input` (string, 必須): JSON テキスト。
- 戻り値: 解析された Lua 値。ルートはどの JSON 型でもかまいません。

型対応:

- JSON object -> Lua table
- JSON array -> Lua sequence table (1-based)
- JSON string -> Lua string
- `i64` に収まる JSON integer -> Lua integer
- それ以外の JSON number -> Lua number
- JSON boolean -> Lua boolean
- JSON null -> Lua `nil`

エラー時の挙動:

- `input` が文字列でない場合はエラーになります。
- JSON 構文エラーでは、`serde_json` のパーサー詳細を含むメッセージで
  エラーになります。

例:

```lua
local data = p.json.parse('{"name":"ptool","features":["json","repl"],"stars":42}')

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.json.stringify

> `v0.3.0` - Introduced.

`ptool.json.stringify(value[, options])` は Lua 値を JSON 文字列へ変換します。

- `value` (JSON 互換 Lua 値, 必須): エンコードする値。
- `options` (table, 任意): シリアライズオプション。
- `options.pretty` (boolean, 任意): `true` のとき見やすく整形された JSON
  を出力します。デフォルトは `false`。
- 戻り値: エンコードされた JSON 文字列。

挙動:

- デフォルト出力は余分な空白のないコンパクト JSON です。
- pretty 出力ではインデント付きの複数行 JSON を使います。
- 値は JSON 互換である必要があります。`function`, `thread`, `userdata`
  などのシリアライズできない Lua 値はエラーになります。

例:

```lua
local text = p.json.stringify({
  name = "ptool",
  features = {"json", "repl"},
  stable = true,
}, { pretty = true })

print(text)
```

注意:

- Lua テーブル内の `nil` 値は `mlua` の serde 変換挙動に従うため、
  JSON オブジェクトのフィールドとして保持されません。
- Lua テーブルが配列かオブジェクトかの判定は `mlua` の serde 変換ルールに
  従います。
