# ハッシュ API

ハッシュヘルパーは `ptool.hash` と `p.hash` にあります。

すべてのハッシュヘルパーは同じ形式です:

- `input` (string, 必須): 入力となる Lua 文字列。ダイジェストまたはチェックサムは文字列の生バイト列から計算されます。
- 戻り値: `string` (小文字の16進数出力)。

## 暗号学的ダイジェスト

- `ptool.hash.sha1(input)`: SHA-1 ダイジェスト。`v0.2.0` で導入。
- `ptool.hash.sha224(input)`: SHA-224 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.sha256(input)`: SHA-256 ダイジェスト。`v0.2.0` で導入。
- `ptool.hash.sha384(input)`: SHA-384 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.sha512(input)`: SHA-512 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.sha512_224(input)`: SHA-512/224 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.sha512_256(input)`: SHA-512/256 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.sha3_224(input)`: SHA3-224 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.sha3_256(input)`: SHA3-256 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.sha3_384(input)`: SHA3-384 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.sha3_512(input)`: SHA3-512 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.blake2s256(input)`: BLAKE2s-256 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.blake2b512(input)`: BLAKE2b-512 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.blake3(input)`: BLAKE3 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.md5(input)`: MD5 ダイジェスト。`v0.2.0` で導入。

## チェックサムと高速な非暗号学的ハッシュ

- `ptool.hash.crc32(input)`: CRC-32/ISO-HDLC チェックサム。`v0.9.0` で導入。
- `ptool.hash.crc64(input)`: CRC-64/ECMA-182 チェックサム。`v0.9.0` で導入。
- `ptool.hash.adler32(input)`: Adler-32 チェックサム。`v0.9.0` で導入。
- `ptool.hash.xxh32(input)`: xxHash32 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.xxh64(input)`: xxHash64 ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.xxh3_64(input)`: XXH3 64ビット ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.xxh3_128(input)`: XXH3 128ビット ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.murmur3_32(input)`: MurmurHash3 32ビット ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.murmur3_128(input)`: MurmurHash3 x64 128ビット ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.fnv1a32(input)`: FNV-1a 32ビット ダイジェスト。`v0.9.0` で導入。
- `ptool.hash.fnv1a64(input)`: FNV-1a 64ビット ダイジェスト。`v0.9.0` で導入。

## 例

```lua
print(ptool.hash.sha256("hello"))
print(ptool.hash.blake3("hello"))
print(ptool.hash.crc32("hello"))
print(ptool.hash.xxh3_64("hello"))
```
