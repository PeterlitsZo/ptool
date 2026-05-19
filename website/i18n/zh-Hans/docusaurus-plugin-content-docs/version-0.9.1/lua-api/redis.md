# Redis API

Redis 连接辅助能力位于 `ptool.redis` 和 `p.redis` 下。

## ptool.redis.connect

> `v0.9.0` - 引入。

`ptool.redis.connect(url_or_options)` 打开 Redis 连接，并返回一个 `Connection` 对象。

参数：

- `url_or_options`（string|table，必填）：
  - 传入字符串时，会被视为 Redis URL。
  - 传入 table 时，目前支持：
    - `url`（string，必填）：Redis URL。

常见 URL 示例：

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")
local db1 = ptool.redis.connect("redis://127.0.0.1/1")
local auth = ptool.redis.connect({
  url = "redis://:secret@cache.internal:6379/0",
})
```

说明：

- 该连接主要用于通过 `conn:call(...)` 直接执行命令。
- 该 API 目前不提供专门的 pub/sub、pipeline 或事务辅助方法。

## Connection

> `v0.9.0` - 引入。

`Connection` 表示由 `ptool.redis.connect()` 返回的已打开 Redis 连接。

它实现为 Lua userdata。

方法：

- `conn:call(command, ...)` -> `any`
- `conn:close()` -> `nil`

命令参数规则：

- `command` 必须是非空字符串。
- 其余参数会作为扁平参数列表传给 Redis。
- 支持的参数值类型包括：
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil`、table、function、thread 和 userdata 不能作为 Redis 命令参数。
- Lua 字符串会按原始字节传递，因此支持二进制安全的 Redis 值。

回复值转换规则：

- 空回复会转换为 `nil`。
- 整数回复会转换为 Lua 整数。
- bulk string、simple string 和 verbatim string 会转换为 Lua 字符串。
- array 和 set 回复会转换为致密 Lua 数组表。
- map 回复会在其键可以表示为 Lua 字符串、整数、数字或布尔值时转换为 Lua table。
- double 回复会转换为 Lua 数字。
- 布尔回复会转换为 Lua 布尔值。
- big number 回复会转换为 Lua 字符串。
- push 回复会转换为形如 `{ kind = "...", data = {...} }` 的 table。

### call

> `v0.9.0` - 引入。

规范 API 名称：`ptool.redis.Connection:call`。

`conn:call(command, ...)` 发送 Redis 命令，并返回转换后的回复值。

示例：

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")

redis:call("SET", "user:1:name", "alice")
print(redis:call("GET", "user:1:name")) -- alice

local count = redis:call("INCR", "counters:signup")
print(count)

local values = redis:call("MGET", "user:1:name", "missing")
print(values[1])        -- alice
print(values[2] == nil) -- true
```

### close

> `v0.9.0` - 引入。

规范 API 名称：`ptool.redis.Connection:close`。

`conn:close()` 关闭该连接。

行为：

- 关闭后，该连接不能再使用。

