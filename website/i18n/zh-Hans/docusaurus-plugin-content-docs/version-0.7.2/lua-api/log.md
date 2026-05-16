# Log API

`ptool.log` 在 `ptool.log` 和 `p.log` 下提供按 level 分类的终端日志辅助能力。

每次日志调用都会输出一行如下格式的文本：

```text
[2026-04-30 14:54:56] INFO message text
```

行为：

- 时间戳使用本地时间，格式为 `YYYY-MM-DD HH:MM:SS`。
- level 标签使用完整名称：`TRACE`、`DEBUG`、`INFO`、`WARN`、`ERROR`。
- 当 `ptool` 正在向终端写入时，会自动启用彩色输出。
- `ptool.log.error(...)` 写入 `stderr`，其余 level 写入 `stdout`。
- 多个参数会用空格连接；非字符串值会以单行 inspect 形式渲染。

## ptool.log.trace

> `v0.4.0` - 引入。

```lua
ptool.log.trace(...)
```

输出一条 trace 级别日志。

## ptool.log.debug

> `v0.4.0` - 引入。

```lua
ptool.log.debug(...)
```

输出一条 debug 级别日志。

## ptool.log.info

> `v0.4.0` - 引入。

```lua
ptool.log.info(...)
```

输出一条 info 级别日志。

## ptool.log.warn

> `v0.4.0` - 引入。

```lua
ptool.log.warn(...)
```

输出一条 warn 级别日志。

## ptool.log.error

> `v0.4.0` - 引入。

```lua
ptool.log.error(...)
```

向 `stderr` 输出一条 error 级别日志。

示例：

```lua
p.log.info("hello", { answer = 42 })
p.log.warn("careful")
p.log.error("boom")
```

示例输出：

```text
[2026-04-30 14:54:56] INFO hello { answer = 42 }
[2026-04-30 14:54:56] WARN careful
[2026-04-30 14:54:56] ERROR boom
```
