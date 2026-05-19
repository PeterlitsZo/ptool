# ANSI API

ANSI styling helpers are available under `ptool.ansi` and `p.ansi`.

## ptool.ansi.style

> `v0.1.0` - Introduced.

`ptool.ansi.style(text[, options])` returns `text` wrapped in ANSI style escape
sequences.

- `text` (string, required): The text to style.
- `options` (table, optional): Style options. Supported fields:
  - `enabled` (boolean, optional): Whether ANSI escapes should be emitted.
    Defaults to whether `ptool` is writing to a terminal.
  - `fg` (string|nil, optional): The foreground color. Supported values are
    `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `purple`, `cyan`,
    `white`, `bright_black`, `bright_red`, `bright_green`,
    `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_purple`,
    `bright_cyan`, and `bright_white`.
  - `bold` (boolean, optional): Whether to apply bold text.
  - `dimmed` (boolean, optional): Whether to apply dimmed text.
  - `italic` (boolean, optional): Whether to apply italic text.
  - `underline` (boolean, optional): Whether to apply underline text.
- Returns: `string`.

Behavior:

- If `enabled = false`, the original text is returned unchanged.
- If `fg = nil` or omitted, no foreground color is applied.
- Unknown option names or invalid option value types raise an error.

Example:

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
`ptool.ansi.cyan`, and `ptool.ansi.white` are convenience helpers with the
following signature:

```lua
ptool.ansi.red(text[, options])
```

They accept the same `text` argument and the same `options` table as
`ptool.ansi.style`, except the foreground color is fixed by the helper itself.
If `options.fg` is also provided, the helper color takes precedence.

Example:

```lua
print(ptool.ansi.green("ok", { bold = true }))
print(ptool.ansi.red("failed", { enabled = true, underline = true }))
```
