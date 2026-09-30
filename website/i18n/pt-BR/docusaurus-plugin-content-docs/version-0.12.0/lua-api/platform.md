# API de plataforma

As utilidades de detecção de plataforma estão disponíveis em `ptool.platform` e `p.platform`.

## ptool.platform.os

> `v0.1.0` - Introduced.

`ptool.platform.os()` retorna o sistema operacional da máquina atual.

- Retorna: `linux | macos | windows`.

```lua
print(ptool.platform.os()) -- macos
```

Comportamento:

- Isso informa a máquina local que está executando `ptool run`.
- Atualmente, `ptool` expõe `linux`, `macos` e `windows`.

## ptool.platform.arch

> `v0.1.0` - Introduced.

`ptool.platform.arch()` retorna a arquitetura de CPU da máquina atual.

- Retorna: `amd64 | arm64 | x86 | arm | riscv64`.

```lua
print(ptool.platform.arch()) -- arm64
```

Comportamento:

- `x86_64` é exposto como `amd64`.
- `aarch64` é exposto como `arm64`.
- Variantes x86 de 32 bits, como `x86` e `i686`, são expostas como `x86`.
- Variantes ARM de 32 bits, como `armv7l`, são expostas como `arm`.
- `riscv64` é exposto como `riscv64`.

## ptool.platform.target

> `v0.1.0` - Introduced.

`ptool.platform.target()` retorna uma string de target de plataforma normalizada para a máquina atual.

- Retorna: `string`.

```lua
local target = ptool.platform.target()
print(target) -- linux-riscv64
```

Comportamento:

- O resultado é sempre `ptool.platform.os() .. "-" .. ptool.platform.arch()`.
- Isso se destina a desvios baseados em plataforma, como selecionar artefatos de download.
- Valores comuns incluem `linux-amd64`, `linux-arm64`, `linux-x86`, `linux-arm`, `linux-riscv64`, `macos-amd64`, `macos-arm64` e `windows-amd64`.
