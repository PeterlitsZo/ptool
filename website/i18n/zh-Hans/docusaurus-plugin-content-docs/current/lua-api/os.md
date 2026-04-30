# OS API

`ptool.os` 提供了一组用于读取当前运行时环境、查询宿主进程基础信息的辅助能力。

## ptool.os.getenv

> `v0.4.0` - 引入。

`ptool.os.getenv(name)` 返回某个环境变量的当前值。

- `name`（string，必填）：环境变量名。
- 返回：`string|nil`。

## ptool.os.env

> `v0.4.0` - 引入。

`ptool.os.env()` 返回当前运行时环境的快照表。

- 返回：`table`。

## ptool.os.setenv

> `v0.4.0` - 引入。

`ptool.os.setenv(name, value)` 在当前 `ptool` 运行时中设置环境变量。

- `name`（string，必填）：环境变量名。
- `value`（string，必填）：环境变量值。

行为：

- 它修改的是当前 `ptool` 运行时环境，不是父级 shell。
- 这里设置的值会反映到 `ptool.os.getenv(...)`、`ptool.os.env()`，以及之后
  通过 `ptool.run(...)` 启动的子进程中。

## ptool.os.unsetenv

> `v0.4.0` - 引入。

`ptool.os.unsetenv(name)` 从当前 `ptool` 运行时中移除环境变量。

- `name`（string，必填）：环境变量名。

## ptool.os.homedir

> `v0.4.0` - 引入。

`ptool.os.homedir()` 返回当前用户的 home 目录。

- 返回：`string|nil`。

## ptool.os.tmpdir

> `v0.4.0` - 引入。

`ptool.os.tmpdir()` 返回系统临时目录。

- 返回：`string`。

## ptool.os.hostname

> `v0.4.0` - 引入。

`ptool.os.hostname()` 返回当前主机名。

- 返回：`string|nil`。

## ptool.os.username

> `v0.4.0` - 引入。

`ptool.os.username()` 返回当前用户名。

- 返回：`string|nil`。

## ptool.os.pid

> `v0.4.0` - 引入。

`ptool.os.pid()` 返回当前 `ptool` 进程 ID。

- 返回：`integer`。

## ptool.os.exepath

> `v0.4.0` - 引入。

`ptool.os.exepath()` 返回当前运行中的 `ptool` 可执行文件解析后路径。

- 返回：`string|nil`。
