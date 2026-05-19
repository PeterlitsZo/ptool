# Network API

URL, IP, and host/port parsing helpers are available under `ptool.net` and `p.net`.

## ptool.net.parse_url

> `v0.2.0` - Introduced.

`ptool.net.parse_url(input)` parses a URL string and returns a normalized parts
table.

Arguments:

- `input` (string, required): The URL to parse.

Returns: A table with the following fields:

- `kind` (string): Always `"url"`.
- `input` (string): The original input string.
- `normalized` (string): The normalized URL string.
- `scheme` (string): The URL scheme.
- `username` (string, optional): The decoded username, if present.
- `password` (string, optional): The decoded password, if present.
- `host` (string, optional): The hostname or IP literal, if present.
- `host_kind` (`"domain"|"ipv4"|"ipv6"`, optional): The host classification,
  if a host is present.
- `port` (integer, optional): The explicit port, if present.
- `path` (string): The URL path.
- `query` (string, optional): The query string without the leading `?`.
- `fragment` (string, optional): The fragment without the leading `#`.

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

> `v0.2.0` - Introduced.

`ptool.net.parse_ip(input)` parses an IPv4 or IPv6 address and returns a
normalized parts table.

Arguments:

- `input` (string, required): The IP address to parse.

Returns: A table with the following fields:

- `kind` (string): Always `"ip"`.
- `input` (string): The original input string.
- `normalized` (string): The normalized IP address.
- `version` (integer): `4` for IPv4 or `6` for IPv6.

```lua
local parts = ptool.net.parse_ip("2001:0db8::1")

print(parts.normalized) -- 2001:db8::1
print(parts.version)    -- 6
```

## ptool.net.parse_host_port

> `v0.2.0` - Introduced.

`ptool.net.parse_host_port(input)` parses a `host:port` string and returns a
normalized parts table.

Arguments:

- `input` (string, required): The host and port string. IPv6 addresses must use
  bracket notation such as `[2001:db8::1]:443`.

Returns: A table with the following fields:

- `kind` (string): Always `"host_port"`.
- `input` (string): The original input string.
- `normalized` (string): The normalized `host:port` string.
- `host` (string): The normalized host value.
- `host_kind` (`"domain"|"ipv4"|"ipv6"`): The host classification.
- `port` (integer): The parsed port.

```lua
local parts = ptool.net.parse_host_port("[2001:0db8::1]:443")

print(parts.host)        -- 2001:db8::1
print(parts.host_kind)   -- ipv6
print(parts.normalized)  -- [2001:db8::1]:443
print(parts.port)        -- 443
```
