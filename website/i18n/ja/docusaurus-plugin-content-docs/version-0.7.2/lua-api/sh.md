# シェル API

シェル解析ヘルパーは `ptool.sh` と `p.sh` にあります。

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split(command)` はシェル風ルールでコマンド文字列を解析し、 引数配列を返します。

- `command` (string, 必須): 分割するコマンド文字列。
- 戻り値: `string[]`。

例:

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

上の `args` は次と等価です。

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```
