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

## ptool.json.get

> Unreleased - Introduced.

`ptool.json.get(input, path)` reads the value at a specified path from JSON
text.

- `input` (string, required): The JSON text.
- `path` ((string|integer)[], required): A non-empty path array, such as
  `{"spec", "template", "metadata", "name"}` or `{"items", 1, "name"}`.
- Returns: The corresponding Lua value, or `nil` if the path does not exist or
  cannot be traversed through the expected object or array type.

Behavior:

- String path segments select object keys.
- Integer path segments select array elements using Lua's 1-based indexing.

Example:

```lua
local text = '{"items":[{"name":"alpha"},{"name":"beta"}]}'
local first_name = p.json.get(text, {"items", 1, "name"})
print(first_name)
```

## ptool.json.set

> Unreleased - Introduced.

`ptool.json.set(input, path, value)` writes a value at a specified path in JSON
text and returns the updated JSON string.

- `input` (string, required): The JSON text.
- `path` ((string|integer)[], required): A non-empty path array.
- `value` (JSON-compatible Lua value, required): The value to write.
- Returns: The updated compact JSON string.

Behavior:

- Existing object keys and array elements are replaced.
- A missing final object key is created.
- Missing intermediate object keys are created when the next path segment is
  also a string key.
- Arrays are not expanded. Every array index in the path must already exist.
- An error is raised when the path cannot be traversed through the expected
  object or array type.

Example:

```lua
local text = '{"service":{"name":"api"},"ports":[8080]}'

text = p.json.set(text, {"service", "enabled"}, true)
text = p.json.set(text, {"ports", 1}, 9090)

print(text)
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
