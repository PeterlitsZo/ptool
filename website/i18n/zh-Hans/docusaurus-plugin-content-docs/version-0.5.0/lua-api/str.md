# String API

字符串辅助能力位于 `ptool.str` 和 `p.str` 下。

## ptool.str.trim

> `v0.1.0` - 引入。

`ptool.str.trim(s)` 移除首尾空白字符。

- `s`（string，必填）：输入字符串。
- 返回：`string`。

```lua
print(ptool.str.trim("  hello\n")) -- hello
```

## ptool.str.trim_start

> `v0.1.0` - 引入。

`ptool.str.trim_start(s)` 移除开头的空白字符。

- `s`（string，必填）：输入字符串。
- 返回：`string`。

```lua
print(ptool.str.trim_start("  hello  ")) -- hello  
```

## ptool.str.trim_end

> `v0.1.0` - 引入。

`ptool.str.trim_end(s)` 移除结尾的空白字符。

- `s`（string，必填）：输入字符串。
- 返回：`string`。

```lua
print(ptool.str.trim_end("  hello  ")) --   hello
```

## ptool.str.is_blank

> `v0.1.0` - 引入。

`ptool.str.is_blank(s)` 检查字符串是否为空，或是否只包含空白字符。

- `s`（string，必填）：输入字符串。
- 返回：`boolean`。

```lua
print(ptool.str.is_blank(" \t\n")) -- true
print(ptool.str.is_blank("x")) -- false
```

## ptool.str.starts_with

> `v0.1.0` - 引入。

`ptool.str.starts_with(s, prefix)` 检查 `s` 是否以 `prefix` 开头。

- `s`（string，必填）：输入字符串。
- `prefix`（string，必填）：要判断的前缀。
- 返回：`boolean`。

```lua
print(ptool.str.starts_with("hello.lua", "hello")) -- true
```

## ptool.str.ends_with

> `v0.1.0` - 引入。

`ptool.str.ends_with(s, suffix)` 检查 `s` 是否以 `suffix` 结尾。

- `s`（string，必填）：输入字符串。
- `suffix`（string，必填）：要判断的后缀。
- 返回：`boolean`。

```lua
print(ptool.str.ends_with("hello.lua", ".lua")) -- true
```

## ptool.str.contains

> `v0.1.0` - 引入。

`ptool.str.contains(s, needle)` 检查 `needle` 是否出现在 `s` 中。

- `s`（string，必填）：输入字符串。
- `needle`（string，必填）：要查找的子串。
- 返回：`boolean`。

```lua
print(ptool.str.contains("hello.lua", "lo.l")) -- true
```

## ptool.str.split

> `v0.1.0` - 引入。

`ptool.str.split(s, sep[, options])` 使用非空分隔符拆分字符串。

- `s`（string，必填）：输入字符串。
- `sep`（string，必填）：分隔符。不允许空字符串。
- `options`（table，可选）：拆分选项。支持：
  - `trim`（boolean，可选）：是否在返回前裁剪每个片段。默认值为 `false`。
  - `skip_empty`（boolean，可选）：是否在可选裁剪后移除空片段。默认值为 `false`。
- 返回：`string[]`。

行为说明：

- 未知选项名或非法选项值类型都会抛出错误。
- `skip_empty = true` 会在 `trim` 之后生效，因此当两者都启用时，只包含空白字符的
  片段也会被移除。

```lua
local parts = ptool.str.split(" a, b ,, c ", ",", {
  trim = true,
  skip_empty = true,
})

print(ptool.inspect(parts)) -- { "a", "b", "c" }
```

## ptool.str.split_lines

> `v0.1.0` - 引入。

`ptool.str.split_lines(s[, options])` 把字符串拆分成多行。

- `s`（string，必填）：输入字符串。
- `options`（table，可选）：按行拆分选项。支持：
  - `keep_ending`（boolean，可选）：是否保留行结束符（`\n`、`\r\n` 或 `\r`）。
    默认值为 `false`。
  - `skip_empty`（boolean，可选）：是否移除空行。默认值为 `false`。
- 返回：`string[]`。

行为说明：

- 支持 Unix（`\n`）和 Windows（`\r\n`）换行，也支持单独的 `\r`。
- 当 `skip_empty = true` 时，仅包含行结束符的行会被视为空行并移除。
- 未知选项名或非法选项值类型都会抛出错误。

```lua
local lines = ptool.str.split_lines("a\n\n b\r\n", {
  skip_empty = true,
})

print(ptool.inspect(lines)) -- { "a", " b" }
```

## ptool.str.join

> `v0.1.0` - 引入。

`ptool.str.join(parts, sep)` 使用分隔符拼接字符串数组。

- `parts`（string[]，必填）：待拼接的字符串片段。
- `sep`（string，必填）：分隔符字符串。
- 返回：`string`。

```lua
print(ptool.str.join({"a", "b", "c"}, "/")) -- a/b/c
```

## ptool.str.replace

> `v0.1.0` - 引入。

`ptool.str.replace(s, from, to[, n])` 将 `from` 替换为 `to`。

- `s`（string，必填）：输入字符串。
- `from`（string，必填）：要替换的子串。不允许空字符串。
- `to`（string，必填）：替换字符串。
- `n`（integer，可选）：最大替换次数。必须大于等于 `0`。如果省略，则替换全部匹配。
- 返回：`string`。

```lua
print(ptool.str.replace("a-b-c", "-", "/")) -- a/b/c
print(ptool.str.replace("a-b-c", "-", "/", 1)) -- a/b-c
```

## ptool.str.repeat

> `v0.1.0` - 引入。

`ptool.str.repeat(s, n)` 将字符串 `s` 重复 `n` 次。

- `s`（string，必填）：输入字符串。
- `n`（integer，必填）：重复次数。必须大于等于 `0`。
- 返回：`string`。

```lua
print(ptool.str.repeat("ab", 3)) -- ababab
```

## ptool.str.cut_prefix

> `v0.1.0` - 引入。

`ptool.str.cut_prefix(s, prefix)` 当 `s` 以 `prefix` 开头时，将其从开头移除。

- `s`（string，必填）：输入字符串。
- `prefix`（string，必填）：要移除的前缀。
- 返回：`string`。

行为说明：

- 如果 `s` 不以 `prefix` 开头，则原样返回原始字符串。

```lua
print(ptool.str.cut_prefix("refs/heads/main", "refs/heads/")) -- main
```

## ptool.str.cut_suffix

> `v0.1.0` - 引入。

`ptool.str.cut_suffix(s, suffix)` 当 `s` 以 `suffix` 结尾时，将其从末尾移除。

- `s`（string，必填）：输入字符串。
- `suffix`（string，必填）：要移除的后缀。
- 返回：`string`。

行为说明：

- 如果 `s` 不以 `suffix` 结尾，则原样返回原始字符串。

```lua
print(ptool.str.cut_suffix("archive.tar.gz", ".gz")) -- archive.tar
```

## ptool.str.indent

> `v0.1.0` - 引入。

`ptool.str.indent(s, prefix[, options])` 为每一行添加 `prefix`。

- `s`（string，必填）：输入字符串。
- `prefix`（string，必填）：插入到每行前面的文本。
- `options`（table，可选）：缩进选项。支持：
  - `skip_first`（boolean，可选）：是否保持第一行不变。默认值为 `false`。
- 返回：`string`。

行为说明：

- 会保留已有的行结束符。
- 空输入会原样返回。
- 未知选项名或非法选项值类型都会抛出错误。

```lua
local text = "first\nsecond\n"
print(ptool.str.indent(text, "> "))
print(ptool.str.indent(text, "  ", { skip_first = true }))
```
