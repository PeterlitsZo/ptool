# API de plataforma

Las utilidades de detección de plataforma están disponibles bajo
`ptool.platform` y `p.platform`.

## ptool.platform.os

> `v0.1.0` - Introduced.

`ptool.platform.os()` devuelve el sistema operativo de la máquina actual.

- Devuelve: `linux | macos | windows`.

```lua
print(ptool.platform.os()) -- macos
```

Comportamiento:

- Esto informa sobre la máquina local que está ejecutando `ptool run`.
- Actualmente `ptool` expone `linux`, `macos` y `windows`.

## ptool.platform.arch

> `v0.1.0` - Introduced.

`ptool.platform.arch()` devuelve la arquitectura de CPU de la máquina actual.

- Devuelve: `amd64 | arm64 | x86 | arm | riscv64`.

```lua
print(ptool.platform.arch()) -- arm64
```

Comportamiento:

- `x86_64` se expone como `amd64`.
- `aarch64` se expone como `arm64`.
- Las variantes x86 de 32 bits como `x86` e `i686` se exponen como `x86`.
- Las variantes ARM de 32 bits como `armv7l` se exponen como `arm`.
- `riscv64` se expone como `riscv64`.

## ptool.platform.target

> `v0.1.0` - Introduced.

`ptool.platform.target()` devuelve una cadena de destino de plataforma
normalizada para la máquina actual.

- Devuelve: `string`.

```lua
local target = ptool.platform.target()
print(target) -- linux-riscv64
```

Comportamiento:

- El resultado siempre es `ptool.platform.os() .. "-" .. ptool.platform.arch()`.
- Esto está pensado para bifurcaciones basadas en plataforma, como seleccionar
  artefactos de descarga.
- Los valores comunes incluyen `linux-amd64`, `linux-arm64`, `linux-x86`,
  `linux-arm`, `linux-riscv64`, `macos-amd64`, `macos-arm64` y
  `windows-amd64`.
