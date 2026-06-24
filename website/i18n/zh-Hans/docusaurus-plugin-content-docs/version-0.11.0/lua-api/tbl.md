# Table API

表辅助能力位于 `ptool.tbl` 和 `p.tbl` 下。

这些 API 只面向从 `1` 开始、整数键连续且无空洞的顺序表。

## ptool.tbl.map

> `v0.6.0` - 引入。

`ptool.tbl.map(list, fn)` 对顺序表中的每一项做映射，并返回一个新的顺序表。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`，并且必须返回非 `nil` 的值。
- 返回：`table`。

行为说明：

- `fn` 会按顺序对每一项调用一次。
- 如果 `fn` 返回 `nil`，会直接抛错，而不是在结果里制造空洞。
- 输入表不会被修改。

```lua
local out = p.tbl.map({ 10, 20, 30 }, function(value, index)
  return value + index
end)

print(ptool.inspect(out)) -- { 11, 22, 33 }
```

## ptool.tbl.filter

> `v0.6.0` - 引入。

`ptool.tbl.filter(list, fn)` 保留回调结果为 truthy 的项，并返回新的致密顺序表。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`。
- 返回：`table`。

行为说明：

- 返回 `nil` 或 `false` 会移除当前项。
- 返回其他任意 Lua 值都会保留当前项。
- 返回值会从 `1` 开始重新编号。

```lua
local out = p.tbl.filter({ "a", "bb", "ccc" }, function(value)
  return #value >= 2
end)

print(ptool.inspect(out)) -- { "bb", "ccc" }
```

## ptool.tbl.any

> `v0.10.0` - 引入。

`ptool.tbl.any(list, fn)` 会在回调对至少一项返回 truthy 结果时返回 `true`。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`。
- 返回：`boolean`。

行为说明：

- `nil` 和 `false` 会被视为 falsy，其他所有 Lua 值都会被视为 truthy。
- 一旦找到 truthy 结果，遍历就会停止。
- 空表会返回 `false`。

```lua
local has_even = p.tbl.any({ 1, 3, 4, 5 }, function(value)
  return value % 2 == 0
end)

print(has_even) -- true
```

## ptool.tbl.all

> `v0.10.0` - 引入。

`ptool.tbl.all(list, fn)` 只有在回调对每一项都返回 truthy 结果时才会返回 `true`。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`。
- 返回：`boolean`。

行为说明：

- `nil` 和 `false` 会被视为 falsy，其他所有 Lua 值都会被视为 truthy。
- 一旦找到 falsy 结果，遍历就会停止。
- 空表会返回 `true`。

```lua
local all_short = p.tbl.all({ "a", "bb", "ccc" }, function(value)
  return #value <= 3
end)

print(all_short) -- true
```

## ptool.tbl.find

> `v0.10.0` - 引入。

`ptool.tbl.find(list, fn)` 会返回回调结果为 truthy 的第一项原始值。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`。
- 返回：`any | nil`。

行为说明：

- `nil` 和 `false` 会被视为 falsy，其他所有 Lua 值都会被视为 truthy。
- 找到第一项匹配后就会停止遍历。
- 空表和无匹配时都会返回 `nil`。

```lua
local found = p.tbl.find({ 10, 15, 20 }, function(value)
  return value > 12
end)

print(found) -- 15
```

## ptool.tbl.find_index

> `v0.10.0` - 引入。

`ptool.tbl.find_index(list, fn)` 会返回回调结果为 truthy 的第一项的 1 基索引。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`。
- 返回：`integer | nil`。

行为说明：

- `nil` 和 `false` 会被视为 falsy，其他所有 Lua 值都会被视为 truthy。
- 找到第一项匹配后就会停止遍历。
- 空表和无匹配时都会返回 `nil`。

```lua
local index = p.tbl.find_index({ "cat", "dog", "eel" }, function(value)
  return value == "dog"
end)

print(index) -- 2
```

## ptool.tbl.count

> `v0.10.0` - 引入。

`ptool.tbl.count(list, fn)` 会统计有多少项的回调结果为 truthy。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`。
- 返回：`integer`。

行为说明：

- `nil` 和 `false` 会被视为 falsy，其他所有 Lua 值都会被视为 truthy。
- 空表会返回 `0`。

```lua
local total = p.tbl.count({ 1, 2, 3, 4, 5 }, function(value)
  return value % 2 == 1
end)

print(total) -- 3
```

## ptool.tbl.includes

> `v0.10.0` - 引入。

`ptool.tbl.includes(list, value)` 会在列表包含给定值时返回 `true`。

- `list`（table，必填）：一个致密的顺序表。
- `value`（any，必填）：要查找的值。
- 返回：`boolean`。

行为说明：

- 列表会从左到右扫描。
- 比较使用标准的 Lua `==` 语义。
- 空表会返回 `false`。

```lua
local has_blue = p.tbl.includes({ "red", "blue", "green" }, "blue")

print(has_blue) -- true
```

## ptool.tbl.reduce

> `v0.10.0` - 引入。

`ptool.tbl.reduce(list, initial, fn)` 会把一个列表折叠成单个值。

- `list`（table，必填）：一个致密的顺序表。
- `initial`（any，必填）：显式提供的初始累加器值。
- `fn`（function，必填）：回调函数，接收 `(acc, value, index)`，并且必须返回非 `nil` 的值。
- 返回：`any`。

行为说明：

- 即使列表非空，也必须始终显式传入 `initial`。
- `fn` 会按顺序对每一项调用一次。
- 每次回调的返回值都会成为下一次的累加器值。
- 如果 `fn` 返回 `nil`，调用会直接报错，而不是丢失累加器状态。
- 空表会原样返回 `initial`。

```lua
local total = p.tbl.reduce({ 5, 10, 15 }, 0, function(acc, value)
  return acc + value
end)

print(total) -- 30
```

## ptool.tbl.flat_map

> `v0.10.0` - 引入。

`ptool.tbl.flat_map(list, fn)` 会把每一项映射为一个致密顺序表，再把这些返回的顺序表按顺序拼接成一个新的致密顺序表。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`，并且必须返回一个致密顺序表。
- 返回：`table`。

行为说明：

- `fn` 会按顺序对每一项调用一次。
- 每次回调的返回值都必须是致密顺序表。
- 如果返回的不是表，或者返回的是非致密列表，就会报错。
- 空表会返回空列表。

```lua
local out = p.tbl.flat_map({ 1, 2, 3 }, function(value)
  return { value, value * 10 }
end)

print(ptool.inspect(out)) -- { 1, 10, 2, 20, 3, 30 }
```

## ptool.tbl.concat

> `v0.6.0` - 引入。

`ptool.tbl.concat(...)` 将一个或多个致密顺序表按顺序拼接成新的顺序表。

- `...`（table，必填）：一个或多个致密顺序表。
- 返回：`table`。

行为说明：

- 参数会从左到右依次追加。
- 允许传入空表。
- 输入表不会被修改。

```lua
local out = p.tbl.concat({ 1, 2 }, { 3 }, {})

print(ptool.inspect(out)) -- { 1, 2, 3 }
```

## ptool.tbl.join

> `v0.6.0` - 引入。

`ptool.tbl.join(...)` 是 `ptool.tbl.concat(...)` 的别名。

- `...`（table，必填）：一个或多个致密顺序表。
- 返回：`table`。

```lua
local out = p.tbl.join({ "x" }, { "y", "z" })

print(ptool.inspect(out)) -- { "x", "y", "z" }
```

## 顺序表规则

`ptool.tbl` 当前只支持致密顺序表。

- 键必须是整数。
- 键必须从 `1` 开始。
- 键必须连续，不能有空洞。
- 稀疏表和字典风格的表都会抛错。
