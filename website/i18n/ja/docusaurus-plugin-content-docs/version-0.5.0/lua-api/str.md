# 文字列 API

文字列ヘルパーは `ptool.str` と `p.str` にあります。

## ptool.str.trim

> `v0.1.0` - Introduced.

`ptool.str.trim(s)` は先頭と末尾の空白を取り除きます。

- `s` (string, 必須): 入力文字列。
- 戻り値: `string`。

```lua
print(ptool.str.trim("  hello\n")) -- hello
```

## ptool.str.trim_start

> `v0.1.0` - Introduced.

`ptool.str.trim_start(s)` は先頭の空白を取り除きます。

- `s` (string, 必須): 入力文字列。
- 戻り値: `string`。

```lua
print(ptool.str.trim_start("  hello  ")) -- hello  
```

## ptool.str.trim_end

> `v0.1.0` - Introduced.

`ptool.str.trim_end(s)` は末尾の空白を取り除きます。

- `s` (string, 必須): 入力文字列。
- 戻り値: `string`。

```lua
print(ptool.str.trim_end("  hello  ")) --   hello
```

## ptool.str.is_blank

> `v0.1.0` - Introduced.

`ptool.str.is_blank(s)` は、文字列が空か空白だけで構成されているかを
確認します。

- `s` (string, 必須): 入力文字列。
- 戻り値: `boolean`。

```lua
print(ptool.str.is_blank(" \t\n")) -- true
print(ptool.str.is_blank("x")) -- false
```

## ptool.str.starts_with

> `v0.1.0` - Introduced.

`ptool.str.starts_with(s, prefix)` は `s` が `prefix` で始まるか確認します。

- `s` (string, 必須): 入力文字列。
- `prefix` (string, 必須): 確認するプレフィックス。
- 戻り値: `boolean`。

```lua
print(ptool.str.starts_with("hello.lua", "hello")) -- true
```

## ptool.str.ends_with

> `v0.1.0` - Introduced.

`ptool.str.ends_with(s, suffix)` は `s` が `suffix` で終わるか確認します。

- `s` (string, 必須): 入力文字列。
- `suffix` (string, 必須): 確認するサフィックス。
- 戻り値: `boolean`。

```lua
print(ptool.str.ends_with("hello.lua", ".lua")) -- true
```

## ptool.str.contains

> `v0.1.0` - Introduced.

`ptool.str.contains(s, needle)` は `needle` が `s` に含まれるか確認します。

- `s` (string, 必須): 入力文字列。
- `needle` (string, 必須): 探す部分文字列。
- 戻り値: `boolean`。

```lua
print(ptool.str.contains("hello.lua", "lo.l")) -- true
```

## ptool.str.split

> `v0.1.0` - Introduced.

`ptool.str.split(s, sep[, options])` は、空でない区切り文字で文字列を
分割します。

- `s` (string, 必須): 入力文字列。
- `sep` (string, 必須): 区切り文字。空文字列は使えません。
- `options` (table, 任意): 分割オプション。サポートされるフィールド:
  - `trim` (boolean, 任意): 返す前に各要素をトリムするかどうか。
    デフォルトは `false`。
  - `skip_empty` (boolean, 任意): トリム後に空要素を取り除くかどうか。
    デフォルトは `false`。
- 戻り値: `string[]`。

挙動:

- 未知のオプション名や不正な値型はエラーになります。
- `skip_empty = true` は `trim` の後に適用されるため、両方を有効にすると
  空白だけの要素も除去できます。

```lua
local parts = ptool.str.split(" a, b ,, c ", ",", {
  trim = true,
  skip_empty = true,
})

print(ptool.inspect(parts)) -- { "a", "b", "c" }
```

## ptool.str.split_lines

> `v0.1.0` - Introduced.

`ptool.str.split_lines(s[, options])` は文字列を行に分割します。

- `s` (string, 必須): 入力文字列。
- `options` (table, 任意): 行分割オプション。サポートされるフィールド:
  - `keep_ending` (boolean, 任意): 行末 (`\n`, `\r\n`, `\r`) を返す要素に
    残すかどうか。デフォルトは `false`。
  - `skip_empty` (boolean, 任意): 空行を取り除くかどうか。デフォルトは
    `false`。
- 戻り値: `string[]`。

挙動:

- Unix (`\n`) と Windows (`\r\n`) の行末、および単独の `\r` を
  サポートします。
- `skip_empty = true` のとき、行末だけを含む行は空行として扱われて
  取り除かれます。
- 未知のオプション名や不正な値型はエラーになります。

```lua
local lines = ptool.str.split_lines("a\n\n b\r\n", {
  skip_empty = true,
})

print(ptool.inspect(lines)) -- { "a", " b" }
```

## ptool.str.join

> `v0.1.0` - Introduced.

`ptool.str.join(parts, sep)` は文字列配列を区切り文字で結合します。

- `parts` (string[], 必須): 結合する文字列要素。
- `sep` (string, 必須): 区切り文字列。
- 戻り値: `string`。

```lua
print(ptool.str.join({"a", "b", "c"}, "/")) -- a/b/c
```

## ptool.str.replace

> `v0.1.0` - Introduced.

`ptool.str.replace(s, from, to[, n])` は `from` の出現を `to` へ置換します。

- `s` (string, 必須): 入力文字列。
- `from` (string, 必須): 置換対象の部分文字列。空文字列は使えません。
- `to` (string, 必須): 置換文字列。
- `n` (integer, 任意): 最大置換回数。`0` 以上でなければなりません。
  省略時はすべての一致を置換します。
- 戻り値: `string`。

```lua
print(ptool.str.replace("a-b-c", "-", "/")) -- a/b/c
print(ptool.str.replace("a-b-c", "-", "/", 1)) -- a/b-c
```

## ptool.str.repeat

> `v0.1.0` - Introduced.

`ptool.str.repeat(s, n)` は文字列を `n` 回繰り返します。

- `s` (string, 必須): 入力文字列。
- `n` (integer, 必須): 繰り返し回数。`0` 以上でなければなりません。
- 戻り値: `string`。

```lua
print(ptool.str.repeat("ab", 3)) -- ababab
```

## ptool.str.cut_prefix

> `v0.1.0` - Introduced.

`ptool.str.cut_prefix(s, prefix)` は `prefix` が存在する場合に、`s` の
先頭からそれを取り除きます。

- `s` (string, 必須): 入力文字列。
- `prefix` (string, 必須): 取り除くプレフィックス。
- 戻り値: `string`。

挙動:

- `s` が `prefix` で始まらない場合は、元の文字列がそのまま返されます。

```lua
print(ptool.str.cut_prefix("refs/heads/main", "refs/heads/")) -- main
```

## ptool.str.cut_suffix

> `v0.1.0` - Introduced.

`ptool.str.cut_suffix(s, suffix)` は `suffix` が存在する場合に、`s` の末尾
からそれを取り除きます。

- `s` (string, 必須): 入力文字列。
- `suffix` (string, 必須): 取り除くサフィックス。
- 戻り値: `string`。

挙動:

- `s` が `suffix` で終わらない場合は、元の文字列がそのまま返されます。

```lua
print(ptool.str.cut_suffix("archive.tar.gz", ".gz")) -- archive.tar
```

## ptool.str.indent

> `v0.1.0` - Introduced.

`ptool.str.indent(s, prefix[, options])` は各行の先頭に `prefix` を
追加します。

- `s` (string, 必須): 入力文字列。
- `prefix` (string, 必須): 各行の前に挿入するテキスト。
- `options` (table, 任意): インデントオプション。サポートされる
  フィールド:
  - `skip_first` (boolean, 任意): 1 行目を変更しないかどうか。デフォルトは
    `false`。
- 戻り値: `string`。

挙動:

- 既存の行末は保持されます。
- 空入力はそのまま返されます。
- 未知のオプション名や不正な値型はエラーになります。

```lua
local text = "first\nsecond\n"
print(ptool.str.indent(text, "> "))
print(ptool.str.indent(text, "  ", { skip_first = true }))
```
