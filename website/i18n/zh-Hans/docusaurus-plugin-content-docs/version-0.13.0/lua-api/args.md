# Args API

CLI 参数模式定义与解析辅助能力位于 `ptool.args` 和 `p.args` 下。

## ptool.args.arg

> `v0.1.0` - 引入。

`ptool.args.arg(id, kind, options)` 创建一个参数构造器，用于 `ptool.args.parse(...).schema.args`。

- `id`（string，必填）：参数标识符。它也会作为返回表中的键。
- `kind`（string，必填）：参数类型。支持：
  - `"flag"`：布尔开关。
  - `"string"`：字符串选项。
  - `"int"`：整数选项（`i64`）。
  - `"positional"`：位置参数。
- `options`（table，可选）：与 `ptool.args.parse` 中参数表支持的可选字段相同， 例如 `long`、`short`、`help`、`required`、`multiple` 和 `default`。

构造器支持链式方法，且每个方法都会返回自身：

- `arg:long(value)`：设置长选项名。仅适用于非 `positional` 参数。
- `arg:short(value)`：设置短选项名。仅适用于非 `positional` 参数。
- `arg:help(value)`：设置帮助文本。
- `arg:required(value)`：设置参数是否必填。如果省略 `value`，默认是 `true`。
- `arg:multiple(value)`：设置参数是否可重复。如果省略 `value`，默认是 `true`。
- `arg:default(value)`：设置默认值。如果 `value = nil`，则清除默认值。

示例：

```lua
local res = ptool.args.parse({
  args = {
    ptool.args.arg("name", "string"):required(),
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("paths", "positional"):multiple(),
  }
})
```

## ptool.args.parse

> `v0.1.0` - 引入。
> 
> `v0.3.0` - 添加 `subcommands` 支持。

`ptool.args.parse(schema)` 使用 `clap` 解析脚本参数，并返回一个以 `id` 为键的表。

脚本参数来自 `ptool run <lua_file> -- ...` 中 `--` 之后的部分。

例如：

```lua
ptool.use("v0.1.0")

local res = ptool.args.parse({
    name = "test",
    about = "The test command",
    args = {
        { id = "name", kind = "string" }
    }
})

print("Hello, " .. res.name .. "!")
```

### Schema 结构

- `name`（string，可选）：命令名称，用于帮助输出。默认是脚本文件名。
- `about`（string，可选）：帮助描述。
- `args`（table，可选）：参数定义数组。每个元素支持两种形式：
  - 参数表。
  - `ptool.args.arg(...)` 返回的构造器对象。
- `subcommands`（table，可选）：子命令名到子命令 schema 的映射。每个子命令 schema 都支持递归定义 `about`、`args` 和 `subcommands`。

`args` 和 `subcommands` 至少要提供一个。

参数表字段：

- `id`（string，必填）：参数标识符。它也会作为返回表中的键。
- `kind`（string，必填）：参数类型。支持：
  - `"flag"`：布尔开关。
  - `"string"`：字符串选项。
  - `"int"`：整数选项（`i64`）。
  - `"positional"`：位置参数。
- `long`（string，可选）：长选项名，例如 `"name"` 对应 `--name`。对于非 `positional` 参数，默认可以从 `id` 推导。
- `short`（string，可选）：短选项名，单个字符，例如 `"v"` 对应 `-v`。
- `help`（string，可选）：参数帮助文本。
- `required`（boolean，可选）：参数是否必填。默认是 `false`。
- `multiple`（boolean，可选）：参数是否允许重复出现。默认是 `false`。
- `default`（string/integer，可选）：默认值。

当存在 `subcommands` 时，当前命令层级里的 `args` 会作为该命令树的共享选项， 可以写在所选子命令之前，也可以写在之后。

带子命令的示例：

```lua
local res = ptool.args.parse({
  name = "demo",
  args = {
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("config", "string"),
  },
  subcommands = {
    build = {
      args = {
        ptool.args.arg("release", "flag"),
      },
      subcommands = {
        web = {
          args = {
            ptool.args.arg("out", "string"):required(),
          },
        },
      },
    },
    clean = {
      args = {
        ptool.args.arg("all", "flag"),
      },
    },
  },
})
```

### 约束

- 下列约束同时适用于参数表形式和构造器语法。
- 非 `positional` 参数可以省略 `long` 和 `short`。如果省略 `long`，会自动使用 `id`。
- `positional` 参数不能设置 `long`、`short` 或 `default`。
- 当 `positional.multiple = true` 时，它必须是 `args` 中最后一个参数。
- `multiple = true` 只支持 `string` 和 `positional`。
- `default` 只支持 `string` 和 `int`，且不能与 `multiple = true` 同时使用。
- 当存在 `subcommands` 时，同一层 schema 中不能定义 `positional` 参数。
- 当顶层存在 `subcommands` 时，参数 id `command_path` 和 `args` 为保留名。
- 在一条实际命中的子命令路径上，祖先和后代子命令不能复用同一个参数 `id`， 因为它们会被合并到同一个 `args` 表里。

### 返回值

返回一个 Lua 表，其中键为 `id`，值类型如下：

- `flag` -> `boolean`
- `string` -> `string`（当 `multiple = true` 时为 `string[]`）
- `int` -> `integer`
- `positional` -> `string`（当 `multiple = true` 时为 `string[]`）

当不存在 `subcommands` 时，返回结构保持上述扁平形式不变。

当存在 `subcommands` 时，返回结构如下：

- 顶层 `args` 的值直接放在顶层表上。
- `command_path` -> `string[]`：命中的子命令路径，例如 `{"build", "web"}`。
- `args` -> `table`：命中的子命令路径上所有参数合并后的结果表。

例如：

```lua
{
  verbose = true,
  config = "cfg.toml",
  command_path = { "build", "web" },
  args = {
    release = true,
    out = "dist",
  },
}
```
