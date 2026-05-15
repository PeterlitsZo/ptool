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
