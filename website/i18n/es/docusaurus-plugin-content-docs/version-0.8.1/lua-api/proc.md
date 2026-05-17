# API de procesos

Las utilidades de procesos locales están disponibles en `ptool.proc` y `p.proc`.

Este módulo sirve para inspeccionar y gestionar procesos locales que ya se están ejecutando. Usa `ptool.run(...)` cuando quieras lanzar un comando nuevo.

## ptool.proc.self

> `v0.8.0` - Introducido.

`ptool.proc.self()` devuelve una tabla de instantánea del proceso `ptool` actual.

- Devuelve: `table`.

La tabla devuelta usa la misma estructura que `ptool.proc.get(...)` y `ptool.proc.find(...)`.

## ptool.proc.get

> `v0.8.0` - Introducido.

`ptool.proc.get(pid)` devuelve una tabla de instantánea para el ID de proceso indicado, o `nil` si el proceso no existe.

- `pid` (integer, obligatorio): ID del proceso.
- Devuelve: `table|nil`.

## ptool.proc.exists

> `v0.8.0` - Introducido.

`ptool.proc.exists(pid)` informa si un ID de proceso existe actualmente.

- `pid` (integer, obligatorio): ID del proceso.
- Devuelve: `boolean`.

## ptool.proc.find

> `v0.8.0` - Introducido.

`ptool.proc.find([options])` enumera procesos locales y devuelve un arreglo de tablas de instantánea.

- `options` (table, opcional): Opciones de filtrado y ordenación.
- Devuelve: `table`.

Campos `options` admitidos:

- `pid` (integer, opcional): Coincide con un único ID de proceso exacto.
- `pids` (integer[], opcional): Coincide con un conjunto de IDs de proceso.
- `ppid` (integer, opcional): Coincide con un ID exacto del proceso padre.
- `name` (string, opcional): Coincide con un nombre de proceso exacto.
- `name_contains` (string, opcional): Coincide con una subcadena en el nombre del proceso.
- `exe` (string, opcional): Coincide con una ruta exacta del ejecutable.
- `exe_contains` (string, opcional): Coincide con una subcadena en la ruta del ejecutable.
- `cmdline_contains` (string, opcional): Coincide con una subcadena en la línea de comandos concatenada.
- `user` (string, opcional): Coincide con un nombre de usuario exacto.
- `cwd` (string, opcional): Coincide con un directorio de trabajo actual exacto.
- `include_self` (boolean, opcional): Si se incluye el proceso actual de `ptool`. Por defecto es `false`.
- `limit` (integer, opcional): Número máximo de entradas devueltas después del filtrado y la ordenación.
- `sort_by` (string, opcional): Clave de ordenación. Valores admitidos:
  - `"pid"` (predeterminado)
  - `"start_time"`
- `reverse` (boolean, opcional): Si se invierte el orden final de clasificación. Por defecto es `false`.

Cada instantánea de proceso devuelta puede contener:

- `pid` (integer): ID del proceso.
- `ppid` (integer|nil): ID del proceso padre.
- `name` (string): Nombre del proceso.
- `exe` (string|nil): Ruta del ejecutable, cuando esté disponible.
- `cwd` (string|nil): Directorio de trabajo actual, cuando esté disponible.
- `user` (string|nil): Nombre del usuario propietario, cuando esté disponible.
- `cmdline` (string|nil): Línea de comandos concatenada, cuando esté disponible.
- `argv` (string[]): Arreglo de argumentos de la línea de comandos.
- `state` (string): Etiqueta de estado del proceso, como `"running"` o `"sleeping"`.
- `start_time_unix_ms` (integer): Hora de inicio del proceso en milisegundos Unix.

Notas:

- Algunos campos pueden ser `nil` cuando la plataforma actual o el nivel de permisos no los expone.
- Las instantáneas de proceso son valores de un momento puntual. No se actualizan por sí mismas.

Ejemplo:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
  include_self = true,
  sort_by = "start_time",
})

for _, proc in ipairs(procs) do
  print(proc.pid, proc.name, proc.cmdline)
end
```

## ptool.proc.kill

> `v0.8.0` - Introducido.

`ptool.proc.kill(targets[, options])` envía una señal a uno o más procesos locales y devuelve una tabla de resultados estructurada.

- `targets` (integer|table, obligatorio): Un pid, una tabla de instantánea de proceso o un arreglo de ellos.
- `options` (table, opcional): Opciones de señal.
- Devuelve: `table`.

Campos `options` admitidos:

- `signal` (string, opcional): Nombre de la señal. Valores admitidos:
  - `"hup"`
  - `"term"` (predeterminado)
  - `"kill"`
  - `"int"`
  - `"quit"`
  - `"stop"`
  - `"cont"`
  - `"user1"`
  - `"user2"`
- `missing_ok` (boolean, opcional): Si los procesos ausentes cuentan como éxito. Por defecto es `true`.
- `allow_self` (boolean, opcional): Si se permite señalar el proceso actual de `ptool`. Por defecto es `false`.
- `check` (boolean, opcional): Si se debe lanzar un error de inmediato cuando el resultado final no sea ok. Por defecto es `false`.
- `confirm` (boolean, opcional): Si se debe pedir confirmación antes de enviar la señal. Por defecto es `false`.

La tabla de resultados devuelta contiene:

- `ok` (boolean): Si toda la operación tuvo éxito con las opciones actuales.
- `signal` (string): La etiqueta de señal solicitada.
- `total` (integer): Número total de objetivos normalizados.
- `sent` (integer): Número de objetivos a los que se envió la señal.
- `missing` (integer): Número de objetivos que ya no existían.
- `failed` (integer): Número de objetivos que fallaron en total.
- `entries` (table): Entradas de resultado por objetivo.
- `assert_ok(self)` (function): Lanza un error estructurado de Lua cuando `ok = false`.

Cada tabla `entries[i]` contiene:

- `pid` (integer): ID del proceso objetivo.
- `ok` (boolean): Si este objetivo tuvo éxito.
- `existed` (boolean): Si el proceso objetivo existía y seguía coincidiendo.
- `signal` (string): La etiqueta de señal solicitada.
- `message` (string|nil): Detalle de estado adicional.

Ejemplo:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
})

local res = p.proc.kill(procs, {
  signal = "term",
  confirm = true,
})

res:assert_ok()
```

## ptool.proc.wait_gone

> `v0.8.0` - Introducido.

`ptool.proc.wait_gone(targets[, options])` espera hasta que uno o más procesos objetivo dejen de existir y luego devuelve una tabla de resultados estructurada.

- `targets` (integer|table, obligatorio): Un pid, una tabla de instantánea de proceso o un arreglo de ellos.
- `options` (table, opcional): Opciones de espera.
- Devuelve: `table`.

Campos `options` admitidos:

- `timeout_ms` (integer, opcional): Tiempo máximo de espera en milisegundos. Si se omite, espera indefinidamente.
- `interval_ms` (integer, opcional): Intervalo de sondeo en milisegundos. Por defecto es `100`.
- `check` (boolean, opcional): Si se debe lanzar un error de inmediato cuando la espera agote el tiempo. Por defecto es `false`.

La tabla de resultados devuelta contiene:

- `ok` (boolean): Si todos los procesos objetivo desaparecieron antes del tiempo límite.
- `timed_out` (boolean): Si se alcanzó el tiempo límite.
- `total` (integer): Número total de objetivos normalizados.
- `gone` (integer[]): IDs de proceso que ya habían desaparecido al final de la espera.
- `remaining` (integer[]): IDs de proceso que seguían presentes cuando terminó la espera.
- `elapsed_ms` (integer): Tiempo total transcurrido de espera en milisegundos.
- `assert_ok(self)` (function): Lanza un error estructurado de Lua cuando `ok = false`.

Ejemplo:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
})

local wait_res = p.proc.wait_gone(procs, {
  timeout_ms = 1000,
  interval_ms = 100,
})

wait_res:assert_ok()
```
