# Core Lua API

`ptool` 会直接在 `ptool` 和 `p` 下暴露这些核心运行时辅助能力。

`ptool run <lua_file>` 会运行 Lua 脚本，并注入全局变量 `ptool`
（以及它的别名 `p`；例如 `p.run` 与 `ptool.run` 等价）。对于以 `.lua`
结尾的文件，`ptool <lua_file>` 也是一个行为相同的 CLI 快捷写法。

内嵌的 Lua 运行时会保留基础 Lua 全局能力，并默认只暴露这些标准库：

- `table`
- `string`
- `math`
- `utf8`

像 `io`、`os`、`package` 这类面向宿主环境的内建模块会被有意禁用。
文件系统、进程、网络等运行时操作应改用 `ptool` 提供的 API。

如果你想向 Lua 脚本传参，可以这样做：

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

随后可以用 `ptool.args.parse(...)` 来解析这些参数。

示例脚本如下：

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

也支持 shebang，因此你可以把下面这行放到文件开头：

```
#!/usr/bin/env ptool
```

## ptool.use

> `v0.1.0` - 引入。

`ptool.use` 用来声明脚本所要求的最低 `ptool` 版本。

```lua
ptool.use("v0.1.0")
```

- 参数是一个语义化版本字符串（SemVer），可选带 `v` 前缀，例如 `v0.1.0` 或
  `0.1.0`。
- 如果要求的版本高于当前 `ptool` 版本，脚本会立即退出，并报错说明当前
  `ptool` 版本过旧。

## ptool.unindent

> `v0.1.0` - 引入。

`ptool.unindent` 处理多行字符串时，会在每行前导缩进之后移除 `| ` 前缀，并裁掉
首尾空白行。

```lua
local str = ptool.unindent([[
  | line 1
  | line 2
]])
```

这等价于：

```lua
local str = [[line 1
line 2]]
```

## ptool.inspect

> `v0.1.0` - 引入。

`ptool.inspect(value[, options])` 会把 Lua 值渲染成可读的 Lua 风格字符串，
主要用于调试和展示 table 内容。

- `value`（任意类型，必填）：要查看的 Lua 值。
- `options`（table，可选）：渲染选项。支持：
  - `indent`（string，可选）：每层嵌套使用的缩进。默认是两个空格。
  - `multiline`（boolean，可选）：table 是否跨多行渲染。默认值为 `true`。
  - `max_depth`（integer，可选）：最大渲染深度。更深的值会被替换为
    `<max-depth>`。
- 返回：`string`。

行为说明：

- 类数组条目（`1..n`）会优先渲染。
- 其余 table 字段会在数组部分之后，按稳定键顺序渲染。
- 类标识符的字符串键会渲染为 `key = value`；其他键会渲染为 `[key] = value`。
- 递归 table 引用会渲染为 `<cycle>`。
- function、thread 和 userdata 会渲染为 `<function>`、`<userdata>` 等占位值。

示例：

```lua
local value = {
  "hello",
  user = { name = "alice", tags = {"dev", "ops"} },
}
value.self = value

print(ptool.inspect(value))
print(ptool.inspect(value, { multiline = false }))
```

## ptool.ask

> `v0.1.0` - 引入。

`ptool.ask(prompt[, options])` 向用户询问一行文本，并返回用户输入。

- `prompt`（string，必填）：展示给用户的提示语。
- `options`（table，可选）：提示选项。支持：
  - `default`（string，可选）：用户提交空输入时采用的默认值。
  - `help`（string，可选）：显示在提示下方的额外帮助文本。
  - `placeholder`（string，可选）：用户开始输入前显示的占位文本。
- 返回：`string`。

行为说明：

- 需要交互式 TTY。在非交互环境中运行会抛出错误。
- 如果用户取消输入，脚本会抛出错误。
- 未知选项名或非法选项值类型都会抛出错误。

示例：

```lua
local name = ptool.ask("Your name?", {
  placeholder = "Alice",
  help = "Press Enter to confirm",
})

local city = ptool.ask("City?", {
  default = "Shanghai",
})

print(string.format("Hello, %s from %s!", name, city))
```

## ptool.config

> `v0.1.0` - 引入。

`ptool.config` 用来设置脚本的运行时配置。

当前支持的字段：

- `run`（table，可选）：`ptool.run` 的默认配置。支持：
  - `echo`（boolean，可选）：默认是否回显。默认值为 `true`。
  - `check`（boolean，可选）：默认在失败时是否抛错。默认值为 `false`。
  - `confirm`（boolean，可选）：默认执行前是否要求确认。默认值为 `false`。
  - `retry`（boolean，可选）：当 `check = true` 时，执行失败后是否询问用户是否重试。
    默认值为 `false`。

示例：

```lua
ptool.config({
  run = {
    echo = false,
    check = true,
    confirm = false,
    retry = false,
  },
})
```

## ptool.cd

> `v0.1.0` - 引入。

`ptool.cd(path)` 更新 `ptool` 的运行时当前目录。

- `path`（string，必填）：目标目录路径，可以是绝对路径或相对路径。

行为说明：

- 相对路径会从当前 `ptool` 运行时目录解析。
- 目标必须存在，而且必须是目录。
- 这会更新 `ptool` 的运行时状态，并影响依赖运行时 cwd 的 API
  （例如 `ptool.run`、`ptool.path.abspath` 和 `ptool.path.relpath`）。

示例：

```lua
ptool.cd("foobar")
local res = ptool.run({ cmd = "pwd", stdout = "capture" })
print(res.stdout)
```

## ptool.script_path

> `v0.4.0` - 引入。

`ptool.script_path()` 返回当前入口脚本的绝对路径。

- 返回：`string|nil`。

行为说明：

- 通过 `ptool run <file>` 运行时，它会返回入口脚本的绝对规范化路径。
- 返回值会在 runtime 启动时固定下来，之后不会随着 `ptool.cd(...)` 改变。
- 在 `ptool repl` 中，它会返回 `nil`。

示例：

```lua
local script_path = ptool.script_path()
local script_dir = ptool.path.dirname(script_path)
local project_root = ptool.path.dirname(script_dir)
```

## ptool.try

> `v0.4.0` - 引入。

`ptool.try(fn)` 会执行 `fn`，并把抛出的错误转换成返回值。

- `fn`（function，必填）：要执行的回调。
- 返回：`ok, value, err`。

返回值规则：

- 成功时，`ok = true`，`err = nil`，`value` 是回调返回结果。
- 如果回调没有返回值，`value` 为 `nil`。
- 如果回调只返回一个值，`value` 就是该值本身。
- 如果回调返回多个值，`value` 是一个类数组 table。
- 失败时，`ok = false`，`value = nil`，`err` 是一个 table。

结构化错误字段：

- `kind`（string）：稳定的错误类别，例如 `io_error`、`command_failed`、
  `invalid_argument`、`http_error` 或 `lua_error`。
- `message`（string）：便于阅读的错误消息。
- `op`（string，可选）：发生错误的 API 或操作名，例如 `ptool.fs.read`。
- `detail`（string，可选）：额外的失败细节。
- `path`（string，可选）：文件系统错误涉及的路径。
- `input`（string，可选）：解析或校验失败时的原始输入。
- `cmd`（string，可选）：命令执行失败时的命令名。
- `status`（integer，可选）：退出码或 HTTP 状态码（如果可用）。
- `stderr`（string，可选）：命令失败时捕获到的 stderr。
- `url`（string，可选）：HTTP 失败涉及的 URL。
- `cwd`（string，可选）：命令执行时实际使用的工作目录。
- `target`（string，可选）：SSH 相关命令失败时的目标主机。
- `retryable`（boolean）：是否适合重试。默认值为 `false`。

行为说明：

- `ptool` 自带 API 抛出的都是结构化错误。`ptool.try` 会把它们转换成上面的
  `err` table，方便调用方根据 `err.kind` 和其他字段做分支处理。
- 普通 Lua 错误也会被捕获。这种情况下，`err.kind` 为 `lua_error`，并且只保证
  `message` 一定存在。
- 对于 `ptool.fs.read`、`ptool.http.request`、`ptool.run(..., { check = true })`
  和 `res:assert_ok()` 这类 API，推荐使用 `ptool.try` 来做错误处理。

示例：

```lua
local ok, content, err = ptool.try(function()
  return ptool.fs.read("missing.txt")
end)

if not ok and err.kind == "io_error" then
  print(err.op, err.path)
end

local ok2, _, err2 = ptool.try(function()
  local res = ptool.run({
    cmd = "sh",
    args = {"-c", "echo bad >&2; exit 7"},
    stderr = "capture",
  })
  res:assert_ok()
end)

if not ok2 and err2.kind == "command_failed" then
  print(err2.cmd, err2.status, err2.stderr)
end
```

## ptool.run

> `v0.1.0` - 引入。

`ptool.run` 从 Rust 执行外部命令。

支持以下调用形式：

```lua
ptool.run("echo hello world")
ptool.run("echo", "hello world")
ptool.run("echo", {"hello", "world"})
ptool.run("echo hello world", { echo = true })
ptool.run("echo", {"hello", "world"}, { echo = true })
ptool.run({ cmd = "echo", args = {"hello", "world"} })
ptool.run({ cmd = "echo", args = {"hello"}, stdout = "capture" })
```

参数规则：

- `ptool.run(cmdline)`：`cmdline` 会按 shell 风格（`shlex`）规则拆分。第一项
  作为命令，其余项作为参数。
- `ptool.run(cmd, argsline)`：`cmd` 直接作为命令，`argsline` 会按 shell 风格
  （`shlex`）规则拆分成参数列表。
- `ptool.run(cmd, args)`：`cmd` 是字符串，`args` 是字符串数组。
- `ptool.run(cmdline, options)`：`options` 会覆盖本次调用的配置，例如 `echo`。
- `ptool.run(cmd, args, options)`：`args` 可以是字符串，也可以是字符串数组；
  `options` 会覆盖本次调用的配置，例如 `echo`。
- `ptool.run(options)`：`options` 是一个 table。
- 当第二个参数是 table 时：如果它是数组（连续整数键 `1..n`），则视为 `args`；
  否则视为 `options`。

返回值规则：

- 总是返回一个 table，包含以下字段：
  - `ok`（boolean）：退出码是否为 `0`。
  - `code`（integer|nil）：进程退出码。如果进程因信号终止，则为 `nil`。
  - `cmd`（string）：本次执行使用的命令名。
  - `cwd`（string）：本次执行实际使用的工作目录。
  - `stdout`（string，可选）：当 `stdout = "capture"` 时提供。
  - `stderr`（string，可选）：当 `stderr = "capture"` 时提供。
  - `assert_ok(self)`（function）：当 `ok = false` 时抛出结构化错误。错误类别
    为 `command_failed`，并且可能包含 `cmd`、`status`、`stderr` 和 `cwd`。
- `check` 的默认值来自 `ptool.config({ run = { check = ... } })`。如果未配置，
  默认是 `false`。当 `check = false` 时，调用方可以自行检查 `ok`，或者调用
  `res:assert_ok()`。
- 当同时设置 `check = true` 和 `retry = true` 时，`ptool.run` 会在最终抛错前，
  询问是否要重试失败的命令。
- 当 `check = true` 时，`ptool.run` 抛出的也是与 `res:assert_ok()` 相同的
  `command_failed` 结构化错误。如果你想在 Lua 里捕获并检查它，使用
  `ptool.try(...)`。

示例：

```lua
ptool.config({ run = { echo = false } })

ptool.run("echo from ptool")
ptool.run("echo", "from ptool")
ptool.run("echo", {"from", "ptool"})
ptool.run("echo from ptool", { echo = true })
ptool.run("echo", {"from", "ptool"}, { echo = true })
ptool.run("pwd")

local res = ptool.run({
  cmd = "sh",
  args = {"-c", "echo bad >&2; exit 7"},
  stderr = "capture",
})
print(res.ok, res.code)
res:assert_ok()
```

也支持 `ptool.run(options)` 形式，其中 `options` 是一个包含以下字段的 table：

- `cmd`（string，必填）：命令名或可执行文件路径。
- `args`（string[]，可选）：参数列表。
- `cwd`（string，可选）：子进程工作目录。
- `env`（table，可选）：附加环境变量，键和值都应为字符串。
- `echo`（boolean，可选）：本次执行是否回显命令信息。如果省略，则使用
  `ptool.config({ run = { echo = ... } })` 中的值；若仍未设置，则默认是 `true`。
- `check`（boolean，可选）：退出码不为 `0` 时是否立即抛错。如果省略，则使用
  `ptool.config({ run = { check = ... } })` 中的值；若仍未设置，则默认是 `false`。
- `confirm`（boolean，可选）：执行前是否询问用户确认。如果省略，则使用
  `ptool.config({ run = { confirm = ... } })` 中的值；若仍未设置，则默认是 `false`。
- `retry`（boolean，可选）：当 `check = true` 时，执行失败后是否询问用户是否重试。
  如果省略，则使用 `ptool.config({ run = { retry = ... } })` 中的值；若仍未设置，
  则默认是 `false`。
- `stdout`（string，可选）：stdout 处理策略。支持：
  - `"inherit"`：继承到当前终端（默认）。
  - `"capture"`：捕获到 `res.stdout`。
  - `"null"`：丢弃输出。
- `stderr`（string，可选）：stderr 处理策略。支持：
  - `"inherit"`：继承到当前终端（默认）。
  - `"capture"`：捕获到 `res.stderr`。
  - `"null"`：丢弃输出。
- 当 `confirm = true` 时：
  - 如果用户拒绝执行，会立即抛出错误。
  - 如果当前环境不是交互式环境（无 TTY），也会立即抛出错误。
- 当 `retry = true` 且 `check = true` 时：
  - 如果命令执行失败，`ptool.run` 会询问是否重试相同命令。
  - 如果当前环境不是交互式环境（无 TTY），则不会弹出重试提示，而是直接抛错。

示例：

```lua
ptool.run({
  cmd = "echo",
  args = {"hello"},
  env = { FOO = "bar" },
})

local res = ptool.run({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2; exit 7"},
  stdout = "capture",
  stderr = "capture",
  check = false,
})
print(res.ok, res.code)
print(res.stdout)
print(res.stderr)
res:assert_ok()
```

## ptool.run_capture

> `Unreleased` - 引入。

`ptool.run_capture` 从 Rust 执行外部命令，调用形式、参数规则、返回值规则和选项
都与 `ptool.run` 相同。

唯一差异是默认流处理方式：

- `stdout` 默认是 `"capture"`。
- `stderr` 默认是 `"capture"`。

你仍然可以在 `options` 中显式覆盖任意一个字段。

示例：

```lua
local res = ptool.run_capture("echo hello world")
print(res.stdout)

local res2 = ptool.run_capture({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2"},
})
print(res2.stdout)
print(res2.stderr)

local res3 = ptool.run_capture("echo hello", {
  stderr = "inherit",
})
print(res3.stdout)
```
