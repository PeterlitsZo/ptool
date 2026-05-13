# Hash API

Hashing helpers are available under `ptool.hash` and `p.hash`.

## ptool.hash.sha256

> `v0.2.0` - Introduced.

`ptool.hash.sha256(input)` returns the SHA-256 digest of a Lua string.

- `input` (string, required): The input Lua string. The digest is computed from the string's raw bytes.
- Returns: `string` (lowercase hexadecimal digest).

```lua
print(ptool.hash.sha256("hello"))
-- 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

## ptool.hash.sha1

> `v0.2.0` - Introduced.

`ptool.hash.sha1(input)` returns the SHA-1 digest of a Lua string.

- `input` (string, required): The input Lua string. The digest is computed from the string's raw bytes.
- Returns: `string` (lowercase hexadecimal digest).

```lua
print(ptool.hash.sha1("hello"))
-- aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
```

## ptool.hash.md5

> `v0.2.0` - Introduced.

`ptool.hash.md5(input)` returns the MD5 digest of a Lua string.

- `input` (string, required): The input Lua string. The digest is computed from the string's raw bytes.
- Returns: `string` (lowercase hexadecimal digest).

```lua
print(ptool.hash.md5("hello"))
-- 5d41402abc4b2a76b9719d911017c592
```
