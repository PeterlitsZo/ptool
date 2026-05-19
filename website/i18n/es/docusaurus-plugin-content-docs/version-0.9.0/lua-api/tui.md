# API TUI

Las utilidades de interfaz de terminal están disponibles bajo `ptool.tui` y `p.tui`.

La primera versión se centra en pantallas interactivas pequeñas controladas por un bucle de eventos en Lua.

## ptool.tui.run

> `v0.6.0` - Introducido.

`ptool.tui.run(options)` inicia una sesión TUI, ejecuta los callbacks del ciclo de vida en Lua y devuelve el valor pasado a `app:quit(value)`. Si se llama a `app:quit()` sin valor, la función devuelve `nil`.

Campos de `options`:

- `tick_ms` (integer, opcional): Intervalo de tick en milisegundos. El valor por defecto es `100`.
- `init` (function, opcional): Se llama una vez antes de dibujar el primer frame. Su valor de retorno se convierte en el `state` inicial.
- `update` (function, obligatorio): Se llama como `update(state, event, app)` después de cada evento de entrada o tick. Si devuelve un valor distinto de `nil`, ese valor se convierte en el siguiente `state`. Si devuelve `nil`, se conserva el `state` actual.
- `view` (function, obligatorio): Se llama como `view(state, app)` para construir el nodo raíz del siguiente frame.

Comportamiento:

- Requiere un TTY interactivo.
- Se ejecuta dentro de la pantalla alternativa del terminal con raw mode activado.
- Restaura el terminal al salir, incluso en rutas de error.

Eventos:

- `{ type = "tick" }`
- `{ type = "resize", width = <integer>, height = <integer> }`
- `{ type = "key", key = <string>, ctrl = <boolean>, alt = <boolean>, shift = <boolean> }`

Los nombres de tecla comunes incluyen `up`, `down`, `left`, `right`, `enter`, `esc`, `tab`, `backspace`, `delete`, `home`, `end`, `pageup` y `pagedown`. Las teclas de caracteres usan el propio carácter, como `"q"` o `"j"`.

Ejemplo:

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

> `v0.6.0` - Introducido.

`app` se pasa a `update(...)` y `view(...)`.

### quit

Canonical API name: `ptool.tui.App:quit`.

`app:quit(value?)` solicita detener la sesión TUI cuando termine el callback actual.

- `value` (valor Lua, opcional): El valor devuelto por `ptool.tui.run(...)`.

## Constructores de nodos

> `v0.6.0` - Introducido.

Los constructores siguientes devuelven tablas de nodo normales. `view(...)` debe devolver uno de estos nodos como árbol de vista raíz.

Campos comunes de nodo:

- `title` (string, opcional): Dibuja un bloque con título alrededor del nodo.
- `border` (boolean, opcional): Dibuja un borde alrededor del nodo. El valor por defecto es `false`.
- `padding` (integer, opcional): Relleno interior uniforme. El valor por defecto es `0`.
- `grow` (integer, opcional): Tamaño relativo cuando el nodo está dentro de una fila o columna. El valor por defecto es `1`.
- `style` (table, opcional): Campos de estilo compartidos:
  - `fg` / `bg`: Uno de `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray` o `dark_gray`.
  - `bold`, `dim`, `italic`, `underlined`, `reversed`: Banderas booleanas de estilo.

## ptool.tui.text

> `v0.6.0` - Introducido.

`ptool.tui.text(text[, options])` crea un nodo de texto.

- `text` (string, obligatorio): El texto a renderizar.
- `options.align` (string, opcional): `left`, `center` o `right`. El valor por defecto es `left`.

## ptool.tui.list

> `v0.6.0` - Introducido.

`ptool.tui.list(items[, options])` crea un nodo de lista vertical.

- `items` (table, obligatorio): Una tabla de lista densa. Los valores de los elementos pueden ser cadenas, números o booleanos.
- `options.selected` (integer, opcional): La fila seleccionada usando el índice base 1 de Lua. Los valores mayores que la cantidad de elementos se ignoran.
- `options.highlight_style` (table, opcional): Estilo aplicado a la fila seleccionada. Usa las mismas claves que `style`.

Notas:

- La selección solo se distingue visualmente cuando `highlight_style` cambia el estilo renderizado.

## ptool.tui.row

> `v0.6.0` - Introducido.

`ptool.tui.row(options)` crea un contenedor horizontal.

- `options.children` (table, obligatorio): Una tabla de lista densa con nodos hijos.

## ptool.tui.column

> `v0.6.0` - Introducido.

`ptool.tui.column(options)` crea un contenedor vertical.

- `options.children` (table, obligatorio): Una tabla de lista densa con nodos hijos.

## Límites actuales

`ptool.tui` actualmente soporta:

- Eventos `tick`, `resize` y de teclado.
- Nodos text, list, row y column.
- Decoración básica de bloques y opciones de estilo.

Todavía no ofrece entradas de texto, popups, tablas, widgets controlados por ratón ni bindings arbitrarios de widgets de ratatui.
