# API JSON

As utilidades de parse e serialização JSON estão disponíveis em `ptool.json` e
`p.json`.

## ptool.json.parse

> `v0.3.0` - Introduced.

`ptool.json.parse(input)` faz o parse de uma string JSON em um valor Lua.

- `input` (string, obrigatório): O texto JSON.
- Retorna: O valor Lua analisado. A raiz pode ser qualquer tipo JSON.

Mapeamento de tipos:

- Objeto JSON -> tabela Lua
- Array JSON -> tabela sequencial Lua (base 1)
- String JSON -> string Lua
- Inteiro JSON que cabe em `i64` -> inteiro Lua
- Outro número JSON -> número Lua
- Booleano JSON -> booleano Lua
- JSON null -> Lua `nil`

Comportamento de erro:

- Um erro é gerado se `input` não for uma string.
- Um erro de sintaxe JSON gera uma mensagem que inclui o detalhe do parser de
  `serde_json`.

Exemplo:

```lua
local data = p.json.parse('{"name":"ptool","features":["json","repl"],"stars":42}')

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.json.stringify

> `v0.3.0` - Introduced.

`ptool.json.stringify(value[, options])` converte um valor Lua em uma string
JSON.

- `value` (valor Lua compatível com JSON, obrigatório): O valor a ser
  codificado.
- `options` (table, opcional): Opções de serialização.
- `options.pretty` (boolean, opcional): Quando `true`, produz JSON formatado.
  O padrão é `false`.
- Retorna: A string JSON codificada.

Comportamento:

- A saída padrão é JSON compacto, sem espaços extras.
- A saída pretty usa JSON indentado em múltiplas linhas.
- Os valores precisam ser compatíveis com JSON. Funções, threads, userdata e
  outros valores Lua não serializáveis geram erro.

Exemplo:

```lua
local text = p.json.stringify({
  name = "ptool",
  features = {"json", "repl"},
  stable = true,
}, { pretty = true })

print(text)
```

Notas:

- Valores `nil` dentro de tabelas Lua seguem o comportamento de conversão
  serde de `mlua` e não são preservados como campos de objetos JSON.
- A detecção de array/objeto em tabelas Lua segue as regras de conversão serde
  de `mlua`.
