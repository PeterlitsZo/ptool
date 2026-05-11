# Table API

Table helpers are available under `ptool.tbl` and `p.tbl`.

These APIs are designed for dense list tables with contiguous integer keys
starting at `1`.

## ptool.tbl.map

> `Unreleased` - Introduced.

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

> `Unreleased` - Introduced.

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

## ptool.tbl.concat

> `Unreleased` - Introduced.

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

> `Unreleased` - Introduced.

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
