# API JSON

Las utilidades para analizar y serializar JSON están disponibles bajo `ptool.json` y `p.json`.

## ptool.json.parse

> `v0.3.0` - Introduced.

`ptool.json.parse(input)` analiza una cadena JSON y la convierte en un valor Lua.

- `input` (string, obligatorio): El texto JSON.
- Devuelve: El valor Lua analizado. La raíz puede ser cualquier tipo JSON.

Asignación de tipos:

- Objeto JSON -> tabla Lua
- Array JSON -> tabla secuencial Lua (base 1)
- Cadena JSON -> cadena Lua
- Entero JSON que cabe en `i64` -> entero Lua
- Otro número JSON -> número Lua
- Booleano JSON -> booleano Lua
- JSON null -> Lua `nil`

Comportamiento ante errores:

- Se produce un error si `input` no es una cadena.
- Un error de sintaxis JSON produce un error cuyo mensaje incluye el detalle del analizador de `serde_json`.

Ejemplo:

```lua
local data = p.json.parse('{"name":"ptool","features":["json","repl"],"stars":42}')

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.json.get

> Sin publicar - Introducido.

`ptool.json.get(input, path)` lee el valor de una ruta especificada en un texto JSON.

- `input` (string, obligatorio): El texto JSON.
- `path` ((string|integer)[], obligatorio): Un arreglo de ruta no vacío, como `{"spec", "template", "metadata", "name"}` o `{"items", 1, "name"}`.
- Devuelve: El valor Lua correspondiente, o `nil` si la ruta no existe o no puede recorrerse mediante el tipo de objeto o array esperado.

Comportamiento:

- Los segmentos de ruta string seleccionan claves de objetos.
- Los segmentos de ruta integer seleccionan elementos de arrays usando índices Lua base 1.

Ejemplo:

```lua
local text = '{"items":[{"name":"alpha"},{"name":"beta"}]}'
local first_name = p.json.get(text, {"items", 1, "name"})
print(first_name)
```

## ptool.json.set

> Sin publicar - Introducido.

`ptool.json.set(input, path, value)` escribe un valor en una ruta especificada de un texto JSON y devuelve la cadena JSON actualizada.

- `input` (string, obligatorio): El texto JSON.
- `path` ((string|integer)[], obligatorio): Un arreglo de ruta no vacío.
- `value` (valor Lua compatible con JSON, obligatorio): El valor que se va a escribir.
- Devuelve: La cadena JSON compacta actualizada.

Comportamiento:

- Se reemplazan las claves de objeto y los elementos de array existentes.
- Se crea la clave de objeto final si no existe.
- Las claves de objeto intermedias que falten se crean cuando el siguiente segmento de ruta también es una clave string.
- Los arrays no se expanden. Todos los índices de array de la ruta deben existir.
- Se produce un error cuando la ruta no puede recorrerse mediante el tipo de objeto o array esperado.

Ejemplo:

```lua
local text = '{"service":{"name":"api"},"ports":[8080]}'

text = p.json.set(text, {"service", "enabled"}, true)
text = p.json.set(text, {"ports", 1}, 9090)

print(text)
```

## ptool.json.stringify

> `v0.3.0` - Introduced.

`ptool.json.stringify(value[, options])` convierte un valor Lua en una cadena JSON.

- `value` (valor Lua compatible con JSON, obligatorio): El valor que se va a codificar.
- `options` (table, opcional): Opciones de serialización.
- `options.pretty` (boolean, opcional): Cuando es `true`, genera JSON con formato legible. Por defecto es `false`.
- Devuelve: La cadena JSON codificada.

Comportamiento:

- La salida por defecto es JSON compacto, sin espacios adicionales.
- La salida pretty usa JSON multilínea con indentación.
- Los valores deben ser compatibles con JSON. Funciones, threads, userdata y otros valores Lua no serializables producen un error.

Ejemplo:

```lua
local text = p.json.stringify({
  name = "ptool",
  features = {"json", "repl"},
  stable = true,
}, { pretty = true })

print(text)
```

Notas:

- Los valores `nil` dentro de tablas Lua siguen el comportamiento de conversión serde de `mlua` y no se conservan como campos de objetos JSON.
- La detección de array/objeto para tablas Lua sigue las reglas de conversión serde de `mlua`.
