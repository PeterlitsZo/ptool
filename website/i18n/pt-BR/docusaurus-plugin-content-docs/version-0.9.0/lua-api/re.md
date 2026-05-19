# API de regex

As utilidades de expressão regular estão disponíveis em `ptool.re` e `p.re`.

## ptool.re.compile

> `v0.1.0` - Introduced.

`ptool.re.compile(pattern[, opts])` compila uma expressão regular e retorna um objeto `Regex`.

- `pattern` (string, obrigatório): O padrão regex.
- `opts` (table, opcional): Opções de compilação. Atualmente, há suporte a:
  - `case_insensitive` (boolean, opcional): Se a correspondência é case insensitive. O padrão é `false`.

Exemplo:

```lua
local re = ptool.re.compile("(?P<name>\\w+)", { case_insensitive = true })
print(re:is_match("Alice")) -- true
```

## ptool.re.escape

> `v0.1.0` - Introduced.

`ptool.re.escape(text)` escapa texto simples para uma string literal de regex.

- `text` (string, obrigatório): O texto a escapar.
- Retorna: A string escapada.

Exemplo:

```lua
local keyword = "a+b?"
local re = ptool.re.compile("^" .. ptool.re.escape(keyword) .. "$")
print(re:is_match("a+b?")) -- true
```

## Regex

> `v0.1.0` - Introduced.

`Regex` representa uma expressão regular compilada retornada por `ptool.re.compile(...)`.

Ela é implementada como userdata Lua.

Métodos:

- `re:is_match(input)` -> `boolean`
- `re:find(input[, init])` -> `Match|nil`
- `re:find_all(input)` -> `Match[]`
- `re:captures(input)` -> `Captures|nil`
- `re:captures_all(input)` -> `Captures[]`
- `re:replace(input, replacement)` -> `string`
- `re:replace_all(input, replacement)` -> `string`
- `re:split(input[, limit])` -> `string[]`

### is_match

Canonical API name: `ptool.re.Regex:is_match`.

`re:is_match(input)` verifica se a regex corresponde a `input`.

- `input` (string, obrigatório): O texto de entrada.
- Retorna: `boolean`.

### find

Canonical API name: `ptool.re.Regex:find`.

`re:find(input[, init])` retorna a primeira correspondência em `input`, ou `nil`.

- `input` (string, obrigatório): O texto de entrada.

Notas sobre parâmetros:

- `init` é uma posição inicial baseada em 1 e o padrão é `1`.
- `limit` deve ser maior que `0`.

Estruturas de retorno:

- `Match`:
  - `start` (integer): O índice inicial baseado em 1.
  - `finish` (integer): O índice final, utilizável diretamente com `string.sub`.
  - `text` (string): O texto correspondente.
- `Captures`:
  - `full` (string): O texto completo correspondente.
  - `groups` (table): Um array dos grupos capturados na ordem de captura. Grupos não correspondidos são `nil`.
  - `named` (table): Um mapeamento de grupos capturados nomeados, indexado pelo nome do grupo.

### find_all

Canonical API name: `ptool.re.Regex:find_all`.

`re:find_all(input)` retorna todas as correspondências em `input` como `Match[]`.

### captures

Canonical API name: `ptool.re.Regex:captures`.

`re:captures(input)` retorna o primeiro conjunto de capturas em `input`, ou `nil`.

### captures_all

Canonical API name: `ptool.re.Regex:captures_all`.

`re:captures_all(input)` retorna todos os conjuntos de capturas em `input` como `Captures[]`.

### replace

Canonical API name: `ptool.re.Regex:replace`.

`re:replace(input, replacement)` substitui a primeira correspondência em `input`.

### replace_all

Canonical API name: `ptool.re.Regex:replace_all`.

`re:replace_all(input, replacement)` substitui todas as correspondências em `input`.

### split

Canonical API name: `ptool.re.Regex:split`.

`re:split(input[, limit])` divide `input` usando a regex como separador.

Exemplo:

```lua
local re = ptool.re.compile("(?P<word>\\w+)")
local cap = re:captures("hello world")
print(cap.full)         -- hello
print(cap.named.word)   -- hello
print(re:replace_all("a b c", "_")) -- _ _ _
```
