# YAML API

YAML parsing and serialization helpers are available under `ptool.yaml` and
`p.yaml`.

## ptool.yaml.parse

> `v0.4.0` - Introduced.

`ptool.yaml.parse(input)` parses a YAML string into a Lua value.

- `input` (string, required): The YAML text.
- Returns: The parsed Lua value. The root can be any supported YAML type.

Type mapping:

- YAML mapping -> Lua table
- YAML sequence -> Lua sequence table (1-based)
- YAML string -> Lua string
- YAML integer that fits in `i64` -> Lua integer
- Other YAML number -> Lua number
- YAML boolean -> Lua boolean
- YAML null -> Lua `nil`

Error behavior:

- An error is raised if `input` is not a string.
- A YAML syntax error raises an error whose message includes parser detail.
- An error is raised if the YAML value cannot be represented as a Lua value in
  `ptool`, such as a mapping with non-string keys or an explicit YAML tag.

Example:

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

> `v0.4.0` - Introduced.

`ptool.yaml.get(input, path)` reads the value at a specified path from YAML
text.

- `input` (string, required): The YAML text.
- `path` ((string|integer)[], required): A non-empty path array, such as
  `{"spec", "template", "metadata", "name"}` or `{"items", 1, "name"}`.
- Returns: The corresponding Lua value, or `nil` if the path does not exist.

Behavior:

- String path segments select mapping keys.
- Integer path segments select sequence elements using Lua's 1-based indexing.

Example:

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

> `v0.4.0` - Introduced.

`ptool.yaml.stringify(value)` converts a Lua value to YAML text.

- `value` (YAML-compatible Lua value, required): The value to encode.
- Returns: The encoded YAML string.

Behavior:

- Values must be YAML-compatible through the same Lua value mapping used by
  `ptool.json.stringify`.
- Lua sequence tables are encoded as YAML sequences.
- Lua string-keyed tables are encoded as YAML mappings.

Example:

```lua
local text = p.yaml.stringify({
  project = "ptool",
  features = {"yaml", "lua"},
  stable = true,
})

print(text)
```

Notes:

- Only single-document YAML is supported.
- YAML mappings must use string keys.
- Explicit YAML tags are not supported.
- The `path` argument for `ptool.yaml.get` must be a non-empty array of strings
  and/or positive integers.
- Integer path segments are 1-based so they match Lua array indexing.
