# API de plantillas

Las utilidades de renderizado de plantillas están disponibles bajo `ptool.template` y `p.template`.

## ptool.template.render

> `v0.1.0` - Introduced.

`ptool.template.render(template, context)` renderiza una cadena de plantilla de estilo Jinja y devuelve el resultado renderizado.

- `template` (string, obligatorio): El texto fuente de la plantilla.
- `context` (cualquier valor Lua serializable, obligatorio): El contexto de la plantilla.
- Devuelve: La cadena renderizada.

Ejemplo:

```lua
local template = ptool.unindent([[
  | {% if user.active %}
  | Hello, {{ user.name }}!
  | {% else %}
  | Inactive user: {{ user.name }}
  | {% endif %}
  | Items:
  | {% for item in items %}
  | - {{ item }}
  | {% endfor %}
]])
local result = ptool.template.render(template, {
  user = { name = "alice", active = true },
  items = { "one", "two", "three" },
})

print(result)
```

Notas:

- El contexto debe poder serializarse como valores de datos.
- Valores Lua como `function`, `thread` y `userdata` no admitidos no se aceptan como valores del contexto de la plantilla.
- Los valores ausentes usan semántica de undefined encadenable. Esto significa que búsquedas anidadas como `foo.bar.baz` pueden pasarse a filtros como `default(...)` sin producir error. Si se renderizan directamente sin un valor de reserva, los valores undefined se convierten en una cadena vacía.

```lua
local template = ptool.unindent([[
  | {{ foo.bar.baz | default("N/A") }}
]])

print(ptool.template.render(template, {})) -- N/A
```
