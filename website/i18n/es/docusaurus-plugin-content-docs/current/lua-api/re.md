# API de regex

Las utilidades de expresiones regulares están disponibles bajo `ptool.re` y
`p.re`.

## ptool.re.compile

> `v0.1.0` - Introduced.

`ptool.re.compile(pattern[, opts])` compila una expresión regular y devuelve un
objeto `Regex`.

- `pattern` (string, obligatorio): El patrón regex.
- `opts` (table, opcional): Opciones de compilación. Actualmente se admite:
  - `case_insensitive` (boolean, opcional): Si la coincidencia distingue entre
    mayúsculas y minúsculas. Por defecto es `false`.

Ejemplo:

```lua
local re = ptool.re.compile("(?P<name>\\w+)", { case_insensitive = true })
print(re:is_match("Alice")) -- true
```

## ptool.re.escape

> `v0.1.0` - Introduced.

`ptool.re.escape(text)` escapa texto plano para convertirlo en una cadena
literal de regex.

- `text` (string, obligatorio): El texto que se va a escapar.
- Devuelve: La cadena escapada.

Ejemplo:

```lua
local keyword = "a+b?"
local re = ptool.re.compile("^" .. ptool.re.escape(keyword) .. "$")
print(re:is_match("a+b?")) -- true
```

## Regex

> `v0.1.0` - Introduced.

`Regex` representa una expresión regular compilada devuelta por
`ptool.re.compile(...)`.

Está implementada como userdata de Lua.

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

`re:is_match(input)` comprueba si la regex coincide con `input`.

- `input` (string, obligatorio): El texto de entrada.
- Devuelve: `boolean`.

### find

Canonical API name: `ptool.re.Regex:find`.

`re:find(input[, init])` devuelve la primera coincidencia en `input`, o `nil`.

- `input` (string, obligatorio): El texto de entrada.

Notas de parámetros:

- `init` es una posición inicial basada en 1 y por defecto vale `1`.
- `limit` debe ser mayor que `0`.

Estructuras devueltas:

- `Match`:
  - `start` (integer): Índice inicial basado en 1.
  - `finish` (integer): Índice final, utilizable directamente con `string.sub`.
  - `text` (string): El texto coincidente.
- `Captures`:
  - `full` (string): El texto completo coincidente.
  - `groups` (table): Un arreglo de grupos capturados en orden de captura.
    Los grupos no coincidentes son `nil`.
  - `named` (table): Un mapa de grupos capturados con nombre, indexado por
    nombre de grupo.

### find_all

Canonical API name: `ptool.re.Regex:find_all`.

`re:find_all(input)` devuelve todas las coincidencias en `input` como
`Match[]`.

### captures

Canonical API name: `ptool.re.Regex:captures`.

`re:captures(input)` devuelve el primer conjunto de capturas en `input`, o
`nil`.

### captures_all

Canonical API name: `ptool.re.Regex:captures_all`.

`re:captures_all(input)` devuelve todos los conjuntos de capturas en `input`
como `Captures[]`.

### replace

Canonical API name: `ptool.re.Regex:replace`.

`re:replace(input, replacement)` reemplaza la primera coincidencia en `input`.

### replace_all

Canonical API name: `ptool.re.Regex:replace_all`.

`re:replace_all(input, replacement)` reemplaza todas las coincidencias en
`input`.

### split

Canonical API name: `ptool.re.Regex:split`.

`re:split(input[, limit])` divide `input` usando la regex como separador.

Ejemplo:

```lua
local re = ptool.re.compile("(?P<word>\\w+)")
local cap = re:captures("hello world")
print(cap.full)         -- hello
print(cap.named.word)   -- hello
print(re:replace_all("a b c", "_")) -- _ _ _
```
