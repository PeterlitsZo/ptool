# SemVer API

バージョンの解析、検証、比較、更新のヘルパーは `ptool.semver` と `p.semver` にあります。

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
- 戻り値: `boolean`.

```lua
print(ptool.semver.is_valid("1.2.3")) -- true
print(ptool.semver.is_valid("x.y.z")) -- false
```

## ptool.semver.compare

> `v0.1.0` - Introduced.

`ptool.semver.compare(a, b)` は 2 つのバージョンを比較します。

- `a` / `b` (string|Version, 必須): バージョン文字列または `Version` オブジェクト。
- 戻り値: `-1 | 0 | 1`.

```lua
print(ptool.semver.compare("1.2.3", "1.2.4")) -- -1
```

## ptool.semver.bump

> `v0.1.0` - Introduced.

`ptool.semver.bump(v, op[, channel])` は、更新を適用した新しい バージョンオブジェクトを返します。

- `v` (string|Version, 必須): 元のバージョン。
- `op` (string, 必須): `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`, `prepatch`, `preminor`, `premajor` のいずれか。
- `channel` (string, 任意): `prepatch`, `preminor`, `premajor` でのみ 使用できます。`alpha`, `beta`, `rc` のいずれかで、デフォルトは `alpha` です。
- 戻り値: `Version`.

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

これは Lua userdata として実装されています。

フィールドとメソッド:

- フィールド:
  - `major` (integer)
  - `minor` (integer)
  - `patch` (integer)
  - `pre` (string|nil)
  - `build` (string|nil)
- メソッド:
  - `v:compare(other)` -> `-1|0|1`
  - `v:bump(op[, channel])` -> `Version`
  - `v:to_string()` -> `string`
- メタメソッド:
  - `tostring(v)` が利用できます。
  - `==`, `<`, `<=` の比較をサポートします。

### compare

Canonical API name: `ptool.semver.Version:compare`.

`v:compare(other)` は現在のバージョンと `other` を比較します。

- `other` (string|Version, 必須): バージョン文字列または別の `Version` オブジェクト。
- 戻り値: `-1 | 0 | 1`.

### bump

Canonical API name: `ptool.semver.Version:bump`.

`v:bump(op[, channel])` は更新を適用した新しい `Version` オブジェクトを 返します。

- `op` (string, 必須): `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`, `prepatch`, `preminor`, `premajor` のいずれか。
- `channel` (string, 任意): `prepatch`, `preminor`, `premajor` でのみ 使用できます。`alpha`, `beta`, `rc` のいずれかで、デフォルトは `alpha` です。
- 戻り値: `Version`.

### to_string

Canonical API name: `ptool.semver.Version:to_string`.

`v:to_string()` は、そのバージョンの正規化された文字列表現を返します。

- 戻り値: `string`.

プレリリース更新ルール:

- 安定版を `alpha`, `beta`, `rc` に更新する場合、まず patch バージョンを 1 つ上げてから、対象チャネルの `.1` から開始します。
- `prepatch`, `preminor`, `premajor` はそれぞれ次の patch、minor、major バージョンから新しいプレリリースを開始します。たとえば `preminor` + `rc` では `1.2.3 -> 1.3.0-rc.1` です。
- 同じチャネル内で更新すると、`alpha.1 -> alpha.2` のように連番が 増えます。
- `release` は `major.minor.patch` を維持したまま prerelease と build メタデータを取り除きます。たとえば `1.2.3-rc.2 -> 1.2.3` です。
- チャネルの昇格 (`alpha -> beta -> rc`) は可能ですが、降格はできません。 たとえば `rc -> beta` はエラーになります。
