# API principal de Lua

`ptool` expone estos helpers principales del runtime directamente bajo `ptool`
y `p`.

`ptool run <lua_file>` ejecuta un script Lua e inyecta la variable global
`ptool` (o su alias `p`; por ejemplo, `p.run` es equivalente a `ptool.run`).
Para archivos que terminan en `.lua`, `ptool <lua_file>` es un atajo de CLI
con el mismo comportamiento.

El runtime Lua embebido conserva los globales básicos de Lua y, por defecto,
solo expone estas bibliotecas estándar:

- `table`
- `string`
- `math`
- `utf8`

Los módulos integrados orientados al host, como `io`, `os` y `package`, no
están disponibles intencionadamente. Usa APIs de `ptool` como `ptool.fs`,
`ptool.os`, `ptool.path` y `ptool.run` para operaciones de sistema de
archivos, entorno, procesos, red y demás tareas de runtime.

Si quieres pasar argumentos a un script Lua, puedes hacerlo así:

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

Luego los argumentos pueden analizarse con `ptool.args.parse(...)`.

Aquí tienes un script de ejemplo:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

Se admite shebang, así que puedes añadir esto al comienzo del archivo:

```
#!/usr/bin/env ptool
```

## ptool.use

> `v0.1.0` - Introduced.

`ptool.use` declara la versión mínima de `ptool` requerida por un script.

```lua
ptool.use("v0.1.0")
```

- El argumento es una cadena de versión semántica (SemVer) y admite un prefijo
  `v` opcional, como `v0.1.0` o `0.1.0`.
- Si la versión requerida es superior a la versión actual de `ptool`, el script
  sale de inmediato con un error indicando que la versión actual es demasiado
  antigua.

## ptool.unindent

> `v0.1.0` - Introduced.

`ptool.unindent` procesa cadenas multilínea eliminando el prefijo `| ` después
de la indentación inicial en cada línea y recortando las líneas en blanco del
principio y del final.

```lua
local str = ptool.unindent([[
  | line 1
  | line 2
]])
```

Esto equivale a:

```lua
local str = [[line 1
line 2]]
```

## ptool.inspect

> `v0.1.0` - Introduced.

`ptool.inspect(value[, options])` representa un valor Lua como una cadena
legible al estilo Lua. Su objetivo principal es depurar y mostrar el contenido
de tablas.

- `value` (any, obligatorio): El valor Lua que se va a inspeccionar.
- `options` (table, opcional): Opciones de renderizado. Campos admitidos:
  - `indent` (string, opcional): Indentación usada en cada nivel de anidación.
    Por defecto son dos espacios.
  - `multiline` (boolean, opcional): Si las tablas se representan en varias
    líneas. Por defecto es `true`.
  - `max_depth` (integer, opcional): Profundidad máxima de anidación que se
    representará. Los valores más profundos se reemplazan por `<max-depth>`.
- Devuelve: `string`.

Comportamiento:

- Las entradas tipo arreglo (`1..n`) se representan primero.
- Los demás campos de la tabla se representan después de la parte tipo arreglo,
  con un orden estable por clave.
- Las claves de cadena con forma de identificador se representan como
  `key = value`; las demás se representan como `[key] = value`.
- Las referencias recursivas a tablas se representan como `<cycle>`.
- Las funciones, threads y userdata se representan con valores marcadores como
  `<function>` y `<userdata>`.

Ejemplo:

```lua
local value = {
  "hello",
  user = { name = "alice", tags = {"dev", "ops"} },
}
value.self = value

print(ptool.inspect(value))
print(ptool.inspect(value, { multiline = false }))
```

## ptool.ask

> `v0.1.0` - Introduced.
> `v0.5.0` - Added validation options and prompt subcommands.

`ptool.ask` ofrece prompts interactivos. Puedes llamarlo directamente para
pedir texto, o usar sus subprompts para confirmación, selección simple,
selección múltiple y entrada secreta.

Comportamiento común:

- Todos los prompts de `ptool.ask` requieren un TTY interactivo. Ejecutarlos
  en un entorno no interactivo produce un error.
- Si el usuario cancela un prompt, el script produce un error.
- Los nombres de opción desconocidos o los tipos de valor no válidos producen
  un error.

### ptool.ask

`ptool.ask(prompt[, options])` pide al usuario una línea de texto y devuelve la
respuesta.

- `prompt` (string, obligatorio): El prompt que se muestra al usuario.
- `options` (table, opcional): Opciones del prompt. Campos admitidos:
  - `default` (string, opcional): Valor por defecto usado cuando el usuario
    envía una respuesta vacía.
  - `help` (string, opcional): Texto de ayuda adicional mostrado bajo el
    prompt.
  - `placeholder` (string, opcional): Texto placeholder mostrado antes de que
    el usuario empiece a escribir.
  - `required` (boolean, opcional): Si la respuesta debe ser no vacía.
  - `allow_empty` (boolean, opcional): Si se acepta una respuesta vacía.
    El valor por defecto es `true`.
  - `trim` (boolean, opcional): Si se deben recortar los espacios al inicio y
    al final antes de devolver la respuesta.
  - `min_length` (integer, opcional): Longitud mínima aceptada.
  - `max_length` (integer, opcional): Longitud máxima aceptada.
  - `pattern` (string, opcional): Expresión regular que la respuesta debe
    cumplir.
- Devuelve: `string`.

Ejemplo:

```lua
local project = ptool.ask("Project name?", {
  placeholder = "my-tool",
  help = "Lowercase letters, digits, and dashes only",
  required = true,
  trim = true,
  pattern = "^[a-z0-9-]+$",
})
```

### ptool.ask.confirm

> `v0.5.0` - Introduced.

`ptool.ask.confirm(prompt[, options])` pide al usuario una respuesta de sí/no.

- `prompt` (string, obligatorio): El prompt que se muestra al usuario.
- `options` (table, opcional): Opciones del prompt. Campos admitidos:
  - `default` (boolean, opcional): Respuesta por defecto cuando el usuario
    pulsa Enter sin escribir.
  - `help` (string, opcional): Texto de ayuda adicional mostrado bajo el
    prompt.
- Devuelve: `boolean`.

Ejemplo:

```lua
local confirmed = ptool.ask.confirm("Continue?", {
  default = true,
})
```

### ptool.ask.select

> `v0.5.0` - Introduced.

`ptool.ask.select(prompt, items[, options])` pide al usuario elegir un elemento
de una lista.

- `prompt` (string, obligatorio): El prompt que se muestra al usuario.
- `items` (table, obligatorio): Elementos candidatos. Cada entrada puede ser:
  - Un string, usado como etiqueta mostrada y como valor devuelto.
  - Un table como `{ label = "Patch", value = "patch" }`.
- `options` (table, opcional): Opciones del prompt. Campos admitidos:
  - `help` (string, opcional): Texto de ayuda adicional mostrado bajo el
    prompt.
  - `page_size` (integer, opcional): Número máximo de filas mostradas a la vez.
  - `default_index` (integer, opcional): Índice 1-based del elemento
    inicialmente seleccionado.
- Devuelve: `string`.

Ejemplo:

```lua
local bump = ptool.ask.select("Select bump type", {
  { label = "Patch", value = "patch" },
  { label = "Minor", value = "minor" },
  { label = "Major", value = "major" },
}, {
  default_index = 2,
})
```

### ptool.ask.multiselect

> `v0.5.0` - Introduced.

`ptool.ask.multiselect(prompt, items[, options])` pide al usuario elegir cero o
más elementos de una lista.

- `prompt` (string, obligatorio): El prompt que se muestra al usuario.
- `items` (table, obligatorio): Elementos candidatos. El formato es el mismo
  que `ptool.ask.select`.
- `options` (table, opcional): Opciones del prompt. Campos admitidos:
  - `help` (string, opcional): Texto de ayuda adicional mostrado bajo el
    prompt.
  - `page_size` (integer, opcional): Número máximo de filas mostradas a la vez.
  - `default_indexes` (table, opcional): Índices 1-based seleccionados por
    defecto.
  - `min_selected` (integer, opcional): Cantidad mínima de elementos que deben
    seleccionarse.
  - `max_selected` (integer, opcional): Cantidad máxima de elementos que pueden
    seleccionarse.
- Devuelve: `table`.

Ejemplo:

```lua
local targets = ptool.ask.multiselect("Select targets", {
  "linux",
  "macos",
  "windows",
}, {
  default_indexes = { 1, 2 },
  min_selected = 1,
})
```

### ptool.ask.secret

> `v0.5.0` - Introduced.

`ptool.ask.secret(prompt[, options])` pide al usuario una entrada secreta, como
un token o una contraseña.

- `prompt` (string, obligatorio): El prompt que se muestra al usuario.
- `options` (table, opcional): Opciones del prompt. Campos admitidos:
  - `help` (string, opcional): Texto de ayuda adicional mostrado bajo el
    prompt.
  - `required` (boolean, opcional): Si la respuesta debe ser no vacía.
  - `allow_empty` (boolean, opcional): Si se acepta una respuesta vacía.
    El valor por defecto es `false`.
  - `confirm` (boolean, opcional): Si se debe pedir al usuario que escriba el
    secreto dos veces. El valor por defecto es `false`.
  - `confirm_prompt` (string, opcional): Prompt personalizado para el paso de
    confirmación.
  - `mismatch_message` (string, opcional): Mensaje de error personalizado
    cuando las dos respuestas no coinciden.
  - `display_toggle` (boolean, opcional): Si se permite mostrar temporalmente
    el secreto escrito.
  - `min_length` (integer, opcional): Longitud mínima aceptada.
  - `max_length` (integer, opcional): Longitud máxima aceptada.
  - `pattern` (string, opcional): Expresión regular que la respuesta debe
    cumplir.
- Devuelve: `string`.

Ejemplo:

```lua
local token = ptool.ask.secret("API token?", {
  confirm = true,
  min_length = 20,
})
```

## ptool.config

> `v0.1.0` - Introduced.

`ptool.config` establece la configuración de runtime del script.

Campos admitidos actualmente:

- `run` (table, opcional): Configuración por defecto de `ptool.run`. Campos
  admitidos:
  - `echo` (boolean, opcional): Interruptor echo por defecto. Por defecto es
    `true`.
  - `check` (boolean, opcional): Si los fallos deben producir error por
    defecto. Por defecto es `false`.
  - `confirm` (boolean, opcional): Si por defecto debe requerirse confirmación
    antes de ejecutar. Por defecto es `false`.
  - `retry` (boolean, opcional): Si debe preguntarse al usuario si quiere
    reintentar después de una ejecución fallida cuando `check = true`. Por
    defecto es `false`.

Ejemplo:

```lua
ptool.config({
  run = {
    echo = false,
    check = true,
    confirm = false,
    retry = false,
  },
})
```

## ptool.cd

> `v0.1.0` - Introduced.

`ptool.cd(path)` actualiza el directorio actual de runtime de `ptool`.

- `path` (string, obligatorio): Ruta del directorio destino, absoluta o
  relativa.

Comportamiento:

- Las rutas relativas se resuelven desde el directorio de runtime actual de
  `ptool`.
- El destino debe existir y debe ser un directorio.
- Esto actualiza el estado interno de runtime de `ptool` y afecta a las APIs
  que usan ese cwd de runtime (como `ptool.run`, `ptool.path.abspath` y
  `ptool.path.relpath`).

Ejemplo:

```lua
ptool.cd("foobar")
local res = ptool.run({ cmd = "pwd", stdout = "capture" })
print(res.stdout)
```

## ptool.script_path

> `v0.4.0` - Introduced.

`ptool.script_path()` devuelve la ruta absoluta del script de entrada actual.

- Devuelve: `string|nil`.

Comportamiento:

- Al ejecutar con `ptool run <file>`, devuelve la ruta del script de entrada
  como una ruta absoluta y normalizada.
- La ruta devuelta queda fijada cuando se inicia el runtime y no cambia después
  de `ptool.cd(...)`.
- En `ptool repl`, devuelve `nil`.

Ejemplo:

```lua
local script_path = ptool.script_path()
local script_dir = ptool.path.dirname(script_path)
local project_root = ptool.path.dirname(script_dir)
```

## ptool.try

> `v0.4.0` - Introduced.

`ptool.try(fn)` ejecuta `fn` y convierte los errores lanzados en valores de
retorno.

- `fn` (function, obligatorio): Callback que se va a ejecutar.
- Devuelve: `ok, value, err`.

Reglas del valor devuelto:

- En caso de éxito, `ok = true`, `err = nil` y `value` contiene el resultado
  del callback.
- Si el callback no devuelve valores, `value` es `nil`.
- Si el callback devuelve un solo valor, `value` es ese valor.
- Si el callback devuelve varios valores, `value` es una tabla tipo arreglo.
- En caso de fallo, `ok = false`, `value = nil` y `err` es una tabla.

Campos de error estructurado:

- `kind` (string): Categoría estable del error, como `io_error`,
  `command_failed`, `invalid_argument`, `http_error` o `lua_error`.
- `message` (string): Mensaje de error legible para humanos.
- `op` (string, opcional): Nombre de la API u operación, como `ptool.fs.read`.
- `detail` (string, opcional): Detalle adicional del fallo.
- `path` (string, opcional): Ruta implicada en un fallo del sistema de
  archivos.
- `input` (string, opcional): Entrada original que no pudo analizarse o
  validarse.
- `cmd` (string, opcional): Nombre del comando en fallos de comandos.
- `status` (integer, opcional): Código de salida o código HTTP cuando esté
  disponible.
- `stderr` (string, opcional): stderr capturado en fallos de comandos.
- `url` (string, opcional): URL implicada en un fallo HTTP.
- `cwd` (string, opcional): Directorio de trabajo efectivo en fallos de
  comandos.
- `target` (string, opcional): Objetivo SSH en fallos de comandos relacionados
  con SSH.
- `retryable` (boolean): Si tiene sentido reintentar. El valor por defecto es
  `false`.

Comportamiento:

- Las APIs de `ptool` lanzan errores estructurados. `ptool.try` los convierte
  en la tabla `err` anterior para que quien llama pueda ramificar según
  `err.kind` y otros campos.
- Los errores Lua normales también se capturan. En ese caso, `err.kind` es
  `lua_error` y solo se garantiza `message`.
- `ptool.try` es la forma recomendada de manejar errores de APIs como
  `ptool.fs.read`, `ptool.http.request`, `ptool.run(..., { check = true })` y
  `res:assert_ok()`.

Ejemplo:

```lua
local ok, content, err = ptool.try(function()
  return ptool.fs.read("missing.txt")
end)

if not ok and err.kind == "io_error" then
  print(err.op, err.path)
end

local ok2, _, err2 = ptool.try(function()
  local res = ptool.run({
    cmd = "sh",
    args = {"-c", "echo bad >&2; exit 7"},
    stderr = "capture",
  })
  res:assert_ok()
end)

if not ok2 and err2.kind == "command_failed" then
  print(err2.cmd, err2.status, err2.stderr)
end
```

## ptool.run

> `v0.1.0` - Introduced.

`ptool.run` ejecuta comandos externos desde Rust.

Se admiten las siguientes formas de llamada:

```lua
ptool.run("echo hello world")
ptool.run("echo", "hello world")
ptool.run("echo", {"hello", "world"})
ptool.run("echo hello world", { echo = true })
ptool.run("echo", {"hello", "world"}, { echo = true })
ptool.run({ cmd = "echo", args = {"hello", "world"} })
ptool.run({ cmd = "echo", args = {"hello"}, stdout = "capture" })
```

Reglas de argumentos:

- `ptool.run(cmdline)`: `cmdline` se divide usando reglas estilo shell
  (`shlex`). El primer elemento se trata como comando y el resto como
  argumentos.
- `ptool.run(cmd, argsline)`: `cmd` se usa directamente como comando y
  `argsline` se divide en una lista de argumentos con reglas estilo shell
  (`shlex`).
- `ptool.run(cmd, args)`: `cmd` es una cadena y `args` es un arreglo de
  cadenas.
- `ptool.run(cmdline, options)`: `options` sobrescribe la configuración de esta
  invocación, como `echo`.
- `ptool.run(cmd, args, options)`: `args` puede ser una cadena o un arreglo de
  cadenas, y `options` sobrescribe la configuración de esta invocación, como
  `echo`.
- `ptool.run(options)`: `options` es una tabla.
- Cuando el segundo argumento es una tabla: si es un arreglo (claves enteras
  consecutivas `1..n`), se trata como `args`; en caso contrario se trata como
  `options`.

Reglas de valor devuelto:

- Siempre se devuelve una tabla con los siguientes campos:
  - `ok` (boolean): Si el código de salida es `0`.
  - `code` (integer|nil): El código de salida del proceso. Si el proceso fue
    terminado por una señal, esto es `nil`.
  - `cmd` (string): Nombre del comando usado para la ejecución.
  - `cwd` (string): Directorio de trabajo efectivo usado para la ejecución.
  - `stdout` (string, opcional): Presente cuando `stdout = "capture"`.
  - `stderr` (string, opcional): Presente cuando `stderr = "capture"`.
  - `assert_ok(self)` (function): Produce un error estructurado cuando
    `ok = false`. El tipo de error es `command_failed` y puede incluir `cmd`,
    `status`, `stderr` y `cwd`.
- El valor por defecto de `check` proviene de
  `ptool.config({ run = { check = ... } })`. Si no se configura, el valor por
  defecto es `false`. Cuando `check = false`, quien llama puede inspeccionar
  `ok` por su cuenta o llamar a `res:assert_ok()`.
- Cuando `check = true` y `retry = true`, `ptool.run` pregunta si debe
  reintentarse el comando fallido antes de producir el error final.
- Cuando `check = true`, `ptool.run` produce el mismo error estructurado
  `command_failed` que `res:assert_ok()`. Usa `ptool.try(...)` si quieres
  capturarlo e inspeccionarlo desde Lua.

Ejemplo:

```lua
ptool.config({ run = { echo = false } })

ptool.run("echo from ptool")
ptool.run("echo", "from ptool")
ptool.run("echo", {"from", "ptool"})
ptool.run("echo from ptool", { echo = true })
ptool.run("echo", {"from", "ptool"}, { echo = true })
ptool.run("pwd")

local res = ptool.run({
  cmd = "sh",
  args = {"-c", "echo bad >&2; exit 7"},
  stderr = "capture",
})
print(res.ok, res.code)
res:assert_ok()
```

También se admite `ptool.run(options)`, donde `options` es una tabla con los
siguientes campos:

- `cmd` (string, obligatorio): El nombre del comando o la ruta del ejecutable.
- `args` (string[], opcional): La lista de argumentos.
- `cwd` (string, opcional): El directorio de trabajo del proceso hijo.
- `env` (table, opcional): Variables de entorno adicionales, donde claves y
  valores son nombres y valores de variable.
- `echo` (boolean, opcional): Si debe imprimirse información del comando para
  esta ejecución. Si se omite, se usa el valor de
  `ptool.config({ run = { echo = ... } })`; si también falta, el valor por
  defecto es `true`.
- `check` (boolean, opcional): Si debe producirse un error inmediatamente
  cuando el código de salida no es `0`. Si se omite, se usa el valor de
  `ptool.config({ run = { check = ... } })`; si también falta, el valor por
  defecto es `false`.
- `confirm` (boolean, opcional): Si debe pedirse confirmación al usuario antes
  de ejecutar. Si se omite, se usa el valor de
  `ptool.config({ run = { confirm = ... } })`; si también falta, el valor por
  defecto es `false`.
- `retry` (boolean, opcional): Si debe preguntarse al usuario si quiere
  reintentar después de un fallo cuando `check = true`. Si se omite, se usa el
  valor de `ptool.config({ run = { retry = ... } })`; si también falta, el
  valor por defecto es `false`.
- `stdout` (string, opcional): Estrategia de manejo de stdout. Valores
  admitidos:
  - `"inherit"`: Heredar hacia el terminal actual (por defecto).
  - `"capture"`: Capturar en `res.stdout`.
  - `"null"`: Descartar la salida.
- `stderr` (string, opcional): Estrategia de manejo de stderr. Valores
  admitidos:
  - `"inherit"`: Heredar hacia el terminal actual (por defecto).
  - `"capture"`: Capturar en `res.stderr`.
  - `"null"`: Descartar la salida.
- Cuando `confirm = true`:
  - Si el usuario rechaza la ejecución, se produce un error de inmediato.
  - Si el entorno actual no es interactivo (sin TTY), se produce un error de
    inmediato.
- Cuando `retry = true` y `check = true`:
  - Si el comando falla, `ptool.run` pregunta si debe reintentarse el mismo
    comando.
  - Si el entorno actual no es interactivo (sin TTY), se produce un error de
    inmediato en lugar de pedir reintento.

Ejemplo:

```lua
ptool.run({
  cmd = "echo",
  args = {"hello"},
  env = { FOO = "bar" },
})

local res = ptool.run({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2; exit 7"},
  stdout = "capture",
  stderr = "capture",
  check = false,
})
print(res.ok, res.code)
print(res.stdout)
print(res.stderr)
res:assert_ok()
```

## ptool.run_capture

> `Unreleased` - Introduced.

`ptool.run_capture` ejecuta comandos externos desde Rust con las mismas formas
de llamada, reglas de argumentos, reglas de valor devuelto y opciones que
`ptool.run`.

La única diferencia es el manejo por defecto de los streams:

- `stdout` por defecto es `"capture"`.
- `stderr` por defecto es `"capture"`.

Puedes seguir sobrescribiendo cualquiera de los dos campos explícitamente en
`options`.

Ejemplo:

```lua
local res = ptool.run_capture("echo hello world")
print(res.stdout)

local res2 = ptool.run_capture({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2"},
})
print(res2.stdout)
print(res2.stderr)

local res3 = ptool.run_capture("echo hello", {
  stderr = "inherit",
})
print(res3.stdout)
```
