# Network API

URL、IP 和 host/port 解析辅助能力位于 `ptool.net` 和 `p.net` 下。

## ptool.net.parse_url

> `v0.2.0` - 引入。

`ptool.net.parse_url(input)` 解析 URL 字符串，并返回一个规范化的组成部分表。

参数：

- `input`（string，必填）：要解析的 URL。

返回：包含以下字段的表：

- `kind`（string）：固定为 `"url"`。
- `input`（string）：原始输入字符串。
- `normalized`（string）：规范化后的 URL 字符串。
- `scheme`（string）：URL 方案。
- `username`（string，可选）：解码后的用户名（如果存在）。
- `password`（string，可选）：解码后的密码（如果存在）。
- `host`（string，可选）：主机名或 IP 字面量（如果存在）。
- `host_kind`（`"domain"|"ipv4"|"ipv6"`，可选）：主机分类，仅当存在主机时提供。
- `port`（integer，可选）：显式端口（如果存在）。
- `path`（string）：URL 路径。
- `query`（string，可选）：不带前导 `?` 的查询字符串。
- `fragment`（string，可选）：不带前导 `#` 的片段。

```lua
local parts = ptool.net.parse_url("https://user:pass@example.com:8443/a/b?q=1#frag")

print(parts.scheme)      -- https
print(parts.host_kind)   -- domain
print(parts.port)        -- 8443
print(parts.path)        -- /a/b
print(parts.query)       -- q=1
print(parts.fragment)    -- frag
```

## ptool.net.parse_ip

> `v0.2.0` - 引入。

`ptool.net.parse_ip(input)` 解析 IPv4 或 IPv6 地址，并返回一个规范化的组成部分表。

参数：

- `input`（string，必填）：要解析的 IP 地址。

返回：包含以下字段的表：

- `kind`（string）：固定为 `"ip"`。
- `input`（string）：原始输入字符串。
- `normalized`（string）：规范化后的 IP 地址。
- `version`（integer）：IPv4 返回 `4`，IPv6 返回 `6`。

```lua
local parts = ptool.net.parse_ip("2001:0db8::1")

print(parts.normalized) -- 2001:db8::1
print(parts.version)    -- 6
```

## ptool.net.parse_host_port

> `v0.2.0` - 引入。

`ptool.net.parse_host_port(input)` 解析 `host:port` 字符串，并返回一个规范化的 组成部分表。

参数：

- `input`（string，必填）：主机和端口字符串。IPv6 地址必须使用方括号形式， 例如 `[2001:db8::1]:443`。

返回：包含以下字段的表：

- `kind`（string）：固定为 `"host_port"`。
- `input`（string）：原始输入字符串。
- `normalized`（string）：规范化后的 `host:port` 字符串。
- `host`（string）：规范化后的主机值。
- `host_kind`（`"domain"|"ipv4"|"ipv6"`）：主机分类。
- `port`（integer）：解析后的端口。

```lua
local parts = ptool.net.parse_host_port("[2001:0db8::1]:443")

print(parts.host)        -- 2001:db8::1
print(parts.host_kind)   -- ipv6
print(parts.normalized)  -- [2001:db8::1]:443
print(parts.port)        -- 443
```
