# Table API

Table helpers are available under `ptool.tbl` and `p.tbl`.

These APIs are designed for dense list tables with contiguous integer keys
starting at `1`.

## ptool.tbl.map

> `v0.6.0` - Introduced.

`ptool.tbl.map(list, fn)` transforms each item in a list table and returns a
new list.

- `list` (table, required): A dense list table.
- `fn` (function, required): A callback that receives `(value, index)` and must
  return a non-`nil` value.
- Returns: `table`.

Behavior:

- `fn` is called once for each item in order.
- If `fn` returns `nil`, the call raises an error instead of creating holes in
  the result.
- The input table is not modified.

```lua
local out = p.tbl.map({ 10, 20, 30 }, function(value, index)
  return value + index
end)

print(ptool.inspect(out)) -- { 11, 22, 33 }
```

## ptool.tbl.filter

> `v0.6.0` - Introduced.

`ptool.tbl.filter(list, fn)` keeps items whose callback result is truthy and
returns them in a new dense list.

- `list` (table, required): A dense list table.
- `fn` (function, required): A callback that receives `(value, index)`.
- Returns: `table`.

Behavior:

- `nil` and `false` remove the current item.
- Any other Lua value keeps the current item.
- The returned table is reindexed from `1`.

```lua
local out = p.tbl.filter({ "a", "bb", "ccc" }, function(value)
  return #value >= 2
end)

print(ptool.inspect(out)) -- { "bb", "ccc" }
```

## ptool.tbl.any

> `Unreleased` - Introduced.

`ptool.tbl.any(list, fn)` returns `true` when the callback produces a truthy
result for at least one item.

- `list` (table, required): A dense list table.
- `fn` (function, required): A callback that receives `(value, index)`.
- Returns: `boolean`.

Behavior:

- `nil` and `false` are treated as falsy. All other Lua values are treated as
  truthy.
- Iteration stops as soon as a truthy result is found.
- Empty lists return `false`.

```lua
local has_even = p.tbl.any({ 1, 3, 4, 5 }, function(value)
  return value % 2 == 0
end)

print(has_even) -- true
```

## ptool.tbl.all

> `Unreleased` - Introduced.

`ptool.tbl.all(list, fn)` returns `true` only when the callback produces a
truthy result for every item.

- `list` (table, required): A dense list table.
- `fn` (function, required): A callback that receives `(value, index)`.
- Returns: `boolean`.

Behavior:

- `nil` and `false` are treated as falsy. All other Lua values are treated as
  truthy.
- Iteration stops as soon as a falsy result is found.
- Empty lists return `true`.

```lua
local all_short = p.tbl.all({ "a", "bb", "ccc" }, function(value)
  return #value <= 3
end)

print(all_short) -- true
```

## ptool.tbl.find

> `Unreleased` - Introduced.

`ptool.tbl.find(list, fn)` returns the first original list value whose callback
result is truthy.

- `list` (table, required): A dense list table.
- `fn` (function, required): A callback that receives `(value, index)`.
- Returns: `any | nil`.

Behavior:

- `nil` and `false` are treated as falsy. All other Lua values are treated as
  truthy.
- Iteration stops at the first match.
- Empty lists and no-match cases return `nil`.

```lua
local found = p.tbl.find({ 10, 15, 20 }, function(value)
  return value > 12
end)

print(found) -- 15
```

## ptool.tbl.find_index

> `Unreleased` - Introduced.

`ptool.tbl.find_index(list, fn)` returns the 1-based index of the first item
whose callback result is truthy.

- `list` (table, required): A dense list table.
- `fn` (function, required): A callback that receives `(value, index)`.
- Returns: `integer | nil`.

Behavior:

- `nil` and `false` are treated as falsy. All other Lua values are treated as
  truthy.
- Iteration stops at the first match.
- Empty lists and no-match cases return `nil`.

```lua
local index = p.tbl.find_index({ "cat", "dog", "eel" }, function(value)
  return value == "dog"
end)

print(index) -- 2
```

## ptool.tbl.count

> `Unreleased` - Introduced.

`ptool.tbl.count(list, fn)` counts how many items produce a truthy callback
result.

- `list` (table, required): A dense list table.
- `fn` (function, required): A callback that receives `(value, index)`.
- Returns: `integer`.

Behavior:

- `nil` and `false` are treated as falsy. All other Lua values are treated as
  truthy.
- Empty lists return `0`.

```lua
local total = p.tbl.count({ 1, 2, 3, 4, 5 }, function(value)
  return value % 2 == 1
end)

print(total) -- 3
```

## ptool.tbl.includes

> `Unreleased` - Introduced.

`ptool.tbl.includes(list, value)` returns `true` when the list contains the
given value.

- `list` (table, required): A dense list table.
- `value` (any, required): The value to search for.
- Returns: `boolean`.

Behavior:

- The list is scanned from left to right.
- Equality uses normal Lua `==` semantics.
- Empty lists return `false`.

```lua
local has_blue = p.tbl.includes({ "red", "blue", "green" }, "blue")

print(has_blue) -- true
```

## ptool.tbl.reduce

> `Unreleased` - Introduced.

`ptool.tbl.reduce(list, initial, fn)` folds a list into a single value.

- `list` (table, required): A dense list table.
- `initial` (any, required): The explicit initial accumulator value.
- `fn` (function, required): A callback that receives `(acc, value, index)` and
  must return a non-`nil` value.
- Returns: `any`.

Behavior:

- `initial` is always required, even for non-empty lists.
- `fn` is called once for each item in order.
- Each callback result becomes the next accumulator value.
- If `fn` returns `nil`, the call raises an error instead of losing the
  accumulator state.
- Empty lists return `initial` unchanged.

```lua
local total = p.tbl.reduce({ 5, 10, 15 }, 0, function(acc, value)
  return acc + value
end)

print(total) -- 30
```

## ptool.tbl.flat_map

> `Unreleased` - Introduced.

`ptool.tbl.flat_map(list, fn)` maps each item to a dense list table and then
concatenates those returned lists into one new dense list.

- `list` (table, required): A dense list table.
- `fn` (function, required): A callback that receives `(value, index)` and must
  return a dense list table.
- Returns: `table`.

Behavior:

- `fn` is called once for each item in order.
- Each callback result must be a dense list table.
- Returning a non-table or a non-dense list raises an error.
- Empty lists return an empty list.

```lua
local out = p.tbl.flat_map({ 1, 2, 3 }, function(value)
  return { value, value * 10 }
end)

print(ptool.inspect(out)) -- { 1, 10, 2, 20, 3, 30 }
```

## ptool.tbl.concat

> `v0.6.0` - Introduced.

`ptool.tbl.concat(...)` concatenates one or more dense list tables into a new
list.

- `...` (table, required): One or more dense list tables.
- Returns: `table`.

Behavior:

- Arguments are appended from left to right.
- Empty lists are allowed.
- The input tables are not modified.

```lua
local out = p.tbl.concat({ 1, 2 }, { 3 }, {})

print(ptool.inspect(out)) -- { 1, 2, 3 }
```

## ptool.tbl.join

> `v0.6.0` - Introduced.

`ptool.tbl.join(...)` is an alias of `ptool.tbl.concat(...)`.

- `...` (table, required): One or more dense list tables.
- Returns: `table`.

```lua
local out = p.tbl.join({ "x" }, { "y", "z" })

print(ptool.inspect(out)) -- { "x", "y", "z" }
```

## List Rules

`ptool.tbl` currently supports only dense list tables.

- Keys must be integers.
- Keys must start at `1`.
- Keys must be contiguous without gaps.
- Sparse tables and dictionary-style tables raise errors.
