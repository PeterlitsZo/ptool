# Shell API

Shell 解析辅助能力位于 `ptool.sh` 和 `p.sh` 下。

## ptool.sh.split

> `v0.1.0` - 引入。

`ptool.sh.split(command)` 使用 shell 风格规则解析命令字符串，并返回参数数组。

- `command`（string，必填）：要拆分的命令字符串。
- 返回：`string[]`。

示例：

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

上面的 `args` 等价于：

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```
