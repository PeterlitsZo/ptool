# JSON API

JSON parsing and serialization helpers are available under `ptool.json` and
`p.json`.

## ptool.json.parse

> `v0.3.0` - Introduced.

`ptool.json.parse(input)` parses a JSON string into a Lua value.

- `input` (string, required): The JSON text.
- Returns: The parsed Lua value. The root can be any JSON type.

Type mapping:

- JSON object -> Lua table
- JSON array -> Lua sequence table (1-based)
- JSON string -> Lua string
- JSON integer that fits in `i64` -> Lua integer
- Other JSON number -> Lua number
- JSON boolean -> Lua boolean
- JSON null -> Lua `nil`

Error behavior:

- An error is raised if `input` is not a string.
- A JSON syntax error raises an error whose message includes the parser detail
  from `serde_json`.

Example:

```lua
local data = p.json.parse('{"name":"ptool","features":["json","repl"],"stars":42}')

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.json.stringify

> `v0.3.0` - Introduced.

`ptool.json.stringify(value[, options])` converts a Lua value to a JSON string.

- `value` (JSON-compatible Lua value, required): The value to encode.
- `options` (table, optional): Serialization options.
- `options.pretty` (boolean, optional): When `true`, output pretty-printed JSON.
  Defaults to `false`.
- Returns: The encoded JSON string.

Behavior:

- Default output is compact JSON with no extra whitespace.
- Pretty output uses indented multi-line JSON.
- Values must be JSON-compatible. Functions, threads, userdata, and other
  non-serializable Lua values raise an error.

Example:

```lua
local text = p.json.stringify({
  name = "ptool",
  features = {"json", "repl"},
  stable = true,
}, { pretty = true })

print(text)
```

Notes:

- `nil` values inside Lua tables follow `mlua`'s serde conversion behavior and
  are not preserved as JSON object fields.
- Array/object detection for Lua tables follows `mlua`'s serde conversion
  rules.
