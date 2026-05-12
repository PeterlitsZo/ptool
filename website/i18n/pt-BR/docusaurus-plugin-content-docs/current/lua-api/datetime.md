# DateTime API

Date and time helpers are available under `ptool.datetime` and `p.datetime`.

`ptool.datetime` works with concrete instants. Every `DateTime` value carries a timezone or numeric offset.

## ptool.datetime.now

> `Unreleased` - Introduced.

`ptool.datetime.now([tz])` returns the current time as a `DateTime`.

- `tz` (string, optional): An IANA timezone such as `UTC`, `America/New_York`, or `Asia/Shanghai`. If omitted, the local system timezone is used.
- Returns: `DateTime`.

```lua
local local_now = p.datetime.now()
local utc_now = p.datetime.now("UTC")

print(local_now)
print(utc_now:format("%Y-%m-%d %H:%M:%S %Z"))
```

## ptool.datetime.parse

> `Unreleased` - Introduced.

`ptool.datetime.parse(input[, options])` parses a datetime string and returns a `DateTime`.

- `input` (string, required): A datetime string.
- `options.timezone` (string, optional): An IANA timezone used only when the input does not already include a timezone or offset.
- Returns: `DateTime`.

Accepted inputs:

- Zoned inputs such as `2024-07-15T16:24:59-04:00`.
- Zoned inputs with bracketed timezone annotations when supported by the parser.
- Naive inputs such as `2024-07-15 16:24:59`, but only when `options.timezone` is provided.

Behavior:

- Empty strings are rejected.
- If `input` already includes a timezone or offset, setting `options.timezone` raises an error.
- Without `options.timezone`, naive inputs are rejected.

```lua
local a = p.datetime.parse("2024-07-15T16:24:59-04:00")
local b = p.datetime.parse("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})

print(a.offset)   -- -04:00
print(b.timezone) -- America/New_York
```

## ptool.datetime.from_unix

> `Unreleased` - Introduced.

`ptool.datetime.from_unix(value[, options])` constructs a `DateTime` from a Unix timestamp.

- `value` (integer, required): The Unix timestamp.
- `options.unit` (string, optional): One of `s`, `ms`, or `ns`. Defaults to `s`.
- `options.timezone` (string, optional): An IANA timezone. If omitted, the timestamp is interpreted in `UTC`.
- Returns: `DateTime`.

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

> `Unreleased` - Introduced.

`ptool.datetime.compare(a, b)` compares two instants.

- `a` / `b` (string|DateTime, required): A datetime string or `DateTime` object.
- Returns: `-1 | 0 | 1`.

String arguments are parsed using the same strict rules as `ptool.datetime.parse(input)`, so they must already include a timezone or offset.

```lua
print(ptool.datetime.compare(
  "2024-07-15T20:24:59+00:00",
  "2024-07-15T16:24:59-04:00"
)) -- 0
```

## ptool.datetime.is_valid

> `Unreleased` - Introduced.

`ptool.datetime.is_valid(input[, options])` checks whether a datetime string can be parsed.

- `input` (string, required): A datetime string.
- `options.timezone` (string, optional): An IANA timezone for naive input.
- Returns: `boolean`.

```lua
print(ptool.datetime.is_valid("2024-07-15T16:24:59-04:00")) -- true
print(ptool.datetime.is_valid("2024-07-15 16:24:59")) -- false
print(ptool.datetime.is_valid("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})) -- true
```

## DateTime

> `Unreleased` - Introduced.

`DateTime` represents a concrete instant returned by `ptool.datetime.now(...)`, `parse(...)`, or `from_unix(...)`.

It is implemented as a Lua userdata.

Fields and methods:

- Fields:
  - `year` (integer)
  - `month` (integer)
  - `day` (integer)
  - `hour` (integer)
  - `minute` (integer)
  - `second` (integer)
  - `nanosecond` (integer)
  - `offset` (string)
  - `timezone` (string)
- Methods:
  - `dt:format(fmt)` -> `string`
  - `dt:to_string()` -> `string`
  - `dt:unix([unit])` -> `integer`
  - `dt:in_tz(tz)` -> `DateTime`
  - `dt:compare(other)` -> `-1|0|1`
- Metamethods:
  - `tostring(dt)` is available.
  - `==`, `<`, and `<=` comparisons are supported.

### format

Canonical API name: `ptool.datetime.DateTime:format`.

`dt:format(fmt)` formats the datetime using `strftime`-style directives.

- `fmt` (string, required): A format string such as `%Y-%m-%d %H:%M:%S %Z`.
- Returns: `string`.

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:format("%Y-%m-%d %H:%M:%S %:z"))
```

### to_string

Canonical API name: `ptool.datetime.DateTime:to_string`.

`dt:to_string()` returns the canonical string form with a numeric offset.

- Returns: `string`.

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:to_string()) -- 2024-07-15T16:24:59-04:00
```

### unix

Canonical API name: `ptool.datetime.DateTime:unix`.

`dt:unix([unit])` returns the Unix timestamp of the instant.

- `unit` (string, optional): One of `s`, `ms`, or `ns`. Defaults to `s`.
- Returns: `integer`.

Notes:

- `ns` may raise an error if the result does not fit in Lua's integer range.

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
print(dt:unix())       -- seconds
print(dt:unix("ms"))   -- milliseconds
```

### in_tz

Canonical API name: `ptool.datetime.DateTime:in_tz`.

`dt:in_tz(tz)` converts the same instant into another timezone.

- `tz` (string, required): An IANA timezone.
- Returns: `DateTime`.

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
local tokyo = dt:in_tz("Asia/Tokyo")

print(dt)
print(tokyo)
```

### compare

Canonical API name: `ptool.datetime.DateTime:compare`.

`dt:compare(other)` compares the current instant with `other`.

- `other` (string|DateTime, required): A datetime string or another `DateTime` object.
- Returns: `-1 | 0 | 1`.

```lua
local a = p.datetime.parse("2024-07-15T20:24:59+00:00")
local b = p.datetime.parse("2024-07-15T21:24:59+00:00")

print(a:compare(b)) -- -1
print(a < b)        -- true
```

## Notes

- `ptool.datetime` does not parse natural-language phrases such as `"tomorrow 8am"`.
- Timezone names should be IANA identifiers such as `UTC`, `Asia/Tokyo`, or `America/New_York`.
- Comparisons operate on the instant, not on the displayed wall-clock fields.
