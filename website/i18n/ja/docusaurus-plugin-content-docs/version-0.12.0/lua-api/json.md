# JSON API

JSON の解析とシリアライズのヘルパーは `ptool.json` と `p.json` に あります。

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
- JSON 構文エラーでは、`serde_json` のパーサー詳細を含むメッセージで エラーになります。

例:

```lua
local data = p.json.parse('{"name":"ptool","features":["json","repl"],"stars":42}')

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.json.get

> 未リリース - 導入。

`ptool.json.get(input, path)` は JSON テキストから指定パスの値を読み取ります。

- `input` (string, 必須): JSON テキスト。
- `path` ((string|integer)[], 必須): `{"spec", "template", "metadata", "name"}` や `{"items", 1, "name"}` のような空でないパス配列。
- 戻り値: 対応する Lua 値。パスが存在しない場合、または想定される object/array 型として辿れない場合は `nil`。

挙動:

- 文字列のパス要素は object のキーを選択します。
- 整数のパス要素は Lua の 1-based インデックスを使って array の要素を選択します。

例:

```lua
local text = '{"items":[{"name":"alpha"},{"name":"beta"}]}'
local first_name = p.json.get(text, {"items", 1, "name"})
print(first_name)
```

## ptool.json.set

> 未リリース - 導入。

`ptool.json.set(input, path, value)` は JSON テキスト内の指定パスに値を書き込み、更新後の JSON 文字列を返します。

- `input` (string, 必須): JSON テキスト。
- `path` ((string|integer)[], 必須): 空でないパス配列。
- `value` (JSON 互換 Lua 値, 必須): 書き込む値。
- 戻り値: 更新後のコンパクトな JSON 文字列。

挙動:

- 既存の object キーと array 要素は置き換えられます。
- 最後の object キーが存在しない場合は作成されます。
- 存在しない中間 object キーは、次のパス要素も文字列キーの場合に作成されます。
- array は拡張されません。パス内のすべての array インデックスが既に存在している必要があります。
- 想定される object/array 型としてパスを辿れない場合はエラーになります。

例:

```lua
local text = '{"service":{"name":"api"},"ports":[8080]}'

text = p.json.set(text, {"service", "enabled"}, true)
text = p.json.set(text, {"ports", 1}, 9090)

print(text)
```

## ptool.json.stringify

> `v0.3.0` - Introduced.

`ptool.json.stringify(value[, options])` は Lua 値を JSON 文字列へ変換します。

- `value` (JSON 互換 Lua 値, 必須): エンコードする値。
- `options` (table, 任意): シリアライズオプション。
- `options.pretty` (boolean, 任意): `true` のとき見やすく整形された JSON を出力します。デフォルトは `false`。
- 戻り値: エンコードされた JSON 文字列。

挙動:

- デフォルト出力は余分な空白のないコンパクト JSON です。
- pretty 出力ではインデント付きの複数行 JSON を使います。
- 値は JSON 互換である必要があります。`function`, `thread`, `userdata` などのシリアライズできない Lua 値はエラーになります。

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

- Lua テーブル内の `nil` 値は `mlua` の serde 変換挙動に従うため、 JSON オブジェクトのフィールドとして保持されません。
- Lua テーブルが配列かオブジェクトかの判定は `mlua` の serde 変換ルールに 従います。
