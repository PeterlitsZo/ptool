# API de SemVer

Las utilidades para analizar, validar, comparar, comprobar requisitos de versión e incrementar versiones viven bajo `ptool.semver` y `p.semver`.

## ptool.semver.parse

> `v0.1.0` - Introduced.

`ptool.semver.parse(version)` analiza una cadena de versión y devuelve un objeto `Version`.

- `version` (string, obligatorio): Una cadena de versión semántica, con prefijo `v` opcional.

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
- Returns: `boolean`.

```lua
print(ptool.semver.is_valid("1.2.3")) -- true
print(ptool.semver.is_valid("x.y.z")) -- false
```

## ptool.semver.parse_req

> `v0.7.0` - Introducido.

`ptool.semver.parse_req(req)` analiza una cadena de requisito de versión semántica al estilo Cargo y devuelve un objeto `VersionReq`.

- `req` (string, obligatorio): Una cadena de requisito de versión.
- Devuelve: `VersionReq`.

Entre los ejemplos admitidos están `^1.2.3`, `~1.2.3`, `>=1.2.3, <2.0.0`, `1.*` y `1.2.*`. Los componentes de versión dentro de un requisito también pueden usar un prefijo `v` opcional, como `>= v0.6.0, < 0.7.0`.

```lua
local req = ptool.semver.parse_req(">= v0.6.0, < 0.7.0")
print(tostring(req)) -- >=0.6.0, <0.7.0
```

## ptool.semver.is_valid_req

> `v0.7.0` - Introducido.

`ptool.semver.is_valid_req(req)` comprueba si una cadena de requisito de versión es válida.

- `req` (string, obligatorio): Una cadena de requisito de versión.
- Returns: `boolean`.

```lua
print(ptool.semver.is_valid_req("^1.2.3")) -- true
print(ptool.semver.is_valid_req(">= 1.2.3, <")) -- false
```

## ptool.semver.compare

> `v0.1.0` - Introduced.

`ptool.semver.compare(a, b)` compara dos versiones.

- `a` / `b` (string|Version, obligatorio): Una cadena de versión o un objeto `Version`.
- Returns: `-1 | 0 | 1`.

```lua
print(ptool.semver.compare("1.2.3", "1.2.4")) -- -1
```

## ptool.semver.matches

> `v0.7.0` - Introducido.

`ptool.semver.matches(req, version)` comprueba si una versión satisface un requisito de versión.

- `req` (string|VersionReq, obligatorio): Una cadena de requisito de versión o un objeto `VersionReq`.
- `version` (string|Version, obligatorio): Una cadena de versión o un objeto `Version`.
- Returns: `boolean`.

```lua
local req = ptool.semver.parse_req("^0.6.0")
print(ptool.semver.matches(req, "0.6.3")) -- true
print(ptool.semver.matches(">=0.6.0, <0.7.0", "0.7.0")) -- false
```

## ptool.semver.bump

> `v0.1.0` - Introduced.

`ptool.semver.bump(v, op[, channel])` devuelve un nuevo objeto de versión tras aplicar el incremento.

- `v` (string|Version, obligatorio): La versión original.
- `op` (string, required): One of `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`, `prepatch`, `preminor`, or `premajor`.
- `channel` (string, optional): Supported only for `prepatch`, `preminor`, and `premajor`. Must be one of `alpha`, `beta`, or `rc`. Defaults to `alpha`.
- Returns: `Version`.

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

`Version` representa una versión semántica analizada devuelta por `ptool.semver.parse(...)` o `ptool.semver.bump(...)`.

It is implemented as a Lua userdata.

Campos y métodos:

- Campos:
  - `major` (entero)
  - `minor` (entero)
  - `patch` (entero)
  - `pre` (cadena|cero)
  - `build` (cadena|cero)
- Methods:
  - `v:compare(other)` -> `-1|0|1`
  - `v:bump(op[, channel])` -> `Version`
  - `v:to_string()` -> `string`
- Metamethods:
  - `tostring(v)` está disponible.
  - Se admiten las comparaciones `==`, `<` y `<=`.

### compare

Nombre canónico de API: `ptool.semver.Version:compare`.

`v:compare(other)` compara la versión actual con `other`.

- `other` (string|Version, obligatorio): Una cadena de versión u otro objeto `Version`.
- Returns: `-1 | 0 | 1`.

### bump

Nombre canónico de API: `ptool.semver.Version:bump`.

`v:bump(op[, channel])` devuelve un nuevo objeto `Version` tras aplicar el incremento.

- `op` (string, required): One of `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`, `prepatch`, `preminor`, or `premajor`.
- `channel` (string, optional): Supported only for `prepatch`, `preminor`, and `premajor`. Must be one of `alpha`, `beta`, or `rc`. Defaults to `alpha`.
- Returns: `Version`.

### to_string

Nombre canónico de API: `ptool.semver.Version:to_string`.

`v:to_string()` devuelve la forma de cadena canónica de la versión.

- Returns: `string`.

## VersionReq

> `v0.7.0` - Introducido.

`VersionReq` representa un requisito de versión semántica analizado devuelto por `ptool.semver.parse_req(...)`.

It is implemented as a Lua userdata.

Methods:

- `req:matches(version)` -> `boolean`
- `req:to_string()` -> `string`

Metamethods:

- `tostring(req)` está disponible.

### matches

Nombre canónico de API: `ptool.semver.VersionReq:matches`.

`req:matches(version)` comprueba si `version` satisface el requisito de versión actual.

- `version` (string|Version, obligatorio): Una cadena de versión o un objeto `Version`.
- Returns: `boolean`.

### to_string

Nombre canónico de API: `ptool.semver.VersionReq:to_string`.

`req:to_string()` devuelve la forma de cadena canónica del requisito de versión.

- Returns: `string`.

Reglas de incremento prerelease:

- Al incrementar una versión estable a `alpha`, `beta` o `rc`, primero se incrementa la versión patch y luego se entra en el canal objetivo desde `.1`.
- `prepatch`, `preminor` y `premajor` inician una nueva prerelease desde la siguiente versión patch, minor o major respectivamente, por ejemplo `1.2.3 -> 1.3.0-rc.1` con `preminor` + `rc`.
- Al incrementar dentro del mismo canal se aumenta el número de secuencia, como `alpha.1 -> alpha.2`.
- `release` elimina los metadatos de prerelease y build manteniendo el mismo `major.minor.patch`, por ejemplo `1.2.3-rc.2 -> 1.2.3`.
- Se permite promocionar de canal (`alpha -> beta -> rc`), pero no degradarlo por ejemplo `rc -> beta` produce un error.
