# Table API

表辅助能力位于 `ptool.tbl` 和 `p.tbl` 下。

这些 API 只面向从 `1` 开始、整数键连续且无空洞的顺序表。

## ptool.tbl.map

> `Unreleased` - 引入。

`ptool.tbl.map(list, fn)` 对顺序表中的每一项做映射，并返回一个新的顺序表。

- `list`（table，必填）：一个致密的顺序表。
- `fn`（function，必填）：回调函数，接收 `(value, index)`，并且必须返回非 `nil`
  的值。
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

> `Unreleased` - 引入。

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

## ptool.tbl.concat

> `Unreleased` - 引入。

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

> `Unreleased` - 引入。

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
