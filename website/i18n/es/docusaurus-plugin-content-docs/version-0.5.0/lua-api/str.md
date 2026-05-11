# API de cadenas

Las utilidades de cadena están disponibles bajo `ptool.str` y `p.str`.

## ptool.str.trim

> `v0.1.0` - Introduced.

`ptool.str.trim(s)` elimina los espacios en blanco iniciales y finales.

- `s` (string, obligatorio): La cadena de entrada.
- Devuelve: `string`.

```lua
print(ptool.str.trim("  hello\n")) -- hello
```

## ptool.str.trim_start

> `v0.1.0` - Introduced.

`ptool.str.trim_start(s)` elimina los espacios en blanco iniciales.

- `s` (string, obligatorio): La cadena de entrada.
- Devuelve: `string`.

```lua
print(ptool.str.trim_start("  hello  ")) -- hello  
```

## ptool.str.trim_end

> `v0.1.0` - Introduced.

`ptool.str.trim_end(s)` elimina los espacios en blanco finales.

- `s` (string, obligatorio): La cadena de entrada.
- Devuelve: `string`.

```lua
print(ptool.str.trim_end("  hello  ")) --   hello
```

## ptool.str.is_blank

> `v0.1.0` - Introduced.

`ptool.str.is_blank(s)` comprueba si una cadena está vacía o contiene solo
espacios en blanco.

- `s` (string, obligatorio): La cadena de entrada.
- Devuelve: `boolean`.

```lua
print(ptool.str.is_blank(" \t\n")) -- true
print(ptool.str.is_blank("x")) -- false
```

## ptool.str.starts_with

> `v0.1.0` - Introduced.

`ptool.str.starts_with(s, prefix)` comprueba si `s` empieza por `prefix`.

- `s` (string, obligatorio): La cadena de entrada.
- `prefix` (string, obligatorio): El prefijo que se comprobará.
- Devuelve: `boolean`.

```lua
print(ptool.str.starts_with("hello.lua", "hello")) -- true
```

## ptool.str.ends_with

> `v0.1.0` - Introduced.

`ptool.str.ends_with(s, suffix)` comprueba si `s` termina en `suffix`.

- `s` (string, obligatorio): La cadena de entrada.
- `suffix` (string, obligatorio): El sufijo que se comprobará.
- Devuelve: `boolean`.

```lua
print(ptool.str.ends_with("hello.lua", ".lua")) -- true
```

## ptool.str.contains

> `v0.1.0` - Introduced.

`ptool.str.contains(s, needle)` comprueba si `needle` aparece en `s`.

- `s` (string, obligatorio): La cadena de entrada.
- `needle` (string, obligatorio): La subcadena que se buscará.
- Devuelve: `boolean`.

```lua
print(ptool.str.contains("hello.lua", "lo.l")) -- true
```

## ptool.str.split

> `v0.1.0` - Introduced.

`ptool.str.split(s, sep[, options])` divide una cadena usando un separador no
vacío.

- `s` (string, obligatorio): La cadena de entrada.
- `sep` (string, obligatorio): El separador. No se permiten cadenas vacías.
- `options` (table, opcional): Opciones de división. Campos admitidos:
  - `trim` (boolean, opcional): Si se recorta cada fragmento antes de
    devolverlo. Por defecto es `false`.
  - `skip_empty` (boolean, opcional): Si se eliminan los fragmentos vacíos
    después del posible recorte. Por defecto es `false`.
- Devuelve: `string[]`.

Comportamiento:

- Los nombres de opción desconocidos o tipos de valor no válidos producen un
  error.
- `skip_empty = true` se aplica después de `trim`, por lo que los fragmentos
  compuestos solo por espacios pueden eliminarse cuando ambas opciones están
  activadas.

```lua
local parts = ptool.str.split(" a, b ,, c ", ",", {
  trim = true,
  skip_empty = true,
})

print(ptool.inspect(parts)) -- { "a", "b", "c" }
```

## ptool.str.split_lines

> `v0.1.0` - Introduced.

`ptool.str.split_lines(s[, options])` divide una cadena en líneas.

- `s` (string, obligatorio): La cadena de entrada.
- `options` (table, opcional): Opciones de división por líneas. Campos
  admitidos:
  - `keep_ending` (boolean, opcional): Si se conservan los finales de línea
    (`\n`, `\r\n` o `\r`) en los elementos devueltos. Por defecto es `false`.
  - `skip_empty` (boolean, opcional): Si se eliminan las líneas vacías. Por
    defecto es `false`.
- Devuelve: `string[]`.

Comportamiento:

- Admite finales de línea Unix (`\n`) y Windows (`\r\n`), y también `\r`
  aislado.
- Cuando `skip_empty = true`, una línea que solo contiene un final de línea se
  considera vacía y se elimina.
- Los nombres de opción desconocidos o tipos de valor no válidos producen un
  error.

```lua
local lines = ptool.str.split_lines("a\n\n b\r\n", {
  skip_empty = true,
})

print(ptool.inspect(lines)) -- { "a", " b" }
```

## ptool.str.join

> `v0.1.0` - Introduced.

`ptool.str.join(parts, sep)` une un arreglo de cadenas con un separador.

- `parts` (string[], obligatorio): Las partes de cadena que se van a unir.
- `sep` (string, obligatorio): La cadena separadora.
- Devuelve: `string`.

```lua
print(ptool.str.join({"a", "b", "c"}, "/")) -- a/b/c
```

## ptool.str.replace

> `v0.1.0` - Introduced.

`ptool.str.replace(s, from, to[, n])` reemplaza ocurrencias de `from` por `to`.

- `s` (string, obligatorio): La cadena de entrada.
- `from` (string, obligatorio): La subcadena que se va a reemplazar. No se
  permiten cadenas vacías.
- `to` (string, obligatorio): La cadena de reemplazo.
- `n` (integer, opcional): Número máximo de reemplazos. Debe ser mayor o igual
  que `0`. Si se omite, se reemplazan todas las coincidencias.
- Devuelve: `string`.

```lua
print(ptool.str.replace("a-b-c", "-", "/")) -- a/b/c
print(ptool.str.replace("a-b-c", "-", "/", 1)) -- a/b-c
```

## ptool.str.repeat

> `v0.1.0` - Introduced.

`ptool.str.repeat(s, n)` repite una cadena `n` veces.

- `s` (string, obligatorio): La cadena de entrada.
- `n` (integer, obligatorio): Número de repeticiones. Debe ser mayor o igual
  que `0`.
- Devuelve: `string`.

```lua
print(ptool.str.repeat("ab", 3)) -- ababab
```

## ptool.str.cut_prefix

> `v0.1.0` - Introduced.

`ptool.str.cut_prefix(s, prefix)` elimina `prefix` del inicio de `s` cuando
está presente.

- `s` (string, obligatorio): La cadena de entrada.
- `prefix` (string, obligatorio): El prefijo que se va a eliminar.
- Devuelve: `string`.

Comportamiento:

- Si `s` no empieza por `prefix`, se devuelve la cadena original sin cambios.

```lua
print(ptool.str.cut_prefix("refs/heads/main", "refs/heads/")) -- main
```

## ptool.str.cut_suffix

> `v0.1.0` - Introduced.

`ptool.str.cut_suffix(s, suffix)` elimina `suffix` del final de `s` cuando
está presente.

- `s` (string, obligatorio): La cadena de entrada.
- `suffix` (string, obligatorio): El sufijo que se va a eliminar.
- Devuelve: `string`.

Comportamiento:

- Si `s` no termina en `suffix`, se devuelve la cadena original sin cambios.

```lua
print(ptool.str.cut_suffix("archive.tar.gz", ".gz")) -- archive.tar
```

## ptool.str.indent

> `v0.1.0` - Introduced.

`ptool.str.indent(s, prefix[, options])` añade `prefix` a cada línea.

- `s` (string, obligatorio): La cadena de entrada.
- `prefix` (string, obligatorio): El texto insertado antes de cada línea.
- `options` (table, opcional): Opciones de indentación. Campos admitidos:
  - `skip_first` (boolean, opcional): Si la primera línea se deja sin cambios.
    Por defecto es `false`.
- Devuelve: `string`.

Comportamiento:

- Se conservan los finales de línea existentes.
- Una entrada vacía se devuelve sin cambios.
- Los nombres de opción desconocidos o tipos de valor no válidos producen un
  error.

```lua
local text = "first\nsecond\n"
print(ptool.str.indent(text, "> "))
print(ptool.str.indent(text, "  ", { skip_first = true }))
```
