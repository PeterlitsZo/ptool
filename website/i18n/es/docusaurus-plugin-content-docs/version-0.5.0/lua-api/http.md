# API HTTP

Las utilidades de cliente HTTP están disponibles bajo `ptool.http` y `p.http`.

## ptool.http.request

> `v0.1.0` - Introduced.

`ptool.http.request(options)` envía una solicitud HTTP y devuelve un objeto `Response`.

Campos de `options`:

- `url` (string, obligatorio): La URL de la solicitud.
- `method` (string, opcional): El método HTTP. Por defecto es `"GET"`.
- `headers` (table, opcional): Cabeceras de la solicitud, donde tanto claves como valores son cadenas.
- `query` (table, opcional): Parámetros de consulta añadidos a la URL de la solicitud. Las claves deben ser cadenas. Los valores pueden ser cadenas, números o booleanos.
- `body` (string, opcional): El cuerpo de la solicitud.
- `json` (valor Lua, opcional): Un valor Lua codificado como JSON y usado como cuerpo de la solicitud. Si `content-type` no está definido, se usa por defecto `application/json`.
- `form` (table, opcional): Campos codificados como `application/x-www-form-urlencoded` y usados como cuerpo de la solicitud. Las claves deben ser cadenas. Los valores pueden ser cadenas, números o booleanos.
- `timeout_ms` (integer, opcional): Tiempo de espera en milisegundos. Por defecto es `30000`.
- `connect_timeout_ms` (integer, opcional): Tiempo límite de conexión en milisegundos.
- `follow_redirects` (boolean, opcional): Si deben seguirse las redirecciones.
- `max_redirects` (integer, opcional): Número máximo de redirecciones a seguir.
- `user_agent` (string, opcional): Establece la cabecera `user-agent`.
- `basic_auth` (table, opcional): Credenciales HTTP Basic con campos de cadena `username` y `password`.
- `bearer_token` (string, opcional): Token Bearer usado para la cabecera `authorization`.
- `fail_on_http_error` (boolean, opcional): Si es `true`, las respuestas HTTP 4xx y 5xx producen un error. El valor por defecto es `false`.

Notas:

- `body`, `json` y `form` son mutuamente excluyentes.
- `basic_auth` y `bearer_token` son mutuamente excluyentes.

Ejemplo:

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

`Response` representa una respuesta HTTP devuelta por `ptool.http.request(...)`.

Campos:

- `status` (integer): El código de estado HTTP.
- `ok` (boolean): Si el código de estado está en el rango 2xx.
- `url` (string): La URL final después de las redirecciones.
- `headers` (table): Vista simplificada de las cabeceras de la respuesta (`table<string, string>`). Las cabeceras repetidas se unen con `, `.

Métodos:

- `resp:text()`: Lee y devuelve el cuerpo de la respuesta como texto.
- `resp:json()`: Lee el cuerpo de la respuesta, lo analiza como JSON y devuelve un valor Lua.
- `resp:bytes()`: Lee y devuelve los bytes sin procesar como una cadena Lua.
- `resp:header(name)`: Devuelve el primer valor de cabecera coincidente, o `nil`.
- `resp:header_values(name)`: Devuelve todos los valores de cabecera coincidentes como un arreglo.
- `resp:raise_for_status()`: Produce un error para respuestas HTTP 4xx y 5xx.

### text

Canonical API name: `ptool.http.Response:text`.

`resp:text()` lee y devuelve el cuerpo de la respuesta como texto.

- Devuelve: `string`.

### json

Canonical API name: `ptool.http.Response:json`.

`resp:json()` lee el cuerpo de la respuesta, lo analiza como JSON y devuelve un valor Lua.

### bytes

Canonical API name: `ptool.http.Response:bytes`.

`resp:bytes()` lee y devuelve el cuerpo de la respuesta como bytes sin procesar.

- Devuelve: `string`.

### header

Canonical API name: `ptool.http.Response:header`.

`resp:header(name)` devuelve el primer valor de cabecera de respuesta que coincide con `name`.

- `name` (string, obligatorio): El nombre de cabecera a buscar.
- Devuelve: `string | nil`.

### header_values

Canonical API name: `ptool.http.Response:header_values`.

`resp:header_values(name)` devuelve todos los valores de cabecera de respuesta que coinciden con `name`.

- `name` (string, obligatorio): El nombre de cabecera a buscar.
- Devuelve: `string[]`.

### raise_for_status

Canonical API name: `ptool.http.Response:raise_for_status`.

`resp:raise_for_status()` produce un error cuando el código de estado de la respuesta está en el rango 4xx o 5xx.

Notas:

- Por defecto, los estados HTTP fuera de 2xx no producen errores. Quien llama puede comprobar `resp.ok`, establecer `fail_on_http_error = true` o llamar a `resp:raise_for_status()`.
- El cuerpo de la respuesta se almacena en caché tras la primera lectura, por lo que `text`, `json` y `bytes` pueden llamarse varias veces.
