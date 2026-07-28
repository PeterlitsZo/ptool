# API YAML

As utilidades de parse e serialização YAML estão disponíveis em `ptool.yaml` e `p.yaml`.

## ptool.yaml.parse

> `v0.4.0` - Introduced.

`ptool.yaml.parse(input)` faz o parse de uma string YAML em um valor Lua.

- `input` (string, obrigatório): O texto YAML.
- Retorna: O valor Lua analisado. A raiz pode ser qualquer tipo YAML compatível.

Mapeamento de tipos:

- YAML mapping -> tabela Lua
- YAML sequence -> tabela sequencial Lua (base 1)
- YAML string -> string Lua
- YAML integer que cabe em `i64` -> inteiro Lua
- Outro YAML number -> número Lua
- YAML boolean -> booleano Lua
- YAML null -> Lua `nil`

Comportamento de erro:

- Um erro é gerado se `input` não for uma string.
- Um erro de sintaxe YAML gera uma mensagem que inclui o detalhe do parser.
- Também ocorre erro se o valor YAML não puder ser representado como um valor Lua de `ptool`, como um mapping com chaves não string ou um valor com tag YAML explícita.

Exemplo:

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

`ptool.yaml.get(input, path)` lê o valor em um caminho específico de um texto YAML.

- `input` (string, obrigatório): O texto YAML.
- `path` ((string|integer)[], obrigatório): Um array de caminho não vazio, como `{"spec", "template", "metadata", "name"}` ou `{"items", 1, "name"}`.
- Retorna: O valor Lua correspondente, ou `nil` se o caminho não existir.

Comportamento:

- Segmentos de caminho string selecionam chaves de mappings.
- Segmentos de caminho integer selecionam elementos de sequências usando índices Lua base 1.

Exemplo:

```lua
local text = [[
items:
  - name: alpha
  - name: beta
]]

local first_name = p.yaml.get(text, {"items", 1, "name"})
print(first_name)
```

## ptool.yaml.set

> Não lançado - Introduzido.

`ptool.yaml.set(input, path, value)` grava um valor em um caminho especificado de um texto YAML e retorna o texto YAML atualizado.

- `input` (string, obrigatório): O texto YAML.
- `path` ((string|integer)[], obrigatório): Um array de caminho não vazio.
- `value` (valor Lua compatível com YAML, obrigatório): O valor a gravar.
- Retorna: O texto YAML atualizado.

Comportamento:

- Chaves de mappings e elementos de sequências existentes são substituídos.
- Uma chave final de mapping ausente é criada.
- Chaves intermediárias de mappings ausentes são criadas quando o próximo segmento do caminho também é uma chave string.
- Sequências não são expandidas. Todos os índices de sequência no caminho já devem existir.
- Um erro é gerado quando o caminho não pode ser percorrido usando o tipo esperado de mapping ou sequência.

Exemplo:

```lua
local text = [[
service:
  name: api
ports:
  - 8080
]]

text = p.yaml.set(text, {"service", "enabled"}, true)
text = p.yaml.set(text, {"ports", 1}, 9090)

print(text)
```

## ptool.yaml.stringify

> `v0.4.0` - Introduced.

`ptool.yaml.stringify(value)` converte um valor Lua em texto YAML.

- `value` (valor Lua compatível com YAML, obrigatório): O valor a ser codificado.
- Retorna: A string YAML codificada.

Comportamento:

- Os valores precisam ser compatíveis com YAML pelo mesmo mapeamento de valores Lua usado por `ptool.json.stringify`.
- Tabelas sequenciais Lua são codificadas como sequências YAML.
- Tabelas Lua com chaves string são codificadas como mappings YAML.

Exemplo:

```lua
local text = p.yaml.stringify({
  project = "ptool",
  features = {"yaml", "lua"},
  stable = true,
})

print(text)
```

Notas:

- Apenas YAML de documento único é suportado.
- Mappings YAML devem usar chaves string.
- Tags YAML explícitas não são suportadas.
- O argumento `path` de `ptool.yaml.get` e `ptool.yaml.set` deve ser um array não vazio de strings e/ou inteiros positivos.
- Segmentos integer são base 1 para combinar com a indexação de arrays do Lua.
