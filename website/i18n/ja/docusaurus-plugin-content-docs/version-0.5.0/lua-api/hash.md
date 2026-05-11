# ハッシュ API

ハッシュヘルパーは `ptool.hash` と `p.hash` にあります。

## ptool.hash.sha256

> `v0.2.0` - Introduced.

`ptool.hash.sha256(input)` は Lua 文字列の SHA-256 ダイジェストを返します。

- `input` (string, 必須): 入力となる Lua 文字列。ダイジェストは文字列の
  生バイト列から計算されます。
- 戻り値: `string` (小文字の 16 進ダイジェスト)。

```lua
print(ptool.hash.sha256("hello"))
-- 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

## ptool.hash.sha1

> `v0.2.0` - Introduced.

`ptool.hash.sha1(input)` は Lua 文字列の SHA-1 ダイジェストを返します。

- `input` (string, 必須): 入力となる Lua 文字列。ダイジェストは文字列の
  生バイト列から計算されます。
- 戻り値: `string` (小文字の 16 進ダイジェスト)。

```lua
print(ptool.hash.sha1("hello"))
-- aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
```

## ptool.hash.md5

> `v0.2.0` - Introduced.

`ptool.hash.md5(input)` は Lua 文字列の MD5 ダイジェストを返します。

- `input` (string, 必須): 入力となる Lua 文字列。ダイジェストは文字列の
  生バイト列から計算されます。
- 戻り値: `string` (小文字の 16 進ダイジェスト)。

```lua
print(ptool.hash.md5("hello"))
-- 5d41402abc4b2a76b9719d911017c592
```
