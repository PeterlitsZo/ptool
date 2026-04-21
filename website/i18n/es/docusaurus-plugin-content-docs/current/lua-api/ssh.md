# API de SSH

Los helpers para conexión SSH, ejecución remota y transferencia de archivos
están disponibles bajo `ptool.ssh` y `p.ssh`.

## ptool.ssh.connect

> `v0.1.0` - Introduced.

`ptool.ssh.connect(target_or_options)` prepara un manejador de conexión SSH
respaldado por el comando `ssh` del sistema y devuelve un objeto `Connection`.

`ssh` debe estar disponible en `PATH`.

Argumentos:

- `target_or_options` (string|table, obligatorio):
  - Cuando se proporciona una cadena, se trata como un destino SSH.
  - Cuando se proporciona una tabla, actualmente admite:
    - `target` (string, opcional): Cadena de destino SSH, como
      `"deploy@example.com"` o `"deploy@example.com:2222"`.
    - `host` (string, opcional): Nombre de host o dirección IP.
    - `user` (string, opcional): Nombre de usuario SSH. Por defecto usa
      `$USER`, o `"root"` si `$USER` no está disponible.
    - `port` (integer, opcional): Puerto SSH. Por defecto es `22`.
    - `auth` (table, opcional): Configuración de autenticación.
    - `host_key` (table, opcional): Configuración de verificación de clave de
      host.
    - `connect_timeout_ms` (integer, opcional): Timeout en milisegundos.
      Por defecto es `10000`.
    - `keepalive_interval_ms` (integer, opcional): Intervalo de keepalive en
      milisegundos.

Ejemplos de cadenas de destino admitidas:

```lua
local a = ptool.ssh.connect("deploy@example.com")
local b = ptool.ssh.connect("deploy@example.com:2222")
local c = ptool.ssh.connect("[2001:db8::10]:2222")
```

Campos de `auth`:

- `private_key_file` (string, opcional): Ruta a un archivo de clave privada.
- `private_key_passphrase` (string, opcional): Frase de contraseña de la clave
  privada. Actualmente no es compatible.
- `password` (string, opcional): Autenticación basada en contraseña.
  Actualmente no es compatible.

Comportamiento de autenticación:

- Si se proporciona `auth.private_key_file`, `ptool` invoca `ssh` con esa
  clave mediante `-i` y también establece `IdentitiesOnly=yes`.
- Si se proporciona `auth.private_key_passphrase` o `auth.password`,
  `ptool.ssh.connect(...)` falla porque esta API no pasa esos secretos al
  comando `ssh` del sistema.
- En caso contrario, la autenticación se delega a la configuración local de
  OpenSSH, incluidas opciones y mecanismos como `IdentityFile`, `ProxyJump`,
  `ProxyCommand`, `ssh-agent` y certificados.
- Las rutas de clave relativas se resuelven desde el directorio de runtime
  actual de `ptool`, por lo que siguen `ptool.cd(...)`.
- `~` y `~/...` se expanden en las rutas de clave.

Campos de `host_key`:

- `verify` (string, opcional): Modo de verificación de la clave de host.
  Valores admitidos:
  - `"known_hosts"`: Verifica contra un archivo `known_hosts` (predeterminado).
  - `"ignore"`: Omite la verificación de la clave de host.
- `known_hosts_file` (string, opcional): Ruta a un archivo `known_hosts`.
  Solo se usa cuando `verify = "known_hosts"`.

Comportamiento de clave de host:

- Si `verify = "ignore"`, `ptool` invoca `ssh` con
  `StrictHostKeyChecking=no` y `UserKnownHostsFile=/dev/null`.
- Si `verify = "known_hosts"` y se proporciona `known_hosts_file`, `ptool`
  invoca `ssh` con `StrictHostKeyChecking=yes` y ese `UserKnownHostsFile`.
- Si `verify = "known_hosts"` y se omite `known_hosts_file`, o si `host_key`
  se omite por completo, el manejo de la clave de host se delega a la
  configuración local de OpenSSH y a sus valores predeterminados.
- Las rutas relativas de `known_hosts_file` se resuelven desde el directorio
  de runtime actual de `ptool`.
- `~` y `~/...` se expanden en `known_hosts_file`.
- Cuando `known_hosts_file` se proporciona explícitamente, reemplaza el valor
  predeterminado de `UserKnownHostsFile` que usa el comando `ssh` local para
  esta conexión.

Ejemplo:

```lua
local ssh = ptool.ssh.connect({
  host = "example.com",
  user = "deploy",
  port = 22,
  auth = {
    private_key_file = "~/.ssh/id_ed25519",
  },
  host_key = {
    verify = "known_hosts",
  },
})
```

## Connection

> `v0.1.0` - Introduced.

`Connection` representa un manejador de conexión respaldado por OpenSSH
devuelto por `ptool.ssh.connect()`.

Está implementado como un userdata de Lua.

Campos y métodos:

- Campos:
  - `conn.host` (string)
  - `conn.user` (string)
  - `conn.port` (integer)
  - `conn.target` (string)
- Métodos:
  - `conn:run(...)` -> `table`
  - `conn:run_capture(...)` -> `table`
  - `conn:path(path)` -> `RemotePath`
  - `conn:exists(path)` -> `boolean`
  - `conn:is_file(path)` -> `boolean`
  - `conn:is_dir(path)` -> `boolean`
  - `conn:upload(local_path, remote_path[, options])` -> `table`
  - `conn:download(remote_path, local_path[, options])` -> `table`
  - `conn:close()` -> `nil`

### run

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:run`.

`conn:run(...)` ejecuta un comando remoto a través de la conexión SSH actual.

Se admiten las siguientes formas de llamada:

```lua
conn:run("hostname")
conn:run("echo", "hello world")
conn:run("echo", {"hello", "world"})
conn:run("hostname", { stdout = "capture" })
conn:run("echo", {"hello", "world"}, { stdout = "capture" })
conn:run({ cmd = "git", args = {"rev-parse", "HEAD"} })
```

Reglas de argumentos:

- `conn:run(cmdline)`: `cmdline` se envía como la cadena de comando remoto.
- `conn:run(cmd, argsline)`: `cmd` se trata como el comando, y `argsline` se
  divide usando reglas de estilo shell (`shlex`).
- `conn:run(cmd, args)`: `cmd` es una cadena y `args` es un arreglo de
  cadenas. Los argumentos se escapan para shell antes de la ejecución remota.
- `conn:run(cmdline, options)`: `options` sobrescribe esta invocación.
- `conn:run(cmd, args, options)`: `options` sobrescribe esta invocación.
- `conn:run(options)`: `options` es una tabla.
- Cuando el segundo argumento es una tabla: si es un arreglo (claves enteras
  consecutivas `1..n`), se trata como `args`; en caso contrario se trata como
  `options`.

Cuando se usa `conn:run(options)`, `options` actualmente admite:

- `cmd` (string, obligatorio): Nombre del comando o ruta del ejecutable.
- `args` (string[], opcional): Lista de argumentos.
- `cwd` (string, opcional): Directorio de trabajo remoto. Se aplica anteponiendo
  `cd ... &&` al comando shell remoto generado.
- `env` (table, opcional): Variables de entorno remotas, donde claves y valores
  son cadenas. Se aplica anteponiendo `export ... &&` al comando shell remoto
  generado.
- `stdin` (string, opcional): Cadena enviada al stdin del proceso remoto.
- `echo` (boolean, opcional): Si debe imprimir el comando remoto antes de
  ejecutarlo. Por defecto es `true`.
- `check` (boolean, opcional): Si debe lanzar un error inmediatamente cuando el
  código de salida no sea `0`. Por defecto es `false`.
- `stdout` (string, opcional): Estrategia de manejo de stdout. Valores
  admitidos:
  - `"inherit"`: Hereda el terminal actual (predeterminado).
  - `"capture"`: Captura en `res.stdout`.
  - `"null"`: Descarta la salida.
- `stderr` (string, opcional): Estrategia de manejo de stderr. Valores
  admitidos:
  - `"inherit"`: Hereda el terminal actual (predeterminado).
  - `"capture"`: Captura en `res.stderr`.
  - `"null"`: Descarta la salida.

Cuando se usan las formas abreviadas, la tabla `options` solo admite:

- `stdin` (string, opcional): Cadena enviada al stdin del proceso remoto.
- `echo` (boolean, opcional): Si debe imprimir el comando remoto antes de
  ejecutarlo. Por defecto es `true`.
- `check` (boolean, opcional): Si debe lanzar un error inmediatamente cuando el
  código de salida no sea `0`. Por defecto es `false`.
- `stdout` (string, opcional): Estrategia de manejo de stdout. Valores
  admitidos:
  - `"inherit"`: Hereda el terminal actual (predeterminado).
  - `"capture"`: Captura en `res.stdout`.
  - `"null"`: Descarta la salida.
- `stderr` (string, opcional): Estrategia de manejo de stderr. Valores
  admitidos:
  - `"inherit"`: Hereda el terminal actual (predeterminado).
  - `"capture"`: Captura en `res.stderr`.
  - `"null"`: Descarta la salida.

Reglas del valor de retorno:

- Siempre se devuelve una tabla con los siguientes campos:
  - `ok` (boolean): Si el código de salida remoto es `0`.
  - `code` (integer|nil): Código de salida remoto. Si el proceso remoto termina
    por señal, este valor es `nil`.
  - `target` (string): Cadena de destino SSH con el formato `user@host:port`.
  - `stdout` (string, opcional): Presente cuando `stdout = "capture"`.
  - `stderr` (string, opcional): Presente cuando `stderr = "capture"`.
  - `assert_ok(self)` (function): Lanza un error cuando `ok = false`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:run("uname -a", { stdout = "capture" })
print(res.target)
print(res.stdout)
local res2 = ssh:run({
  cmd = "git",
  args = {"rev-parse", "HEAD"},
  cwd = "/srv/app",
  env = {
    FOO = "bar",
  },
  stdout = "capture",
  check = true,
})

print(res2.stdout)
```

### run_capture

> `Unreleased` - Introduced.

Canonical API name: `ptool.ssh.Connection:run_capture`.

`conn:run_capture(...)` ejecuta un comando remoto a través de la conexión SSH
actual.

Acepta las mismas formas de llamada, reglas de argumentos, reglas del valor de
retorno y opciones que `conn:run(...)`.

La única diferencia es el manejo predeterminado de flujos:

- `stdout` usa `"capture"` por defecto.
- `stderr` usa `"capture"` por defecto.

Todavía puedes sobrescribir cualquiera de esos campos explícitamente en
`options`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:run_capture("uname -a")
print(res.stdout)

local res2 = ssh:run_capture({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2"},
  cwd = "/srv/app",
})
print(res2.stdout)
print(res2.stderr)

local res3 = ssh:run_capture("echo hello", {
  stderr = "inherit",
})
print(res3.stdout)
```

### path

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:path`.

`conn:path(path)` crea un objeto `RemotePath` reutilizable vinculado a la
conexión SSH actual.

- `path` (string, obligatorio): La ruta remota.
- Devuelve: Un objeto `RemotePath` que puede pasarse a
  `conn:upload(...)`, `conn:download(...)` y `ptool.fs.copy(...)`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

ssh:download(remote_release, "./tmp/current.tar.gz")
```

### exists

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:exists`.

`conn:exists(path)` comprueba si existe una ruta remota.

- `path` (string|remote path, obligatorio): La ruta remota que se debe
  comprobar. Puede ser una cadena o un valor creado por `conn:path(...)`.
- Devuelve: `true` cuando la ruta remota existe; de lo contrario, `false`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

print(ssh:exists("/srv/app"))
print(ssh:path("/srv/app/releases/current.tar.gz"):exists())
```

### is_file

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:is_file`.

`conn:is_file(path)` comprueba si una ruta remota existe y es un archivo
regular.

- `path` (string|remote path, obligatorio): La ruta remota que se debe
  comprobar. Puede ser una cadena o un valor creado por `conn:path(...)`.
- Devuelve: `true` cuando la ruta remota es un archivo; de lo contrario,
  `false`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if ssh:is_file(remote_tarball) then
  print("release tarball exists")
end
```

### is_dir

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:is_dir`.

`conn:is_dir(path)` comprueba si una ruta remota existe y es un directorio.

- `path` (string|remote path, obligatorio): La ruta remota que se debe
  comprobar. Puede ser una cadena o un valor creado por `conn:path(...)`.
- Devuelve: `true` cuando la ruta remota es un directorio; de lo contrario,
  `false`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```

### upload

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:upload`.

`conn:upload(local_path, remote_path[, options])` sube un archivo o directorio
local al host remoto.

- `local_path` (string, obligatorio): El archivo o directorio local que se va a
  subir.
- `remote_path` (string|remote path, obligatorio): La ruta de destino en el
  host remoto. Puede ser una cadena o un valor creado por `conn:path(...)`.
- `options` (table, opcional): Opciones de transferencia.
- Devuelve: Una tabla con los siguientes campos:
  - `bytes` (integer): El número de bytes de archivos regulares subidos. Cuando
    se sube un directorio, es la suma de los tamaños de los archivos subidos.
  - `from` (string): La ruta local de origen.
  - `to` (string): La ruta remota de destino.

Opciones de transferencia admitidas:

- `parents` (boolean, opcional): Crea el directorio padre de `remote_path`
  antes de subir. Por defecto es `false`.
- `overwrite` (boolean, opcional): Si se puede reemplazar un archivo de destino
  existente. Por defecto es `true`.
- `echo` (boolean, opcional): Si debe imprimir la transferencia antes de
  ejecutarla. Por defecto es `false`.

Comportamiento con directorios:

- Cuando `local_path` es un archivo, el comportamiento no cambia.
- Cuando `local_path` es un directorio y `remote_path` no existe,
  `remote_path` se convierte en la raíz del directorio de destino.
- Cuando `local_path` es un directorio y `remote_path` ya existe como
  directorio, el directorio de origen se crea dentro de él usando el basename
  del directorio de origen.
- `overwrite = false` rechaza un directorio de destino ya existente para la
  raíz final del directorio.
- Las subidas de directorios requieren que `tar` esté disponible en el host
  remoto.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

local res = ssh:upload("./dist/app.tar.gz", remote_tarball, {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.to)
```

Ejemplo con directorio:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:upload("./dist/assets", "/srv/app/releases", {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.to) -- deploy@example.com:22:/srv/app/releases
```

### download

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:download`.

`conn:download(remote_path, local_path[, options])` descarga un archivo o
directorio remoto a una ruta local.

- `remote_path` (string|remote path, obligatorio): La ruta de origen en el host
  remoto. Puede ser una cadena o un valor creado por `conn:path(...)`.
- `local_path` (string, obligatorio): La ruta de destino local.
- `options` (table, opcional): Opciones de transferencia.
- Devuelve: Una tabla con los siguientes campos:
  - `bytes` (integer): El número de bytes de archivos regulares descargados.
    Cuando se descarga un directorio, es la suma de los tamaños de los archivos
    descargados.
  - `from` (string): La ruta remota de origen.
  - `to` (string): La ruta local de destino.

Opciones de transferencia admitidas:

- `parents` (boolean, opcional): Crea el directorio padre de `local_path`
  antes de descargar. Por defecto es `false`.
- `overwrite` (boolean, opcional): Si se puede reemplazar un archivo de destino
  existente. Por defecto es `true`.
- `echo` (boolean, opcional): Si debe imprimir la transferencia antes de
  ejecutarla. Por defecto es `false`.

Comportamiento con directorios:

- Cuando `remote_path` es un archivo, el comportamiento no cambia.
- Cuando `remote_path` es un directorio y `local_path` no existe,
  `local_path` se convierte en la raíz del directorio de destino.
- Cuando `remote_path` es un directorio y `local_path` ya existe como
  directorio, el directorio remoto de origen se crea dentro de él usando el
  basename del directorio remoto.
- `overwrite = false` rechaza un directorio de destino ya existente para la
  raíz final del directorio.
- Las descargas de directorios requieren que `tar` esté disponible en el host
  remoto.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:download("/srv/app/logs/app.log", "./tmp/app.log", {
  parents = true,
  overwrite = false,
  echo = true,
})

print(res.bytes)
print(res.from)
```

Ejemplo con directorio:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:download("/srv/app/releases/assets", "./tmp/releases", {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.from)
```

### close

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:close`.

`conn:close()` cierra el manejador de conexión SSH.

Comportamiento:

- Después de cerrarla, la conexión ya no puede usarse.
- Cerrar una conexión que ya está cerrada está permitido y no tiene efecto.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
ssh:close()
```

## RemotePath

> `v0.1.0` - Introduced.

`RemotePath` representa una ruta remota vinculada a un `Connection` y devuelta
por `conn:path(path)`.

Está implementado como un userdata de Lua.

Métodos:

- `remote:exists()` -> `boolean`
- `remote:is_file()` -> `boolean`
- `remote:is_dir()` -> `boolean`

### exists

`remote:exists()` comprueba si existe la ruta remota.

- Devuelve: `true` cuando la ruta remota existe; de lo contrario, `false`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

print(remote_release:exists())
```

### is_file

`remote:is_file()` comprueba si la ruta remota existe y es un archivo regular.

- Devuelve: `true` cuando la ruta remota es un archivo; de lo contrario,
  `false`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if remote_tarball:is_file() then
  print("release tarball exists")
end
```

### is_dir

`remote:is_dir()` comprueba si la ruta remota existe y es un directorio.

- Devuelve: `true` cuando la ruta remota es un directorio; de lo contrario,
  `false`.

Ejemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```
