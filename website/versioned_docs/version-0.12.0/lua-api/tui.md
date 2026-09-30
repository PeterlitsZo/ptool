# TUI API

Terminal UI helpers are available under `ptool.tui` and `p.tui`.

The first version focuses on small interactive terminal screens driven by a Lua
event loop.

## ptool.tui.run

> `v0.6.0` - Introduced.

`ptool.tui.run(options)` starts a TUI session, runs the Lua lifecycle
callbacks, and returns the value passed to `app:quit(value)`. If `app:quit()`
is called without a value, the function returns `nil`.

`options` fields:

- `tick_ms` (integer, optional): Tick interval in milliseconds. Defaults to
  `100`.
- `init` (function, optional): Called once before the first frame is drawn. Its
  return value becomes the initial `state`.
- `update` (function, required): Called as `update(state, event, app)` after
  each input event or tick. If it returns a non-`nil` value, that value becomes
  the next `state`. Returning `nil` keeps the current `state`.
- `view` (function, required): Called as `view(state, app)` to build the root
  node for the next frame.

Behavior:

- Requires an interactive TTY.
- Runs inside the terminal alternate screen with raw mode enabled.
- Restores the terminal when the session exits, including error paths.

Events:

- `{ type = "tick" }`
- `{ type = "resize", width = <integer>, height = <integer> }`
- `{ type = "key", key = <string>, ctrl = <boolean>, alt = <boolean>, shift = <boolean> }`

Common key names include `up`, `down`, `left`, `right`, `enter`, `esc`,
`tab`, `backspace`, `delete`, `home`, `end`, `pageup`, and `pagedown`.
Character keys use the character itself, such as `"q"` or `"j"`.

Example:

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

> `v0.6.0` - Introduced.

`app` is passed to `update(...)` and `view(...)`.

### quit

Canonical API name: `ptool.tui.App:quit`.

`app:quit(value?)` requests that the TUI session stop after the current
callback completes.

- `value` (Lua value, optional): The value returned by `ptool.tui.run(...)`.

## Node Constructors

> `v0.6.0` - Introduced.

The constructor helpers below return plain node tables. `view(...)` must return
one of these nodes as the root widget tree.

Common node fields:

- `title` (string, optional): Draws a block title around the node.
- `border` (boolean, optional): Draws a border around the node. Defaults to
  `false`.
- `padding` (integer, optional): Uniform inner padding. Defaults to `0`.
- `grow` (integer, optional): Relative size when the node is inside a row or
  column. Defaults to `1`.
- `style` (table, optional): Shared style fields:
  - `fg` / `bg`: One of `black`, `red`, `green`, `yellow`, `blue`, `magenta`,
    `cyan`, `white`, `gray`, or `dark_gray`.
  - `bold`, `dim`, `italic`, `underlined`, `reversed`: Boolean style toggles.

## ptool.tui.text

> `v0.6.0` - Introduced.

`ptool.tui.text(text[, options])` creates a text node.

- `text` (string, required): The text to render.
- `options.align` (string, optional): `left`, `center`, or `right`. Defaults to
  `left`.

## ptool.tui.list

> `v0.6.0` - Introduced.

`ptool.tui.list(items[, options])` creates a vertical list node.

- `items` (table, required): A dense list table. Item values may be strings,
  numbers, or booleans.
- `options.selected` (integer, optional): The selected row using Lua's 1-based
  indexing. Values beyond the item count are ignored.
- `options.highlight_style` (table, optional): Style applied to the selected
  row. Uses the same keys as `style`.

Notes:

- Selection is only visually distinct when `highlight_style` changes the
  rendered style.

## ptool.tui.row

> `v0.6.0` - Introduced.

`ptool.tui.row(options)` creates a horizontal container.

- `options.children` (table, required): A dense list table of child nodes.

## ptool.tui.column

> `v0.6.0` - Introduced.

`ptool.tui.column(options)` creates a vertical container.

- `options.children` (table, required): A dense list table of child nodes.

## Current Limits

`ptool.tui` currently supports:

- `tick`, `resize`, and keyboard events.
- Text, list, row, and column nodes.
- Basic block decoration and style options.

It does not yet provide text inputs, popups, tables, mouse-driven widgets, or
arbitrary ratatui widget bindings.
