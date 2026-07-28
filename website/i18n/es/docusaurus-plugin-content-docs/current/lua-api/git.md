# API de Git

Los ayudantes de repositorio de Git están disponibles en `ptool.git` y `p.git`.

Este módulo se basa en `git2` / `libgit2`, no en invocar la herramienta de línea de comandos `git`.

Las operaciones que modifican el repositorio aceptan `confirm = true` en su tabla de opciones para pedir confirmación antes de ejecutarse. Las operaciones destructivas también usan indicadores de seguridad independientes, como `force = true`; la confirmación nunca sustituye el indicador de seguridad requerido.

Los nombres de opciones desconocidos se rechazan en lugar de ignorarse silenciosamente.

## ptool.git.init

> `Unreleased` - Introducido.

`ptool.git.init(path?, options?)` inicializa un repositorio y devuelve un objeto `Repo`.

Opciones:

- `bare` (boolean): Crea un repositorio bare. El valor predeterminado es `false`.
- `initial_head` (string): Nombre de la rama inicial, por ejemplo `"main"`.
- `confirm` (boolean): Solicita confirmación antes de crear el repositorio.

Las rutas relativas se resuelven desde el directorio de tiempo de ejecución `ptool` actual.

```lua
local repo = p.git.init("tmp/project", {
  initial_head = "main",
})
```

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
  - `depth` (integer, opcional): Profundidad positiva para una clonación superficial.
  - `checkout` (boolean, opcional): Indica si se debe extraer la rama seleccionada. El valor predeterminado es `true`.
  - `remote` (string, opcional): Nombre asignado al remoto clonado. El valor predeterminado es `"origin"`.
  - `tags` (string, opcional): `"auto"`, `"all"` o `"none"`. El valor predeterminado es `"auto"`.
  - `confirm` (booleano, opcional): Si se debe pedir confirmación antes de clonar. El valor predeterminado es `false`.
  - `auth` (tabla, opcional): Configuración de autenticación remota.

Campos `auth`:

- `kind` (string, required): Modo de autenticación. Valores admitidos:
  - `"default"`: Utilice las credenciales predeterminadas de libgit2.
  - `"ssh_agent"`: Autenticar a través del agente SSH local.
  - `"ssh_key"`: Autentica mediante una clave SSH privada.
  - `"userpass"`: Utiliza un nombre de usuario y una contraseña en texto plano.
  - `"credential_helper"`: Consulta el gestor de credenciales de Git configurado.
- `username` (string): Opcional para `"ssh_agent"` y `"credential_helper"`; obligatorio para `"ssh_key"` y `"userpass"`.
- `private_key` (string, obligatorio para `"ssh_key"`): Ruta de la clave privada. Las rutas relativas se resuelven desde el directorio de ejecución actual de `ptool`.
- `public_key` (string, opcional): Ruta de la clave pública.
- `passphrase` (string, opcional): Frase de contraseña de la clave SSH privada.
- `password` (string, obligatorio para `"userpass"`): Contraseña.

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

`Repo` representa un identificador de repositorio Git abierto devuelto por `ptool.git.init()`, `ptool.git.open()`, `ptool.git.discover()` o `ptool.git.clone()`.

Se implementa como datos de usuario de Lua.

Los métodos se agrupan a continuación. Los métodos existentes de repositorio, estado, preparación, commit, checkout, switch, fetch y push mantienen la compatibilidad. Las API de flujo de trabajo descritas en las secciones siguientes aún no se han publicado.

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
  - `confirm` (booleano, opcional): Si se debe pedir confirmación antes de añadir rutas al área de preparación. El valor predeterminado es `false`.

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
  - `amend` (boolean, opcional): Sustituye el commit HEAD actual. El valor predeterminado es `false`.
  - `allow_empty` (boolean, opcional): Permite un commit cuyo árbol no haya cambiado. El valor predeterminado es `true` por compatibilidad.
  - `confirm` (booleano, opcional): Si se debe pedir confirmación antes de crear el commit. El valor predeterminado es `false`.

Campos de firma:

- `name` (string, requerido)
- `email` (string, requerido)
- `time_seconds` (integer, opcional): Marca de tiempo Unix.
- `offset_minutes` (integer, opcional): Desplazamiento de zona horaria respecto a UTC.

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
  - `confirm` (booleano, opcional): Si se debe pedir confirmación antes de hacer checkout de la revisión. El valor predeterminado es `false`.

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
  - `track` (string, opcional): Referencia upstream de la nueva rama.
  - `orphan` (boolean, opcional): Crea una rama huérfana.
  - `confirm` (booleano, opcional): Si se debe pedir confirmación antes de cambiar de rama. El valor predeterminado es `false`.

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
  - `depth` (integer, opcional): Profundidad positiva para un fetch superficial.
  - `prune` (boolean, opcional): Elimina referencias remotas obsoletas.
  - `tags` (string, opcional): `"auto"`, `"all"` o `"none"`.
  - `update_fetchhead` (boolean, opcional): Actualiza `FETCH_HEAD`. El valor predeterminado es `true`.
  - `confirm` (booleano, opcional): Si se debe pedir confirmación antes de hacer fetch. El valor predeterminado es `false`.
  - `auth` (tabla, opcional): Configuración de autenticación remota. Utiliza la misma estructura que `ptool.git.clone(...)`.

Devuelve:

- `received_objects` (entero)
- `indexed_objects` (entero)
- `local_objects` (entero)
- `total_objects` (entero)
- `received_bytes` (entero)
- `updated_refs` (string[])

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
  - `force` (boolean, opcional): Fuerza cada refspec de push. El valor predeterminado es `false`.
  - `set_upstream` (boolean, opcional): Configura la rama actual enviada para seguir la rama remota de destino.
  - `confirm` (booleano, opcional): Si se debe pedir confirmación antes de hacer push. El valor predeterminado es `false`.
  - `auth` (tabla, opcional): Configuración de autenticación remota. Utiliza la misma estructura que `ptool.git.clone(...)`.

Comportamiento:

- Cuando se omite `refspecs`, `ptool` intenta hacer push de la rama local actual a la rama del mismo nombre en el remoto.
- Omitir `refspecs` cuando HEAD está detached genera un error.
- La tabla devuelta contiene `ok`, `refspecs` y `rejected`. Cada entrada rechazada contiene `reference` y `message`.

Ejemplo:

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```


## API de flujo de trabajo de Git

> `Unreleased` - Introducido.

Las API de esta sección cubren mantenimiento de repositorios, publicaciones, inspección del historial, colaboración y automatización de CI. Se acepta una cadena o un array denso de cadenas cuando un parámetro se documenta como `string|string[]`.

### Tablas de resultados compartidas

La información de commit devuelta por `commit_info()` y `log()` contiene `oid`, `message`, `summary`, `author`, `committer` y `parent_oids`. Una tabla de firma contiene `name`, `email`, `time_seconds` y `offset_minutes`.

Los métodos de integración devuelven:

```lua
{
  outcome = "up_to_date" | "fast_forward" | "merged" | "conflicted",
  oid = "..." | nil,
  conflicts = {
    { path = "file", ancestor = true, ours = true, theirs = true },
  },
}
```

Los resultados de rebase usan los estados `"rebased"` o `"conflicted"` y añaden `current` y `total`. Las operaciones detenidas por conflictos pueden continuarse o abortarse con la API correspondiente.

### Repositorio y estado

```lua
repo:path() -> string
repo:root() -> string|nil
repo:is_bare() -> boolean
repo:head() -> GitHeadInfo
repo:current_branch() -> string|nil
repo:status(options?) -> GitStatusSummary
repo:is_clean(options?) -> boolean
```

Las opciones de `status()` e `is_clean()` son `include_untracked` (predeterminado `true`), `include_ignored`, `recurse_untracked_dirs` (predeterminado `true`) y `paths`. `GitStatusSummary` contiene `root`, `branch`, `head`, `upstream`, `ahead`, `behind`, `clean` y `entries`.

### Historial y diff

```lua
repo:resolve(rev) -> GitObjectInfo
repo:commit_info(rev?) -> GitCommitInfo
repo:log(options?) -> GitCommitInfo[]
repo:diff(options?) -> GitDiff
repo:describe(options?) -> string|nil
```

- `resolve()` devuelve `oid`, `kind` y `shorthand`.
- `commit_info()` usa `HEAD` de forma predeterminada.
- `log()` acepta `rev`, `max_count` (predeterminado `100`), `skip`, `first_parent`, `reverse` y `paths`.
- `diff()` acepta `from`, `to`, `cached`, `paths`, `context_lines` (predeterminado `3`) y `find_renames` (predeterminado `true`). Sin `from` o `to`, compara el estado apropiado del worktree, índice o HEAD.
- Un resultado de diff contiene `patch`, `files_changed`, `insertions`, `deletions` y `deltas`. Cada delta contiene `status`, `old_path`, `new_path` y `binary`.
- `describe()` acepta `rev`, `pattern`, `always`, `abbrev` (predeterminado `7`) y `dirty_suffix`.

```lua
local commits = repo:log({
  rev = "HEAD",
  max_count = 20,
  first_parent = true,
  paths = {"crates/ptool-engine"},
})
local changes = repo:diff({ from = "v0.10.0", to = "HEAD" })
```

### Ramas

```lua
repo:branches(options?) -> GitBranchInfo[]
repo:branch_create(name, options?) -> GitBranchInfo
repo:branch_delete(name, options?) -> nil
repo:branch_rename(old_name, new_name, options?) -> GitBranchInfo
repo:branch_set_upstream(name, upstream_or_nil, options?) -> nil
```

- `branches()` acepta `kind = "local" | "remote" | "all"`; el valor predeterminado es `"local"`.
- La información de rama contiene `name`, `kind`, `oid`, `head`, `upstream`, `ahead` y `behind`.
- `branch_create()` acepta `start_point`, `force`, `checkout`, `upstream` y `confirm`. El punto de inicio predeterminado es `HEAD`.
- `branch_delete()` acepta `force` y `confirm`. No se puede eliminar la rama actual, y eliminar una rama sin fusionar requiere `force = true`.
- `branch_rename()` acepta `force` y `confirm`.
- Pasa `nil` a `branch_set_upstream()` para eliminar el upstream. Sus opciones solo contienen `confirm`.

### Etiquetas

```lua
repo:tags(pattern?) -> GitTagInfo[]
repo:tag_create(name, target?, options?) -> GitTagInfo
repo:tag_delete(name, options?) -> nil
```

`tag_create()` apunta a `HEAD` de forma predeterminada. Sin `message` crea una etiqueta ligera; con `message` crea una etiqueta anotada. Las opciones son `message`, `tagger`, `force` y `confirm`. El tagger usa los campos de firma compartidos.

La información de etiqueta contiene `name`, `oid`, `target_oid`, `target_kind`, `annotated`, `message` y `tagger`. `tags(pattern)` usa patrones glob de Git. Eliminar una etiqueta solo cambia el repositorio local; elimina una etiqueta remota mediante un refspec de push explícito.

```lua
local tag = repo:tag_create("v1.0.0", "HEAD", {
  message = "Release v1.0.0",
})
repo:push("origin", "refs/tags/v1.0.0:refs/tags/v1.0.0")
```

### Remotos y transferencia

```lua
repo:remotes() -> GitRemoteInfo[]
repo:remote(name) -> GitRemoteInfo
repo:remote_add(name, url, options?) -> GitRemoteInfo
repo:remote_remove(name, options?) -> nil
repo:remote_rename(name, new_name, options?) -> GitRemoteInfo
repo:remote_set_url(name, url, options?) -> GitRemoteInfo
repo:fetch(remote?, options?) -> GitFetchStats
repo:push(remote?, refspecs?, options?) -> GitPushResult
repo:pull(remote?, branch?, options?) -> GitIntegrateResult
```

La información de remoto contiene `name`, `url`, `push_url`, `fetch_refspecs` y `push_refspecs`. `remote_add()` acepta `push_url` y `confirm`. `remote_set_url()` acepta `push = true` para cambiar la URL de push en vez de la URL de fetch. Las operaciones de eliminación, cambio de nombre y URL aceptan `confirm`.

`fetch()` acepta `refspecs`, `auth`, `depth`, `prune`, `tags`, `update_fetchhead` y `confirm`. `push()` acepta `auth`, `force`, `set_upstream` y `confirm`.

`pull()` usa de forma predeterminada el remoto `"origin"`, la rama actual y `strategy = "ff_only"`. También acepta `strategy = "merge" | "rebase"`, las opciones de fetch `auth`, `depth`, `prune`, `tags` y `update_fetchhead`, además de `signature`, `message` y `confirm`. Pull requiere un repositorio limpio antes de comenzar.

### Worktree, índice y recuperación

```lua
repo:add(paths, options?) -> nil
repo:restore(paths, options?) -> nil
repo:reset(rev?, options?) -> nil
repo:remove(paths, options?) -> nil
repo:clean(options?) -> string[]
```

- `restore()` acepta `source` (predeterminado `"HEAD"`), `staged`, `worktree` y `confirm`. Si solo se especifica `staged = true`, el worktree no cambia.
- `reset()` acepta `mode = "soft" | "mixed" | "hard"`, `force` y `confirm`. Un reset hard requiere `force = true`.
- `remove()` acepta `cached`, `force` y `confirm`.
- `clean()` acepta `dry_run`, `force`, `dirs`, `ignored`, `paths` y `confirm`. El valor predeterminado es `dry_run = true`. La eliminación real requiere `dry_run = false` y `force = true`. Los directorios no se modifican salvo que `dirs = true`.

```lua
local candidates = repo:clean()
repo:clean({ dry_run = false, force = true, dirs = true, confirm = true })
```

### Configuración

```lua
repo:config_get(name, options?) -> string|boolean|integer|nil
repo:config_list(options?) -> GitConfigEntry[]
repo:config_set(name, value, options?) -> nil
repo:config_remove(name, options?) -> nil
```

`scope` es `"local"`, `"global"` o `"system"`; para lecturas se usa de forma predeterminada el valor disponible de mayor prioridad y para escrituras `"local"`. Las entradas contienen `name`, `value` y `scope`. La configuración del sistema es de solo lectura. `config_set()` y `config_remove()` globales requieren `confirm = true` y siempre muestran una confirmación.

### Merge, cherry-pick y revert

```lua
repo:state() -> string
repo:conflicts() -> GitConflictEntry[]
repo:merge_analysis(rev) -> string
repo:merge(rev, options?) -> GitIntegrateResult
repo:merge_abort(options?) -> nil
repo:cherry_pick(rev, options?) -> GitIntegrateResult
repo:cherry_pick_abort(options?) -> nil
repo:revert(rev, options?) -> GitIntegrateResult
repo:revert_abort(options?) -> nil
```

`merge()` acepta `ff = "allow" | "only" | "never"`, `signature`, `message` y `confirm`. Merge, cherry-pick y revert requieren un repositorio limpio antes de comenzar para que abort pueda restaurar `ORIG_HEAD` de forma segura.

Cherry-pick y revert aceptan `commit` (predeterminado `true`), `signature`, `message`, `mainline` y `confirm`. Usa `commit = false` para actualizar el índice y el worktree sin crear un commit. Los métodos abort aceptan `confirm`.

### Stash y rebase

```lua
repo:stash_save(message?, options?) -> string
repo:stashes() -> GitStashInfo[]
repo:stash_apply(index?, options?) -> GitIntegrateResult
repo:stash_pop(index?, options?) -> GitIntegrateResult
repo:stash_drop(index?, options?) -> nil
repo:rebase(options) -> GitRebaseResult
repo:rebase_continue(options?) -> GitRebaseResult
repo:rebase_abort(options?) -> nil
```

Los índices de stash usan `0` de forma predeterminada. `stash_save()` acepta `include_untracked`, `include_ignored`, `keep_index`, `signature` y `confirm`. Apply y pop aceptan `reinstate_index` y `confirm`; drop acepta `confirm`. La información de stash contiene `index`, `message` y `oid`.

`rebase()` requiere `upstream` y acepta `onto`, `branch` (predeterminado `"HEAD"`), `signature` y `confirm`. La primera versión admite operaciones pick no interactivas; squash, fixup, reword y edit interactivas no están disponibles. Continue acepta `signature` y `confirm`; abort acepta `confirm`.

### Repositorios avanzados

```lua
repo:worktrees() -> GitWorktreeInfo[]
repo:worktree_add(name, path, options?) -> GitWorktreeInfo
repo:worktree_lock(name, reason?, options?) -> nil
repo:worktree_unlock(name, options?) -> nil
repo:worktree_prune(name, options?) -> nil
repo:submodules() -> GitSubmoduleInfo[]
repo:submodule_init(name?, options?) -> nil
repo:submodule_update(name?, options?) -> nil
repo:submodule_sync(name?, options?) -> nil
repo:blame(path, options?) -> GitBlameHunk[]
```

- La información de worktree contiene `name`, `path`, `locked`, `lock_reason` y `valid`. Las opciones de add son `reference`, `lock`, `checkout_existing` y `confirm`. Las opciones de prune son `valid`, `locked`, `working_tree`, `force` y `confirm`; lock y unlock también aceptan `confirm`.
- La información de submódulo contiene `name`, `path`, `url`, `branch`, `head_oid`, `index_oid` y `workdir_oid`. Las opciones de init son `overwrite`, `recursive` y `confirm`. Las opciones de update son `init`, `recursive`, `allow_fetch`, `auth` y `confirm`. Las opciones de sync son `recursive` y `confirm`.
- El procesamiento recursivo de submódulos está desactivado salvo que `recursive = true`.
- `blame()` acepta `newest`, `oldest`, `min_line`, `max_line`, `first_parent`, indicadores de seguimiento de copias/movimientos, `ignore_whitespace` y `use_mailmap`. Cada bloque contiene `final_start_line`, `original_start_line`, `line_count`, `commit_oid`, `author`, `origin_path` y `boundary`.

## Seguridad y compatibilidad

Todos los métodos de mutación que aceptan `confirm` usan `false` de forma predeterminada. `force` y `confirm` expresan intenciones distintas: `force` habilita un comportamiento que de otro modo se rechaza, mientras que `confirm` pregunta al usuario antes de ejecutar una acción ya válida.

Se mantienen los valores predeterminados existentes: el remoto es `"origin"`, push sin refspecs envía la rama actual, status incluye archivos sin seguimiento y los commits vacíos siguen permitidos salvo que se indique `allow_empty = false`.

La eliminación de etiquetas remotas se expresa mediante un refspec de push:

```lua
repo:push("origin", ":refs/tags/v1.0.0", { confirm = true })
```
