# Shell API

Shell 解析辅助能力位于 `ptool.sh` 和 `p.sh` 下。

这些辅助能力工作在 shell word 这一层级。它们用于按照 POSIX shell 风格规则拆分、引用和拼接参数字符串，而不是用于解析完整的 shell 语法，例如管道、重定向、命令替换或变量展开。

## ptool.sh.split

> `v0.1.0` - 引入。

`ptool.sh.split(command)` 使用 shell 风格规则解析命令字符串，并返回参数数组。

- `command`（string，必填）：要拆分的命令字符串。
- 返回：`string[]`。

行为说明：

- 这里只解析 shell words。本 API 不会解释 shell 操作符，也不会执行展开。

示例：

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

上面的 `args` 等价于：

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```

## ptool.sh.quote

> `Unreleased` - 引入。

`ptool.sh.quote(word)` 会对单个 shell word 进行引用，使其可以安全地嵌入到 shell 命令字符串中。

- `word`（string，必填）：要引用的 shell word。
- 返回：`string`。

行为说明：

- 返回的字符串对 shell 是安全的，并且在语义上等价于输入的 word。
- 这里保留的是 shell word 的语义，而不是原始文本的书写形式。

示例：

```lua
local word = ptool.sh.quote("hello world")
print(word) -- 'hello world'
```

## ptool.sh.join

> `Unreleased` - 引入。

`ptool.sh.join(words)` 会把参数数组拼成一个 shell 命令字符串，并在需要时对其中的 word 进行引用。

- `words`（string[]，必填）：要拼接的 shell words。
- 返回：`string`。

行为说明：

- 相邻 words 会使用单个空格连接。
- 输出结果适合传递给 POSIX 风格的 shell。
- 这里追求 shell word 级别的往返一致性，因此 `ptool.sh.split(ptool.sh.join(words))` 与 `words` 等价。
- `ptool.sh.join(ptool.sh.split(command))` 可能会规范化引用方式和空格，而不是保留原始命令文本。

示例：

```lua
local cmd = ptool.sh.join({"git", "commit", "-m", "hello world"})
print(cmd) -- git commit -m 'hello world'
```
