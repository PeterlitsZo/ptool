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
- A TOML syntax error raises an error whose message includes line and column
  information.

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

`ptool.toml.get(input, path)` reads the value at a specified path from TOML
text.

- `input` (string, required): The TOML text.
- `path` (string[], required): A non-empty path array, such as `{"package",
  "version"}`.
- Returns: The corresponding Lua value, or `nil` if the path does not exist.

Example:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)
```

## ptool.toml.set

> `v0.1.0` - Introduced.

`ptool.toml.set(input, path, value)` sets the value at a specified path and
returns the updated TOML text.

- `input` (string, required): The TOML text.
- `path` (string[], required): A non-empty path array, such as `{"package",
  "version"}`.
- `value` (string|integer|number|boolean, required): The value to write.
- Returns: The updated TOML string.

Behavior:

- Missing intermediate paths are created automatically as tables.
- If an intermediate path exists but is not a table, an error is raised.
- Parsing and writing back are based on `toml_edit`, which preserves original
  comments and formatting as much as possible.

Example:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.set(text, {"package", "version"}, "0.2.0")
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.remove

> `v0.1.0` - Introduced.

`ptool.toml.remove(input, path)` removes the specified path and returns the
updated TOML text.

- `input` (string, required): The TOML text.
- `path` (string[], required): A non-empty path array, such as `{"package",
  "name"}`.
- Returns: The updated TOML string.

Behavior:

- If the path does not exist, no error is raised and the original text (or an
  equivalent form) is returned.
- If an intermediate path exists but is not a table, an error is raised.

Example:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.remove(text, {"package", "description"})
ptool.fs.write("Cargo.toml", updated)
```

Notes:

- The `path` argument for `ptool.toml.get/set/remove` must be a non-empty string
  array.
- `set` currently supports writing only scalar types
  (`string`/`integer`/`number`/`boolean`).
