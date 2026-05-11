# API ANSI

As utilidades de estilo ANSI estão disponíveis em `ptool.ansi` e `p.ansi`.

## ptool.ansi.style

> `v0.1.0` - Introduced.

`ptool.ansi.style(text[, options])` retorna `text` envolto em sequências de
escape de estilo ANSI.

- `text` (string, obrigatório): O texto a ser estilizado.
- `options` (table, opcional): Opções de estilo. Campos suportados:
  - `enabled` (boolean, opcional): Se escapes ANSI devem ser emitidos. O padrão
    depende de `ptool` estar escrevendo em um terminal.
  - `fg` (string|nil, opcional): A cor de primeiro plano. Os valores
    suportados são `black`, `red`, `green`, `yellow`, `blue`, `magenta`,
    `purple`, `cyan`, `white`, `bright_black`, `bright_red`,
    `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`,
    `bright_purple`, `bright_cyan` e `bright_white`.
  - `bold` (boolean, opcional): Se aplica texto em negrito.
  - `dimmed` (boolean, opcional): Se aplica texto esmaecido.
  - `italic` (boolean, opcional): Se aplica itálico.
  - `underline` (boolean, opcional): Se aplica sublinhado.
- Retorna: `string`.

Comportamento:

- Se `enabled = false`, o texto original é retornado sem alteração.
- Se `fg = nil` ou for omitido, nenhuma cor de primeiro plano é aplicada.
- Nomes de opção desconhecidos ou tipos de valor inválidos geram erro.

Exemplo:

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
`ptool.ansi.cyan` e `ptool.ansi.white` são utilitários de conveniência com a
seguinte assinatura:

```lua
ptool.ansi.red(text[, options])
```

Eles aceitam o mesmo argumento `text` e a mesma tabela `options` que
`ptool.ansi.style`, exceto que a cor de primeiro plano é fixada pelo próprio
helper. Se `options.fg` também for fornecido, a cor do helper tem prioridade.

Exemplo:

```lua
print(ptool.ansi.green("ok", { bold = true }))
print(ptool.ansi.red("failed", { enabled = true, underline = true }))
```
