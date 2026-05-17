# OS API

`ptool.os` 提供了一组用于读取当前运行时环境、查询宿主进程基础信息的辅助能力。

## ptool.os.getenv

> `v0.4.0` - 引入。

`ptool.os.getenv(name)` 返回某个环境变量的当前值。

- `name`（string，必填）：环境变量名。
- 返回：`string|nil`。

行为说明：

- 当变量未设置时返回 `nil`。
- 读取当前 `ptool` 运行时环境，其中也包括通过 `ptool.os.setenv(...)` 和 `ptool.os.unsetenv(...)` 修改过的值。
- 当 `name` 为空，或包含 `=` 之类的非法字符时会抛出错误。

示例：

```lua
local home = p.os.getenv("HOME")
print(home)
```

## ptool.os.env

> `v0.4.0` - 引入。

`ptool.os.env()` 返回当前运行时环境的快照表。

- 返回：`table`。

行为说明：

- 返回的 table 会把变量名映射到字符串值。
- 通过 `ptool.os.setenv(...)` 和 `ptool.os.unsetenv(...)` 修改的值也会反映到这个快照中。

示例：

```lua
local env = p.os.env()
print(env.HOME)
```

## ptool.os.setenv

> `v0.4.0` - 引入。

`ptool.os.setenv(name, value)` 在当前 `ptool` 运行时中设置环境变量。

- `name`（string，必填）：环境变量名。
- `value`（string，必填）：环境变量值。

行为说明：

- 它修改的是当前 `ptool` 运行时环境，不是父级 shell。
- 这里设置的值会反映到 `ptool.os.getenv(...)`、`ptool.os.env()`，以及之后通过 `ptool.run(...)` 启动的子进程中。
- 当 `name` 为空、包含 `=`，或 `value` 包含 NUL 时会抛出错误。

示例：

```lua
p.os.setenv("APP_ENV", "dev")
print(p.os.getenv("APP_ENV"))
```

## ptool.os.unsetenv

> `v0.4.0` - 引入。

`ptool.os.unsetenv(name)` 从当前 `ptool` 运行时中移除环境变量。

- `name`（string，必填）：环境变量名。

行为说明：

- 这会影响之后对 `ptool.os.getenv(...)`、`ptool.os.env()` 的调用，以及由 `ptool.run(...)` 启动的子进程。
- 当 `name` 为空，或包含 `=` 之类的非法字符时会抛出错误。

示例：

```lua
p.os.unsetenv("APP_ENV")
assert(p.os.getenv("APP_ENV") == nil)
```

## ptool.os.homedir

> `v0.4.0` - 引入。

`ptool.os.homedir()` 返回当前用户的 home 目录。

- 返回：`string|nil`。

示例：

```lua
local home = p.os.homedir()
```

## ptool.os.tmpdir

> `v0.4.0` - 引入。

`ptool.os.tmpdir()` 返回系统临时目录。

- 返回：`string`。

示例：

```lua
local tmp = p.os.tmpdir()
```

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

示例：

```lua
print(p.os.hostname(), p.os.username(), p.os.pid())
print(p.os.exepath())
```
