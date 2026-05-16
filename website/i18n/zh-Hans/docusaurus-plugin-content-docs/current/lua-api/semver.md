# SemVer API

版本解析、校验、比较、版本要求匹配和提升辅助能力位于 `ptool.semver` 和 `p.semver` 下。

## ptool.semver.parse

> `v0.1.0` - 引入。

`ptool.semver.parse(version)` 解析版本字符串，并返回 `Version` 对象。

- `version`（string，必填）：语义化版本字符串，可选带 `v` 前缀。

示例：

```lua
local v = ptool.semver.parse("v1.2.3-alpha.1+build.9")
print(v.major, v.minor, v.patch)
print(v.pre, v.build)
print(tostring(v))
```

## ptool.semver.is_valid

> `v0.1.0` - 引入。

`ptool.semver.is_valid(version)` 检查版本字符串是否合法。

- `version`（string，必填）：语义化版本字符串。
- 返回：`boolean`。

```lua
print(ptool.semver.is_valid("1.2.3")) -- true
print(ptool.semver.is_valid("x.y.z")) -- false
```

## ptool.semver.parse_req

> `v0.7.0` - 引入。

`ptool.semver.parse_req(req)` 解析 Cargo 风格的语义化版本要求字符串，并返回 `VersionReq` 对象。

- `req`（string，必填）：版本要求字符串。
- 返回：`VersionReq`。

支持的示例包括 `^1.2.3`、`~1.2.3`、`>=1.2.3, <2.0.0`、`1.*` 和 `1.2.*`。版本要求中的版本组件也可以使用可选的 `v` 前缀，例如 `>= v0.6.0, < 0.7.0`。

```lua
local req = ptool.semver.parse_req(">= v0.6.0, < 0.7.0")
print(tostring(req)) -- >=0.6.0, <0.7.0
```

## ptool.semver.is_valid_req

> `v0.7.0` - 引入。

`ptool.semver.is_valid_req(req)` 检查版本要求字符串是否合法。

- `req`（string，必填）：版本要求字符串。
- 返回：`boolean`。

```lua
print(ptool.semver.is_valid_req("^1.2.3")) -- true
print(ptool.semver.is_valid_req(">= 1.2.3, <")) -- false
```

## ptool.semver.compare

> `v0.1.0` - 引入。

`ptool.semver.compare(a, b)` 比较两个版本。

- `a` / `b`（string|Version，必填）：版本字符串或 `Version` 对象。
- 返回：`-1 | 0 | 1`。

```lua
print(ptool.semver.compare("1.2.3", "1.2.4")) -- -1
```

## ptool.semver.matches

> `v0.7.0` - 引入。

`ptool.semver.matches(req, version)` 检查某个版本是否满足版本要求。

- `req`（string|VersionReq，必填）：版本要求字符串或 `VersionReq` 对象。
- `version`（string|Version，必填）：版本字符串或 `Version` 对象。
- 返回：`boolean`。

```lua
local req = ptool.semver.parse_req("^0.6.0")
print(ptool.semver.matches(req, "0.6.3")) -- true
print(ptool.semver.matches(">=0.6.0, <0.7.0", "0.7.0")) -- false
```

## ptool.semver.bump

> `v0.1.0` - 引入。

`ptool.semver.bump(v, op[, channel])` 应用版本提升操作，并返回新的版本对象。

- `v`（string|Version，必填）：原始版本。
- `op`（string，必填）：取值之一：`major`、`minor`、`patch`、`release`、 `alpha`、`beta`、`rc`、`prepatch`、`preminor` 或 `premajor`。
- `channel`（string，可选）：仅对 `prepatch`、`preminor` 和 `premajor` 有效。 必须是 `alpha`、`beta` 或 `rc` 之一，默认值为 `alpha`。
- 返回：`Version`。

```lua
local v = ptool.semver.bump("1.2.3", "alpha")
print(tostring(v)) -- 1.2.4-alpha.1

local next_minor_rc = ptool.semver.bump("1.2.3", "preminor", "rc")
print(tostring(next_minor_rc)) -- 1.3.0-rc.1

local stable = ptool.semver.bump("1.2.4-rc.2", "release")
print(tostring(stable)) -- 1.2.4
```

## Version

> `v0.1.0` - 引入。

`Version` 表示由 `ptool.semver.parse(...)` 或 `ptool.semver.bump(...)` 返回的已解析语义化版本。

它实现为 Lua 用户数据（userdata）。

字段和方法：

- 字段：
  - `major`（integer）
  - `minor`（integer）
  - `patch`（integer）
  - `pre`（string|nil）
  - `build`（string|nil）
- 方法：
  - `v:compare(other)` -> `-1|0|1`
  - `v:bump(op[, channel])` -> `Version`
  - `v:to_string()` -> `string`
- 元方法：
  - 支持 `tostring(v)`。
  - 支持 `==`、`<` 和 `<=` 比较。

### compare

规范 API 名称：`ptool.semver.Version:compare`。

`v:compare(other)` 比较当前版本和 `other`。

- `other`（string|Version，必填）：版本字符串或另一个 `Version` 对象。
- 返回：`-1 | 0 | 1`。

### bump

规范 API 名称：`ptool.semver.Version:bump`。

`v:bump(op[, channel])` 应用版本提升操作，并返回新的 `Version` 对象。

- `op`（string，必填）：取值之一：`major`、`minor`、`patch`、`release`、 `alpha`、`beta`、`rc`、`prepatch`、`preminor` 或 `premajor`。
- `channel`（string，可选）：仅对 `prepatch`、`preminor` 和 `premajor` 有效。 必须是 `alpha`、`beta` 或 `rc` 之一，默认值为 `alpha`。
- 返回：`Version`。

### to_string

规范 API 名称：`ptool.semver.Version:to_string`。

`v:to_string()` 返回版本的规范字符串形式。

- 返回：`string`。

## VersionReq

> `v0.7.0` - 引入。

`VersionReq` 表示由 `ptool.semver.parse_req(...)` 返回的已解析语义化版本要求。

它实现为 Lua 用户数据（userdata）。

方法：

- `req:matches(version)` -> `boolean`
- `req:to_string()` -> `string`

元方法：

- 支持 `tostring(req)`。

### matches

规范 API 名称：`ptool.semver.VersionReq:matches`。

`req:matches(version)` 检查 `version` 是否满足当前版本要求。

- `version`（string|Version，必填）：版本字符串或 `Version` 对象。
- 返回：`boolean`。

### to_string

规范 API 名称：`ptool.semver.VersionReq:to_string`。

`req:to_string()` 返回版本要求的规范字符串形式。

- 返回：`string`。

预发布提升规则：

- 将稳定版本提升到 `alpha`、`beta` 或 `rc` 时，会先递增 patch 版本，再进入目标通道并从 `.1` 开始。
- `prepatch`、`preminor` 和 `premajor` 会分别从下一个 patch、minor 或 major 版本开始全新的预发布，例如使用 `preminor` + `rc` 时，`1.2.3 -> 1.3.0-rc.1`。
- 在同一通道内继续提升时，会递增序号，例如 `alpha.1 -> alpha.2`。
- `release` 会移除预发布和 build 元数据，同时保持 `major.minor.patch` 不变，例如 `1.2.3-rc.2 -> 1.2.3`。
- 允许通道升级（`alpha -> beta -> rc`），但不允许通道降级（例如 `rc -> beta` 会抛出错误）。
