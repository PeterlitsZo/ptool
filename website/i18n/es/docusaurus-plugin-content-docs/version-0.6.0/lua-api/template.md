# Template API

Template rendering helpers are available under `ptool.template` and `p.template`.

## ptool.template.render

> `v0.1.0` - Introduced.

`ptool.template.render(template, context)` renders a Jinja-style template string and returns the rendered result.

- `template` (string, required): The template source text.
- `context` (any serializable Lua value, required): The template context.
- Returns: The rendered string.

Example:

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

Notes:

- The context must be serializable to data values.
- Lua values such as `function`, `thread`, and unsupported `userdata` are not accepted as template context values.
- Missing values use chainable undefined semantics. This means nested lookups such as `foo.bar.baz` can be passed to filters like `default(...)` without raising an error. When rendered directly without a fallback, undefined values become an empty string.

```lua
local template = ptool.unindent([[
  | {{ foo.bar.baz | default("N/A") }}
]])

print(ptool.template.render(template, {})) -- N/A
```
