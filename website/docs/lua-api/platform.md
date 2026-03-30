# Platform API

Platform detection helpers are available under `ptool.platform` and `p.platform`.

## ptool.platform.os

> `v0.1.0` - Introduced.

`ptool.platform.os()` returns the operating system of the current machine.

- Returns: `linux | macos | windows`.

```lua
print(ptool.platform.os()) -- macos
```

Behavior:

- This reports the local machine running `ptool run`.
- Only `linux`, `macos`, and `windows` are supported.

## ptool.platform.arch

> `v0.1.0` - Introduced.

`ptool.platform.arch()` returns the CPU architecture of the current machine.

- Returns: `amd64 | arm64`.

```lua
print(ptool.platform.arch()) -- arm64
```

Behavior:

- `x86_64` is exposed as `amd64`.
- `aarch64` is exposed as `arm64`.
- Only `x86_64` and `aarch64` are supported by `ptool`.

## ptool.platform.target

> `v0.1.0` - Introduced.

`ptool.platform.target()` returns a normalized platform target string for the
current machine.

- Returns: `string`.

```lua
local target = ptool.platform.target()
print(target) -- macos-arm64
```

Behavior:

- The result is always `ptool.platform.os() .. "-" .. ptool.platform.arch()`.
- This is intended for platform-based branching such as selecting download
  artifacts.
