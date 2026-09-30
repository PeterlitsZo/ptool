# API JSON

As utilidades de parse e serialização JSON estão disponíveis em `ptool.json` e `p.json`.

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
- Um erro de sintaxe JSON gera uma mensagem que inclui o detalhe do parser de `serde_json`.

Exemplo:

```lua
local data = p.json.parse('{"name":"ptool","features":["json","repl"],"stars":42}')

print(data.name)
print(data.features[1])
print(data.stars)
```

## ptool.json.get

> Não lançado - Introduzido.

`ptool.json.get(input, path)` lê o valor em um caminho especificado de um texto JSON.

- `input` (string, obrigatório): O texto JSON.
- `path` ((string|integer)[], obrigatório): Um array de caminho não vazio, como `{"spec", "template", "metadata", "name"}` ou `{"items", 1, "name"}`.
- Retorna: O valor Lua correspondente, ou `nil` se o caminho não existir ou não puder ser percorrido usando o tipo esperado de objeto ou array.

Comportamento:

- Segmentos de caminho string selecionam chaves de objetos.
- Segmentos de caminho integer selecionam elementos de arrays usando índices Lua base 1.

Exemplo:

```lua
local text = '{"items":[{"name":"alpha"},{"name":"beta"}]}'
local first_name = p.json.get(text, {"items", 1, "name"})
print(first_name)
```

## ptool.json.set

> Não lançado - Introduzido.

`ptool.json.set(input, path, value)` grava um valor em um caminho especificado de um texto JSON e retorna a string JSON atualizada.

- `input` (string, obrigatório): O texto JSON.
- `path` ((string|integer)[], obrigatório): Um array de caminho não vazio.
- `value` (valor Lua compatível com JSON, obrigatório): O valor a gravar.
- Retorna: A string JSON compacta atualizada.

Comportamento:

- Chaves de objeto e elementos de array existentes são substituídos.
- Uma chave de objeto final ausente é criada.
- Chaves de objeto intermediárias ausentes são criadas quando o próximo segmento do caminho também é uma chave string.
- Arrays não são expandidos. Todos os índices de array no caminho já devem existir.
- Um erro é gerado quando o caminho não pode ser percorrido usando o tipo esperado de objeto ou array.

Exemplo:

```lua
local text = '{"service":{"name":"api"},"ports":[8080]}'

text = p.json.set(text, {"service", "enabled"}, true)
text = p.json.set(text, {"ports", 1}, 9090)

print(text)
```

## ptool.json.stringify

> `v0.3.0` - Introduced.

`ptool.json.stringify(value[, options])` converte um valor Lua em uma string JSON.

- `value` (valor Lua compatível com JSON, obrigatório): O valor a ser codificado.
- `options` (table, opcional): Opções de serialização.
- `options.pretty` (boolean, opcional): Quando `true`, produz JSON formatado. O padrão é `false`.
- Retorna: A string JSON codificada.

Comportamento:

- A saída padrão é JSON compacto, sem espaços extras.
- A saída pretty usa JSON indentado em múltiplas linhas.
- Os valores precisam ser compatíveis com JSON. Funções, threads, userdata e outros valores Lua não serializáveis geram erro.

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

- Valores `nil` dentro de tabelas Lua seguem o comportamento de conversão serde de `mlua` e não são preservados como campos de objetos JSON.
- A detecção de array/objeto em tabelas Lua segue as regras de conversão serde de `mlua`.
