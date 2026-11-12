# Regex API

正则表达式辅助能力位于 `ptool.re` 和 `p.re` 下。

## ptool.re.compile

> `v0.1.0` - 引入。

`ptool.re.compile(pattern[, opts])` 编译正则表达式，并返回一个 `Regex` 对象。

- `pattern`（string，必填）：正则模式。
- `opts`（table，可选）：编译选项。目前支持：
  - `case_insensitive`（boolean，可选）：是否大小写不敏感。默认值是 `false`。

示例：

```lua
local re = ptool.re.compile("(?P<name>\\w+)", { case_insensitive = true })
print(re:is_match("Alice")) -- true
```

## ptool.re.escape

> `v0.1.0` - 引入。

`ptool.re.escape(text)` 把普通文本转义成正则字面量字符串。

- `text`（string，必填）：要转义的文本。
- 返回：转义后的字符串。

示例：

```lua
local keyword = "a+b?"
local re = ptool.re.compile("^" .. ptool.re.escape(keyword) .. "$")
print(re:is_match("a+b?")) -- true
```

## Regex

> `v0.1.0` - 引入。

`Regex` 表示由 `ptool.re.compile(...)` 返回的已编译正则表达式。

它实现为 Lua userdata。

方法：

- `re:is_match(input)` -> `boolean`
- `re:find(input[, init])` -> `Match|nil`
- `re:find_all(input)` -> `Match[]`
- `re:captures(input)` -> `Captures|nil`
- `re:captures_all(input)` -> `Captures[]`
- `re:replace(input, replacement)` -> `string`
- `re:replace_all(input, replacement)` -> `string`
- `re:split(input[, limit])` -> `string[]`

### is_match

规范 API 名称：`ptool.re.Regex:is_match`。

`re:is_match(input)` 检查正则是否能匹配 `input`。

- `input`（string，必填）：输入文本。
- 返回：`boolean`。

### find

规范 API 名称：`ptool.re.Regex:find`。

`re:find(input[, init])` 返回 `input` 中第一个匹配项；如果没有匹配，则返回 `nil`。

- `input`（string，必填）：输入文本。

参数说明：

- `init` 是从 `1` 开始的起始位置，默认值为 `1`。
- `limit` 必须大于 `0`。

返回结构：

- `Match`：
  - `start`（integer）：从 `1` 开始的起始索引。
  - `finish`（integer）：结束索引，可直接用于 `string.sub`。
  - `text`（string）：匹配到的文本。
- `Captures`：
  - `full`（string）：完整匹配文本。
  - `groups`（table）：按捕获顺序排列的捕获组数组。未命中的组为 `nil`。
  - `named`（table）：命名捕获组映射，键为组名。

### find_all

规范 API 名称：`ptool.re.Regex:find_all`。

`re:find_all(input)` 以 `Match[]` 形式返回 `input` 中的全部匹配项。

### captures

规范 API 名称：`ptool.re.Regex:captures`。

`re:captures(input)` 返回 `input` 中第一组捕获结果；如果没有匹配，则返回 `nil`。

### captures_all

规范 API 名称：`ptool.re.Regex:captures_all`。

`re:captures_all(input)` 以 `Captures[]` 形式返回 `input` 中全部捕获结果。

### replace

规范 API 名称：`ptool.re.Regex:replace`。

`re:replace(input, replacement)` 替换 `input` 中第一个匹配项。

### replace_all

规范 API 名称：`ptool.re.Regex:replace_all`。

`re:replace_all(input, replacement)` 替换 `input` 中全部匹配项。

### split

规范 API 名称：`ptool.re.Regex:split`。

`re:split(input[, limit])` 使用正则作为分隔符拆分 `input`。

示例：

```lua
local re = ptool.re.compile("(?P<word>\\w+)")
local cap = re:captures("hello world")
print(cap.full)         -- hello
print(cap.named.word)   -- hello
print(re:replace_all("a b c", "_")) -- _ _ _
```
