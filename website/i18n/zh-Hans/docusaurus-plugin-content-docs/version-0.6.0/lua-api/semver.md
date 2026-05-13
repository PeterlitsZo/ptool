# SemVer API

Version parsing, validation, comparison, and bumping helpers live under `ptool.semver` and `p.semver`.

## ptool.semver.parse

> `v0.1.0` - Introduced.

`ptool.semver.parse(version)` parses a version string and returns a `Version` object.

- `version` (string, required): A semantic version string, optionally prefixed with `v`.

Example:

```lua
local v = ptool.semver.parse("v1.2.3-alpha.1+build.9")
print(v.major, v.minor, v.patch)
print(v.pre, v.build)
print(tostring(v))
```

## ptool.semver.is_valid

> `v0.1.0` - Introduced.

`ptool.semver.is_valid(version)` checks whether a version string is valid.

- `version` (string, required): A semantic version string.
- Returns: `boolean`.

```lua
print(ptool.semver.is_valid("1.2.3")) -- true
print(ptool.semver.is_valid("x.y.z")) -- false
```

## ptool.semver.compare

> `v0.1.0` - Introduced.

`ptool.semver.compare(a, b)` compares two versions.

- `a` / `b` (string|Version, required): A version string or a `Version` object.
- Returns: `-1 | 0 | 1`.

```lua
print(ptool.semver.compare("1.2.3", "1.2.4")) -- -1
```

## ptool.semver.bump

> `v0.1.0` - Introduced.

`ptool.semver.bump(v, op[, channel])` returns a new version object after applying the bump.

- `v` (string|Version, required): The original version.
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

`Version` represents a parsed semantic version returned by `ptool.semver.parse(...)` or `ptool.semver.bump(...)`.

It is implemented as a Lua userdata.

Fields and methods:

- Fields:
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
  - `tostring(v)` is available.
  - `==`, `<`, and `<=` comparisons are supported.

### compare

Canonical API name: `ptool.semver.Version:compare`.

`v:compare(other)` compares the current version with `other`.

- `other` (string|Version, required): A version string or another `Version` object.
- Returns: `-1 | 0 | 1`.

### bump

Canonical API name: `ptool.semver.Version:bump`.

`v:bump(op[, channel])` returns a new `Version` object after applying the bump.

- `op` (string, required): One of `major`, `minor`, `patch`, `release`, `alpha`, `beta`, `rc`, `prepatch`, `preminor`, or `premajor`.
- `channel` (string, optional): Supported only for `prepatch`, `preminor`, and `premajor`. Must be one of `alpha`, `beta`, or `rc`. Defaults to `alpha`.
- Returns: `Version`.

### to_string

Canonical API name: `ptool.semver.Version:to_string`.

`v:to_string()` returns the canonical string form of the version.

- Returns: `string`.

Prerelease bump rules:

- Bumping a stable version to `alpha`, `beta`, or `rc` first increments the patch version, then enters the target channel starting from `.1`.
- `prepatch`, `preminor`, and `premajor` start a fresh prerelease from the next patch, minor, or major version respectively, such as `1.2.3 -> 1.3.0-rc.1` with `preminor` + `rc`.
- Bumping within the same channel increments the sequence number, such as `alpha.1 -> alpha.2`.
- `release` removes prerelease and build metadata while keeping the same `major.minor.patch` values, such as `1.2.3-rc.2 -> 1.2.3`.
- Channel promotion is allowed (`alpha -> beta -> rc`), but channel downgrade is not (for example, `rc -> beta` raises an error).
