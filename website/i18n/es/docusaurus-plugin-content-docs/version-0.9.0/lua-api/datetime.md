# DateTime API

Los ayudantes de fecha y hora están disponibles en `ptool.datetime` y `p.datetime`.

`ptool.datetime` trabaja con instantes concretos. Cada valor `DateTime` incluye una zona horaria o un desplazamiento numérico.

## ptool.datetime.now

> `v0.6.0` - Introducido.

`ptool.datetime.now([tz])` devuelve la hora actual como `DateTime`.

- `tz` (cadena, opcional): una zona horaria de IANA como `UTC`, `America/New_York` o `Asia/Shanghai`. Si se omite, se utiliza la zona horaria del sistema local.
- Devuelve: `DateTime`.

```lua
local local_now = p.datetime.now()
local utc_now = p.datetime.now("UTC")

print(local_now)
print(utc_now:format("%Y-%m-%d %H:%M:%S %Z"))
```

## ptool.datetime.parse

> `v0.6.0` - Introducido.

`ptool.datetime.parse(input[, options])` analiza una cadena de fecha y hora y devuelve un `DateTime`.

- `input` (cadena, obligatorio): Una cadena de fecha y hora.
- `options.timezone` (cadena, opcional): Una zona horaria IANA utilizada solo cuando la entrada no incluye ya una zona horaria o desplazamiento.
- Devuelve: `DateTime`.

Entradas aceptadas:

- Entradas zonificadas como `2024-07-15T16:24:59-04:00`.
- Entradas zonificadas con anotaciones de zona horaria entre corchetes cuando el analizador lo admite.
- Entradas ingenuas como `2024-07-15 16:24:59`, pero solo cuando se proporciona `options.timezone`.

Comportamiento:

- Las cadenas vacías se rechazan.
- Si `input` ya incluye una zona horaria o desplazamiento, la configuración de `options.timezone` genera un error.
- Sin `options.timezone`, se rechazan las entradas ingenuas.

```lua
local a = p.datetime.parse("2024-07-15T16:24:59-04:00")
local b = p.datetime.parse("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})

print(a.offset)   -- -04:00
print(b.timezone) -- America/New_York
```

## ptool.datetime.from_unix

> `v0.6.0` - Introducido.

`ptool.datetime.from_unix(value[, options])` construye un `DateTime` a partir de una marca de tiempo de Unix.

- `value` (entero, requerido): La marca de tiempo de Unix.
- `options.unit` (cadena, opcional): uno de `s`, `ms` o `ns`. El valor predeterminado es `s`.
- `options.timezone` (cadena, opcional): Una zona horaria IANA. Si se omite, la marca de tiempo se interpreta en `UTC`.
- Devuelve: `DateTime`.

```lua
local a = p.datetime.from_unix(1721075099)
local b = p.datetime.from_unix(1721075099000, {
  unit = "ms",
  timezone = "Asia/Tokyo",
})

print(a) -- 2024-07-15T20:24:59+00:00
print(b)
```

## ptool.datetime.compare

> `v0.6.0` - Introducido.

`ptool.datetime.compare(a, b)` compara dos instantes.

- `a` / `b` (string|DateTime, requerido): Una cadena de fecha y hora u objeto `DateTime`.
- Devuelve: `-1 | 0 | 1`.

Los argumentos de cadena se analizan utilizando las mismas reglas estrictas que `ptool.datetime.parse(input)`, por lo que ya deben incluir una zona horaria o un desplazamiento.

```lua
print(ptool.datetime.compare(
  "2024-07-15T20:24:59+00:00",
  "2024-07-15T16:24:59-04:00"
)) -- 0
```

## ptool.datetime.is_valid

> `v0.6.0` - Introducido.

`ptool.datetime.is_valid(input[, options])` comprueba si se puede analizar una cadena de fecha y hora.

- `input` (cadena, obligatorio): Una cadena de fecha y hora.
- `options.timezone` (cadena, opcional): Una zona horaria IANA para la entrada ingenua.
- Devuelve: `boolean`.

```lua
print(ptool.datetime.is_valid("2024-07-15T16:24:59-04:00")) -- true
print(ptool.datetime.is_valid("2024-07-15 16:24:59")) -- false
print(ptool.datetime.is_valid("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})) -- true
```

## DateTime

> `v0.6.0` - Introducido.

`DateTime` representa un instante concreto devuelto por `ptool.datetime.now(...)`, `parse(...)` o `from_unix(...)`.

Se implementa como datos de usuario de Lua.

Campos y métodos:

- Campos:
  - `year` (entero)
  - `month` (entero)
  - `day` (entero)
  - `hour` (entero)
  - `minute` (entero)
  - `second` (entero)
  - `nanosecond` (entero)
  - `offset` (cadena)
  - `timezone` (cadena)
- Métodos:
  - `dt:format(fmt)` -> `string`
  - `dt:to_string()` -> `string`
  - `dt:unix([unit])` -> `integer`
  - `dt:in_tz(tz)` -> `DateTime`
  - `dt:compare(other)` -> `-1|0|1`
- Metamétodos:
  - `tostring(dt)` está disponible.
  - Se admiten comparaciones `==`, `<` y `<=`.

### format

Nombre de la API canónica: `ptool.datetime.DateTime:format`.

`dt:format(fmt)` formatea la fecha y hora utilizando directivas de estilo `strftime`.

- `fmt` (cadena, obligatorio): Una cadena de formato como `%Y-%m-%d %H:%M:%S %Z`.
- Devuelve: `string`.

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:format("%Y-%m-%d %H:%M:%S %:z"))
```

### to_string

Nombre de la API canónica: `ptool.datetime.DateTime:to_string`.

`dt:to_string()` devuelve la forma de cadena canónica con un desplazamiento numérico.

- Devuelve: `string`.

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:to_string()) -- 2024-07-15T16:24:59-04:00
```

### unix

Nombre de la API canónica: `ptool.datetime.DateTime:unix`.

`dt:unix([unit])` devuelve la marca de tiempo Unix del instante.

- `unit` (cadena, opcional): uno de `s`, `ms` o `ns`. El valor predeterminado es `s`.
- Devuelve: `integer`.

Notas:

- `ns` puede generar un error si el resultado no encaja en el rango entero de Lua.

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
print(dt:unix())       -- seconds
print(dt:unix("ms"))   -- milliseconds
```

### in_tz

Nombre de la API canónica: `ptool.datetime.DateTime:in_tz`.

`dt:in_tz(tz)` convierte el mismo instante en otra zona horaria.

- `tz` (cadena, obligatorio): Una zona horaria IANA.
- Devuelve: `DateTime`.

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
local tokyo = dt:in_tz("Asia/Tokyo")

print(dt)
print(tokyo)
```

### compare

Nombre de la API canónica: `ptool.datetime.DateTime:compare`.

`dt:compare(other)` compara el instante actual con `other`.

- `other` (string|DateTime, requerido): Una cadena de fecha y hora u otro objeto `DateTime`.
- Devuelve: `-1 | 0 | 1`.

```lua
local a = p.datetime.parse("2024-07-15T20:24:59+00:00")
local b = p.datetime.parse("2024-07-15T21:24:59+00:00")

print(a:compare(b)) -- -1
print(a < b)        -- true
```

## Notas

- `ptool.datetime` no analiza frases en lenguaje natural como `"tomorrow 8am"`.
- Los nombres de zonas horarias deben ser identificadores IANA como `UTC`, `Asia/Tokyo` o `America/New_York`.
- Las comparaciones operan en el instante, no en los campos de reloj de pared mostrados.
