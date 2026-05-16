# HTTP API

HTTP 客户端辅助能力位于 `ptool.http` 和 `p.http` 下。

## ptool.http.request

> `v0.1.0` - 引入。

`ptool.http.request(options)` 发送一个 HTTP 请求，并返回 `Response` 对象。

如果要从远程 SSH 主机发出相同形状的请求，请使用 `ptool.ssh.Connection:http_request(options)`。

`options` 字段：

- `url`（string，必填）：请求 URL。
- `method`（string，可选）：HTTP 方法。默认值为 `"GET"`。
- `headers`（table，可选）：请求头，键和值都必须是字符串。
- `query`（table，可选）：追加到请求 URL 的查询参数。键必须是字符串，值可以 是字符串、数字或布尔值。
- `body`（string，可选）：请求体。
- `json`（Lua 值，可选）：会被编码为 JSON 并作为请求体发送。当未显式设置 `content-type` 时，默认使用 `application/json`。
- `form`（table，可选）：会被编码为 `application/x-www-form-urlencoded` 并作为请求体发送。键必须是字符串，值 可以是字符串、数字或布尔值。
- `timeout_ms`（integer，可选）：超时时间，单位毫秒。默认值为 `30000`。
- `connect_timeout_ms`（integer，可选）：连接超时时间，单位毫秒。
- `follow_redirects`（boolean，可选）：是否跟随重定向。
- `max_redirects`（integer，可选）：允许跟随的最大重定向次数。
- `user_agent`（string，可选）：设置 `user-agent` 请求头。
- `basic_auth`（table，可选）：HTTP Basic 认证信息，包含字符串字段 `username` 和 `password`。
- `bearer_token`（string，可选）：用于 `authorization` 请求头的 Bearer Token。
- `fail_on_http_error`（boolean，可选）：为 `true` 时，4xx 和 5xx HTTP 响应会直接抛错。默认值为 `false`。

说明：

- `body`、`json` 和 `form` 互斥。
- `basic_auth` 和 `bearer_token` 互斥。

示例：

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

> `v0.1.0` - 引入。

`Response` 表示 `ptool.http.request(...)` 返回的 HTTP 响应。

字段：

- `status`（integer）：HTTP 状态码。
- `ok`（boolean）：状态码是否位于 2xx 范围内。
- `url`（string）：重定向后的最终 URL。
- `headers`（table）：响应头的扁平化便捷视图（`table<string, string>`）。 重复响应头会使用 `, ` 合并。

方法：

- `resp:text()`：读取并以文本形式返回响应体。
- `resp:json()`：读取响应体，按 JSON 解析后返回 Lua 值。
- `resp:bytes()`：读取并返回原始字节（作为 Lua 字符串）。
- `resp:header(name)`：返回第一个匹配的响应头值，若不存在则返回 `nil`。
- `resp:header_values(name)`：返回所有匹配的响应头值数组。
- `resp:raise_for_status()`：对 4xx 和 5xx HTTP 响应抛出错误。

### text

规范 API 名称：`ptool.http.Response:text`。

`resp:text()` 读取并以文本形式返回响应体。

- 返回：`string`。

### json

规范 API 名称：`ptool.http.Response:json`。

`resp:json()` 读取响应体，按 JSON 解析后返回 Lua 值。

### bytes

规范 API 名称：`ptool.http.Response:bytes`。

`resp:bytes()` 读取并以原始字节形式返回响应体。

- 返回：`string`。

### header

规范 API 名称：`ptool.http.Response:header`。

`resp:header(name)` 返回第一个匹配 `name` 的响应头值。

- `name`（string，必填）：要查找的响应头名称。
- 返回：`string | nil`。

### header_values

规范 API 名称：`ptool.http.Response:header_values`。

`resp:header_values(name)` 返回所有匹配 `name` 的响应头值。

- `name`（string，必填）：要查找的响应头名称。
- 返回：`string[]`。

### raise_for_status

规范 API 名称：`ptool.http.Response:raise_for_status`。

`resp:raise_for_status()` 会在响应状态码处于 4xx 或 5xx 范围时抛出错误。

说明：

- 默认情况下，非 2xx HTTP 状态不会抛错。调用方可以检查 `resp.ok`、设置 `fail_on_http_error = true`，或调用 `resp:raise_for_status()`。
- 响应体会在首次读取后缓存，因此 `text`、`json` 和 `bytes` 可以重复调用。
