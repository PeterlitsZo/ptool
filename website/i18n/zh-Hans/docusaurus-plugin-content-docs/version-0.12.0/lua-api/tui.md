# TUI API

终端 UI 辅助能力位于 `ptool.tui` 和 `p.tui` 下。

第一版聚焦于由 Lua 事件循环驱动的小型交互式终端界面。

## ptool.tui.run

> `v0.6.0` - 引入。

`ptool.tui.run(options)` 启动一个 TUI 会话，运行 Lua 生命周期回调，并返回 `app:quit(value)` 传入的值。如果调用的是 `app:quit()`，则返回 `nil`。

`options` 字段：

- `tick_ms`（integer，可选）：tick 间隔，单位毫秒。默认值为 `100`。
- `init`（function，可选）：在绘制第一帧前调用一次。它的返回值会成为初始 `state`。
- `update`（function，必填）：以 `update(state, event, app)` 的形式调用，在每次输入事件或 tick 后执行。如果它返回非 `nil` 值，该值会成为下一次的 `state`。返回 `nil` 会保留当前 `state`。
- `view`（function，必填）：以 `view(state, app)` 的形式调用，用来构建下一帧的根节点。

行为：

- 需要交互式 TTY。
- 运行时会进入终端 alternate screen 并启用 raw mode。
- 退出时会恢复终端状态，包括错误路径。

事件：

- `{ type = "tick" }`
- `{ type = "resize", width = <integer>, height = <integer> }`
- `{ type = "key", key = <string>, ctrl = <boolean>, alt = <boolean>, shift = <boolean> }`

常见按键名包括 `up`、`down`、`left`、`right`、`enter`、`esc`、`tab`、`backspace`、`delete`、`home`、`end`、`pageup` 和 `pagedown`。字符键直接使用字符本身，例如 `"q"` 或 `"j"`。

示例：

```lua
local result = p.tui.run({
  tick_ms = 200,
  init = function()
    return {
      items = {"alpha", "beta", "gamma"},
      selected = 1,
    }
  end,
  update = function(state, event, app)
    if event.type == "key" then
      if event.key == "q" then
        app:quit(state.items[state.selected])
      elseif event.key == "down" then
        state.selected = math.min(#state.items, state.selected + 1)
      elseif event.key == "up" then
        state.selected = math.max(1, state.selected - 1)
      end
    end
  end,
  view = function(state)
    return p.tui.column({
      title = "Demo",
      border = true,
      padding = 1,
      children = {
        p.tui.text("Press q to quit", {
          align = "center",
        }),
        p.tui.list(state.items, {
          selected = state.selected,
          highlight_style = {
            reversed = true,
          },
        }),
      },
    })
  end,
})

print("selected:", result)
```

## App

> `v0.6.0` - 引入。

`app` 会传给 `update(...)` 和 `view(...)`。

### quit

规范 API 名称：`ptool.tui.App:quit`。

`app:quit(value?)` 会请求在当前回调结束后停止 TUI 会话。

- `value`（Lua 值，可选）：`ptool.tui.run(...)` 的返回值。

## 节点构造器

> `v0.6.0` - 引入。

下面这些构造器返回普通的节点 table。`view(...)` 必须返回其中一个节点作为根视图树。

通用节点字段：

- `title`（string，可选）：为节点绘制带标题的 block。
- `border`（boolean，可选）：为节点绘制边框。默认值为 `false`。
- `padding`（integer，可选）：统一的内边距。默认值为 `0`。
- `grow`（integer，可选）：节点位于 row 或 column 中时的相对尺寸。默认值为 `1`。
- `style`（table，可选）：通用样式字段：
  - `fg` / `bg`：可选值为 `black`、`red`、`green`、`yellow`、`blue`、`magenta`、`cyan`、`white`、`gray`、`dark_gray`。
  - `bold`、`dim`、`italic`、`underlined`、`reversed`：布尔样式开关。

## ptool.tui.text

> `v0.6.0` - 引入。

`ptool.tui.text(text[, options])` 创建一个文本节点。

- `text`（string，必填）：要渲染的文本。
- `options.align`（string，可选）：`left`、`center` 或 `right`。默认值为 `left`。

## ptool.tui.list

> `v0.6.0` - 引入。

`ptool.tui.list(items[, options])` 创建一个垂直列表节点。

- `items`（table，必填）：一个致密 list table。元素值可以是字符串、数字或布尔值。
- `options.selected`（integer，可选）：使用 Lua 的 1-based 索引指定选中行。超出元素数量的值会被忽略。
- `options.highlight_style`（table，可选）：应用到选中行的样式。使用与 `style` 相同的键。

说明：

- 只有当 `highlight_style` 改变渲染样式时，选中项才会在视觉上明显区分。

## ptool.tui.row

> `v0.6.0` - 引入。

`ptool.tui.row(options)` 创建一个水平容器。

- `options.children`（table，必填）：一个由子节点组成的致密 list table。

## ptool.tui.column

> `v0.6.0` - 引入。

`ptool.tui.column(options)` 创建一个垂直容器。

- `options.children`（table，必填）：一个由子节点组成的致密 list table。

## 当前限制

`ptool.tui` 当前支持：

- `tick`、`resize` 和键盘事件。
- text、list、row 和 column 节点。
- 基础的 block 装饰和样式选项。

暂时还不提供文本输入框、弹窗、表格、鼠标驱动控件或任意 ratatui widget 绑定。
