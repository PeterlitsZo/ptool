# String API

String helpers are available under `ptool.str` and `p.str`.

## ptool.str.trim

> `v0.1.0` - Introduced.

`ptool.str.trim(s)` removes leading and trailing whitespace.

- `s` (string, required): The input string.
- Returns: `string`.

```lua
print(ptool.str.trim("  hello\n")) -- hello
```

## ptool.str.trim_start

> `v0.1.0` - Introduced.

`ptool.str.trim_start(s)` removes leading whitespace.

- `s` (string, required): The input string.
- Returns: `string`.

```lua
print(ptool.str.trim_start("  hello  ")) -- hello  
```

## ptool.str.trim_end

> `v0.1.0` - Introduced.

`ptool.str.trim_end(s)` removes trailing whitespace.

- `s` (string, required): The input string.
- Returns: `string`.

```lua
print(ptool.str.trim_end("  hello  ")) --   hello
```

## ptool.str.is_blank

> `v0.1.0` - Introduced.

`ptool.str.is_blank(s)` checks whether a string is empty or contains only
whitespace.

- `s` (string, required): The input string.
- Returns: `boolean`.

```lua
print(ptool.str.is_blank(" \t\n")) -- true
print(ptool.str.is_blank("x")) -- false
```

## ptool.str.starts_with

> `v0.1.0` - Introduced.

`ptool.str.starts_with(s, prefix)` checks whether `s` starts with `prefix`.

- `s` (string, required): The input string.
- `prefix` (string, required): The prefix to test.
- Returns: `boolean`.

```lua
print(ptool.str.starts_with("hello.lua", "hello")) -- true
```

## ptool.str.ends_with

> `v0.1.0` - Introduced.

`ptool.str.ends_with(s, suffix)` checks whether `s` ends with `suffix`.

- `s` (string, required): The input string.
- `suffix` (string, required): The suffix to test.
- Returns: `boolean`.

```lua
print(ptool.str.ends_with("hello.lua", ".lua")) -- true
```

## ptool.str.contains

> `v0.1.0` - Introduced.

`ptool.str.contains(s, needle)` checks whether `needle` appears in `s`.

- `s` (string, required): The input string.
- `needle` (string, required): The substring to search for.
- Returns: `boolean`.

```lua
print(ptool.str.contains("hello.lua", "lo.l")) -- true
```

## ptool.str.split

> `v0.1.0` - Introduced.

`ptool.str.split(s, sep[, options])` splits a string by a non-empty separator.

- `s` (string, required): The input string.
- `sep` (string, required): The separator. Empty strings are not allowed.
- `options` (table, optional): Split options. Supported fields:
  - `trim` (boolean, optional): Whether to trim each piece before returning it.
    Defaults to `false`.
  - `skip_empty` (boolean, optional): Whether to remove empty pieces after
    optional trimming. Defaults to `false`.
- Returns: `string[]`.

Behavior:

- Unknown option names or invalid option value types raise an error.
- `skip_empty = true` is applied after `trim`, so whitespace-only pieces can be
  removed when both are enabled.

```lua
local parts = ptool.str.split(" a, b ,, c ", ",", {
  trim = true,
  skip_empty = true,
})

print(ptool.inspect(parts)) -- { "a", "b", "c" }
```

## ptool.str.split_lines

> `v0.1.0` - Introduced.

`ptool.str.split_lines(s[, options])` splits a string into lines.

- `s` (string, required): The input string.
- `options` (table, optional): Line-splitting options. Supported fields:
  - `keep_ending` (boolean, optional): Whether to keep line endings (`\n`,
    `\r\n`, or `\r`) in returned items. Defaults to `false`.
  - `skip_empty` (boolean, optional): Whether to remove empty lines. Defaults
    to `false`.
- Returns: `string[]`.

Behavior:

- Supports Unix (`\n`) and Windows (`\r\n`) line endings, and also lone `\r`.
- When `skip_empty = true`, a line containing only a line ending is treated as
  empty and is removed.
- Unknown option names or invalid option value types raise an error.

```lua
local lines = ptool.str.split_lines("a\n\n b\r\n", {
  skip_empty = true,
})

print(ptool.inspect(lines)) -- { "a", " b" }
```

## ptool.str.join

> `v0.1.0` - Introduced.

`ptool.str.join(parts, sep)` joins a string array with a separator.

- `parts` (string[], required): The string parts to join.
- `sep` (string, required): The separator string.
- Returns: `string`.

```lua
print(ptool.str.join({"a", "b", "c"}, "/")) -- a/b/c
```

## ptool.str.replace

> `v0.1.0` - Introduced.

`ptool.str.replace(s, from, to[, n])` replaces occurrences of `from` with `to`.

- `s` (string, required): The input string.
- `from` (string, required): The substring to replace. Empty strings are not
  allowed.
- `to` (string, required): The replacement string.
- `n` (integer, optional): Maximum replacement count. Must be greater than or
  equal to `0`. If omitted, all matches are replaced.
- Returns: `string`.

```lua
print(ptool.str.replace("a-b-c", "-", "/")) -- a/b/c
print(ptool.str.replace("a-b-c", "-", "/", 1)) -- a/b-c
```

## ptool.str.repeat

> `v0.1.0` - Introduced.

`ptool.str.repeat(s, n)` repeats a string `n` times.

- `s` (string, required): The input string.
- `n` (integer, required): Repeat count. Must be greater than or equal to `0`.
- Returns: `string`.

```lua
print(ptool.str.repeat("ab", 3)) -- ababab
```

## ptool.str.cut_prefix

> `v0.1.0` - Introduced.

`ptool.str.cut_prefix(s, prefix)` removes `prefix` from the start of `s` when
it is present.

- `s` (string, required): The input string.
- `prefix` (string, required): The prefix to remove.
- Returns: `string`.

Behavior:

- If `s` does not start with `prefix`, the original string is returned
  unchanged.

```lua
print(ptool.str.cut_prefix("refs/heads/main", "refs/heads/")) -- main
```

## ptool.str.cut_suffix

> `v0.1.0` - Introduced.

`ptool.str.cut_suffix(s, suffix)` removes `suffix` from the end of `s` when it
is present.

- `s` (string, required): The input string.
- `suffix` (string, required): The suffix to remove.
- Returns: `string`.

Behavior:

- If `s` does not end with `suffix`, the original string is returned unchanged.

```lua
print(ptool.str.cut_suffix("archive.tar.gz", ".gz")) -- archive.tar
```

## ptool.str.indent

> `v0.1.0` - Introduced.

`ptool.str.indent(s, prefix[, options])` adds `prefix` to each line.

- `s` (string, required): The input string.
- `prefix` (string, required): The text inserted before each line.
- `options` (table, optional): Indent options. Supported fields:
  - `skip_first` (boolean, optional): Whether to leave the first line unchanged.
    Defaults to `false`.
- Returns: `string`.

Behavior:

- Existing line endings are preserved.
- Empty input is returned unchanged.
- Unknown option names or invalid option value types raise an error.

```lua
local text = "first\nsecond\n"
print(ptool.str.indent(text, "> "))
print(ptool.str.indent(text, "  ", { skip_first = true }))
```
