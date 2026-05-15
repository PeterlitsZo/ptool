# ANSI API

ANSI スタイルヘルパーは `ptool.ansi` と `p.ansi` にあります。

## ptool.ansi.style

> `v0.1.0` - Introduced.

`ptool.ansi.style(text[, options])` は、ANSI スタイルのエスケープ シーケンスで包んだ `text` を返します。

- `text` (string, 必須): 装飾するテキスト。
- `options` (table, 任意): スタイルオプション。サポートされるフィールド:
  - `enabled` (boolean, 任意): ANSI エスケープを出力するかどうか。 デフォルトは `ptool` が端末へ書き込んでいるかどうかに従います。
  - `fg` (string|nil, 任意): 前景色。`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `purple`, `cyan`, `white`, `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_purple`, `bright_cyan`, `bright_white` が 使えます。
  - `bold` (boolean, 任意): 太字を適用するかどうか。
  - `dimmed` (boolean, 任意): 淡色表示を適用するかどうか。
  - `italic` (boolean, 任意): イタリックを適用するかどうか。
  - `underline` (boolean, 任意): 下線を適用するかどうか。
- 戻り値: `string`。

挙動:

- `enabled = false` の場合、元のテキストがそのまま返されます。
- `fg = nil` または未指定の場合、前景色は適用されません。
- 未知のオプション名や不正な値型はエラーになります。

例:

```lua
print(ptool.ansi.style("warning", {
  fg = "bright_yellow",
  bold = true,
}))
```

## ptool.ansi.\<color\>

> `v0.1.0` - Introduced.

`ptool.ansi.black`, `ptool.ansi.red`, `ptool.ansi.green`, `ptool.ansi.yellow`, `ptool.ansi.blue`, `ptool.ansi.magenta`, `ptool.ansi.cyan`, `ptool.ansi.white` は、次のシグネチャを持つ 簡易ヘルパーです。

```lua
ptool.ansi.red(text[, options])
```

これらは `ptool.ansi.style` と同じ `text` 引数と同じ `options` テーブルを 受け取りますが、前景色はヘルパー自身によって固定されます。 `options.fg` も指定された場合は、ヘルパー側の色が優先されます。

例:

```lua
print(ptool.ansi.green("ok", { bold = true }))
print(ptool.ansi.red("failed", { enabled = true, underline = true }))
```
