# Platform API

平台检测辅助能力位于 `ptool.platform` 和 `p.platform` 下。

## ptool.platform.os

> `v0.1.0` - 引入。

`ptool.platform.os()` 返回当前机器的操作系统。

- 返回：`linux | macos | windows`。

```lua
print(ptool.platform.os()) -- macos
```

行为说明：

- 这里报告的是运行 `ptool run` 的本机环境。
- `ptool` 当前会暴露 `linux`、`macos` 和 `windows`。

## ptool.platform.arch

> `v0.1.0` - 引入。

`ptool.platform.arch()` 返回当前机器的 CPU 架构。

- 返回：`amd64 | arm64 | x86 | arm | riscv64`。

```lua
print(ptool.platform.arch()) -- arm64
```

行为说明：

- `x86_64` 会暴露为 `amd64`。
- `aarch64` 会暴露为 `arm64`。
- `x86`、`i686` 等 32 位 x86 变体会统一暴露为 `x86`。
- `armv7l` 等 32 位 ARM 变体会统一暴露为 `arm`。
- `riscv64` 会暴露为 `riscv64`。

## ptool.platform.target

> `v0.1.0` - 引入。

`ptool.platform.target()` 返回当前机器的规范化平台目标字符串。

- 返回：`string`。

```lua
local target = ptool.platform.target()
print(target) -- linux-riscv64
```

行为说明：

- 结果始终等于 `ptool.platform.os() .. "-" .. ptool.platform.arch()`。
- 这个值适合做基于平台的分支判断，例如选择下载产物。
- 常见值包括 `linux-amd64`、`linux-arm64`、`linux-x86`、`linux-arm`、 `linux-riscv64`、`macos-amd64`、`macos-arm64` 和 `windows-amd64`。
