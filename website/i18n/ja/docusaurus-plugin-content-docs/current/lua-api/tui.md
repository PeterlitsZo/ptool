# TUI API

端末 UI ヘルパーは `ptool.tui` と `p.tui` で利用できます。

最初のバージョンは、Lua のイベントループで駆動する小さな対話型端末画面に焦点を当てています。

## ptool.tui.run

> `v0.6.0` - 導入されました。

`ptool.tui.run(options)` は TUI セッションを開始し、Lua のライフサイクルコールバックを実行し、`app:quit(value)` に渡した値を返します。`app:quit()` を値なしで呼んだ場合は `nil` を返します。

`options` のフィールド:

- `tick_ms` (integer, optional): tick 間隔をミリ秒で指定します。既定値は `100` です。
- `init` (function, optional): 最初のフレームを描画する前に一度だけ呼ばれます。戻り値は初期 `state` になります。
- `update` (function, required): `update(state, event, app)` として、各入力イベントまたは tick の後に呼ばれます。`nil` 以外の値を返した場合、その値が次の `state` になります。`nil` を返すと現在の `state` を維持します。
- `view` (function, required): `view(state, app)` として呼ばれ、次のフレームのルートノードを構築します。

動作:

- 対話型 TTY が必要です。
- 端末の alternate screen と raw mode を使って実行されます。
- エラー経路を含め、終了時に端末状態を復元します。

イベント:

- `{ type = "tick" }`
- `{ type = "resize", width = <integer>, height = <integer> }`
- `{ type = "key", key = <string>, ctrl = <boolean>, alt = <boolean>, shift = <boolean> }`

主なキー名には `up`、`down`、`left`、`right`、`enter`、`esc`、`tab`、`backspace`、`delete`、`home`、`end`、`pageup`、`pagedown` があります。文字キーは `"q"` や `"j"` のように文字そのものを使います。

例:

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

> `v0.6.0` - 導入されました。

`app` は `update(...)` と `view(...)` に渡されます。

### quit

Canonical API name: `ptool.tui.App:quit`.

`app:quit(value?)` は、現在のコールバックが終わったあとに TUI セッションを停止するよう要求します。

- `value` (Lua value, optional): `ptool.tui.run(...)` の戻り値になります。

## ノードコンストラクタ

> `v0.6.0` - 導入されました。

以下のコンストラクタは通常のノードテーブルを返します。`view(...)` は、ルートのビュー木としてこれらのノードのいずれかを返す必要があります。

共通ノードフィールド:

- `title` (string, optional): ノードの周囲にタイトル付きブロックを描画します。
- `border` (boolean, optional): ノードの周囲に枠線を描画します。既定値は `false` です。
- `padding` (integer, optional): 均一な内側余白です。既定値は `0` です。
- `grow` (integer, optional): ノードが row または column の中にあるときの相対サイズです。既定値は `1` です。
- `style` (table, optional): 共通のスタイルフィールド:
  - `fg` / `bg`: `black`、`red`、`green`、`yellow`、`blue`、`magenta`、`cyan`、`white`、`gray`、`dark_gray` のいずれか。
  - `bold`、`dim`、`italic`、`underlined`、`reversed`: 真偽値のスタイルフラグ。

## ptool.tui.text

> `v0.6.0` - 導入されました。

`ptool.tui.text(text[, options])` はテキストノードを作成します。

- `text` (string, required): 描画するテキスト。
- `options.align` (string, optional): `left`、`center`、`right`。既定値は `left` です。

## ptool.tui.list

> `v0.6.0` - 導入されました。

`ptool.tui.list(items[, options])` は縦方向のリストノードを作成します。

- `items` (table, required): 密なリストテーブル。要素の値には文字列、数値、真偽値を使えます。
- `options.selected` (integer, optional): Lua の 1 始まりの添字で選択行を指定します。要素数を超える値は無視されます。
- `options.highlight_style` (table, optional): 選択行に適用するスタイルです。`style` と同じキーを使います。

注意:

- `highlight_style` が描画スタイルを変えない限り、選択は視覚的にはっきり区別されません。

## ptool.tui.row

> `v0.6.0` - 導入されました。

`ptool.tui.row(options)` は横方向コンテナを作成します。

- `options.children` (table, required): 子ノードを並べた密なリストテーブル。

## ptool.tui.column

> `v0.6.0` - 導入されました。

`ptool.tui.column(options)` は縦方向コンテナを作成します。

- `options.children` (table, required): 子ノードを並べた密なリストテーブル。

## 現在の制限

`ptool.tui` が現在サポートするもの:

- `tick`、`resize`、キーボードイベント。
- text、list、row、column ノード。
- 基本的なブロック装飾とスタイルオプション。

現時点では、テキスト入力、ポップアップ、表、マウス駆動のウィジェット、任意の ratatui ウィジェットバインディングはまだ提供していません。
