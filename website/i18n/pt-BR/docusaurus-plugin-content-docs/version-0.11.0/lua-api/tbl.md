# API de tabelas

Os utilitários de tabelas estão disponíveis em `ptool.tbl` e `p.tbl`.

Essas APIs foram projetadas para tabelas de lista densas, com chaves inteiras contíguas começando em `1`.

## ptool.tbl.map

> `v0.6.0` - Introduzido.

`ptool.tbl.map(list, fn)` transforma cada item de uma tabela de lista e retorna uma nova lista.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)` e deve retornar um valor diferente de `nil`.
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

> `v0.6.0` - Introduzido.

`ptool.tbl.filter(list, fn)` mantém os itens cujo resultado do callback seja truthy e os retorna em uma nova lista densa.

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

## ptool.tbl.any

> `v0.10.0` - Introduzido.

`ptool.tbl.any(list, fn)` retorna `true` quando o callback produz um resultado truthy para pelo menos um item.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)`.
- Retorna: `boolean`.

Comportamento:

- `nil` e `false` são tratados como falsy. Todos os outros valores Lua são tratados como truthy.
- A iteração para assim que um resultado truthy é encontrado.
- Listas vazias retornam `false`.

```lua
local has_even = p.tbl.any({ 1, 3, 4, 5 }, function(value)
  return value % 2 == 0
end)

print(has_even) -- true
```

## ptool.tbl.all

> `v0.10.0` - Introduzido.

`ptool.tbl.all(list, fn)` retorna `true` somente quando o callback produz um resultado truthy para cada item.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)`.
- Retorna: `boolean`.

Comportamento:

- `nil` e `false` são tratados como falsy. Todos os outros valores Lua são tratados como truthy.
- A iteração para assim que um resultado falsy é encontrado.
- Listas vazias retornam `true`.

```lua
local all_short = p.tbl.all({ "a", "bb", "ccc" }, function(value)
  return #value <= 3
end)

print(all_short) -- true
```

## ptool.tbl.find

> `v0.10.0` - Introduzido.

`ptool.tbl.find(list, fn)` retorna o primeiro valor original da lista cujo resultado do callback é truthy.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)`.
- Retorna: `any | nil`.

Comportamento:

- `nil` e `false` são tratados como falsy. Todos os outros valores Lua são tratados como truthy.
- A iteração para na primeira correspondência.
- Listas vazias e casos sem correspondência retornam `nil`.

```lua
local found = p.tbl.find({ 10, 15, 20 }, function(value)
  return value > 12
end)

print(found) -- 15
```

## ptool.tbl.find_index

> `v0.10.0` - Introduzido.

`ptool.tbl.find_index(list, fn)` retorna o índice baseado em 1 do primeiro item cujo resultado do callback é truthy.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)`.
- Retorna: `integer | nil`.

Comportamento:

- `nil` e `false` são tratados como falsy. Todos os outros valores Lua são tratados como truthy.
- A iteração para na primeira correspondência.
- Listas vazias e casos sem correspondência retornam `nil`.

```lua
local index = p.tbl.find_index({ "cat", "dog", "eel" }, function(value)
  return value == "dog"
end)

print(index) -- 2
```

## ptool.tbl.count

> `v0.10.0` - Introduzido.

`ptool.tbl.count(list, fn)` conta quantos itens produzem um resultado truthy no callback.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)`.
- Retorna: `integer`.

Comportamento:

- `nil` e `false` são tratados como falsy. Todos os outros valores Lua são tratados como truthy.
- Listas vazias retornam `0`.

```lua
local total = p.tbl.count({ 1, 2, 3, 4, 5 }, function(value)
  return value % 2 == 1
end)

print(total) -- 3
```

## ptool.tbl.includes

> `v0.10.0` - Introduzido.

`ptool.tbl.includes(list, value)` retorna `true` quando a lista contém o valor fornecido.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `value` (any, obrigatório): O valor a ser procurado.
- Retorna: `boolean`.

Comportamento:

- A lista é percorrida da esquerda para a direita.
- A igualdade usa a semântica normal de `==` do Lua.
- Listas vazias retornam `false`.

```lua
local has_blue = p.tbl.includes({ "red", "blue", "green" }, "blue")

print(has_blue) -- true
```

## ptool.tbl.reduce

> `v0.10.0` - Introduzido.

`ptool.tbl.reduce(list, initial, fn)` reduz uma lista a um único valor.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `initial` (any, obrigatório): O valor inicial explícito do acumulador.
- `fn` (function, obrigatório): Um callback que recebe `(acc, value, index)` e deve retornar um valor diferente de `nil`.
- Retorna: `any`.

Comportamento:

- `initial` é sempre obrigatório, mesmo para listas não vazias.
- `fn` é chamado uma vez para cada item, em ordem.
- Cada resultado do callback se torna o próximo valor do acumulador.
- Se `fn` retornar `nil`, a chamada gera um erro em vez de perder o estado do acumulador.
- Listas vazias retornam `initial` sem alterações.

```lua
local total = p.tbl.reduce({ 5, 10, 15 }, 0, function(acc, value)
  return acc + value
end)

print(total) -- 30
```

## ptool.tbl.flat_map

> `v0.10.0` - Introduzido.

`ptool.tbl.flat_map(list, fn)` mapeia cada item para uma tabela de lista densa e então concatena essas listas retornadas em uma nova lista densa.

- `list` (table, obrigatório): Uma tabela de lista densa.
- `fn` (function, obrigatório): Um callback que recebe `(value, index)` e deve retornar uma tabela de lista densa.
- Retorna: `table`.

Comportamento:

- `fn` é chamado uma vez para cada item, em ordem.
- Cada resultado do callback deve ser uma tabela de lista densa.
- Retornar um valor que não seja tabela ou uma lista não densa gera um erro.
- Listas vazias retornam uma lista vazia.

```lua
local out = p.tbl.flat_map({ 1, 2, 3 }, function(value)
  return { value, value * 10 }
end)

print(ptool.inspect(out)) -- { 1, 10, 2, 20, 3, 30 }
```

## ptool.tbl.concat

> `v0.6.0` - Introduzido.

`ptool.tbl.concat(...)` concatena uma ou mais tabelas de lista densas em uma nova lista.

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

> `v0.6.0` - Introduzido.

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
