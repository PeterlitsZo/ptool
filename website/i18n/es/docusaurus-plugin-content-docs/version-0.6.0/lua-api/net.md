# API de red

Las utilidades para analizar URL, IP y host/puerto están disponibles bajo `ptool.net` y `p.net`.

## ptool.net.parse_url

> `v0.2.0` - Introduced.

`ptool.net.parse_url(input)` analiza una cadena URL y devuelve una tabla de partes normalizadas.

Argumentos:

- `input` (string, obligatorio): La URL que se va a analizar.

Devuelve: una tabla con los siguientes campos:

- `kind` (string): Siempre `"url"`.
- `input` (string): La cadena de entrada original.
- `normalized` (string): La cadena URL normalizada.
- `scheme` (string): El esquema de la URL.
- `username` (string, opcional): El nombre de usuario decodificado, si existe.
- `password` (string, opcional): La contraseña decodificada, si existe.
- `host` (string, opcional): El hostname o literal IP, si existe.
- `host_kind` (`"domain"|"ipv4"|"ipv6"`, opcional): La clasificación del host si hay host.
- `port` (integer, opcional): El puerto explícito, si existe.
- `path` (string): La ruta de la URL.
- `query` (string, opcional): La cadena de consulta sin el `?` inicial.
- `fragment` (string, opcional): El fragmento sin el `#` inicial.

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

`ptool.net.parse_ip(input)` analiza una dirección IPv4 o IPv6 y devuelve una tabla de partes normalizadas.

Argumentos:

- `input` (string, obligatorio): La dirección IP que se va a analizar.

Devuelve: una tabla con los siguientes campos:

- `kind` (string): Siempre `"ip"`.
- `input` (string): La cadena de entrada original.
- `normalized` (string): La dirección IP normalizada.
- `version` (integer): `4` para IPv4 o `6` para IPv6.

```lua
local parts = ptool.net.parse_ip("2001:0db8::1")

print(parts.normalized) -- 2001:db8::1
print(parts.version)    -- 6
```

## ptool.net.parse_host_port

> `v0.2.0` - Introduced.

`ptool.net.parse_host_port(input)` analiza una cadena `host:port` y devuelve una tabla de partes normalizadas.

Argumentos:

- `input` (string, obligatorio): La cadena host y puerto. Las direcciones IPv6 deben usar notación con corchetes como `[2001:db8::1]:443`.

Devuelve: una tabla con los siguientes campos:

- `kind` (string): Siempre `"host_port"`.
- `input` (string): La cadena de entrada original.
- `normalized` (string): La cadena `host:port` normalizada.
- `host` (string): El valor de host normalizado.
- `host_kind` (`"domain"|"ipv4"|"ipv6"`): La clasificación del host.
- `port` (integer): El puerto analizado.

```lua
local parts = ptool.net.parse_host_port("[2001:0db8::1]:443")

print(parts.host)        -- 2001:db8::1
print(parts.host_kind)   -- ipv6
print(parts.normalized)  -- [2001:db8::1]:443
print(parts.port)        -- 443
```
