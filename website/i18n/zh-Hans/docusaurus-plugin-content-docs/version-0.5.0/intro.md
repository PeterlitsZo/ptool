# 快速开始

`ptool` 用来运行 Lua 脚本，并注入一套面向实用自动化场景的标准库。

目前的主要入口是：

```sh
ptool run <file>
```

对于 `.lua` 文件，也可以使用快捷形式：

```sh
ptool <file.lua>
```

如果你想交互式试用 Lua 表达式或 `ptool` API，也可以使用：

```sh
ptool repl
```

脚本运行时，`ptool` 会通过全局表 `ptool` 以及更短的别名 `p` 暴露 API。

## 安装

在 Linux 和 macOS 上，可以使用发布安装脚本安装 `ptool`：

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash
```

安装脚本会下载当前平台最新的预编译发布包，将 `ptool` 安装到
`~/.local/bin/ptool`，并在需要时打印 PATH 提示。

如果你想安装某个指定的发布标签，而不是最新稳定版：

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- v0.2.0
```

## 最小脚本

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

`ptool.use(...)` 用来声明脚本要求的最低 `ptool` 版本。
这样可以明确脚本依赖的 API 版本，并在运行时版本过旧时尽早失败。
详见 [Core Lua API](./lua-api/core.md)。

运行方式：

```sh
ptool run script.lua
ptool script.lua
```

## 传递参数

你可以在脚本路径后继续传入 CLI 参数：

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

然后在脚本内部使用 `ptool.args.parse(...)` 解析这些参数。

## Shebang 脚本

`ptool` 支持 shebang 文件。配合 `.lua` 快捷形式，脚本可以这样开头：

```text
#!/usr/bin/env ptool
```

这样一来，只要脚本设置了可执行位，就可以直接执行。

## 你能得到什么

- 一个支持 shebang 文件的脚本运行器。
- 一个可交互使用的 REPL，方便直接试验 Lua 表达式和 `ptool` API。
- 一组面向 Lua 的辅助能力，涵盖 semver、路径、文件、TOML、正则、字符串、
  HTTP、SSH、数据库和模板。
- 一组面向命令行场景的辅助能力，用于执行命令、解析参数和进行交互式输入。

## 下一步

- 查看 [REPL](./repl.md) 了解交互式使用方式、多行输入和键盘行为。
- 使用 [Lua API 概览](./lua-api/index.md) 浏览核心 API 和可用模块。
- 从 [Core Lua API](./lua-api/core.md) 开始了解版本门禁、进程执行、配置和脚本
  生命周期辅助能力。
- 当你需要某个具体能力的详细参考时，可以直接打开对应模块页面，例如
  [Args API](./lua-api/args.md)。
