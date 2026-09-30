# API de Redis

Las utilidades de conexión a Redis están disponibles bajo `ptool.redis` y `p.redis`.

## ptool.redis.connect

> `v0.9.0` - Introducido.

`ptool.redis.connect(url_or_options)` abre una conexión Redis y devuelve un objeto `Connection`.

Argumentos:

- `url_or_options` (string|table, obligatorio):
  - Cuando se proporciona una cadena, se trata como la URL de Redis.
  - Cuando se proporciona una tabla, actualmente admite:
    - `url` (string, obligatorio): La URL de Redis.

Ejemplos comunes de URL:

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")
local db1 = ptool.redis.connect("redis://127.0.0.1/1")
local auth = ptool.redis.connect({
  url = "redis://:secret@cache.internal:6379/0",
})
```

Notas:

- La conexión está pensada para ejecutar comandos directamente mediante `conn:call(...)`.
- Actualmente esta API no ofrece utilidades dedicadas para pub/sub, pipeline ni transacciones.

## Connection

> `v0.9.0` - Introducido.

`Connection` representa una conexión Redis abierta devuelta por `ptool.redis.connect()`.

Está implementado como un userdata de Lua.

Métodos:

- `conn:call(command, ...)` -> `any`
- `conn:close()` -> `nil`

Reglas de los argumentos del comando:

- `command` debe ser una cadena no vacía.
- Los argumentos restantes se envían a Redis como una lista plana de argumentos.
- Los tipos de valor admitidos para los argumentos son:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil`, tablas, funciones, hilos y userdata no están admitidos como argumentos de comandos Redis.
- Las cadenas de Lua se pasan como bytes sin procesar, así que los valores Redis binary-safe están admitidos.

Reglas de conversión de respuestas:

- Las respuestas nulas se convierten en `nil`.
- Las respuestas enteras se convierten en enteros de Lua.
- Las bulk strings, simple strings y verbatim strings se convierten en cadenas de Lua.
- Las respuestas de array y set se convierten en tablas array densas de Lua.
- Las respuestas de mapa se convierten en tablas Lua cuando las claves del mapa pueden representarse como cadenas, enteros, números o booleanos de Lua.
- Las respuestas dobles se convierten en números de Lua.
- Las respuestas booleanas se convierten en booleanos de Lua.
- Las respuestas de números grandes se convierten en cadenas de Lua.
- Las respuestas push se convierten en tablas con forma `{ kind = "...", data = {...} }`.

### call

> `v0.9.0` - Introducido.

Nombre canónico de la API: `ptool.redis.Connection:call`.

`conn:call(command, ...)` envía un comando Redis y devuelve la respuesta convertida.

Ejemplo:

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")

redis:call("SET", "user:1:name", "alice")
print(redis:call("GET", "user:1:name")) -- alice

local count = redis:call("INCR", "counters:signup")
print(count)

local values = redis:call("MGET", "user:1:name", "missing")
print(values[1])        -- alice
print(values[2] == nil) -- true
```

### close

> `v0.9.0` - Introducido.

Nombre canónico de la API: `ptool.redis.Connection:close`.

`conn:close()` cierra la conexión.

Comportamiento:

- Después de cerrarla, la conexión ya no puede usarse.

