# Hash API

哈希辅助能力位于 `ptool.hash` 和 `p.hash` 下。

所有哈希辅助方法都采用相同的形式：

- `input`（string，必填）：输入的 Lua 字符串。摘要或校验和基于字符串的原始字节计算。
- 返回：`string`（小写十六进制输出）。

## 加密哈希摘要

- `ptool.hash.sha1(input)`：SHA-1 摘要。于 `v0.2.0` 引入。
- `ptool.hash.sha224(input)`：SHA-224 摘要。于 `v0.9.0` 引入。
- `ptool.hash.sha256(input)`：SHA-256 摘要。于 `v0.2.0` 引入。
- `ptool.hash.sha384(input)`：SHA-384 摘要。于 `v0.9.0` 引入。
- `ptool.hash.sha512(input)`：SHA-512 摘要。于 `v0.9.0` 引入。
- `ptool.hash.sha512_224(input)`：SHA-512/224 摘要。于 `v0.9.0` 引入。
- `ptool.hash.sha512_256(input)`：SHA-512/256 摘要。于 `v0.9.0` 引入。
- `ptool.hash.sha3_224(input)`：SHA3-224 摘要。于 `v0.9.0` 引入。
- `ptool.hash.sha3_256(input)`：SHA3-256 摘要。于 `v0.9.0` 引入。
- `ptool.hash.sha3_384(input)`：SHA3-384 摘要。于 `v0.9.0` 引入。
- `ptool.hash.sha3_512(input)`：SHA3-512 摘要。于 `v0.9.0` 引入。
- `ptool.hash.blake2s256(input)`：BLAKE2s-256 摘要。于 `v0.9.0` 引入。
- `ptool.hash.blake2b512(input)`：BLAKE2b-512 摘要。于 `v0.9.0` 引入。
- `ptool.hash.blake3(input)`：BLAKE3 摘要。于 `v0.9.0` 引入。
- `ptool.hash.md5(input)`：MD5 摘要。于 `v0.2.0` 引入。

## 校验和与快速非加密哈希

- `ptool.hash.crc32(input)`：CRC-32/ISO-HDLC 校验和。于 `v0.9.0` 引入。
- `ptool.hash.crc64(input)`：CRC-64/ECMA-182 校验和。于 `v0.9.0` 引入。
- `ptool.hash.adler32(input)`：Adler-32 校验和。于 `v0.9.0` 引入。
- `ptool.hash.xxh32(input)`：xxHash32 摘要。于 `v0.9.0` 引入。
- `ptool.hash.xxh64(input)`：xxHash64 摘要。于 `v0.9.0` 引入。
- `ptool.hash.xxh3_64(input)`：XXH3 64 位摘要。于 `v0.9.0` 引入。
- `ptool.hash.xxh3_128(input)`：XXH3 128 位摘要。于 `v0.9.0` 引入。
- `ptool.hash.murmur3_32(input)`：MurmurHash3 32 位摘要。于 `v0.9.0` 引入。
- `ptool.hash.murmur3_128(input)`：MurmurHash3 x64 128 位摘要。于 `v0.9.0` 引入。
- `ptool.hash.fnv1a32(input)`：FNV-1a 32 位摘要。于 `v0.9.0` 引入。
- `ptool.hash.fnv1a64(input)`：FNV-1a 64 位摘要。于 `v0.9.0` 引入。

## 示例

```lua
print(ptool.hash.sha256("hello"))
print(ptool.hash.blake3("hello"))
print(ptool.hash.crc32("hello"))
print(ptool.hash.xxh3_64("hello"))
```
