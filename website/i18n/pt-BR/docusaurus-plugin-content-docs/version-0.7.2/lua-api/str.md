# API de strings

As utilidades de string estão disponíveis em `ptool.str` e `p.str`.

## ptool.str.trim

> `v0.1.0` - Introduced.

`ptool.str.trim(s)` remove espaços em branco no início e no fim.

- `s` (string, obrigatório): A string de entrada.
- Retorna: `string`.

```lua
print(ptool.str.trim("  hello\n")) -- hello
```

## ptool.str.trim_start

> `v0.1.0` - Introduced.

`ptool.str.trim_start(s)` remove espaços em branco no início.

- `s` (string, obrigatório): A string de entrada.
- Retorna: `string`.

```lua
print(ptool.str.trim_start("  hello  ")) -- hello  
```

## ptool.str.trim_end

> `v0.1.0` - Introduced.

`ptool.str.trim_end(s)` remove espaços em branco no fim.

- `s` (string, obrigatório): A string de entrada.
- Retorna: `string`.

```lua
print(ptool.str.trim_end("  hello  ")) --   hello
```

## ptool.str.is_blank

> `v0.1.0` - Introduced.

`ptool.str.is_blank(s)` verifica se uma string está vazia ou contém apenas espaços em branco.

- `s` (string, obrigatório): A string de entrada.
- Retorna: `boolean`.

```lua
print(ptool.str.is_blank(" \t\n")) -- true
print(ptool.str.is_blank("x")) -- false
```

## ptool.str.starts_with

> `v0.1.0` - Introduced.

`ptool.str.starts_with(s, prefix)` verifica se `s` começa com `prefix`.

- `s` (string, obrigatório): A string de entrada.
- `prefix` (string, obrigatório): O prefixo a testar.
- Retorna: `boolean`.

```lua
print(ptool.str.starts_with("hello.lua", "hello")) -- true
```

## ptool.str.ends_with

> `v0.1.0` - Introduced.

`ptool.str.ends_with(s, suffix)` verifica se `s` termina com `suffix`.

- `s` (string, obrigatório): A string de entrada.
- `suffix` (string, obrigatório): O sufixo a testar.
- Retorna: `boolean`.

```lua
print(ptool.str.ends_with("hello.lua", ".lua")) -- true
```

## ptool.str.contains

> `v0.1.0` - Introduced.

`ptool.str.contains(s, needle)` verifica se `needle` aparece em `s`.

- `s` (string, obrigatório): A string de entrada.
- `needle` (string, obrigatório): A substring a procurar.
- Retorna: `boolean`.

```lua
print(ptool.str.contains("hello.lua", "lo.l")) -- true
```

## ptool.str.split

> `v0.1.0` - Introduced.

`ptool.str.split(s, sep[, options])` divide uma string por um separador não vazio.

- `s` (string, obrigatório): A string de entrada.
- `sep` (string, obrigatório): O separador. Strings vazias não são permitidas.
- `options` (table, opcional): Opções de divisão. Campos suportados:
  - `trim` (boolean, opcional): Se cada parte deve ser aparada antes de ser retornada. O padrão é `false`.
  - `skip_empty` (boolean, opcional): Se partes vazias devem ser removidas após a eventual remoção de espaços. O padrão é `false`.
- Retorna: `string[]`.

Comportamento:

- Nomes de opção desconhecidos ou tipos de valor inválidos geram erro.
- `skip_empty = true` é aplicado depois de `trim`, então partes compostas só por espaços podem ser removidas quando ambas as opções estão habilitadas.

```lua
local parts = ptool.str.split(" a, b ,, c ", ",", {
  trim = true,
  skip_empty = true,
})

print(ptool.inspect(parts)) -- { "a", "b", "c" }
```

## ptool.str.split_lines

> `v0.1.0` - Introduced.

`ptool.str.split_lines(s[, options])` divide uma string em linhas.

- `s` (string, obrigatório): A string de entrada.
- `options` (table, opcional): Opções de divisão de linhas. Campos suportados:
  - `keep_ending` (boolean, opcional): Se os finais de linha (`\n`, `\r\n` ou `\r`) devem ser mantidos nos itens retornados. O padrão é `false`.
  - `skip_empty` (boolean, opcional): Se linhas vazias devem ser removidas. O padrão é `false`.
- Retorna: `string[]`.

Comportamento:

- Suporta finais de linha Unix (`\n`), Windows (`\r\n`) e também `\r` isolado.
- Quando `skip_empty = true`, uma linha que contém apenas um final de linha é tratada como vazia e removida.
- Nomes de opção desconhecidos ou tipos de valor inválidos geram erro.

```lua
local lines = ptool.str.split_lines("a\n\n b\r\n", {
  skip_empty = true,
})

print(ptool.inspect(lines)) -- { "a", " b" }
```

## ptool.str.join

> `v0.1.0` - Introduced.

`ptool.str.join(parts, sep)` junta um array de strings com um separador.

- `parts` (string[], obrigatório): As partes de string a unir.
- `sep` (string, obrigatório): A string separadora.
- Retorna: `string`.

```lua
print(ptool.str.join({"a", "b", "c"}, "/")) -- a/b/c
```

## ptool.str.replace

> `v0.1.0` - Introduced.

`ptool.str.replace(s, from, to[, n])` substitui ocorrências de `from` por `to`.

- `s` (string, obrigatório): A string de entrada.
- `from` (string, obrigatório): A substring a ser substituída. Strings vazias não são permitidas.
- `to` (string, obrigatório): A string de substituição.
- `n` (integer, opcional): Número máximo de substituições. Deve ser maior ou igual a `0`. Se omitido, todas as correspondências são substituídas.
- Retorna: `string`.

```lua
print(ptool.str.replace("a-b-c", "-", "/")) -- a/b/c
print(ptool.str.replace("a-b-c", "-", "/", 1)) -- a/b-c
```

## ptool.str.repeat

> `v0.1.0` - Introduced.

`ptool.str.repeat(s, n)` repete uma string `n` vezes.

- `s` (string, obrigatório): A string de entrada.
- `n` (integer, obrigatório): A contagem de repetição. Deve ser maior ou igual a `0`.
- Retorna: `string`.

```lua
print(ptool.str.repeat("ab", 3)) -- ababab
```

## ptool.str.cut_prefix

> `v0.1.0` - Introduced.

`ptool.str.cut_prefix(s, prefix)` remove `prefix` do começo de `s` quando ele está presente.

- `s` (string, obrigatório): A string de entrada.
- `prefix` (string, obrigatório): O prefixo a remover.
- Retorna: `string`.

Comportamento:

- Se `s` não começar com `prefix`, a string original é retornada sem alteração.

```lua
print(ptool.str.cut_prefix("refs/heads/main", "refs/heads/")) -- main
```

## ptool.str.cut_suffix

> `v0.1.0` - Introduced.

`ptool.str.cut_suffix(s, suffix)` remove `suffix` do fim de `s` quando ele está presente.

- `s` (string, obrigatório): A string de entrada.
- `suffix` (string, obrigatório): O sufixo a remover.
- Retorna: `string`.

Comportamento:

- Se `s` não terminar com `suffix`, a string original é retornada sem alteração.

```lua
print(ptool.str.cut_suffix("archive.tar.gz", ".gz")) -- archive.tar
```

## ptool.str.indent

> `v0.1.0` - Introduced.

`ptool.str.indent(s, prefix[, options])` adiciona `prefix` a cada linha.

- `s` (string, obrigatório): A string de entrada.
- `prefix` (string, obrigatório): O texto inserido antes de cada linha.
- `options` (table, opcional): Opções de indentação. Campos suportados:
  - `skip_first` (boolean, opcional): Se a primeira linha deve permanecer inalterada. O padrão é `false`.
- Retorna: `string`.

Comportamento:

- Os finais de linha existentes são preservados.
- Uma entrada vazia é retornada sem alterações.
- Nomes de opção desconhecidos ou tipos de valor inválidos geram erro.

```lua
local text = "first\nsecond\n"
print(ptool.str.indent(text, "> "))
print(ptool.str.indent(text, "  ", { skip_first = true }))
```
