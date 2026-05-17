# API de Git

Los ayudantes de repositorio de Git están disponibles en `ptool.git` y `p.git`.

Este módulo se basa en `git2` / `libgit2`, no en invocar la herramienta de línea de comandos `git`.

## ptool.git.open

> `v0.6.0` - Introducido.

`ptool.git.open(path?)` abre un repositorio directamente y devuelve un objeto `Repo`.

Argumentos:

- `path` (cadena, opcional): Ruta del repositorio. Si se omite, se utiliza el directorio de tiempo de ejecución `ptool` actual.

Comportamiento:

- Las rutas relativas se resuelven desde el directorio de tiempo de ejecución `ptool` actual, por lo que siguen a `ptool.cd(...)`.
- Esto no busca directorios principales. Utilice `ptool.git.discover(...)` cuando desee un comportamiento de descubrimiento de repositorios.

Ejemplo:

```lua
local repo = ptool.git.open(".")
print(repo:path())
```

## ptool.git.discover

> `v0.6.0` - Introducido.

`ptool.git.discover(path?)` encuentra un repositorio que comienza desde `path` y avanza por los directorios principales, luego devuelve un objeto `Repo`.

Argumentos:

- `path` (cadena, opcional): Ruta de inicio. Si se omite, se utiliza el directorio de tiempo de ejecución `ptool` actual.

Comportamiento:

- Las rutas relativas se resuelven desde el directorio de tiempo de ejecución `ptool` actual.
- Esto es útil cuando un script puede ejecutarse desde un subdirectorio dentro de un árbol de trabajo.

Ejemplo:

```lua
local repo = ptool.git.discover("src")
print(repo:root())
```

## ptool.git.clone

> `v0.6.0` - Introducido.

`ptool.git.clone(url, path[, options])` clona un repositorio y devuelve un objeto `Repo` para el repositorio clonado.

Argumentos:

- `url` (string, requerido): URL del repositorio remoto.
- `path` (string, required): Ruta de destino.
- `options` (tabla, opcional): Opciones de clonación. Campos admitidos:
  - `branch` (cadena, opcional): Nombre de la rama a hacer checkout después de clonar.
  - `bare` (booleano, opcional): Si se debe crear un repositorio bare. El valor predeterminado es `false`.
  - `auth` (tabla, opcional): Configuración de autenticación remota.

Campos `auth`:

- `kind` (string, required): Modo de autenticación. Valores admitidos:
  - `"default"`: Utilice las credenciales predeterminadas de libgit2.
  - `"ssh_agent"`: Autenticar a través del agente SSH local.
  - `"userpass"`: Utiliza un nombre de usuario y una contraseña en texto plano.
- `username` (cadena, opcional): Nombre de usuario para `"ssh_agent"`.
- `username` (string, requerido): Nombre de usuario para `"userpass"`.
- `password` (string, requerido): Contraseña para `"userpass"`.

Comportamiento:

- Las rutas de destino relativas se resuelven desde el directorio de tiempo de ejecución `ptool` actual.
- Las opciones de autenticación también son utilizadas por `repo:fetch(...)` y `repo:push(...)`.

Ejemplo:

```lua
local repo = ptool.git.clone(
  "git@github.com:example/project.git",
  "tmp/project",
  {
    branch = "main",
    auth = {
      kind = "ssh_agent",
    },
  }
)

print(repo:root())
```

## Repo

> `v0.6.0` - Introducido.

`Repo` representa un identificador de repositorio Git abierto devuelto por `ptool.git.open()`, `ptool.git.discover()` o `ptool.git.clone()`.

Se implementa como datos de usuario de Lua.

Métodos:

- `repo:path()` -> `string`
- `repo:root()` -> `string|nil`
- `repo:is_bare()` -> `boolean`
- `repo:head()` -> `table`
- `repo:current_branch()` -> `string|nil`
- `repo:status([options])` -> `table`
- `repo:is_clean([options])` -> `boolean`
- `repo:add(paths[, options])` -> `nil`
- `repo:commit(message[, options])` -> `string`
- `repo:checkout(rev[, options])` -> `nil`
- `repo:switch(branch[, options])` -> `nil`
- `repo:fetch([remote[, options]])` -> `table`
- `repo:push([remote[, refspecs[, options]]])` -> `nil`

### path

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:path`.

`repo:path()` devuelve la ruta del directorio git del repositorio.

- Devuelve: `string`.

Notas:

- Para un repositorio no bare, normalmente este es el directorio `.git`.
- Para un repositorio bare, este es el propio directorio del repositorio.

### root

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:root`.

`repo:root()` devuelve el directorio raíz del árbol de trabajo.

- Devuelve: `string|nil`.

Notas:

- Esto devuelve `nil` para repositorios bare.

### is_bare

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:is_bare`.

`repo:is_bare()` informa si el repositorio es bare.

- Devuelve: `boolean`.

### head

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:head`.

`repo:head()` devuelve información de HEAD en una tabla con:

- `oid` (string|nil): El OID del commit actual, si está disponible.
- `shorthand` (string|nil): Un nombre corto para HEAD, como el nombre de una rama.
- `detached` (booleano): Si HEAD está detached.
- `unborn` (booleano): Si el repositorio todavía no tiene un commit inicial.

Ejemplo:

```lua
local head = repo:head()
print(head.oid)
print(head.detached)
```

### current_branch

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:current_branch`.

`repo:current_branch()` devuelve el nombre de la rama local actual.

- Devuelve: `string|nil`.

Notas:

- Esto devuelve `nil` cuando HEAD está detached.
- Esto también devuelve `nil` para una rama unborn antes del primer commit.

### status

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:status`.

`repo:status([options])` resume el estado del repositorio y devuelve una tabla con:

- `root` (string|nil): El directorio raíz del árbol de trabajo.
- `branch` (string|nil): El nombre de la rama local actual.
- `head` (tabla): La misma información de HEAD devuelta por `repo:head()`.
- `upstream` (string|nil): El nombre de la rama upstream, cuando está configurada.
- `ahead` (entero): Número de commits por delante del upstream.
- `behind` (entero): Número de commits por detrás del upstream.
- `clean` (booleano): si el repositorio no tiene entradas de estado visibles.
- `entries` (tabla): Una matriz de tablas de entrada de estado.

`entries[i]` contiene:

- `path` (string): Ruta relativa al repositorio.
- `index_status` (string|nil): Estado del lado del índice. Los valores admitidos actualmente incluyen `"new"`, `"modified"`, `"deleted"`, `"renamed"` y `"typechange"`.
- `worktree_status` (string|nil): estado del lado del árbol de trabajo. Los valores admitidos actualmente incluyen `"new"`, `"modified"`, `"deleted"`, `"renamed"`, `"typechange"` y `"ignored"`.
- `conflicted` (booleano): si la ruta está en conflicto.
- `ignored` (booleano): si se ignora la ruta.

Campos `options`:

- `include_untracked` (booleano, opcional): si se deben incluir archivos no rastreados. El valor predeterminado es `true`.
- `include_ignored` (booleano, opcional): si se deben incluir archivos ignorados. El valor predeterminado es `false`.
- `recurse_untracked_dirs` (booleano, opcional): Si se recurre en directorios no rastreados. El valor predeterminado es `true`.

Ejemplo:

```lua
local st = repo:status()
print(st.clean)
print(st.branch)

for _, entry in ipairs(st.entries) do
  print(entry.path, entry.index_status, entry.worktree_status)
end
```

### is_clean

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:is_clean`.

`repo:is_clean([options])` devuelve si el repositorio está limpio.

- `options` (tabla, opcional): Las mismas opciones aceptadas por `repo:status(...)`.
- Devuelve: `boolean`.

### add

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:add`.

`repo:add(paths[, options])` añade una o más rutas al índice.

Argumentos:

- `paths` (string|string[], requerido): Una ruta o una matriz de rutas.
- `options` (tabla, opcional): Añadir opciones. Campos admitidos:
  - `update` (booleano, opcional): Actualiza solo las rutas ya conocidas por el índice. El valor predeterminado es `false`.

Comportamiento:

- Las rutas se interpretan en relación con el árbol de trabajo del repositorio.

Ejemplo:

```lua
repo:add("README.md")
repo:add({"src", "Cargo.toml"})
```

### commit

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:commit`.

`repo:commit(message[, options])` crea un commit a partir del índice actual y devuelve el nuevo OID del commit.

Argumentos:

- `message` (cadena, obligatorio): Mensaje del commit.
- `options` (tabla, opcional): Opciones de confirmación. Campos admitidos:
  - `author` (tabla, opcional): Firma del autor.
  - `committer` (tabla, opcional): Firma del comitente.

Campos de firma:

- `name` (string, requerido)
- `email` (string, requerido)

Comportamiento:

- Cuando se omiten `author` y `committer`, `ptool` intenta utilizar la identidad del repositorio Git desde la configuración.
- Si no se configura ninguna identidad y no se proporciona ninguna firma explícita, se genera un error.

Ejemplo:

```lua
local oid = repo:commit("Release v0.7.0", {
  author = {
    name = "Release Bot",
    email = "bot@example.com",
  },
})

print(oid)
```

### checkout

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:checkout`.

`repo:checkout(rev[, options])` hace checkout de una revisión.

Argumentos:

- `rev` (string, required): expresión de revisión como un nombre de rama, nombre de etiqueta u Oid de confirmación.
- `options` (tabla, opcional): Opciones de checkout. Campos admitidos:
  - `force` (booleano, opcional): Si se debe forzar el checkout. El valor predeterminado es `false`.

Comportamiento:

- Esto puede dejar HEAD en detached cuando `rev` no se resuelve a una referencia con nombre.

### switch

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:switch`.

`repo:switch(branch[, options])` cambia HEAD a una rama local.

Argumentos:

- `branch` (cadena, obligatorio): Nombre de la rama local.
- `options` (tabla, opcional): Opciones de switch. Campos admitidos:
  - `create` (booleano, opcional): Si se crea primero la rama. El valor predeterminado es `false`.
  - `force` (booleano, opcional): Si forzar el pago. El valor predeterminado es `false`.
  - `start_point` (cadena, opcional): Revisión para ramificar desde cuando `create = true`. El valor predeterminado es `HEAD`.

Ejemplo:

```lua
repo:switch("release")
repo:switch("release-next", {
  create = true,
  start_point = "origin/main",
})
```

### fetch

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:fetch`.

`repo:fetch([remote[, options]])` hace fetch desde un remoto y devuelve estadísticas de transferencia.

Argumentos:

- `remote` (cadena, opcional): Nombre remoto. El valor predeterminado es `"origin"`.
- `options` (tabla, opcional): Opciones de fetch. Campos admitidos:
  - `refspecs` (string|string[], opcional): Una refspec o una matriz de refspecs.
  - `auth` (tabla, opcional): Configuración de autenticación remota. Utiliza la misma estructura que `ptool.git.clone(...)`.

Devuelve:

- `received_objects` (entero)
- `indexed_objects` (entero)
- `local_objects` (entero)
- `total_objects` (entero)
- `received_bytes` (entero)

Ejemplo:

```lua
local stats = repo:fetch("origin", {
  auth = {
    kind = "ssh_agent",
  },
})

print(stats.received_objects, stats.received_bytes)
```

### push

> `v0.6.0` - Introducido.

Nombre de la API canónica: `ptool.git.Repo:push`.

`repo:push([remote[, refspecs[, options]]])` hace push de refs a un remoto.

Argumentos:

- `remote` (cadena, opcional): Nombre remoto. El valor predeterminado es `"origin"`.
- `refspecs` (string|string[], opcional): Una refspec o una matriz de refspecs.
- `options` (tabla, opcional): Opciones de push. Campos admitidos:
  - `auth` (tabla, opcional): Configuración de autenticación remota. Utiliza la misma estructura que `ptool.git.clone(...)`.

Comportamiento:

- Cuando se omite `refspecs`, `ptool` intenta hacer push de la rama local actual a la rama del mismo nombre en el remoto.
- Omitir `refspecs` cuando HEAD está detached genera un error.

Ejemplo:

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```
