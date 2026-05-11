# プラットフォーム API

プラットフォーム検出ヘルパーは `ptool.platform` と `p.platform` に
あります。

## ptool.platform.os

> `v0.1.0` - Introduced.

`ptool.platform.os()` は現在のマシンのオペレーティングシステムを返します。

- 戻り値: `linux | macos | windows`。

```lua
print(ptool.platform.os()) -- macos
```

挙動:

- これは `ptool run` を実行しているローカルマシンを報告します。
- `ptool` は現在 `linux`, `macos`, `windows` を公開しています。

## ptool.platform.arch

> `v0.1.0` - Introduced.

`ptool.platform.arch()` は現在のマシンの CPU アーキテクチャを返します。

- 戻り値: `amd64 | arm64 | x86 | arm | riscv64`。

```lua
print(ptool.platform.arch()) -- arm64
```

挙動:

- `x86_64` は `amd64` として公開されます。
- `aarch64` は `arm64` として公開されます。
- `x86` や `i686` などの 32 ビット x86 系は `x86` として公開されます。
- `armv7l` などの 32 ビット ARM 系は `arm` として公開されます。
- `riscv64` は `riscv64` として公開されます。

## ptool.platform.target

> `v0.1.0` - Introduced.

`ptool.platform.target()` は現在のマシン向けの正規化されたプラットフォーム
target 文字列を返します。

- 戻り値: `string`。

```lua
local target = ptool.platform.target()
print(target) -- linux-riscv64
```

挙動:

- 結果は常に `ptool.platform.os() .. "-" .. ptool.platform.arch()` です。
- これはダウンロード用アーティファクトの選択など、プラットフォームに
  基づく分岐のために使うことを想定しています。
- よくある値には `linux-amd64`, `linux-arm64`, `linux-x86`, `linux-arm`,
  `linux-riscv64`, `macos-amd64`, `macos-arm64`, `windows-amd64` が
  含まれます。
