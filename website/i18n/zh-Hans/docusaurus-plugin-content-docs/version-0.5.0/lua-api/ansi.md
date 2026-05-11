# ANSI API

ANSI 样式辅助能力位于 `ptool.ansi` 和 `p.ansi` 下。

## ptool.ansi.style

> `v0.1.0` - 引入。

`ptool.ansi.style(text[, options])` 返回一个用 ANSI 样式转义序列包裹后的
`text`。

- `text`（string，必填）：要添加样式的文本。
- `options`（table，可选）：样式选项。支持以下字段：
  - `enabled`（boolean，可选）：是否输出 ANSI 转义序列。默认取决于 `ptool`
    当前是否写入终端。
  - `fg`（string|nil，可选）：前景色。支持的值包括 `black`、`red`、
    `green`、`yellow`、`blue`、`magenta`、`purple`、`cyan`、`white`、
    `bright_black`、`bright_red`、`bright_green`、`bright_yellow`、
    `bright_blue`、`bright_magenta`、`bright_purple`、`bright_cyan` 和
    `bright_white`。
  - `bold`（boolean，可选）：是否加粗。
  - `dimmed`（boolean，可选）：是否使用弱化显示。
  - `italic`（boolean，可选）：是否使用斜体。
  - `underline`（boolean，可选）：是否加下划线。
- 返回：`string`。

行为说明：

- 如果 `enabled = false`，会原样返回原始文本。
- 如果 `fg = nil` 或省略，则不设置前景色。
- 未知选项名或非法选项值类型都会抛出错误。

示例：

```lua
print(ptool.ansi.style("warning", {
  fg = "bright_yellow",
  bold = true,
}))
```

## ptool.ansi.\<color\>

> `v0.1.0` - 引入。

`ptool.ansi.black`、`ptool.ansi.red`、`ptool.ansi.green`、
`ptool.ansi.yellow`、`ptool.ansi.blue`、`ptool.ansi.magenta`、
`ptool.ansi.cyan` 和 `ptool.ansi.white` 都是便捷辅助函数，签名如下：

```lua
ptool.ansi.red(text[, options])
```

它们接受与 `ptool.ansi.style` 相同的 `text` 参数和 `options` 表，只是前景色
由辅助函数自身固定。如果同时提供了 `options.fg`，则以辅助函数自身的颜色为准。

示例：

```lua
print(ptool.ansi.green("ok", { bold = true }))
print(ptool.ansi.red("failed", { enabled = true, underline = true }))
```
