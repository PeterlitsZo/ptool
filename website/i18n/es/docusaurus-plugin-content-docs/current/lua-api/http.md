# API HTTP

Las utilidades de cliente HTTP están disponibles bajo `ptool.http` y `p.http`.

## ptool.http.request

> `v0.1.0` - Introduced.

`ptool.http.request(options)` envía una solicitud HTTP y devuelve un objeto
`Response`.

Campos de `options`:

- `url` (string, obligatorio): La URL de la solicitud.
- `method` (string, opcional): El método HTTP. Por defecto es `"GET"`.
- `headers` (table, opcional): Cabeceras de la solicitud, donde tanto claves
  como valores son cadenas.
- `body` (string, opcional): El cuerpo de la solicitud.
- `timeout_ms` (integer, opcional): Tiempo de espera en milisegundos. Por
  defecto es `30000`.

Ejemplo:

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

`Response` representa una respuesta HTTP devuelta por
`ptool.http.request(...)`.

Campos:

- `status` (integer): El código de estado HTTP.
- `ok` (boolean): Si el código de estado está en el rango 2xx.
- `url` (string): La URL final después de las redirecciones.
- `headers` (table): Cabeceras de la respuesta (`table<string, string>`).

Métodos:

- `resp:text()`: Lee y devuelve el cuerpo de la respuesta como texto.
- `resp:json()`: Lee el cuerpo de la respuesta, lo analiza como JSON y devuelve
  un valor Lua.
- `resp:bytes()`: Lee y devuelve los bytes sin procesar como una cadena Lua.

### text

Canonical API name: `ptool.http.Response:text`.

`resp:text()` lee y devuelve el cuerpo de la respuesta como texto.

- Devuelve: `string`.

### json

Canonical API name: `ptool.http.Response:json`.

`resp:json()` lee el cuerpo de la respuesta, lo analiza como JSON y devuelve un
valor Lua.

### bytes

Canonical API name: `ptool.http.Response:bytes`.

`resp:bytes()` lee y devuelve el cuerpo de la respuesta como bytes sin procesar.

- Devuelve: `string`.

Notas:

- Los estados HTTP que no son 2xx no producen errores. Quien llama debe
  comprobar `resp.ok` por su cuenta.
- El cuerpo solo puede consumirse una vez. Llamar a `text`, `json` o `bytes`
  más de una vez produce un error.
