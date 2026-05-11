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
>
> `v0.4.0` - Added numeric path segments for array indexing.

`ptool.toml.get(input, path)` lee el valor de una ruta determinada dentro de un
texto TOML.

- `input` (string, obligatorio): El texto TOML.
- `path` ((string|integer)[], obligatorio): Un arreglo de ruta no vacío, como
  `{"package", "version"}` o `{"bin", 1, "name"}`.
- Devuelve: El valor Lua correspondiente, o `nil` si la ruta no existe.

Comportamiento:

- Los segmentos de ruta de tipo string seleccionan claves de tabla.
- Los segmentos de ruta de tipo integer seleccionan elementos de array usando
  el índice base 1 de Lua.

Ejemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)

local first_bin_name = ptool.toml.get(text, {"bin", 1, "name"})
print(first_bin_name)
```

## ptool.toml.set

> `v0.1.0` - Introduced.
>
> `v0.4.0` - Added composite value writes and numeric path segments.

`ptool.toml.set(input, path, value)` establece el valor en una ruta
determinada y devuelve el texto TOML actualizado.

- `input` (string, obligatorio): El texto TOML.
- `path` ((string|integer)[], obligatorio): Un arreglo de ruta no vacío, como
  `{"package", "version"}` o `{"bin", 1, "name"}`.
- `value` (string|integer|number|boolean|table, obligatorio): El valor que se
  va a escribir.
- Devuelve: La cadena TOML actualizada.

Comportamiento:

- Las rutas intermedias que falten se crean automáticamente como tablas.
- Si una ruta intermedia existe pero no es una tabla, se produce un error.
- Las tablas Lua con solo claves string se escriben como tablas TOML.
- Las tablas secuenciales de Lua se escriben como arrays TOML.
- Una secuencia Lua de tablas con claves string se escribe como un array of
  tables de TOML.
- Las tablas Lua vacías se escriben actualmente como tablas TOML.
- El análisis y la reescritura se basan en `toml_edit`, que preserva en lo
  posible los comentarios y el formato originales.

Ejemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.set(text, {"package", "version"}, "0.2.0")
ptool.fs.write("Cargo.toml", updated)

local text2 = ptool.toml.set(text, {"package", "keywords"}, {"lua", "toml"})
local text3 = ptool.toml.set(text2, {"package", "metadata"}, {
  channel = "stable",
  maintainers = {"peterlits"},
})
```

## ptool.toml.remove

> `v0.1.0` - Introduced.
>
> `v0.4.0` - Added numeric path segments for array indexing.

`ptool.toml.remove(input, path)` elimina la ruta indicada y devuelve el texto
TOML actualizado.

- `input` (string, obligatorio): El texto TOML.
- `path` ((string|integer)[], obligatorio): Un arreglo de ruta no vacío, como
  `{"package", "name"}` o `{"bin", 1}`.
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

## ptool.toml.stringify

> `v0.4.0` - Introduced.

`ptool.toml.stringify(value)` convierte un valor Lua en texto TOML.

- `value` (table, obligatorio): La tabla TOML raíz que se va a codificar.
- Devuelve: La cadena TOML codificada.

Comportamiento:

- El valor raíz debe ser una tabla Lua que represente una tabla TOML.
- Las tablas Lua anidadas siguen las mismas reglas de table/array que
  `ptool.toml.set`.
- Las tablas Lua vacías se codifican actualmente como tablas TOML.

Ejemplo:

```lua
local text = ptool.toml.stringify({
  package = {
    name = "ptool",
    version = "0.4.0",
    keywords = {"lua", "toml"},
  },
})

print(text)
```

Notas:

- El argumento `path` de `ptool.toml.get/set/remove` debe ser un arreglo no
  vacío de strings y/o enteros positivos.
- Los segmentos de ruta enteros son base 1 para coincidir con el indexado de
  arrays en Lua.
- Los valores TOML datetime/date/time todavía se leen como strings de Lua.
