# API de shell

Las utilidades de análisis de shell están disponibles bajo `ptool.sh` y `p.sh`.

Estas utilidades trabajan al nivel de shell words. Están pensadas para dividir, citar y unir cadenas de argumentos usando reglas de shell al estilo POSIX, no para analizar la sintaxis completa de shell como tuberías, redirecciones, sustitución de comandos o expansión de variables.

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split(command)` analiza una cadena de comando usando reglas de estilo shell y devuelve un arreglo de argumentos.

- `command` (string, obligatorio): La cadena de comando que se va a dividir.
- Devuelve: `string[]`.

Comportamiento:

- Esto analiza únicamente shell words. No interpreta operadores de shell ni ejecuta expansiones.

Ejemplo:

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

El `args` anterior equivale a:

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```

## ptool.sh.quote

> `v0.10.0` - Introducido.

`ptool.sh.quote(word)` cita una única shell word para que pueda incrustarse de forma segura en una cadena de comando de shell.

- `word` (string, obligatorio): La shell word que se va a citar.
- Devuelve: `string`.

Comportamiento:

- La cadena devuelta es segura para shell y semánticamente equivalente a la word de entrada.
- Esto preserva el significado como shell word, no la forma textual original.

Ejemplo:

```lua
local word = ptool.sh.quote("hello world")
print(word) -- 'hello world'
```

## ptool.sh.join

> `v0.10.0` - Introducido.

`ptool.sh.join(words)` une un arreglo de argumentos en una cadena de comando de shell, citando las words cuando hace falta.

- `words` (string[], obligatorio): Las shell words que se van a unir.
- Devuelve: `string`.

Comportamiento:

- Las words consecutivas se unen con un solo espacio.
- La salida es adecuada para pasarla a un shell de estilo POSIX.
- Esto busca round-tripping al nivel de shell words, por lo que `ptool.sh.split(ptool.sh.join(words))` es equivalente a `words`.
- `ptool.sh.join(ptool.sh.split(command))` puede normalizar el citado y los espacios en lugar de preservar el texto original del comando.

Ejemplo:

```lua
local cmd = ptool.sh.join({"git", "commit", "-m", "hello world"})
print(cmd) -- git commit -m 'hello world'
```
