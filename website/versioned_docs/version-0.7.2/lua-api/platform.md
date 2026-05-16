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
- `ptool` currently exposes `linux`, `macos`, and `windows`.

## ptool.platform.arch

> `v0.1.0` - Introduced.

`ptool.platform.arch()` returns the CPU architecture of the current machine.

- Returns: `amd64 | arm64 | x86 | arm | riscv64`.

```lua
print(ptool.platform.arch()) -- arm64
```

Behavior:

- `x86_64` is exposed as `amd64`.
- `aarch64` is exposed as `arm64`.
- 32-bit x86 variants such as `x86` and `i686` are exposed as `x86`.
- 32-bit ARM variants such as `armv7l` are exposed as `arm`.
- `riscv64` is exposed as `riscv64`.

## ptool.platform.target

> `v0.1.0` - Introduced.

`ptool.platform.target()` returns a normalized platform target string for the
current machine.

- Returns: `string`.

```lua
local target = ptool.platform.target()
print(target) -- linux-riscv64
```

Behavior:

- The result is always `ptool.platform.os() .. "-" .. ptool.platform.arch()`.
- This is intended for platform-based branching such as selecting download
  artifacts.
- Common values include `linux-amd64`, `linux-arm64`, `linux-x86`,
  `linux-arm`, `linux-riscv64`, `macos-amd64`, `macos-arm64`, and
  `windows-amd64`.
