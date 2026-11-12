# Consul API

Consul KV helpers are available under `ptool.consul` and `p.consul`.

## ptool.consul.connect

> `v0.10.0` - 引入。

`ptool.consul.connect(options)` opens a Consul connection and returns a `Connection` object.

`options` fields:

- `address` (string, optional): The Consul HTTP address such as `http://127.0.0.1:8500` or `127.0.0.1:8500`.
- `token` (string, optional): The Consul ACL token sent through the `x-consul-token` header.
- `datacenter` (string, optional): Default `dc` query parameter for requests sent through this connection.
- `timeout_ms` (integer, optional): Default request timeout in milliseconds.

Environment fallback:

- `address` falls back to `CONSUL_HTTP_ADDR`.
- `token` falls back to `CONSUL_HTTP_TOKEN`.
- Environment fallback uses `ptool`'s runtime environment view, so values set through `p.os.setenv(...)` are also visible to `ptool.consul.connect(...)`.

Example:

```lua
local consul = ptool.consul.connect({
  address = "127.0.0.1:8500",
  token = p.os.getenv("CONSUL_HTTP_TOKEN"),
  datacenter = "dc1",
  timeout_ms = 10_000,
})
```

## Connection

> `v0.10.0` - 引入。

`Connection` represents an open Consul connection returned by `ptool.consul.connect()`.

It is implemented as a Lua userdata.

Methods:

- `conn:get(key[, options])` -> `table | nil`
- `conn:put(key, value[, options])` -> `boolean`
- `conn:delete(key[, options])` -> `boolean`
- `conn:list(prefix[, options])` -> `table`
- `conn:request(options)` -> `Response`

KV entry table shape:

- `key` (string): The Consul key.
- `value` (string | nil): The decoded raw value as a Lua string.
- `flags` (integer): The Consul KV flags value.
- `create_index` (integer): The create index.
- `modify_index` (integer): The modify index.
- `lock_index` (integer): The lock index.
- `session` (string | nil): The lock session when present.
- `namespace` (string | nil): The Consul Enterprise namespace when present.
- `partition` (string | nil): The Consul Enterprise partition when present.

Read options:

- `consistent` (boolean, optional): Adds the `consistent` query flag.
- `stale` (boolean, optional): Adds the `stale` query flag.
- `wait_index` (integer, optional): Adds the blocking-query `index` parameter.
- `wait_time` (string, optional): Adds the blocking-query `wait` parameter such as `"30s"` or `"5m"`.

### get

> `v0.10.0` - 引入。

Canonical API name: `ptool.consul.Connection:get`.

`conn:get(key[, options])` reads a single Consul KV entry.

- `key` (string, required): The Consul key.
- `options` (table, optional): Read options.
- Returns: `table | nil`.

Behavior:

- Returns `nil` when the key does not exist.
- The returned `value` field is the decoded raw bytes as a Lua string.

Example:

```lua
local consul = ptool.consul.connect({ address = "127.0.0.1:8500" })
local item = consul:get("apps/api/config.json")

if item then
  local config = p.json.parse(item.value)
  print(item.modify_index, config.port)
end
```

### put

> `v0.10.0` - 引入。

Canonical API name: `ptool.consul.Connection:put`.

`conn:put(key, value[, options])` writes a Consul KV entry.

- `key` (string, required): The Consul key.
- `value` (string, required): The raw bytes to store.
- `options` (table, optional):
  - `flags` (integer, optional): Consul KV flags.
  - `cas` (integer, optional): Check-and-set modify index.
- Returns: `boolean`.

Behavior:

- Returns the boolean result produced by Consul.

Example:

```lua
local consul = ptool.consul.connect({ address = "127.0.0.1:8500" })
local item = consul:get("apps/api/version")

local ok = consul:put("apps/api/version", "v2\n", {
  cas = item and item.modify_index or 0,
})

print(ok)
```

### delete

> `v0.10.0` - 引入。

Canonical API name: `ptool.consul.Connection:delete`.

`conn:delete(key[, options])` deletes a Consul KV entry or subtree.

- `key` (string, required): The Consul key or prefix.
- `options` (table, optional):
  - `cas` (integer, optional): Check-and-set modify index.
  - `recurse` (boolean, optional): When `true`, delete the whole subtree.
- Returns: `boolean`.

Notes:

- `cas` and `recurse` are mutually exclusive.

### list

> `v0.10.0` - 引入。

Canonical API name: `ptool.consul.Connection:list`.

`conn:list(prefix[, options])` lists Consul KV entries under a prefix.

- `prefix` (string, required): The prefix to list.
- `options` (table, optional): Read options.
- Returns: `table`.

Behavior:

- Returns a dense Lua array of KV entry tables.
- Returns an empty array when the prefix does not exist.

Example:

```lua
local consul = ptool.consul.connect({ address = "127.0.0.1:8500" })
local items = consul:list("apps/api/")

for _, item in ipairs(items) do
  print(item.key, item.modify_index)
end
```

### request

> `v0.10.0` - 引入。

Canonical API name: `ptool.consul.Connection:request`.

`conn:request(options)` sends a raw HTTP request through the current Consul connection defaults and returns the same `Response` object shape as `ptool.http.request(...)`.

`options` supports the same request fields as `ptool.http.request(options)`, plus:

- `path` (string, optional): A Consul-relative path such as `"/v1/status/leader"`.
- `url` (string, optional): An explicit absolute URL.

Notes:

- Exactly one of `path` or `url` is required.
- Connection defaults automatically add `x-consul-token`, `dc`, and `timeout_ms` when they are configured and not overridden by the request.

Example:

```lua
local consul = ptool.consul.connect({ address = "127.0.0.1:8500" })
local resp = consul:request({
  path = "/v1/status/leader",
  fail_on_http_error = true,
})

print(resp:text())
```
