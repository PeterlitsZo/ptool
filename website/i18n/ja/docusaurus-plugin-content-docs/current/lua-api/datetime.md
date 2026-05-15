# 日時API

日付と時刻のヘルパーは、`ptool.datetime` および `p.datetime` で使用できます。

`ptool.datetime` は具体的なインスタントで動作します。すべての `DateTime` 値にはタイムゾーンまたは数値オフセットが含まれます。

## ptool.datetime.now

> `Unreleased` - 導入されました。

`ptool.datetime.now([tz])` は現在時刻を `DateTime` として返します。

- `tz` (文字列、オプション): `UTC`、`America/New_York`、`Asia/Shanghai` などの IANA タイムゾーン。省略した場合は、ローカル システムのタイムゾーンが使用されます。
- 戻り値: `DateTime`。

```lua
local local_now = p.datetime.now()
local utc_now = p.datetime.now("UTC")

print(local_now)
print(utc_now:format("%Y-%m-%d %H:%M:%S %Z"))
```

## ptool.datetime.parse

> `Unreleased` - 導入されました。

`ptool.datetime.parse(input[, options])` は日時文字列を解析し、`DateTime` を返します。

- `input` (文字列、必須): 日時文字列。
- `options.timezone` (文字列、オプション): 入力にタイムゾーンまたはオフセットが含まれていない場合にのみ使用される IANA タイムゾーン。
- 戻り値: `DateTime`。

受け入れられる入力:

- `2024-07-15T16:24:59-04:00` などのゾーン入力。
- パーサーでサポートされている場合、括弧で囲まれたタイムゾーン注釈を含むゾーン入力。
- `2024-07-15 16:24:59` などの単純な入力。ただし、`options.timezone` が指定された場合に限ります。

挙動:

- 空の文字列は拒否されます。
- `input` にすでにタイムゾーンまたはオフセットが含まれている場合、`options.timezone` を設定するとエラーが発生します。
- `options.timezone` がないと、単純な入力は拒否されます。

```lua
local a = p.datetime.parse("2024-07-15T16:24:59-04:00")
local b = p.datetime.parse("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})

print(a.offset)   -- -04:00
print(b.timezone) -- America/New_York
```

## ptool.datetime.from_unix

> `Unreleased` - 導入されました。

`ptool.datetime.from_unix(value[, options])` は、Unix タイムスタンプから `DateTime` を構築します。

- `value` (整数、必須): Unix タイムスタンプ。
- `options.unit` (文字列、オプション): `s`、`ms`、または `ns` のいずれか。デフォルトは`s`です。
- `options.timezone` (文字列、オプション): IANA タイムゾーン。省略した場合、タイムスタンプは `UTC` で解釈されます。
- 戻り値: `DateTime`。

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

> `Unreleased` - 導入されました。

`ptool.datetime.compare(a, b)` は 2 つの瞬間を比較します。

- `a` / `b` (文字列|DateTime、必須): 日時文字列または `DateTime` オブジェクト。
- 戻り値: `-1 | 0 | 1`.

文字列引数は `ptool.datetime.parse(input)` と同じ厳密なルールを使用して解析されるため、タイムゾーンまたはオフセットがすでに含まれている必要があります。

```lua
print(ptool.datetime.compare(
  "2024-07-15T20:24:59+00:00",
  "2024-07-15T16:24:59-04:00"
)) -- 0
```

## ptool.datetime.is_valid

> `Unreleased` - 導入されました。

`ptool.datetime.is_valid(input[, options])` は、日時文字列を解析できるかどうかをチェックします。

- `input` (文字列、必須): 日時文字列。
- `options.timezone` (文字列、オプション): 単純な入力用の IANA タイムゾーン。
- 戻り値: `boolean`。

```lua
print(ptool.datetime.is_valid("2024-07-15T16:24:59-04:00")) -- true
print(ptool.datetime.is_valid("2024-07-15 16:24:59")) -- false
print(ptool.datetime.is_valid("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})) -- true
```

## DateTime

> `Unreleased` - 導入されました。

`DateTime` は、`ptool.datetime.now(...)`、`parse(...)`、または `from_unix(...)` によって返される具体的なインスタントを表します。

これは Lua userdata として実装されています。

フィールドとメソッド:

- フィールド:
  - `year` (整数)
  - `month` (整数)
  - `day` (整数)
  - `hour` (整数)
  - `minute` (整数)
  - `second` (整数)
  - `nanosecond` (整数)
  - `offset` (文字列)
  - `timezone` (文字列)
- メソッド:
  - `dt:format(fmt)` -> `string`
  - `dt:to_string()` -> `string`
  - `dt:unix([unit])` -> `integer`
  - `dt:in_tz(tz)` -> `DateTime`
  - `dt:compare(other)` -> `-1|0|1`
- メタメソッド:
  - `tostring(dt)`が利用可能です。
  - `==`, `<`, `<=` の比較をサポートします。

### format

正規 API 名: `ptool.datetime.DateTime:format`。

`dt:format(fmt)` は、`strftime` スタイルのディレクティブを使用して日時をフォーマットします。

- `fmt` (文字列、必須): `%Y-%m-%d %H:%M:%S %Z` などの形式文字列。
- 戻り値: `string`。

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:format("%Y-%m-%d %H:%M:%S %:z"))
```

### to_string

正規 API 名: `ptool.datetime.DateTime:to_string`。

`dt:to_string()` は、数値オフセットを含む正規の文字列形式を返します。

- 戻り値: `string`。

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:to_string()) -- 2024-07-15T16:24:59-04:00
```

### unix

正規 API 名: `ptool.datetime.DateTime:unix`。

`dt:unix([unit])` は、その時点の Unix タイムスタンプを返します。

- `unit` (文字列、オプション): `s`、`ms`、または `ns` のいずれか。デフォルトは`s`です。
- 戻り値: `integer`。

注意:

- 結果が Lua の整数範囲に収まらない場合、`ns` はエラーを引き起こす可能性があります。

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
print(dt:unix())       -- seconds
print(dt:unix("ms"))   -- milliseconds
```

### in_tz

正規 API 名: `ptool.datetime.DateTime:in_tz`。

`dt:in_tz(tz)` は、同じ瞬間を別のタイムゾーンに変換します。

- `tz` (文字列、必須): IANA タイムゾーン。
- 戻り値: `DateTime`。

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
local tokyo = dt:in_tz("Asia/Tokyo")

print(dt)
print(tokyo)
```

### compare

正規 API 名: `ptool.datetime.DateTime:compare`。

`dt:compare(other)` は現在の瞬間を `other` と比較します。

- `other` (文字列|DateTime、必須): 日時文字列または別の `DateTime` オブジェクト。
- 戻り値: `-1 | 0 | 1`.

```lua
local a = p.datetime.parse("2024-07-15T20:24:59+00:00")
local b = p.datetime.parse("2024-07-15T21:24:59+00:00")

print(a:compare(b)) -- -1
print(a < b)        -- true
```

## 注意

- `ptool.datetime` は、`"tomorrow 8am"` などの自然言語フレーズを解析しません。
- タイムゾーン名は、`UTC`、`Asia/Tokyo`、`America/New_York` などの IANA 識別子である必要があります。
- 比較は、表示されている実時間フィールドではなく、瞬間に基づいて行われます。
