# API ANSI

Las utilidades de estilo ANSI están disponibles bajo `ptool.ansi` y `p.ansi`.

## ptool.ansi.style

> `v0.1.0` - Introduced.

`ptool.ansi.style(text[, options])` devuelve `text` envuelto con secuencias de
escape de estilo ANSI.

- `text` (string, obligatorio): El texto al que se aplicará estilo.
- `options` (table, opcional): Opciones de estilo. Campos admitidos:
  - `enabled` (boolean, opcional): Si deben emitirse escapes ANSI. Por defecto
    depende de si `ptool` está escribiendo en un terminal.
  - `fg` (string|nil, opcional): El color de primer plano. Los valores
    admitidos son `black`, `red`, `green`, `yellow`, `blue`, `magenta`,
    `purple`, `cyan`, `white`, `bright_black`, `bright_red`,
    `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`,
    `bright_purple`, `bright_cyan` y `bright_white`.
  - `bold` (boolean, opcional): Si se aplica texto en negrita.
  - `dimmed` (boolean, opcional): Si se aplica texto atenuado.
  - `italic` (boolean, opcional): Si se aplica cursiva.
  - `underline` (boolean, opcional): Si se aplica subrayado.
- Devuelve: `string`.

Comportamiento:

- Si `enabled = false`, se devuelve el texto original sin cambios.
- Si `fg = nil` o se omite, no se aplica color de primer plano.
- Los nombres de opción desconocidos o tipos de valor no válidos producen un
  error.

Ejemplo:

```lua
print(ptool.ansi.style("warning", {
  fg = "bright_yellow",
  bold = true,
}))
```

## ptool.ansi.\<color\>

> `v0.1.0` - Introduced.

`ptool.ansi.black`, `ptool.ansi.red`, `ptool.ansi.green`,
`ptool.ansi.yellow`, `ptool.ansi.blue`, `ptool.ansi.magenta`,
`ptool.ansi.cyan` y `ptool.ansi.white` son utilidades de conveniencia con la
siguiente firma:

```lua
ptool.ansi.red(text[, options])
```

Aceptan el mismo argumento `text` y la misma tabla `options` que
`ptool.ansi.style`, salvo que el color de primer plano queda fijado por la
propia utilidad. Si también se proporciona `options.fg`, el color de la
utilidad tiene prioridad.

Ejemplo:

```lua
print(ptool.ansi.green("ok", { bold = true }))
print(ptool.ansi.red("failed", { enabled = true, underline = true }))
```
