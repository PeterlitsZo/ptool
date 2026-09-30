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

## ptool.template.write

> `v0.12.0` - Introducido.

`ptool.template.write(path, template, context)` renderiza una cadena de plantilla de estilo Jinja y escribe el resultado renderizado directamente en un archivo.

- `path` (string, obligatorio): La ruta del archivo de destino.
- `template` (string, obligatorio): El texto fuente de la plantilla.
- `context` (cualquier valor Lua serializable, obligatorio): El contexto de la plantilla.
- Devuelve: Nada.

Ejemplo:

```lua
local template = ptool.unindent([[
  | server_name = {{ server.name }}
  | port = {{ server.port }}
]])

ptool.template.write("server.conf", template, {
  server = { name = "example.com", port = 8080 },
})
```

Notas:

- El renderizado utiliza la misma conversión de contexto y la misma semántica de plantilla que `ptool.template.render(...)`.
- El archivo de destino se crea si no existe y se trunca si ya existe, igual que con `ptool.fs.write(...)`.
- Los directorios padre no se crean automáticamente.
