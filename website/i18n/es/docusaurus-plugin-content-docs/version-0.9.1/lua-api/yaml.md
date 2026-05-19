# API YAML

Las utilidades para analizar y serializar YAML están disponibles bajo `ptool.yaml` y `p.yaml`.

## ptool.yaml.parse

> `v0.4.0` - Introduced.

`ptool.yaml.parse(input)` analiza una cadena YAML y la convierte en un valor Lua.

- `input` (string, obligatorio): El texto YAML.
- Devuelve: El valor Lua analizado. La raíz puede ser cualquier tipo YAML compatible.

Asignación de tipos:

- YAML mapping -> tabla Lua
- YAML sequence -> tabla secuencial Lua (base 1)
- YAML string -> cadena Lua
- YAML integer que cabe en `i64` -> entero Lua
- Otro YAML number -> número Lua
- YAML boolean -> booleano Lua
- YAML null -> Lua `nil`

Comportamiento ante errores:

- Se produce un error si `input` no es una cadena.
- Un error de sintaxis YAML produce un error cuyo mensaje incluye el detalle del analizador.
- También se produce un error si el valor YAML no puede representarse como un valor Lua de `ptool`, como un mapping con claves no string o un valor con un tag YAML explícito.

Ejemplo:

```lua
local data = p.yaml.parse([[
name: ptool
features:
  - yaml
  - repl
stars: 42
]])

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.yaml.get

> `v0.4.0` - Introduced.

`ptool.yaml.get(input, path)` lee el valor en una ruta específica dentro de un texto YAML.

- `input` (string, obligatorio): El texto YAML.
- `path` ((string|integer)[], obligatorio): Un arreglo de ruta no vacío, como `{"spec", "template", "metadata", "name"}` o `{"items", 1, "name"}`.
- Devuelve: El valor Lua correspondiente, o `nil` si la ruta no existe.

Comportamiento:

- Los segmentos de ruta string seleccionan claves de mappings.
- Los segmentos de ruta integer seleccionan elementos de secuencias usando índices Lua base 1.

Ejemplo:

```lua
local text = [[
items:
  - name: alpha
  - name: beta
]]

local first_name = p.yaml.get(text, {"items", 1, "name"})
print(first_name)
```

## ptool.yaml.stringify

> `v0.4.0` - Introduced.

`ptool.yaml.stringify(value)` convierte un valor Lua en texto YAML.

- `value` (valor Lua compatible con YAML, obligatorio): El valor que se va a codificar.
- Devuelve: La cadena YAML codificada.

Comportamiento:

- Los valores deben ser compatibles con YAML mediante el mismo mapeo de valores Lua usado por `ptool.json.stringify`.
- Las tablas secuenciales Lua se codifican como secuencias YAML.
- Las tablas Lua con claves string se codifican como mappings YAML.

Ejemplo:

```lua
local text = p.yaml.stringify({
  project = "ptool",
  features = {"yaml", "lua"},
  stable = true,
})

print(text)
```

Notas:

- Solo se admite YAML de un único documento.
- Los mappings YAML deben usar claves string.
- Los tags YAML explícitos no están soportados.
- El argumento `path` de `ptool.yaml.get` debe ser un arreglo no vacío de strings y/o enteros positivos.
- Los segmentos integer son base 1 para coincidir con la indexación de arrays de Lua.
