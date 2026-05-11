# API de SemVer

Las utilidades para analizar, validar, comparar e incrementar versiones viven
bajo `ptool.semver` y `p.semver`.

## ptool.semver.parse

> `v0.1.0` - Introduced.

`ptool.semver.parse(version)` analiza una cadena de versión y devuelve un
objeto `Version`.

- `version` (string, obligatorio): Una cadena de versión semántica, con prefijo
  `v` opcional.

Ejemplo:

```lua
local v = ptool.semver.parse("v1.2.3-alpha.1+build.9")
print(v.major, v.minor, v.patch)
print(v.pre, v.build)
print(tostring(v))
```

## ptool.semver.is_valid

> `v0.1.0` - Introduced.

`ptool.semver.is_valid(version)` comprueba si una cadena de versión es válida.

- `version` (string, obligatorio): Una cadena de versión semántica.
- Devuelve: `boolean`.

```lua
print(ptool.semver.is_valid("1.2.3")) -- true
print(ptool.semver.is_valid("x.y.z")) -- false
```

## ptool.semver.compare

> `v0.1.0` - Introduced.

`ptool.semver.compare(a, b)` compara dos versiones.

- `a` / `b` (string|Version, obligatorio): Una cadena de versión o un objeto
  `Version`.
- Devuelve: `-1 | 0 | 1`.

```lua
print(ptool.semver.compare("1.2.3", "1.2.4")) -- -1
```

## ptool.semver.bump

> `v0.1.0` - Introduced.

`ptool.semver.bump(v, op[, channel])` devuelve un nuevo objeto de versión tras
aplicar el incremento.

- `v` (string|Version, obligatorio): La versión original.
- `op` (string, obligatorio): Uno de `major`, `minor`, `patch`, `release`,
  `alpha`, `beta`, `rc`, `prepatch`, `preminor` o `premajor`.
- `channel` (string, opcional): Solo se admite para `prepatch`, `preminor` y
  `premajor`. Debe ser `alpha`, `beta` o `rc`. Por defecto es `alpha`.
- Devuelve: `Version`.

```lua
local v = ptool.semver.bump("1.2.3", "alpha")
print(tostring(v)) -- 1.2.4-alpha.1

local next_minor_rc = ptool.semver.bump("1.2.3", "preminor", "rc")
print(tostring(next_minor_rc)) -- 1.3.0-rc.1

local stable = ptool.semver.bump("1.2.4-rc.2", "release")
print(tostring(stable)) -- 1.2.4
```

## Version

> `v0.1.0` - Introduced.

`Version` representa una versión semántica analizada devuelta por
`ptool.semver.parse(...)` o `ptool.semver.bump(...)`.

Está implementado como userdata de Lua.

Campos y métodos:

- Campos:
  - `major` (integer)
  - `minor` (integer)
  - `patch` (integer)
  - `pre` (string|nil)
  - `build` (string|nil)
- Métodos:
  - `v:compare(other)` -> `-1|0|1`
  - `v:bump(op[, channel])` -> `Version`
  - `v:to_string()` -> `string`
- Metamétodos:
  - `tostring(v)` está disponible.
  - Se admiten las comparaciones `==`, `<` y `<=`.

### compare

Nombre canónico de API: `ptool.semver.Version:compare`.

`v:compare(other)` compara la versión actual con `other`.

- `other` (string|Version, obligatorio): Una cadena de versión u otro objeto
  `Version`.
- Devuelve: `-1 | 0 | 1`.

### bump

Nombre canónico de API: `ptool.semver.Version:bump`.

`v:bump(op[, channel])` devuelve un nuevo objeto `Version` tras aplicar el
incremento.

- `op` (string, obligatorio): Uno de `major`, `minor`, `patch`, `release`,
  `alpha`, `beta`, `rc`, `prepatch`, `preminor` o `premajor`.
- `channel` (string, opcional): Solo se admite para `prepatch`, `preminor` y
  `premajor`. Debe ser `alpha`, `beta` o `rc`. Por defecto es `alpha`.
- Devuelve: `Version`.

### to_string

Nombre canónico de API: `ptool.semver.Version:to_string`.

`v:to_string()` devuelve la forma de cadena canónica de la versión.

- Devuelve: `string`.

Reglas de incremento prerelease:

- Al incrementar una versión estable a `alpha`, `beta` o `rc`, primero se
  incrementa la versión patch y luego se entra en el canal objetivo desde `.1`.
- `prepatch`, `preminor` y `premajor` inician una nueva prerelease desde la
  siguiente versión patch, minor o major respectivamente, por ejemplo
  `1.2.3 -> 1.3.0-rc.1` con `preminor` + `rc`.
- Al incrementar dentro del mismo canal se aumenta el número de secuencia,
  como `alpha.1 -> alpha.2`.
- `release` elimina los metadatos de prerelease y build manteniendo el mismo
  `major.minor.patch`, por ejemplo `1.2.3-rc.2 -> 1.2.3`.
- Se permite promocionar de canal (`alpha -> beta -> rc`), pero no degradarlo
  por ejemplo `rc -> beta` produce un error.
