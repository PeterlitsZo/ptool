# API de sistema operativo

`ptool.os` expone utilidades para leer el entorno actual del runtime y consultar
detalles básicos del proceso anfitrión.

## ptool.os.getenv

> `v0.4.0` - Introduced.

`ptool.os.getenv(name)` devuelve el valor actual de una variable de entorno.

- `name` (string, obligatorio): Nombre de la variable de entorno.
- Devuelve: `string|nil`.

Comportamiento:

- Devuelve `nil` cuando la variable no está definida.
- Lee el entorno actual del runtime de `ptool`, incluidos los valores cambiados
  por `ptool.os.setenv(...)` y `ptool.os.unsetenv(...)`.
- Lanza un error cuando `name` está vacío o contiene caracteres inválidos como
  `=`.

## ptool.os.env

> `v0.4.0` - Introduced.

`ptool.os.env()` devuelve una tabla instantánea del entorno actual del runtime.

- Devuelve: `table`.

## ptool.os.setenv

> `v0.4.0` - Introduced.

`ptool.os.setenv(name, value)` define una variable de entorno en el runtime
actual de `ptool`.

- `name` (string, obligatorio): Nombre de la variable.
- `value` (string, obligatorio): Valor de la variable.

Comportamiento:

- Esto actualiza el entorno del runtime actual de `ptool`, no el shell padre.
- Los valores definidos aquí son visibles para `ptool.os.getenv(...)`,
  `ptool.os.env()` y los procesos hijos lanzados después mediante
  `ptool.run(...)`.

## ptool.os.unsetenv

> `v0.4.0` - Introduced.

`ptool.os.unsetenv(name)` elimina una variable de entorno del runtime actual de
`ptool`.

- `name` (string, obligatorio): Nombre de la variable.

## ptool.os.homedir

> `v0.4.0` - Introduced.

`ptool.os.homedir()` devuelve el directorio personal del usuario actual.

- Devuelve: `string|nil`.

## ptool.os.tmpdir

> `v0.4.0` - Introduced.

`ptool.os.tmpdir()` devuelve el directorio temporal del sistema.

- Devuelve: `string`.

## ptool.os.hostname

> `v0.4.0` - Introduced.

`ptool.os.hostname()` devuelve el nombre del host actual.

- Devuelve: `string|nil`.

## ptool.os.username

> `v0.4.0` - Introduced.

`ptool.os.username()` devuelve el nombre del usuario actual.

- Devuelve: `string|nil`.

## ptool.os.pid

> `v0.4.0` - Introduced.

`ptool.os.pid()` devuelve el PID del proceso actual de `ptool`.

- Devuelve: `integer`.

## ptool.os.exepath

> `v0.4.0` - Introduced.

`ptool.os.exepath()` devuelve la ruta resuelta del ejecutable `ptool` en
ejecución.

- Devuelve: `string|nil`.
