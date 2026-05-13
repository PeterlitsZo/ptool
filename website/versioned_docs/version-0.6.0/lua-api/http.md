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
- `query` (table, optional): Query parameters appended to the request URL. Keys
  must be strings. Values may be strings, numbers, or booleans.
- `body` (string, optional): The request body.
- `json` (Lua value, optional): A Lua value encoded as JSON and used as the
  request body. When `content-type` is not already set, it defaults to
  `application/json`.
- `form` (table, optional): Form fields encoded as
  `application/x-www-form-urlencoded` and used as the request body. Keys must
  be strings. Values may be strings, numbers, or booleans.
- `timeout_ms` (integer, optional): Timeout in milliseconds. Defaults to
  `30000`.
- `connect_timeout_ms` (integer, optional): Connection timeout in milliseconds.
- `follow_redirects` (boolean, optional): Whether redirects should be followed.
- `max_redirects` (integer, optional): Maximum number of redirects to follow.
- `user_agent` (string, optional): Sets the `user-agent` request header.
- `basic_auth` (table, optional): HTTP basic auth credentials with string
  fields `username` and `password`.
- `bearer_token` (string, optional): Bearer token used for the
  `authorization` header.
- `fail_on_http_error` (boolean, optional): When `true`, raise an error for 4xx
  and 5xx HTTP responses. Defaults to `false`.

Notes:

- `body`, `json`, and `form` are mutually exclusive.
- `basic_auth` and `bearer_token` are mutually exclusive.

Example:

```lua
local resp = ptool.http.request({
  url = "https://httpbin.org/post",
  method = "POST",
  query = {
    page = 1,
    draft = false,
  },
  json = {
    name = "alice",
    tags = {"admin", "beta"},
  },
  user_agent = "ptool-script/1.0",
  fail_on_http_error = true,
})

print(resp.status, resp.ok)
print(resp:header("content-type"))
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
- `headers` (table): A flattened convenience view of response headers
  (`table<string, string>`). Repeated headers are merged with `, `.

Methods:

- `resp:text()`: Reads and returns the response body as text.
- `resp:json()`: Reads the response body, parses it as JSON, and returns a Lua
  value.
- `resp:bytes()`: Reads and returns the raw bytes (as a Lua string).
- `resp:header(name)`: Returns the first matching response header value, or
  `nil`.
- `resp:header_values(name)`: Returns all matching response header values as an
  array.
- `resp:raise_for_status()`: Raises an error for 4xx and 5xx HTTP responses.

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

### header

Canonical API name: `ptool.http.Response:header`.

`resp:header(name)` returns the first response header value matching `name`.

- `name` (string, required): The header name to look up.
- Returns: `string | nil`.

### header_values

Canonical API name: `ptool.http.Response:header_values`.

`resp:header_values(name)` returns all response header values matching `name`.

- `name` (string, required): The header name to look up.
- Returns: `string[]`.

### raise_for_status

Canonical API name: `ptool.http.Response:raise_for_status`.

`resp:raise_for_status()` raises an error when the response status code is in
the 4xx or 5xx range.

Notes:

- Non-2xx HTTP statuses do not raise errors by default. Callers can check
  `resp.ok`, set `fail_on_http_error = true`, or call `resp:raise_for_status()`.
- The response body is cached after the first read. Calling `text`, `json`, and
  `bytes` multiple times is allowed.
