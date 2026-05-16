# API de shell

Las utilidades de análisis de shell están disponibles bajo `ptool.sh` y `p.sh`.

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split(command)` analiza una cadena de comando usando reglas de estilo shell y devuelve un arreglo de argumentos.

- `command` (string, obligatorio): La cadena de comando que se va a dividir.
- Devuelve: `string[]`.

Ejemplo:

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

El `args` anterior equivale a:

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```
