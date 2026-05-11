# YAML API

YAML 解析与序列化辅助能力位于 `ptool.yaml` 和 `p.yaml` 下。

## ptool.yaml.parse

> `v0.4.0` - 引入。

`ptool.yaml.parse(input)` 将 YAML 字符串解析为 Lua 值。

- `input`（string，必填）：YAML 文本。
- 返回：解析后的 Lua 值。根节点可以是任意受支持的 YAML 类型。

类型映射：

- YAML mapping -> Lua table
- YAML sequence -> Lua 序列表（从 1 开始）
- YAML string -> Lua string
- 可放入 `i64` 的 YAML integer -> Lua integer
- 其他 YAML number -> Lua number
- YAML boolean -> Lua boolean
- YAML null -> Lua `nil`

错误行为：

- 如果 `input` 不是字符串，会抛出错误。
- 如果 YAML 语法有误，会抛出错误，错误信息中包含解析器细节。
- 如果 YAML 值无法表示为 `ptool` 支持的 Lua 值，也会抛出错误，例如 非字符串 key 的 mapping，或带显式 YAML tag 的值。

示例：

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

> `v0.4.0` - 引入。

`ptool.yaml.get(input, path)` 从 YAML 文本中读取指定路径上的值。

- `input`（string，必填）：YAML 文本。
- `path`（(string|integer)[]，必填）：非空路径数组，例如 `{"spec", "template", "metadata", "name"}` 或 `{"items", 1, "name"}`。
- 返回：对应的 Lua 值；如果路径不存在，则返回 `nil`。

行为说明：

- 字符串路径段用于选择 mapping key。
- 整数路径段用于选择 sequence 元素，使用 Lua 的 1-based 索引。

示例：

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

> `v0.4.0` - 引入。

`ptool.yaml.stringify(value)` 将 Lua 值编码为 YAML 文本。

- `value`（兼容 YAML 的 Lua 值，必填）：要编码的值。
- 返回：编码后的 YAML 字符串。

行为说明：

- 值必须能通过与 `ptool.json.stringify` 相同的 Lua 值映射表示为 YAML。
- Lua 序列表会编码为 YAML sequence。
- 仅包含字符串 key 的 Lua table 会编码为 YAML mapping。

示例：

```lua
local text = p.yaml.stringify({
  project = "ptool",
  features = {"yaml", "lua"},
  stable = true,
})

print(text)
```

说明：

- 目前只支持单文档 YAML。
- YAML mapping 的 key 必须是字符串。
- 不支持显式 YAML tag。
- `ptool.yaml.get` 的 `path` 参数必须是由字符串和/或正整数构成的非空数组。
- 整数路径段为 1-based，以与 Lua 数组索引保持一致。
