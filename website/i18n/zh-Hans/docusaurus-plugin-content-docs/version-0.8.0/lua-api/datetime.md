# DateTime API

日期和时间辅助能力位于 `ptool.datetime` 和 `p.datetime` 下。

`ptool.datetime` 处理的是具体时间点。每个 `DateTime` 值都会携带一个时区或数值偏移。

## ptool.datetime.now

> `v0.6.0` - 引入。

`ptool.datetime.now([tz])` 返回当前时间，并以 `DateTime` 表示。

- `tz`（string，可选）：IANA 时区，例如 `UTC`、`America/New_York` 或 `Asia/Shanghai`。省略时使用本地系统时区。
- 返回：`DateTime`。

```lua
local local_now = p.datetime.now()
local utc_now = p.datetime.now("UTC")

print(local_now)
print(utc_now:format("%Y-%m-%d %H:%M:%S %Z"))
```

## ptool.datetime.parse

> `v0.6.0` - 引入。

`ptool.datetime.parse(input[, options])` 解析一个日期时间字符串，并返回 `DateTime`。

- `input`（string，必填）：日期时间字符串。
- `options.timezone`（string，可选）：仅当输入本身未包含时区或偏移时，才会使用的 IANA 时区。
- 返回：`DateTime`。

可接受的输入：

- 带时区的输入，例如 `2024-07-15T16:24:59-04:00`。
- 在解析器支持时，带方括号时区注解的带时区输入。
- 不带时区的朴素输入，例如 `2024-07-15 16:24:59`，但仅当提供了 `options.timezone` 时才允许。

行为说明：

- 空字符串会被拒绝。
- 如果 `input` 已经包含时区或偏移，再设置 `options.timezone` 会报错。
- 未提供 `options.timezone` 时，朴素输入会被拒绝。

```lua
local a = p.datetime.parse("2024-07-15T16:24:59-04:00")
local b = p.datetime.parse("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})

print(a.offset)   -- -04:00
print(b.timezone) -- America/New_York
```

## ptool.datetime.from_unix

> `v0.6.0` - 引入。

`ptool.datetime.from_unix(value[, options])` 从 Unix 时间戳构造一个 `DateTime`。

- `value`（integer，必填）：Unix 时间戳。
- `options.unit`（string，可选）：`s`、`ms` 或 `ns` 之一。默认值为 `s`。
- `options.timezone`（string，可选）：IANA 时区。省略时，时间戳会按 `UTC` 解释。
- 返回：`DateTime`。

```lua
local a = p.datetime.from_unix(1721075099)
local b = p.datetime.from_unix(1721075099000, {
  unit = "ms",
  timezone = "Asia/Tokyo",
})

print(a) -- 2024-07-15T20:24:59+00:00
print(b)
```

## ptool.datetime.compare

> `v0.6.0` - 引入。

`ptool.datetime.compare(a, b)` 比较两个时间点。

- `a` / `b`（string|DateTime，必填）：日期时间字符串或 `DateTime` 对象。
- 返回：`-1 | 0 | 1`。

字符串参数会按 `ptool.datetime.parse(input)` 相同的严格规则解析，因此它们必须已经包含时区或偏移。

```lua
print(ptool.datetime.compare(
  "2024-07-15T20:24:59+00:00",
  "2024-07-15T16:24:59-04:00"
)) -- 0
```

## ptool.datetime.is_valid

> `v0.6.0` - 引入。

`ptool.datetime.is_valid(input[, options])` 检查某个日期时间字符串是否可被解析。

- `input`（string，必填）：日期时间字符串。
- `options.timezone`（string，可选）：用于朴素输入的 IANA 时区。
- 返回：`boolean`。

```lua
print(ptool.datetime.is_valid("2024-07-15T16:24:59-04:00")) -- true
print(ptool.datetime.is_valid("2024-07-15 16:24:59")) -- false
print(ptool.datetime.is_valid("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})) -- true
```

## DateTime

> `v0.6.0` - 引入。

`DateTime` 表示一个由 `ptool.datetime.now(...)`、`parse(...)` 或 `from_unix(...)` 返回的具体时间点。

它实现为 Lua userdata。

字段和方法：

- 字段：
  - `year`（integer）
  - `month`（integer）
  - `day`（integer）
  - `hour`（integer）
  - `minute`（integer）
  - `second`（integer）
  - `nanosecond`（integer）
  - `offset`（string）
  - `timezone`（string）
- 方法：
  - `dt:format(fmt)` -> `string`
  - `dt:to_string()` -> `string`
  - `dt:unix([unit])` -> `integer`
  - `dt:in_tz(tz)` -> `DateTime`
  - `dt:compare(other)` -> `-1|0|1`
- 元方法：
  - 支持 `tostring(dt)`。
  - 支持 `==`、`<` 和 `<=` 比较。

### format

规范 API 名称：`ptool.datetime.DateTime:format`。

`dt:format(fmt)` 使用 `strftime` 风格的指令格式化日期时间。

- `fmt`（string，必填）：格式字符串，例如 `%Y-%m-%d %H:%M:%S %Z`。
- 返回：`string`。

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:format("%Y-%m-%d %H:%M:%S %:z"))
```

### to_string

规范 API 名称：`ptool.datetime.DateTime:to_string`。

`dt:to_string()` 返回带数值偏移的规范字符串形式。

- 返回：`string`。

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:to_string()) -- 2024-07-15T16:24:59-04:00
```

### unix

规范 API 名称：`ptool.datetime.DateTime:unix`。

`dt:unix([unit])` 返回该时间点的 Unix 时间戳。

- `unit`（string，可选）：`s`、`ms` 或 `ns` 之一。默认值为 `s`。
- 返回：`integer`。

说明：

- 当结果无法放进 Lua 整数范围时，`ns` 可能会报错。

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
print(dt:unix())       -- seconds
print(dt:unix("ms"))   -- milliseconds
```

### in_tz

规范 API 名称：`ptool.datetime.DateTime:in_tz`。

`dt:in_tz(tz)` 把同一个时间点转换到另一个时区中表示。

- `tz`（string，必填）：IANA 时区。
- 返回：`DateTime`。

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
local tokyo = dt:in_tz("Asia/Tokyo")

print(dt)
print(tokyo)
```

### compare

规范 API 名称：`ptool.datetime.DateTime:compare`。

`dt:compare(other)` 将当前时间点与 `other` 进行比较。

- `other`（string|DateTime，必填）：日期时间字符串或另一个 `DateTime` 对象。
- 返回：`-1 | 0 | 1`。

```lua
local a = p.datetime.parse("2024-07-15T20:24:59+00:00")
local b = p.datetime.parse("2024-07-15T21:24:59+00:00")

print(a:compare(b)) -- -1
print(a < b)        -- true
```

## 说明

- `ptool.datetime` 不会解析像 `"tomorrow 8am"` 这样的自然语言短语。
- 时区名称应使用 IANA 标识符，例如 `UTC`、`Asia/Tokyo` 或 `America/New_York`。
- 比较针对的是时间点本身，而不是显示出来的本地时间字段。
