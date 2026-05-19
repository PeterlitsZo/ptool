# テンプレート API

テンプレートレンダリングヘルパーは `ptool.template` と `p.template` に あります。

## ptool.template.render

> `v0.1.0` - Introduced.

`ptool.template.render(template, context)` は Jinja 風テンプレート文字列を レンダリングし、結果の文字列を返します。

- `template` (string, 必須): テンプレートのソーステキスト。
- `context` (任意のシリアライズ可能な Lua 値, 必須): テンプレート コンテキスト。
- 戻り値: レンダリング結果の文字列。

例:

```lua
local template = ptool.unindent([[
  | {% if user.active %}
  | Hello, {{ user.name }}!
  | {% else %}
  | Inactive user: {{ user.name }}
  | {% endif %}
  | Items:
  | {% for item in items %}
  | - {{ item }}
  | {% endfor %}
]])
local result = ptool.template.render(template, {
  user = { name = "alice", active = true },
  items = { "one", "two", "three" },
})

print(result)
```

注意:

- コンテキストはデータ値としてシリアライズ可能である必要があります。
- `function`, `thread`, 未対応の `userdata` などの Lua 値はテンプレート コンテキスト値として使えません。
- 欠けている値は chainable undefined semantics を使います。つまり `foo.bar.baz` のようなネストした参照を `default(...)` のような フィルターへ渡してもエラーになりません。フォールバックなしで直接 レンダリングした場合、undefined 値は空文字列になります。

```lua
local template = ptool.unindent([[
  | {{ foo.bar.baz | default("N/A") }}
]])

print(ptool.template.render(template, {})) -- N/A
```
