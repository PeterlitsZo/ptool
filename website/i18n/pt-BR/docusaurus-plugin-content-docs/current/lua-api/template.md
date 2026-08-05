# API de templates

As utilidades de renderização de templates estão disponíveis em `ptool.template` e `p.template`.

## ptool.template.render

> `v0.1.0` - Introduced.

`ptool.template.render(template, context)` renderiza uma string de template no estilo Jinja e retorna o resultado renderizado.

- `template` (string, obrigatório): O texto-fonte do template.
- `context` (qualquer valor Lua serializável, obrigatório): O contexto do template.
- Retorna: A string renderizada.

Exemplo:

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

- O contexto precisa ser serializável como valores de dados.
- Valores Lua como `function`, `thread` e `userdata` não suportado não são aceitos como valores de contexto do template.
- Valores ausentes usam semântica de undefined encadeável. Isso significa que buscas aninhadas como `foo.bar.baz` podem ser passadas para filtros como `default(...)` sem gerar erro. Quando renderizados diretamente sem fallback, valores undefined tornam-se string vazia.

```lua
local template = ptool.unindent([[
  | {{ foo.bar.baz | default("N/A") }}
]])

print(ptool.template.render(template, {})) -- N/A
```

## ptool.template.write

> `v0.12.0` - Introduzido.

`ptool.template.write(path, template, context)` renderiza uma string de template no estilo Jinja e grava o resultado renderizado diretamente em um arquivo.

- `path` (string, obrigatório): O caminho do arquivo de destino.
- `template` (string, obrigatório): O texto-fonte do template.
- `context` (qualquer valor Lua serializável, obrigatório): O contexto do template.
- Retorna: Nada.

Exemplo:

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

- A renderização usa a mesma conversão de contexto e a mesma semântica de template que `ptool.template.render(...)`.
- O arquivo de destino é criado se não existir e truncado se já existir, como em `ptool.fs.write(...)`.
- Os diretórios pai não são criados automaticamente.
