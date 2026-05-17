# Hash API

哈希辅助能力位于 `ptool.hash` 和 `p.hash` 下。

## ptool.hash.sha256

> `v0.2.0` - 引入。

`ptool.hash.sha256(input)` 返回 Lua 字符串的 SHA-256 摘要。

- `input`（string，必填）：输入的 Lua 字符串。摘要基于字符串的原始字节计算。
- 返回：`string`（小写十六进制摘要）。

```lua
print(ptool.hash.sha256("hello"))
-- 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

## ptool.hash.sha1

> `v0.2.0` - 引入。

`ptool.hash.sha1(input)` 返回 Lua 字符串的 SHA-1 摘要。

- `input`（string，必填）：输入的 Lua 字符串。摘要基于字符串的原始字节计算。
- 返回：`string`（小写十六进制摘要）。

```lua
print(ptool.hash.sha1("hello"))
-- aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
```

## ptool.hash.md5

> `v0.2.0` - 引入。

`ptool.hash.md5(input)` 返回 Lua 字符串的 MD5 摘要。

- `input`（string，必填）：输入的 Lua 字符串。摘要基于字符串的原始字节计算。
- 返回：`string`（小写十六进制摘要）。

```lua
print(ptool.hash.md5("hello"))
-- 5d41402abc4b2a76b9719d911017c592
```
