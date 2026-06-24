# ネットワーク API

URL、IP、host/port 解析ヘルパーは `ptool.net` と `p.net` にあります。

## ptool.net.parse_url

> `v0.2.0` - Introduced.

`ptool.net.parse_url(input)` は URL 文字列を解析し、正規化された各部の テーブルを返します。

引数:

- `input` (string, 必須): 解析する URL。

戻り値: 次のフィールドを持つテーブル。

- `kind` (string): 常に `"url"`。
- `input` (string): 元の入力文字列。
- `normalized` (string): 正規化された URL 文字列。
- `scheme` (string): URL スキーム。
- `username` (string, 任意): 存在する場合のデコード済みユーザー名。
- `password` (string, 任意): 存在する場合のデコード済みパスワード。
- `host` (string, 任意): 存在する場合のホスト名または IP リテラル。
- `host_kind` (`"domain"|"ipv4"|"ipv6"`, 任意): ホストが存在する場合の ホスト分類。
- `port` (integer, 任意): 存在する場合の明示的なポート。
- `path` (string): URL パス。
- `query` (string, 任意): 先頭の `?` を含まないクエリ文字列。
- `fragment` (string, 任意): 先頭の `#` を含まないフラグメント。

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

`ptool.net.parse_ip(input)` は IPv4 または IPv6 アドレスを解析し、 正規化された各部のテーブルを返します。

引数:

- `input` (string, 必須): 解析する IP アドレス。

戻り値: 次のフィールドを持つテーブル。

- `kind` (string): 常に `"ip"`。
- `input` (string): 元の入力文字列。
- `normalized` (string): 正規化された IP アドレス。
- `version` (integer): IPv4 の場合は `4`、IPv6 の場合は `6`。

```lua
local parts = ptool.net.parse_ip("2001:0db8::1")

print(parts.normalized) -- 2001:db8::1
print(parts.version)    -- 6
```

## ptool.net.parse_host_port

> `v0.2.0` - Introduced.

`ptool.net.parse_host_port(input)` は `host:port` 文字列を解析し、 正規化された各部のテーブルを返します。

引数:

- `input` (string, 必須): ホストとポートの文字列。IPv6 アドレスは `[2001:db8::1]:443` のように角括弧表記を使う必要があります。

戻り値: 次のフィールドを持つテーブル。

- `kind` (string): 常に `"host_port"`。
- `input` (string): 元の入力文字列。
- `normalized` (string): 正規化された `host:port` 文字列。
- `host` (string): 正規化された host 値。
- `host_kind` (`"domain"|"ipv4"|"ipv6"`): ホスト分類。
- `port` (integer): 解析されたポート。

```lua
local parts = ptool.net.parse_host_port("[2001:0db8::1]:443")

print(parts.host)        -- 2001:db8::1
print(parts.host_kind)   -- ipv6
print(parts.normalized)  -- [2001:db8::1]:443
print(parts.port)        -- 443
```
