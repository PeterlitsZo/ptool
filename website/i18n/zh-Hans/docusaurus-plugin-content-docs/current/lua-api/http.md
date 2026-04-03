# HTTP API

HTTP 客户端辅助能力位于 `ptool.http` 和 `p.http` 下。

## ptool.http.request

> `v0.1.0` - 引入。

`ptool.http.request(options)` 发送一个 HTTP 请求，并返回 `Response` 对象。

`options` 字段：

- `url`（string，必填）：请求 URL。
- `method`（string，可选）：HTTP 方法。默认值为 `"GET"`。
- `headers`（table，可选）：请求头，键和值都必须是字符串。
- `body`（string，可选）：请求体。
- `timeout_ms`（integer，可选）：超时时间，单位毫秒。默认值为 `30000`。

示例：

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

> `v0.1.0` - 引入。

`Response` 表示 `ptool.http.request(...)` 返回的 HTTP 响应。

字段：

- `status`（integer）：HTTP 状态码。
- `ok`（boolean）：状态码是否位于 2xx 范围内。
- `url`（string）：重定向后的最终 URL。
- `headers`（table）：响应头（`table<string, string>`）。

方法：

- `resp:text()`：读取并以文本形式返回响应体。
- `resp:json()`：读取响应体，按 JSON 解析后返回 Lua 值。
- `resp:bytes()`：读取并返回原始字节（作为 Lua 字符串）。

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

说明：

- 非 2xx HTTP 状态不会抛错，调用方需要自行检查 `resp.ok`。
- 响应体只能消费一次。`text`、`json` 和 `bytes` 中任意方法调用一次后，
  再次调用任一方法都会抛出错误。
