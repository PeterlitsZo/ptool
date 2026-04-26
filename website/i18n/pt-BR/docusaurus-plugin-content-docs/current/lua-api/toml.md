# API TOML

As utilidades de parse e edição de TOML estão disponíveis em `ptool.toml` e
`p.toml`.

## ptool.toml.parse

> `v0.1.0` - Introduced.

`ptool.toml.parse(input)` faz o parse de uma string TOML em uma tabela Lua.

- `input` (string, obrigatório): O texto TOML.
- Retorna: Uma tabela Lua (o nó raiz sempre é uma tabela).

Mapeamento de tipos:

- Tabela TOML / tabela inline -> tabela Lua
- Array TOML -> tabela sequencial Lua (base 1)
- String TOML -> string Lua
- Inteiro TOML -> inteiro Lua
- Float TOML -> número Lua
- Booleano TOML -> booleano Lua
- datetime/date/time TOML -> string Lua

Comportamento de erro:

- Um erro é gerado se `input` não for uma string.
- Um erro de sintaxe TOML gera uma mensagem que inclui informações de linha e
  coluna.

Exemplo:

```lua
local text = ptool.fs.read("ptool.toml")
local conf = ptool.toml.parse(text)

print(conf.project.name)
print(conf.build.jobs)
print(conf.release_date) -- datetime/date/time values are strings
```

## ptool.toml.get

> `v0.1.0` - Introduced.
>
> `v0.4.0` - Added numeric path segments for array indexing.

`ptool.toml.get(input, path)` lê o valor em um caminho especificado dentro de
um texto TOML.

- `input` (string, obrigatório): O texto TOML.
- `path` ((string|integer)[], obrigatório): Um array de caminho não vazio, como
  `{"package", "version"}` ou `{"bin", 1, "name"}`.
- Retorna: O valor Lua correspondente, ou `nil` se o caminho não existir.

Comportamento:

- Segmentos de caminho do tipo string selecionam chaves de tabela.
- Segmentos de caminho do tipo integer selecionam elementos de array usando o
  índice base 1 do Lua.

Exemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)

local first_bin_name = ptool.toml.get(text, {"bin", 1, "name"})
print(first_bin_name)
```

## ptool.toml.set

> `v0.1.0` - Introduced.
>
> `v0.4.0` - Added composite value writes and numeric path segments.

`ptool.toml.set(input, path, value)` define o valor em um caminho especificado
e retorna o texto TOML atualizado.

- `input` (string, obrigatório): O texto TOML.
- `path` ((string|integer)[], obrigatório): Um array de caminho não vazio, como
  `{"package", "version"}` ou `{"bin", 1, "name"}`.
- `value` (string|integer|number|boolean|table, obrigatório): O valor a ser
  escrito.
- Retorna: A string TOML atualizada.

Comportamento:

- Caminhos intermediários ausentes são criados automaticamente como tabelas.
- Se um caminho intermediário existir mas não for uma tabela, um erro é gerado.
- Tabelas Lua com apenas chaves string são gravadas como tabelas TOML.
- Tabelas sequenciais de Lua são gravadas como arrays TOML.
- Uma sequência Lua de tabelas com chaves string é gravada como array of
  tables TOML.
- Tabelas Lua vazias são gravadas atualmente como tabelas TOML.
- O parse e a regravação se baseiam em `toml_edit`, que preserva comentários e
  formatação originais tanto quanto possível.

Exemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.set(text, {"package", "version"}, "0.2.0")
ptool.fs.write("Cargo.toml", updated)

local text2 = ptool.toml.set(text, {"package", "keywords"}, {"lua", "toml"})
local text3 = ptool.toml.set(text2, {"package", "metadata"}, {
  channel = "stable",
  maintainers = {"peterlits"},
})
```

## ptool.toml.remove

> `v0.1.0` - Introduced.
>
> `v0.4.0` - Added numeric path segments for array indexing.

`ptool.toml.remove(input, path)` remove o caminho especificado e retorna o
texto TOML atualizado.

- `input` (string, obrigatório): O texto TOML.
- `path` ((string|integer)[], obrigatório): Um array de caminho não vazio, como
  `{"package", "name"}` ou `{"bin", 1}`.
- Retorna: A string TOML atualizada.

Comportamento:

- Se o caminho não existir, nenhum erro é gerado e o texto original ou uma
  forma equivalente é retornado.
- Se um caminho intermediário existir mas não for uma tabela, um erro é gerado.

Exemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.remove(text, {"package", "description"})
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.stringify

> `v0.4.0` - Introduced.

`ptool.toml.stringify(value)` converte um valor Lua em texto TOML.

- `value` (table, obrigatório): A tabela TOML raiz a ser codificada.
- Retorna: A string TOML codificada.

Comportamento:

- O valor raiz deve ser uma tabela Lua que represente uma tabela TOML.
- Tabelas Lua aninhadas seguem as mesmas regras de table/array de
  `ptool.toml.set`.
- Tabelas Lua vazias são codificadas atualmente como tabelas TOML.

Exemplo:

```lua
local text = ptool.toml.stringify({
  package = {
    name = "ptool",
    version = "0.4.0",
    keywords = {"lua", "toml"},
  },
})

print(text)
```

Notas:

- O argumento `path` de `ptool.toml.get/set/remove` deve ser um array não vazio
  de strings e/ou inteiros positivos.
- Segmentos de caminho inteiros usam base 1 para combinar com o indexamento de
  arrays do Lua.
- Valores TOML datetime/date/time ainda são lidos como strings Lua.
