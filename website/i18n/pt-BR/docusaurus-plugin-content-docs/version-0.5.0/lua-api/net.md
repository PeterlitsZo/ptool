# API de rede

As utilidades de parse de URL, IP e host/porta estão disponíveis em
`ptool.net` e `p.net`.

## ptool.net.parse_url

> `v0.2.0` - Introduced.

`ptool.net.parse_url(input)` faz o parse de uma string de URL e retorna uma
tabela de partes normalizadas.

Argumentos:

- `input` (string, obrigatório): A URL a ser analisada.

Retorna: uma tabela com os seguintes campos:

- `kind` (string): Sempre `"url"`.
- `input` (string): A string de entrada original.
- `normalized` (string): A string de URL normalizada.
- `scheme` (string): O esquema da URL.
- `username` (string, opcional): O nome de usuário decodificado, se presente.
- `password` (string, opcional): A senha decodificada, se presente.
- `host` (string, opcional): O hostname ou literal de IP, se presente.
- `host_kind` (`"domain"|"ipv4"|"ipv6"`, opcional): A classificação do host,
  se houver host.
- `port` (integer, opcional): A porta explícita, se presente.
- `path` (string): O caminho da URL.
- `query` (string, opcional): A query string sem o `?` inicial.
- `fragment` (string, opcional): O fragmento sem o `#` inicial.

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

`ptool.net.parse_ip(input)` faz o parse de um endereço IPv4 ou IPv6 e retorna
uma tabela de partes normalizadas.

Argumentos:

- `input` (string, obrigatório): O endereço IP a ser analisado.

Retorna: uma tabela com os seguintes campos:

- `kind` (string): Sempre `"ip"`.
- `input` (string): A string de entrada original.
- `normalized` (string): O endereço IP normalizado.
- `version` (integer): `4` para IPv4 ou `6` para IPv6.

```lua
local parts = ptool.net.parse_ip("2001:0db8::1")

print(parts.normalized) -- 2001:db8::1
print(parts.version)    -- 6
```

## ptool.net.parse_host_port

> `v0.2.0` - Introduced.

`ptool.net.parse_host_port(input)` faz o parse de uma string `host:port` e
retorna uma tabela de partes normalizadas.

Argumentos:

- `input` (string, obrigatório): A string com host e porta. Endereços IPv6
  devem usar notação com colchetes, como `[2001:db8::1]:443`.

Retorna: uma tabela com os seguintes campos:

- `kind` (string): Sempre `"host_port"`.
- `input` (string): A string de entrada original.
- `normalized` (string): A string `host:port` normalizada.
- `host` (string): O valor normalizado do host.
- `host_kind` (`"domain"|"ipv4"|"ipv6"`): A classificação do host.
- `port` (integer): A porta analisada.

```lua
local parts = ptool.net.parse_host_port("[2001:0db8::1]:443")

print(parts.host)        -- 2001:db8::1
print(parts.host_kind)   -- ipv6
print(parts.normalized)  -- [2001:db8::1]:443
print(parts.port)        -- 443
```
