# Regex API

Regular expression helpers are available under `ptool.re` and `p.re`.

## ptool.re.compile

> `v0.1.0` - Introduced.

`ptool.re.compile(pattern[, opts])` compiles a regular expression and returns a
`Regex` object.

- `pattern` (string, required): The regex pattern.
- `opts` (table, optional): Compile options. Currently supported:
  - `case_insensitive` (boolean, optional): Whether matching is
    case-insensitive. Defaults to `false`.

Example:

```lua
local re = ptool.re.compile("(?P<name>\\w+)", { case_insensitive = true })
print(re:is_match("Alice")) -- true
```

## ptool.re.escape

> `v0.1.0` - Introduced.

`ptool.re.escape(text)` escapes plain text into a regex literal string.

- `text` (string, required): The text to escape.
- Returns: The escaped string.

Example:

```lua
local keyword = "a+b?"
local re = ptool.re.compile("^" .. ptool.re.escape(keyword) .. "$")
print(re:is_match("a+b?")) -- true
```

### Regex Methods

`ptool.re.compile(...)` returns a `Regex` UserData with the following methods:

- `re:is_match(input)` -> `boolean`
- `re:find(input[, init])` -> `Match|nil`
- `re:find_all(input)` -> `Match[]`
- `re:captures(input)` -> `Captures|nil`
- `re:captures_all(input)` -> `Captures[]`
- `re:replace(input, replacement)` -> `string`
- `re:replace_all(input, replacement)` -> `string`
- `re:split(input[, limit])` -> `string[]`

Parameter notes:

- `init` is a 1-based start position and defaults to `1`.
- `limit` must be greater than `0`.

Return structures:

- `Match`:
  - `start` (integer): The 1-based start index.
  - `finish` (integer): The end index, directly usable with `string.sub`.
  - `text` (string): The matched text.
- `Captures`:
  - `full` (string): The full matched text.
  - `groups` (table): An array of capture groups in capture order. Unmatched
    groups are `nil`.
  - `named` (table): A mapping of named capture groups, keyed by group name.

Example:

```lua
local re = ptool.re.compile("(?P<word>\\w+)")
local cap = re:captures("hello world")
print(cap.full)         -- hello
print(cap.named.word)   -- hello
print(re:replace_all("a b c", "_")) -- _ _ _
```
