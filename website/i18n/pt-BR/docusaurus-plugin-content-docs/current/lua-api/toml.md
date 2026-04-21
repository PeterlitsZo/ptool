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

`ptool.toml.get(input, path)` lê o valor em um caminho especificado dentro de
um texto TOML.

- `input` (string, obrigatório): O texto TOML.
- `path` (string[], obrigatório): Um array de caminho não vazio, como
  `{"package", "version"}`.
- Retorna: O valor Lua correspondente, ou `nil` se o caminho não existir.

Exemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local version = ptool.toml.get(text, {"package", "version"})
print(version)
```

## ptool.toml.set

> `v0.1.0` - Introduced.

`ptool.toml.set(input, path, value)` define o valor em um caminho especificado
e retorna o texto TOML atualizado.

- `input` (string, obrigatório): O texto TOML.
- `path` (string[], obrigatório): Um array de caminho não vazio, como
  `{"package", "version"}`.
- `value` (string|integer|number|boolean, obrigatório): O valor a ser escrito.
- Retorna: A string TOML atualizada.

Comportamento:

- Caminhos intermediários ausentes são criados automaticamente como tabelas.
- Se um caminho intermediário existir mas não for uma tabela, um erro é gerado.
- O parse e a regravação se baseiam em `toml_edit`, que preserva comentários e
  formatação originais tanto quanto possível.

Exemplo:

```lua
local text = ptool.fs.read("Cargo.toml")
local updated = ptool.toml.set(text, {"package", "version"}, "0.2.0")
ptool.fs.write("Cargo.toml", updated)
```

## ptool.toml.remove

> `v0.1.0` - Introduced.

`ptool.toml.remove(input, path)` remove o caminho especificado e retorna o
texto TOML atualizado.

- `input` (string, obrigatório): O texto TOML.
- `path` (string[], obrigatório): Um array de caminho não vazio, como
  `{"package", "name"}`.
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

Notas:

- O argumento `path` de `ptool.toml.get/set/remove` deve ser um array não vazio
  de strings.
- Atualmente, `set` só suporta escrita de tipos escalares
  (`string`/`integer`/`number`/`boolean`).
