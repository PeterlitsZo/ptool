# API de SemVer

As utilidades para analisar, validar, comparar, verificar requisitos de versão e incrementar versões vivem em `ptool.semver` e `p.semver`.

## ptool.semver.parse

> `v0.1.0` - Introduced.

`ptool.semver.parse(version)` faz o parse de uma string de versão e retorna um objeto `Version`.

- `version` (string, obrigatório): Uma string de versão semântica, com prefixo `v` opcional.

Exemplo:

```lua
local v = ptool.semver.parse("v1.2.3-alpha.1+build.9")
print(v.major, v.minor, v.patch)
print(v.pre, v.build)
print(tostring(v))
```

## ptool.semver.is_valid

> `v0.1.0` - Introduced.

`ptool.semver.is_valid(version)` verifica se uma string de versão é válida.

- `version` (string, obrigatório): Uma string de versão semântica.
- Returns: `boolean`.

```lua
print(ptool.semver.is_valid("1.2.3")) -- true
print(ptool.semver.is_valid("x.y.z")) -- false
```

## ptool.semver.parse_req

> `v0.7.0` - Introduced.

`ptool.semver.parse_req(req)` analisa uma string de requisito de versão semântica no estilo Cargo e retorna um objeto `VersionReq`.

- `req` (string, obrigatório): Uma string de requisito de versão.
- Retorna: `VersionReq`.

Exemplos suportados incluem `^1.2.3`, `~1.2.3`, `>=1.2.3, <2.0.0`, `1.*` e `1.2.*`. Componentes de versão dentro de um requisito também podem usar um prefixo `v` opcional, como `>= v0.6.0, < 0.7.0`.

```lua
local req = ptool.semver.parse_req(">= v0.6.0, < 0.7.0")
print(tostring(req)) -- >=0.6.0, <0.7.0
```

## ptool.semver.is_valid_req

> `v0.7.0` - Introduced.

`ptool.semver.is_valid_req(req)` verifica se uma string de requisito de versão é válida.

- `req` (string, obrigatório): Uma string de requisito de versão.
- Returns: `boolean`.

```lua
print(ptool.semver.is_valid_req("^1.2.3")) -- true
print(ptool.semver.is_valid_req(">= 1.2.3, <")) -- false
```

## ptool.semver.compare

> `v0.1.0` - Introduced.

`ptool.semver.compare(a, b)` compara duas versões.

- `a` / `b` (string|Version, obrigatório): Uma string de versão ou um objeto `Version`.
- Returns: `-1 | 0 | 1`.

```lua
print(ptool.semver.compare("1.2.3", "1.2.4")) -- -1
```

## ptool.semver.matches

> `v0.7.0` - Introduced.

`ptool.semver.matches(req, version)` verifica se uma versão satisfaz um requisito de versão.

- `req` (string|VersionReq, obrigatório): Uma string de requisito de versão ou um objeto `VersionReq`.
- `version` (string|Version, obrigatório): Uma string de versão ou um objeto `Version`.
- Returns: `boolean`.

```lua
local req = ptool.semver.parse_req("^0.6.0")
print(ptool.semver.matches(req, "0.6.3")) -- true
print(ptool.semver.matches(">=0.6.0, <0.7.0", "0.7.0")) -- false
```

## ptool.semver.bump

> `v0.1.0` - Introduced.

`ptool.semver.bump(v, op[, channel])` retorna um novo objeto de versão depois de aplicar o incremento.

- `v` (string|Version, obrigatório): A versão original.
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

`Version` representa uma versão semântica analisada retornada por `ptool.semver.parse(...)` ou `ptool.semver.bump(...)`.

It is implemented as a Lua userdata.

Campos e métodos:

- Campos:
  - `major` (integer)
  - `minor` (integer)
  - `patch` (integer)
  - `pre` (string|nil)
  - `build` (string|nil)
- Methods:
  - `v:compare(other)` -> `-1|0|1`
  - `v:bump(op[, channel])` -> `Version`
  - `v:to_string()` -> `string`
- Metamethods:
  - `tostring(v)` está disponível.
  - Comparações `==`, `<` e `<=` são suportadas.

## VersionReq

> `v0.7.0` - Introduced.

`VersionReq` representa um requisito de versão semântica analisado retornado por `ptool.semver.parse_req(...)`.

It is implemented as a Lua userdata.

Methods:

- `req:matches(version)` -> `boolean`
- `req:to_string()` -> `string`

Metamethods:

- `tostring(req)` está disponível.

### compare

Nome canônico da API: `ptool.semver.Version:compare`.

`v:compare(other)` compara a versão atual com `other`.

- `other` (string|Version, obrigatório): Uma string de versão ou outro objeto `Version`.
- Returns: `-1 | 0 | 1`.

### bump

Nome canônico da API: `ptool.semver.Version:bump`.

`v:bump(op[, channel])` retorna um novo objeto `Version` depois de aplicar o incremento.

- `op` (string, required): One of `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`, `prepatch`, `preminor`, or `premajor`.
- `channel` (string, optional): Supported only for `prepatch`, `preminor`, and `premajor`. Must be one of `alpha`, `beta`, or `rc`. Defaults to `alpha`.
- Returns: `Version`.

### to_string

Nome canônico da API: `ptool.semver.Version:to_string`.

`v:to_string()` retorna a forma canônica da versão em string.

- Returns: `string`.

### matches

Nome canônico da API: `ptool.semver.VersionReq:matches`.

`req:matches(version)` verifica se `version` satisfaz o requisito de versão atual.

- `version` (string|Version, obrigatório): Uma string de versão ou um objeto `Version`.
- Returns: `boolean`.

### to_string

Nome canônico da API: `ptool.semver.VersionReq:to_string`.

`req:to_string()` retorna a forma canônica em string do requisito de versão.

- Returns: `string`.

Regras de incremento de prerelease:

- Ao incrementar uma versão estável para `alpha`, `beta` ou `rc`, primeiro a versão patch é incrementada e depois o canal alvo começa em `.1`.
- `prepatch`, `preminor` e `premajor` iniciam uma nova prerelease a partir da próxima versão patch, minor ou major, respectivamente, por exemplo `1.2.3 -> 1.3.0-rc.1` com `preminor` + `rc`.
- Incrementar dentro do mesmo canal aumenta o número de sequência, como em `alpha.1 -> alpha.2`.
- `release` remove metadados de prerelease e build, mantendo o mesmo `major.minor.patch`, por exemplo `1.2.3-rc.2 -> 1.2.3`.
- Promoção de canal é permitida (`alpha -> beta -> rc`), mas rebaixamento não é permitido, por exemplo `rc -> beta` gera erro.
