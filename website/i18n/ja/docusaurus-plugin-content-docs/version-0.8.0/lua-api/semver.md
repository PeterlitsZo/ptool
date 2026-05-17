# SemVer API

バージョンの解析、検証、比較、バージョン要件の照合、更新のヘルパーは `ptool.semver` と `p.semver` にあります。

## ptool.semver.parse

> `v0.1.0` - Introduced.

`ptool.semver.parse(version)` はバージョン文字列を解析し、`Version` オブジェクトを返します。

- `version` (string, 必須): セマンティックバージョン文字列。先頭の `v` は 省略可能です。

例:

```lua
local v = ptool.semver.parse("v1.2.3-alpha.1+build.9")
print(v.major, v.minor, v.patch)
print(v.pre, v.build)
print(tostring(v))
```

## ptool.semver.is_valid

> `v0.1.0` - Introduced.

`ptool.semver.is_valid(version)` はバージョン文字列が有効かどうかを 確認します。

- `version` (string, 必須): セマンティックバージョン文字列。
- Returns: `boolean`.

```lua
print(ptool.semver.is_valid("1.2.3")) -- true
print(ptool.semver.is_valid("x.y.z")) -- false
```

## ptool.semver.parse_req

> `v0.7.0` - Introduced.

`ptool.semver.parse_req(req)` は Cargo 形式のセマンティックバージョン要件文字列を解析し、`VersionReq` オブジェクトを返します。

- `req` (string, 必須): バージョン要件文字列。
- 戻り値: `VersionReq`.

サポートされる例には `^1.2.3`、`~1.2.3`、`>=1.2.3, <2.0.0`、`1.*`、`1.2.*` が含まれます。要件内のバージョン要素では、`>= v0.6.0, < 0.7.0` のように任意の `v` プレフィックスも使用できます。

```lua
local req = ptool.semver.parse_req(">= v0.6.0, < 0.7.0")
print(tostring(req)) -- >=0.6.0, <0.7.0
```

## ptool.semver.is_valid_req

> `v0.7.0` - Introduced.

`ptool.semver.is_valid_req(req)` はバージョン要件文字列が有効かどうかを確認します。

- `req` (string, 必須): バージョン要件文字列。
- Returns: `boolean`.

```lua
print(ptool.semver.is_valid_req("^1.2.3")) -- true
print(ptool.semver.is_valid_req(">= 1.2.3, <")) -- false
```

## ptool.semver.compare

> `v0.1.0` - Introduced.

`ptool.semver.compare(a, b)` は 2 つのバージョンを比較します。

- `a` / `b` (string|Version, 必須): バージョン文字列または `Version` オブジェクト。
- Returns: `-1 | 0 | 1`.

```lua
print(ptool.semver.compare("1.2.3", "1.2.4")) -- -1
```

## ptool.semver.matches

> `v0.7.0` - Introduced.

`ptool.semver.matches(req, version)` は、あるバージョンがバージョン要件を満たすかどうかを確認します。

- `req` (string|VersionReq, 必須): バージョン要件文字列または `VersionReq` オブジェクト。
- `version` (string|Version, 必須): バージョン文字列または `Version` オブジェクト。
- Returns: `boolean`.

```lua
local req = ptool.semver.parse_req("^0.6.0")
print(ptool.semver.matches(req, "0.6.3")) -- true
print(ptool.semver.matches(">=0.6.0, <0.7.0", "0.7.0")) -- false
```

## ptool.semver.bump

> `v0.1.0` - Introduced.

`ptool.semver.bump(v, op[, channel])` は、更新を適用した新しい バージョンオブジェクトを返します。

- `v` (string|Version, 必須): 元のバージョン。
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

`Version` は `ptool.semver.parse(...)` または `ptool.semver.bump(...)` が返す、解析済みのセマンティックバージョンを表します。

It is implemented as a Lua userdata.

フィールドとメソッド:

- フィールド:
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
  - `tostring(v)` が利用できます。
  - `==`, `<`, `<=` の比較をサポートします。

### compare

Canonical API name: `ptool.semver.Version:compare`.

`v:compare(other)` は現在のバージョンと `other` を比較します。

- `other` (string|Version, 必須): バージョン文字列または別の `Version` オブジェクト。
- Returns: `-1 | 0 | 1`.

### bump

Canonical API name: `ptool.semver.Version:bump`.

`v:bump(op[, channel])` は更新を適用した新しい `Version` オブジェクトを 返します。

- `op` (string, required): One of `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`, `prepatch`, `preminor`, or `premajor`.
- `channel` (string, optional): Supported only for `prepatch`, `preminor`, and `premajor`. Must be one of `alpha`, `beta`, or `rc`. Defaults to `alpha`.
- Returns: `Version`.

### to_string

Canonical API name: `ptool.semver.Version:to_string`.

`v:to_string()` は、そのバージョンの正規化された文字列表現を返します。

- Returns: `string`.

## VersionReq

> `v0.7.0` - Introduced.

`VersionReq` は `ptool.semver.parse_req(...)` が返す、解析済みのセマンティックバージョン要件を表します。

It is implemented as a Lua userdata.

Methods:

- `req:matches(version)` -> `boolean`
- `req:to_string()` -> `string`

Metamethods:

- `tostring(req)` が利用できます。

### matches

Canonical API name: `ptool.semver.VersionReq:matches`.

`req:matches(version)` は `version` が現在のバージョン要件を満たすかどうかを確認します。

- `version` (string|Version, 必須): バージョン文字列または `Version` オブジェクト。
- Returns: `boolean`.

### to_string

Canonical API name: `ptool.semver.VersionReq:to_string`.

`req:to_string()` は、そのバージョン要件の正規化された文字列表現を返します。

- Returns: `string`.

プレリリース更新ルール:

- 安定版を `alpha`, `beta`, `rc` に更新する場合、まず patch バージョンを 1 つ上げてから、対象チャネルの `.1` から開始します。
- `prepatch`, `preminor`, `premajor` はそれぞれ次の patch、minor、major バージョンから新しいプレリリースを開始します。たとえば `preminor` + `rc` では `1.2.3 -> 1.3.0-rc.1` です。
- 同じチャネル内で更新すると、`alpha.1 -> alpha.2` のように連番が 増えます。
- `release` は `major.minor.patch` を維持したまま prerelease と build メタデータを取り除きます。たとえば `1.2.3-rc.2 -> 1.2.3` です。
- チャネルの昇格 (`alpha -> beta -> rc`) は可能ですが、降格はできません。 たとえば `rc -> beta` はエラーになります。
