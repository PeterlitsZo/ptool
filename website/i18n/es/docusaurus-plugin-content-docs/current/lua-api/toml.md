# API TOML

Las utilidades para analizar y editar TOML están disponibles bajo `ptool.toml`
y `p.toml`.

## ptool.toml.parse

> `v0.1.0` - Introduced.

`ptool.toml.parse(input)` analiza una cadena TOML y la convierte en una tabla
Lua.

- `input` (string, obligatorio): El texto TOML.
- Devuelve: Una tabla Lua (el nodo raíz siempre es una tabla).

Asignación de tipos:

- Tabla TOML / tabla inline -> tabla Lua
- Array TOML -> tabla secuencial Lua (base 1)
- Cadena TOML -> cadena Lua
- Entero TOML -> entero Lua
- Float TOML -> número Lua
- Booleano TOML -> booleano Lua
- datetime/date/time TOML -> cadena Lua

Comportamiento ante errores:

- Se produce un error si `input` no es una cadena.
- Un error de sintaxis TOML produce un error cuyo mensaje incluye información
  de línea y columna.

Ejemplo:

```lua
local text = ptool.fs.read("ptool.toml")
local conf = ptool.toml.parse(text)

print(conf.project.name)
print(conf.build.jobs)
print(conf.release_date) -- datetime/date/time values are strings
```

## ptool.toml.get

> `v0.1.0` - Introduced.

`ptool.toml.get(input, path)` lee el valor de una ruta determinada dentro de un
texto TOML.

- `input` (string, obligatorio): El texto TOML.
- `path` (string[], obligatorio): Un arreglo de ruta no vacío, como
  `{"package", "version"}`.
- Devuelve: El valor Lua correspondiente, o `nil` si la ruta no existe.

Ejemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)
```

## ptool.toml.set

> `v0.1.0` - Introduced.

`ptool.toml.set(input, path, value)` establece el valor en una ruta
determinada y devuelve el texto TOML actualizado.

- `input` (string, obligatorio): El texto TOML.
- `path` (string[], obligatorio): Un arreglo de ruta no vacío, como
  `{"package", "version"}`.
- `value` (string|integer|number|boolean, obligatorio): El valor que se va a
  escribir.
- Devuelve: La cadena TOML actualizada.

Comportamiento:

- Las rutas intermedias que falten se crean automáticamente como tablas.
- Si una ruta intermedia existe pero no es una tabla, se produce un error.
- El análisis y la reescritura se basan en `toml_edit`, que preserva en lo
  posible los comentarios y el formato originales.

Ejemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.set(text, {"package", "version"}, "0.2.0")
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.remove

> `v0.1.0` - Introduced.

`ptool.toml.remove(input, path)` elimina la ruta indicada y devuelve el texto
TOML actualizado.

- `input` (string, obligatorio): El texto TOML.
- `path` (string[], obligatorio): Un arreglo de ruta no vacío, como
  `{"package", "name"}`.
- Devuelve: La cadena TOML actualizada.

Comportamiento:

- Si la ruta no existe, no se produce ningún error y se devuelve el texto
  original o una forma equivalente.
- Si una ruta intermedia existe pero no es una tabla, se produce un error.

Ejemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.remove(text, {"package", "description"})
ptool.fs.write("Cargo.toml", updated)
```

Notas:

- El argumento `path` de `ptool.toml.get/set/remove` debe ser un arreglo no
  vacío de cadenas.
- Actualmente `set` solo admite la escritura de tipos escalares
  (`string`/`integer`/`number`/`boolean`).
