# Hash API

Hashing helpers are available under `ptool.hash` and `p.hash`.

All hash helpers share the same shape:

- `input` (string, required): The input Lua string. The digest or checksum is
  computed from the string's raw bytes.
- Returns: `string` (lowercase hexadecimal output).

## Cryptographic digests

- `ptool.hash.sha1(input)`: SHA-1 digest. Introduced in `v0.2.0`.
- `ptool.hash.sha224(input)`: SHA-224 digest. Introduced in `v0.9.0`.
- `ptool.hash.sha256(input)`: SHA-256 digest. Introduced in `v0.2.0`.
- `ptool.hash.sha384(input)`: SHA-384 digest. Introduced in `v0.9.0`.
- `ptool.hash.sha512(input)`: SHA-512 digest. Introduced in `v0.9.0`.
- `ptool.hash.sha512_224(input)`: SHA-512/224 digest. Introduced in `v0.9.0`.
- `ptool.hash.sha512_256(input)`: SHA-512/256 digest. Introduced in `v0.9.0`.
- `ptool.hash.sha3_224(input)`: SHA3-224 digest. Introduced in `v0.9.0`.
- `ptool.hash.sha3_256(input)`: SHA3-256 digest. Introduced in `v0.9.0`.
- `ptool.hash.sha3_384(input)`: SHA3-384 digest. Introduced in `v0.9.0`.
- `ptool.hash.sha3_512(input)`: SHA3-512 digest. Introduced in `v0.9.0`.
- `ptool.hash.blake2s256(input)`: BLAKE2s-256 digest. Introduced in `v0.9.0`.
- `ptool.hash.blake2b512(input)`: BLAKE2b-512 digest. Introduced in `v0.9.0`.
- `ptool.hash.blake3(input)`: BLAKE3 digest. Introduced in `v0.9.0`.
- `ptool.hash.md5(input)`: MD5 digest. Introduced in `v0.2.0`.

## Checksums and fast non-cryptographic hashes

- `ptool.hash.crc32(input)`: CRC-32/ISO-HDLC checksum. Introduced in `v0.9.0`.
- `ptool.hash.crc64(input)`: CRC-64/ECMA-182 checksum. Introduced in `v0.9.0`.
- `ptool.hash.adler32(input)`: Adler-32 checksum. Introduced in `v0.9.0`.
- `ptool.hash.xxh32(input)`: xxHash32 digest. Introduced in `v0.9.0`.
- `ptool.hash.xxh64(input)`: xxHash64 digest. Introduced in `v0.9.0`.
- `ptool.hash.xxh3_64(input)`: XXH3 64-bit digest. Introduced in `v0.9.0`.
- `ptool.hash.xxh3_128(input)`: XXH3 128-bit digest. Introduced in `v0.9.0`.
- `ptool.hash.murmur3_32(input)`: MurmurHash3 32-bit digest. Introduced in `v0.9.0`.
- `ptool.hash.murmur3_128(input)`: MurmurHash3 x64 128-bit digest. Introduced in `v0.9.0`.
- `ptool.hash.fnv1a32(input)`: FNV-1a 32-bit digest. Introduced in `v0.9.0`.
- `ptool.hash.fnv1a64(input)`: FNV-1a 64-bit digest. Introduced in `v0.9.0`.

## Example

```lua
print(ptool.hash.sha256("hello"))
print(ptool.hash.blake3("hello"))
print(ptool.hash.crc32("hello"))
print(ptool.hash.xxh3_64("hello"))
```
