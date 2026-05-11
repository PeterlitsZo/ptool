# TOML API

TOML 解析与编辑辅助能力位于 `ptool.toml` 和 `p.toml` 下。

## ptool.toml.parse

> `v0.1.0` - 引入。

`ptool.toml.parse(input)` 将 TOML 字符串解析为 Lua 表。

- `input`（string，必填）：TOML 文本。
- 返回：Lua 表（根节点始终是 table）。

类型映射：

- TOML table / inline table -> Lua table
- TOML array -> Lua 序列表（从 1 开始）
- TOML string -> Lua string
- TOML integer -> Lua integer
- TOML float -> Lua number
- TOML boolean -> Lua boolean
- TOML datetime/date/time -> Lua string

错误行为：

- 如果 `input` 不是字符串，会抛出错误。
- 如果 TOML 语法有误，会抛出错误，错误信息中包含行号和列号。

示例：

```lua
local text = ptool.fs.read("ptool.toml")
local conf = ptool.toml.parse(text)

print(conf.project.name)
print(conf.build.jobs)
print(conf.release_date) -- datetime/date/time values are strings
```

## ptool.toml.get

> `v0.1.0` - 引入。
>
> `v0.4.0` - 新增数组索引用的数字路径段。

`ptool.toml.get(input, path)` 从 TOML 文本中读取指定路径上的值。

- `input`（string，必填）：TOML 文本。
- `path`（(string|integer)[]，必填）：非空路径数组，例如
  `{"package", "version"}` 或 `{"bin", 1, "name"}`。
- 返回：对应的 Lua 值；如果路径不存在，则返回 `nil`。

行为说明：

- 字符串路径段用于选择 table 键。
- 整数路径段用于选择数组元素，采用 Lua 的 1-based 索引。

示例：

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)

local first_bin_name = ptool.toml.get(text, {"bin", 1, "name"})
print(first_bin_name)
```

## ptool.toml.set

> `v0.1.0` - 引入。
>
> `v0.4.0` - 新增复合值写入和数字路径段。

`ptool.toml.set(input, path, value)` 设置指定路径上的值，并返回更新后的 TOML
文本。

- `input`（string，必填）：TOML 文本。
- `path`（(string|integer)[]，必填）：非空路径数组，例如
  `{"package", "version"}` 或 `{"bin", 1, "name"}`。
- `value`（string|integer|number|boolean|table，必填）：要写入的值。
- 返回：更新后的 TOML 字符串。

行为说明：

- 缺失的中间路径会自动创建为 table。
- 如果某个中间路径已存在但不是 table，会抛出错误。
- 仅包含字符串键的 Lua table 会写成 TOML table。
- Lua 序列表会写成 TOML array。
- 由“仅包含字符串键的 Lua table”组成的序列表会写成 TOML array of tables。
- 空 Lua table 目前会按 TOML table 写入。
- 解析和回写基于 `toml_edit`，会尽可能保留原始注释和格式。

示例：

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

> `v0.1.0` - 引入。
>
> `v0.4.0` - 新增数组索引用的数字路径段。

`ptool.toml.remove(input, path)` 删除指定路径，并返回更新后的 TOML 文本。

- `input`（string，必填）：TOML 文本。
- `path`（(string|integer)[]，必填）：非空路径数组，例如
  `{"package", "name"}` 或 `{"bin", 1}`。
- 返回：更新后的 TOML 字符串。

行为说明：

- 如果路径不存在，不会报错，而是返回原始文本或其等价形式。
- 如果某个中间路径已存在但不是 table，会抛出错误。

示例：

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.remove(text, {"package", "description"})
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.stringify

> `v0.4.0` - 引入。

`ptool.toml.stringify(value)` 将 Lua 值编码为 TOML 文本。

- `value`（table，必填）：要编码的根 TOML table。
- 返回：编码后的 TOML 字符串。

行为说明：

- 根值必须是表示 TOML table 的 Lua table。
- 嵌套 Lua table 与 `ptool.toml.set` 使用相同的 table/array 判定规则。
- 空 Lua table 目前会编码为 TOML table。

示例：

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

说明：

- `ptool.toml.get/set/remove` 的 `path` 参数必须是由字符串和/或正整数构成的非空数组。
- 整数路径段是 1-based 的，与 Lua 数组索引保持一致。
- TOML datetime/date/time 值目前仍会读取为 Lua string。
