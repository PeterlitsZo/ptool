# HTTP API

HTTP client helpers are available under `ptool.http` and `p.http`.

## ptool.http.request

> `v0.1.0` - Introduced.

`ptool.http.request(options)` sends an HTTP request and returns a `Response`
object.

`options` fields:

- `url` (string, required): The request URL.
- `method` (string, optional): The HTTP method. Defaults to `"GET"`.
- `headers` (table, optional): Request headers, where both keys and values are
  strings.
- `body` (string, optional): The request body.
- `timeout_ms` (integer, optional): Timeout in milliseconds. Defaults to
  `30000`.

Example:

```lua
local resp = ptool.http.request({
  url = "https://httpbin.org/post",
  method = "POST",
  headers = {
    ["content-type"] = "application/json",
  },
  body = [[{"name":"alice"}]],
})

print(resp.status, resp.ok)
local data = resp:json()
print(data.json.name)
```

## Response

> `v0.1.0` - Introduced.

`Response` represents an HTTP response returned by `ptool.http.request(...)`.

Fields:

- `status` (integer): The HTTP status code.
- `ok` (boolean): Whether the status code is in the 2xx range.
- `url` (string): The final URL after redirects.
- `headers` (table): Response headers (`table<string, string>`).

Methods:

- `resp:text()`: Reads and returns the response body as text.
- `resp:json()`: Reads the response body, parses it as JSON, and returns a Lua
  value.
- `resp:bytes()`: Reads and returns the raw bytes (as a Lua string).

### text

Canonical API name: `ptool.http.Response:text`.

`resp:text()` reads and returns the response body as text.

- Returns: `string`.

### json

Canonical API name: `ptool.http.Response:json`.

`resp:json()` reads the response body, parses it as JSON, and returns a Lua
value.

### bytes

Canonical API name: `ptool.http.Response:bytes`.

`resp:bytes()` reads and returns the response body as raw bytes.

- Returns: `string`.

Notes:

- Non-2xx HTTP statuses do not raise errors. Callers should check `resp.ok`
  themselves.
- The body can only be consumed once. Calling any of `text`, `json`, or `bytes`
  more than once raises an error.
