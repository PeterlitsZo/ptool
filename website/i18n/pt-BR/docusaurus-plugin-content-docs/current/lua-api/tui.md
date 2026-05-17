# API TUI

As utilidades de interface de terminal estão disponíveis em `ptool.tui` e `p.tui`.

A primeira versão se concentra em telas interativas pequenas dirigidas por um loop de eventos em Lua.

## ptool.tui.run

> `v0.6.0` - Introduzido.

`ptool.tui.run(options)` inicia uma sessão TUI, executa os callbacks do ciclo de vida em Lua e retorna o valor passado para `app:quit(value)`. Se `app:quit()` for chamado sem valor, a função retorna `nil`.

Campos de `options`:

- `tick_ms` (integer, opcional): Intervalo de tick em milissegundos. O padrão é `100`.
- `init` (function, opcional): É chamado uma vez antes de desenhar o primeiro frame. O valor retornado se torna o `state` inicial.
- `update` (function, obrigatório): É chamado como `update(state, event, app)` após cada evento de entrada ou tick. Se retornar um valor diferente de `nil`, esse valor se torna o próximo `state`. Retornar `nil` mantém o `state` atual.
- `view` (function, obrigatório): É chamado como `view(state, app)` para montar o nó raiz do próximo frame.

Comportamento:

- Requer um TTY interativo.
- Roda dentro da alternate screen do terminal com raw mode ativado.
- Restaura o terminal ao sair, inclusive em caminhos de erro.

Eventos:

- `{ type = "tick" }`
- `{ type = "resize", width = <integer>, height = <integer> }`
- `{ type = "key", key = <string>, ctrl = <boolean>, alt = <boolean>, shift = <boolean> }`

Nomes de tecla comuns incluem `up`, `down`, `left`, `right`, `enter`, `esc`, `tab`, `backspace`, `delete`, `home`, `end`, `pageup` e `pagedown`. Teclas de caractere usam o próprio caractere, como `"q"` ou `"j"`.

Exemplo:

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

> `v0.6.0` - Introduzido.

`app` é passado para `update(...)` e `view(...)`.

### quit

Canonical API name: `ptool.tui.App:quit`.

`app:quit(value?)` solicita que a sessão TUI pare depois que o callback atual terminar.

- `value` (valor Lua, opcional): O valor retornado por `ptool.tui.run(...)`.

## Construtores de nó

> `v0.6.0` - Introduzido.

Os construtores abaixo retornam tabelas de nó comuns. `view(...)` deve retornar um desses nós como a árvore de visualização raiz.

Campos comuns de nó:

- `title` (string, opcional): Desenha um bloco com título ao redor do nó.
- `border` (boolean, opcional): Desenha uma borda ao redor do nó. O padrão é `false`.
- `padding` (integer, opcional): Espaçamento interno uniforme. O padrão é `0`.
- `grow` (integer, opcional): Tamanho relativo quando o nó está dentro de um row ou column. O padrão é `1`.
- `style` (table, opcional): Campos de estilo compartilhados:
  - `fg` / `bg`: Um de `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray` ou `dark_gray`.
  - `bold`, `dim`, `italic`, `underlined`, `reversed`: Flags booleanas de estilo.

## ptool.tui.text

> `v0.6.0` - Introduzido.

`ptool.tui.text(text[, options])` cria um nó de texto.

- `text` (string, obrigatório): O texto a renderizar.
- `options.align` (string, opcional): `left`, `center` ou `right`. O padrão é `left`.

## ptool.tui.list

> `v0.6.0` - Introduzido.

`ptool.tui.list(items[, options])` cria um nó de lista vertical.

- `items` (table, obrigatório): Uma tabela de lista densa. Os valores dos itens podem ser strings, números ou booleanos.
- `options.selected` (integer, opcional): A linha selecionada usando a indexação base 1 do Lua. Valores maiores que a quantidade de itens são ignorados.
- `options.highlight_style` (table, opcional): Estilo aplicado à linha selecionada. Usa as mesmas chaves de `style`.

Notas:

- A seleção só fica visualmente distinta quando `highlight_style` muda o estilo renderizado.

## ptool.tui.row

> `v0.6.0` - Introduzido.

`ptool.tui.row(options)` cria um contêiner horizontal.

- `options.children` (table, obrigatório): Uma tabela de lista densa com nós filhos.

## ptool.tui.column

> `v0.6.0` - Introduzido.

`ptool.tui.column(options)` cria um contêiner vertical.

- `options.children` (table, obrigatório): Uma tabela de lista densa com nós filhos.

## Limites atuais

`ptool.tui` atualmente suporta:

- Eventos `tick`, `resize` e de teclado.
- Nós text, list, row e column.
- Decoração básica de bloco e opções de estilo.

Ainda não oferece campos de texto, popups, tabelas, widgets dirigidos por mouse nem bindings arbitrários de widgets do ratatui.
