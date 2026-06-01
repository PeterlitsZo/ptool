# Etcd API

Etcd v3 KV helpers are available under `ptool.etcd` and `p.etcd`.

## ptool.etcd.connect

> `v0.11.0` - Introduced.

`ptool.etcd.connect(options)` opens an etcd connection and returns a `Connection` object.

`options` fields:

- `address` (string, required): The etcd HTTP address such as `http://127.0.0.1:2379` or `127.0.0.1:2379`.
- `token` (string, optional): Authorization header value sent with requests.
- `username` (string, optional): etcd auth username.
- `password` (string, optional): etcd auth password.
- `api_prefix` (string, optional): API prefix appended under `address`. Defaults to `"v3"`, which targets the standard v3 gRPC-gateway endpoints.
- `timeout_ms` (integer, optional): Default request timeout in milliseconds.

Notes:

- `token` cannot be combined with `username` or `password`.
- `username` and `password` must be provided together.
- When `username` and `password` are used, `ptool.etcd.connect(...)` authenticates through `auth/authenticate` and stores the returned token on the connection.

Example:

```lua
local etcd = ptool.etcd.connect({
  address = "127.0.0.1:2379",
  username = "root",
  password = p.os.getenv("ETCD_PASSWORD"),
  timeout_ms = 10_000,
})
```

## Connection

> `v0.11.0` - Introduced.

`Connection` represents an open etcd connection returned by `ptool.etcd.connect()`.

It is implemented as a Lua userdata.

Methods:

- `conn:get(key[, options])` -> `table | nil`
- `conn:put(key, value[, options])` -> `table`
- `conn:delete(key[, options])` -> `table`
- `conn:list([prefix[, options]])` -> `table`
- `conn:request(options)` -> `Response`

KV entry table shape:

- `key` (string): The decoded raw key bytes as a Lua string.
- `value` (string): The decoded raw value bytes as a Lua string.
- `create_revision` (integer): The key creation revision.
- `mod_revision` (integer): The last modification revision.
- `version` (integer): The number of times the key has been modified.
- `lease` (integer): The attached lease ID, or `0` when no lease is attached.

Response header table shape:

- `cluster_id` (string): The etcd cluster ID rendered as a decimal string.
- `member_id` (string): The responding member ID rendered as a decimal string.
- `revision` (integer): The store revision for the response.
- `raft_term` (string): The raft term rendered as a decimal string.

### get

> `v0.11.0` - Introduced.

Canonical API name: `ptool.etcd.Connection:get`.

`conn:get(key[, options])` reads a single etcd KV entry.

- `key` (string, required): The raw key bytes.
- `options` (table, optional):
  - `revision` (integer, optional): Reads the key at a specific revision.
  - `serializable` (boolean, optional): Uses serializable reads instead of linearizable reads.
- Returns: `table | nil`.

Behavior:

- Returns `nil` when the key does not exist.
- `key` and `value` are binary-safe Lua strings.

Example:

```lua
local etcd = ptool.etcd.connect({ address = "127.0.0.1:2379" })
local item = etcd:get("/apps/api/config.json")

if item then
  local config = p.json.parse(item.value)
  print(item.mod_revision, config.port)
end
```

### put

> `v0.11.0` - Introduced.

Canonical API name: `ptool.etcd.Connection:put`.

`conn:put(key, value[, options])` writes an etcd KV entry.

- `key` (string, required): The raw key bytes.
- `value` (string, required): The raw value bytes.
- `options` (table, optional):
  - `lease` (integer, optional): Lease ID to attach to the key.
  - `prev_kv` (boolean, optional): When `true`, include the previous KV entry.
  - `ignore_value` (boolean, optional): Reuse the current value.
  - `ignore_lease` (boolean, optional): Reuse the current lease.
- Returns: `table`.

Return table shape:

- `header` (table | nil): Response header metadata.
- `prev_kv` (table | nil): Previous KV entry when `prev_kv = true` and a previous key existed.

Example:

```lua
local etcd = ptool.etcd.connect({ address = "127.0.0.1:2379" })
local resp = etcd:put("/apps/api/version", "v2\n", {
  prev_kv = true,
})

if resp.prev_kv then
  print(resp.prev_kv.value)
end
```

### delete

> `v0.11.0` - Introduced.

Canonical API name: `ptool.etcd.Connection:delete`.

`conn:delete(key[, options])` deletes one key or a whole key prefix.

- `key` (string, required): The raw key bytes. When `prefix = true`, an empty string deletes the full keyspace.
- `options` (table, optional):
  - `prefix` (boolean, optional): When `true`, delete the full prefix range.
  - `prev_kv` (boolean, optional): When `true`, include deleted KV entries.
- Returns: `table`.

Return table shape:

- `header` (table | nil): Response header metadata.
- `deleted` (integer): Number of deleted keys.
- `prev_kvs` (table): Dense Lua array of deleted KV entries.

### list

> `v0.11.0` - Introduced.

Canonical API name: `ptool.etcd.Connection:list`.

`conn:list([prefix[, options]])` lists etcd KV entries under a prefix.

- `prefix` (string, optional): The prefix to list. When omitted or empty, the whole keyspace is scanned.
- `options` (table, optional):
  - `limit` (integer, optional): Maximum number of returned entries.
  - `revision` (integer, optional): Reads at a specific revision.
  - `serializable` (boolean, optional): Uses serializable reads.
  - `keys_only` (boolean, optional): Return keys without values.
  - `count_only` (boolean, optional): Return only the count.
  - `min_mod_revision` (integer, optional): Minimum modification revision.
  - `max_mod_revision` (integer, optional): Maximum modification revision.
  - `min_create_revision` (integer, optional): Minimum creation revision.
  - `max_create_revision` (integer, optional): Maximum creation revision.
  - `sort_order` (string, optional): One of `"none"`, `"ascend"`, or `"descend"`.
  - `sort_target` (string, optional): One of `"key"`, `"version"`, `"create"`, `"mod"`, or `"value"`.
- Returns: `table`.

Return table shape:

- `header` (table | nil): Response header metadata.
- `kvs` (table): Dense Lua array of KV entries.
- `count` (integer): Number of matched keys reported by etcd.
- `more` (boolean): Whether more results remain on the server.

Example:

```lua
local etcd = ptool.etcd.connect({ address = "127.0.0.1:2379" })
local resp = etcd:list("/apps/api/", {
  keys_only = true,
  sort_order = "ascend",
  sort_target = "key",
})

for _, item in ipairs(resp.kvs) do
  print(item.key)
end
```

### request

> `v0.11.0` - Introduced.

Canonical API name: `ptool.etcd.Connection:request`.

`conn:request(options)` sends a raw HTTP request through the current etcd connection defaults and returns the same `Response` object shape as `ptool.http.request(...)`.

`options` supports the same request fields as `ptool.http.request(options)`, plus:

- `path` (string, optional): An etcd-relative path such as `"/kv/txn"` or `"/v3/kv/txn"`.
- `url` (string, optional): An explicit absolute URL.

Notes:

- Exactly one of `path` or `url` is required.
- Connection defaults automatically add the configured authorization header and `timeout_ms` when they are not overridden by the request.
- Relative `path` values are resolved against the configured `api_prefix`.

Example:

```lua
local etcd = ptool.etcd.connect({ address = "127.0.0.1:2379" })
local resp = etcd:request({
  path = "/kv/txn",
  json = {
    compare = {},
    success = {},
    failure = {},
  },
  fail_on_http_error = true,
})

print(resp:text())
```
