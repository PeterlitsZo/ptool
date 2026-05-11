# API de tabelas

Os utilitários de tabelas estão disponíveis em `ptool.tbl` e `p.tbl`.

Essas APIs foram projetadas para tabelas de lista densas, com chaves inteiras
contíguas começando em `1`.

## ptool.tbl.map

> `Unreleased` - Introduced.

`ptool.tbl.map(list, fn)` transforma cada item de uma tabela de lista e retorna
uma nova lista.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)` e deve
  retornar um valor diferente de `nil`.
- Retorna: `table`.

Comportamento:

- `fn` é chamado uma vez para cada item, em ordem.
- Se `fn` retornar `nil`, a chamada falha em vez de criar buracos no resultado.
- A tabela de entrada não é modificada.

```lua
local out = p.tbl.map({ 10, 20, 30 }, function(value, index)
  return value + index
end)

print(ptool.inspect(out)) -- { 11, 22, 33 }
```

## ptool.tbl.filter

> `Unreleased` - Introduced.

`ptool.tbl.filter(list, fn)` mantém os itens cujo resultado do callback seja
truthy e os retorna em uma nova lista densa.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)`.
- Retorna: `table`.

Comportamento:

- `nil` e `false` removem o item atual.
- Qualquer outro valor Lua mantém o item atual.
- A tabela retornada é reindexada a partir de `1`.

```lua
local out = p.tbl.filter({ "a", "bb", "ccc" }, function(value)
  return #value >= 2
end)

print(ptool.inspect(out)) -- { "bb", "ccc" }
```

## ptool.tbl.concat

> `Unreleased` - Introduced.

`ptool.tbl.concat(...)` concatena uma ou mais tabelas de lista densas em uma
nova lista.

- `...` (table, obrigatório): Uma ou mais tabelas de lista densas.
- Retorna: `table`.

Comportamento:

- Os argumentos são anexados da esquerda para a direita.
- Listas vazias são permitidas.
- As tabelas de entrada não são modificadas.

```lua
local out = p.tbl.concat({ 1, 2 }, { 3 }, {})

print(ptool.inspect(out)) -- { 1, 2, 3 }
```

## ptool.tbl.join

> `Unreleased` - Introduced.

`ptool.tbl.join(...)` é um alias de `ptool.tbl.concat(...)`.

- `...` (table, obrigatório): Uma ou mais tabelas de lista densas.
- Retorna: `table`.

```lua
local out = p.tbl.join({ "x" }, { "y", "z" })

print(ptool.inspect(out)) -- { "x", "y", "z" }
```

## Regras de lista

`ptool.tbl` atualmente suporta apenas tabelas de lista densas.

- As chaves devem ser inteiras.
- As chaves devem começar em `1`.
- As chaves devem ser contíguas, sem buracos.
- Tabelas esparsas e tabelas no estilo dicionário geram erro.
