# JSON API

JSON 解析与序列化辅助能力位于 `ptool.json` 和 `p.json` 下。

## ptool.json.parse

> `v0.3.0` - 引入。

`ptool.json.parse(input)` 将 JSON 字符串解析为 Lua 值。

- `input`（string，必填）：JSON 文本。
- 返回：解析后的 Lua 值。根节点可以是任意 JSON 类型。

类型映射：

- JSON object -> Lua table
- JSON array -> Lua 序列表（从 1 开始）
- JSON string -> Lua string
- 可放入 `i64` 的 JSON integer -> Lua integer
- 其他 JSON number -> Lua number
- JSON boolean -> Lua boolean
- JSON null -> Lua `nil`

错误行为：

- 如果 `input` 不是字符串，会抛出错误。
- 如果 JSON 语法有误，会抛出错误，错误信息中包含 `serde_json` 返回的解析细节。

示例：

```lua
local data = p.json.parse('{"name":"ptool","features":["json","repl"],"stars":42}')

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.json.stringify

> `v0.3.0` - 引入。

`ptool.json.stringify(value[, options])` 将 Lua 值编码为 JSON 字符串。

- `value`（兼容 JSON 的 Lua 值，必填）：要编码的值。
- `options`（table，可选）：序列化选项。
- `options.pretty`（boolean，可选）：当为 `true` 时输出带缩进的易读 JSON。
  默认为 `false`。
- 返回：编码后的 JSON 字符串。

行为说明：

- 默认输出紧凑 JSON，不额外插入空白。
- pretty 输出为带缩进的多行 JSON。
- 值必须与 JSON 兼容。函数、thread、userdata 以及其他不可序列化的 Lua 值会报错。

示例：

```lua
local text = p.json.stringify({
  name = "ptool",
  features = {"json", "repl"},
  stable = true,
}, { pretty = true })

print(text)
```

说明：

- Lua table 中的 `nil` 值遵循 `mlua` 的 serde 转换行为，不会以 JSON 对象字段的形式保留下来。
- Lua table 该被视为数组还是对象，遵循 `mlua` 的 serde 转换规则。
