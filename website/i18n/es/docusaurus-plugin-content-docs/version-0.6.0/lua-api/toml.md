# TOML API

TOML parsing and editing helpers are available under `ptool.toml` and `p.toml`.

## ptool.toml.parse

> `v0.1.0` - Introduced.

`ptool.toml.parse(input)` parses a TOML string into a Lua table.

- `input` (string, required): The TOML text.
- Returns: A Lua table (the root node is always a table).

Type mapping:

- TOML table / inline table -> Lua table
- TOML array -> Lua sequence table (1-based)
- TOML string -> Lua string
- TOML integer -> Lua integer
- TOML float -> Lua number
- TOML boolean -> Lua boolean
- TOML datetime/date/time -> Lua string

Error behavior:

- An error is raised if `input` is not a string.
- A TOML syntax error raises an error whose message includes line and column information.

Example:

```lua
local text = ptool.fs.read("ptool.toml")
local conf = ptool.toml.parse(text)

print(conf.project.name)
print(conf.build.jobs)
print(conf.release_date) -- datetime/date/time values are strings
```

## ptool.toml.get

> `v0.1.0` - Introduced.
> 
> `v0.4.0` - Added numeric path segments for array indexing.

`ptool.toml.get(input, path)` reads the value at a specified path from TOML text.

- `input` (string, required): The TOML text.
- `path` ((string|integer)[], required): A non-empty path array, such as `{"package", "version"}` or `{"bin", 1, "name"}`.
- Returns: The corresponding Lua value, or `nil` if the path does not exist.

Behavior:

- String path segments select table keys.
- Integer path segments select array elements using Lua's 1-based indexing.

Example:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)

local first_bin_name = ptool.toml.get(text, {"bin", 1, "name"})
print(first_bin_name)
```

## ptool.toml.set

> `v0.1.0` - Introduced.
> 
> `v0.4.0` - Added composite value writes and numeric path segments.

`ptool.toml.set(input, path, value)` sets the value at a specified path and returns the updated TOML text.

- `input` (string, required): The TOML text.
- `path` ((string|integer)[], required): A non-empty path array, such as `{"package", "version"}` or `{"bin", 1, "name"}`.
- `value` (string|integer|number|boolean|table, required): The value to write.
- Returns: The updated TOML string.

Behavior:

- Missing intermediate paths are created automatically as tables.
- If an intermediate path exists but is not a table, an error is raised.
- Lua tables with only string keys are written as TOML tables.
- Lua sequence tables are written as TOML arrays.
- A Lua sequence of string-keyed tables is written as a TOML array of tables.
- Empty Lua tables are currently written as TOML tables.
- Parsing and writing back are based on `toml_edit`, which preserves original comments and formatting as much as possible.

Example:

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

> `v0.1.0` - Introduced.
> 
> `v0.4.0` - Added numeric path segments for array indexing.

`ptool.toml.remove(input, path)` removes the specified path and returns the updated TOML text.

- `input` (string, required): The TOML text.
- `path` ((string|integer)[], required): A non-empty path array, such as `{"package", "name"}` or `{"bin", 1}`.
- Returns: The updated TOML string.

Behavior:

- If the path does not exist, no error is raised and the original text (or an equivalent form) is returned.
- If an intermediate path exists but is not a table, an error is raised.

Example:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.remove(text, {"package", "description"})
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.stringify

> `v0.4.0` - Introduced.

`ptool.toml.stringify(value)` converts a Lua value to TOML text.

- `value` (table, required): The root TOML table to encode.
- Returns: The encoded TOML string.

Behavior:

- The root value must be a Lua table representing a TOML table.
- Nested Lua tables follow the same table/array rules as `ptool.toml.set`.
- Empty Lua tables are currently encoded as TOML tables.

Example:

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

Notes:

- The `path` argument for `ptool.toml.get/set/remove` must be a non-empty array of strings and/or positive integers.
- Integer path segments are 1-based so they match Lua array indexing.
- TOML datetime/date/time values are still read back as Lua strings.
