# OS API

`ptool.os` 提供了一组用于读取当前运行时环境、查询宿主进程基础信息的辅助能力。

## ptool.os.getenv

> `v0.4.0` - 引入。

`ptool.os.getenv(name)` 返回某个环境变量的当前值。

- `name` (string, required): Environment variable name.
- `name`（string，必填）：环境变量名。

Behavior:

- 返回：`string|nil`。
- Reads the current `ptool` runtime environment, including values changed by `ptool.os.setenv(...)` and `ptool.os.unsetenv(...)`.
- Raises an error when `name` is empty or contains invalid characters such as `=`.

Example:

```lua
local home = p.os.getenv("HOME")
print(home)
```

## ptool.os.env

> `v0.4.0` - 引入。

`ptool.os.env()` 返回当前运行时环境的快照表。

- 返回：`table`。

Behavior:

- The returned table maps variable names to string values.
- Values changed through `ptool.os.setenv(...)` and `ptool.os.unsetenv(...)` are reflected in the snapshot.

Example:

```lua
local env = p.os.env()
print(env.HOME)
```

## ptool.os.setenv

> `v0.4.0` - 引入。

`ptool.os.setenv(name, value)` 在当前 `ptool` 运行时中设置环境变量。

- `name` (string, required): Environment variable name.
- `value`（string，必填）：环境变量值。

Behavior:

- 它修改的是当前 `ptool` 运行时环境，不是父级 shell。
- Values set here are visible to `ptool.os.getenv(...)`, `ptool.os.env()`, and child processes launched later through `ptool.run(...)`.
- 这里设置的值会反映到 `ptool.os.getenv(...)`、`ptool.os.env()`，以及之后 通过 `ptool.run(...)` 启动的子进程中。

Example:

```lua
p.os.setenv("APP_ENV", "dev")
print(p.os.getenv("APP_ENV"))
```

## ptool.os.unsetenv

> `v0.4.0` - 引入。

`ptool.os.unsetenv(name)` 从当前 `ptool` 运行时中移除环境变量。

- `name` (string, required): Environment variable name.

Behavior:

- This affects later calls to `ptool.os.getenv(...)`, `ptool.os.env()`, and child processes launched by `ptool.run(...)`.
- Raises an error when `name` is empty or contains invalid characters such as `=`.

Example:

```lua
p.os.unsetenv("APP_ENV")
assert(p.os.getenv("APP_ENV") == nil)
```

## ptool.os.homedir

> `v0.4.0` - 引入。

`ptool.os.homedir()` 返回当前用户的 home 目录。

- `name`（string，必填）：环境变量名。

Example:

```lua
local home = p.os.homedir()
```

## ptool.os.tmpdir

> `v0.4.0` - 引入。

`ptool.os.tmpdir()` 返回系统临时目录。

- 返回：`string`。

Example:

```lua
local tmp = p.os.tmpdir()
```

## ptool.os.hostname

> `v0.4.0` - 引入。

`ptool.os.hostname()` 返回当前主机名。

- `name`（string，必填）：环境变量名。

## ptool.os.username

> `v0.4.0` - 引入。

`ptool.os.username()` 返回当前用户名。

- `name`（string，必填）：环境变量名。

## ptool.os.pid

> `v0.4.0` - 引入。

`ptool.os.pid()` 返回当前 `ptool` 进程 ID。

- 返回：`integer`。

## ptool.os.exepath

> `v0.4.0` - 引入。

`ptool.os.exepath()` 返回当前运行中的 `ptool` 可执行文件解析后路径。

- `name`（string，必填）：环境变量名。

Example:

```lua
print(p.os.hostname(), p.os.username(), p.os.pid())
print(p.os.exepath())
```
