# Redis API

Redis connection helpers are available under `ptool.redis` and `p.redis`.

## ptool.redis.connect

> `v0.9.0` - Introduced.

`ptool.redis.connect(url_or_options)` opens a Redis connection and returns a
`Connection` object.

Arguments:

- `url_or_options` (string|table, required):
  - When a string is provided, it is treated as the Redis URL.
  - When a table is provided, it currently supports:
    - `url` (string, required): The Redis URL.

Common URL examples:

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")
local db1 = ptool.redis.connect("redis://127.0.0.1/1")
local auth = ptool.redis.connect({
  url = "redis://:secret@cache.internal:6379/0",
})
```

Notes:

- The connection is intended for direct command execution through
  `conn:call(...)`.
- This API does not currently provide dedicated pub/sub, pipeline, or
  transaction helpers.

## Connection

> `v0.9.0` - Introduced.

`Connection` represents an open Redis connection returned by
`ptool.redis.connect()`.

It is implemented as a Lua userdata.

Methods:

- `conn:call(command, ...)` -> `any`
- `conn:close()` -> `nil`

Command argument rules:

- `command` must be a non-empty string.
- Remaining arguments are passed to Redis as a flat argument list.
- Supported argument value types are:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil`, tables, functions, threads, and userdata are not supported as Redis
  command arguments.
- Lua strings are passed as raw bytes, so binary-safe Redis values are
  supported.

Reply conversion rules:

- Null replies become `nil`.
- Integer replies become Lua integers.
- Bulk strings, simple strings, and verbatim strings become Lua strings.
- Array and set replies become dense Lua array tables.
- Map replies become Lua tables when the map keys can be represented as Lua
  strings, integers, numbers, or booleans.
- Double replies become Lua numbers.
- Boolean replies become Lua booleans.
- Big number replies become Lua strings.
- Push replies become tables shaped like `{ kind = "...", data = {...} }`.

### call

> `v0.9.0` - Introduced.

Canonical API name: `ptool.redis.Connection:call`.

`conn:call(command, ...)` sends a Redis command and returns the converted
reply.

Example:

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")

redis:call("SET", "user:1:name", "alice")
print(redis:call("GET", "user:1:name")) -- alice

local count = redis:call("INCR", "counters:signup")
print(count)

local values = redis:call("MGET", "user:1:name", "missing")
print(values[1])        -- alice
print(values[2] == nil) -- true
```

### close

> `v0.9.0` - Introduced.

Canonical API name: `ptool.redis.Connection:close`.

`conn:close()` closes the connection.

Behavior:

- After closing, the connection can no longer be used.

