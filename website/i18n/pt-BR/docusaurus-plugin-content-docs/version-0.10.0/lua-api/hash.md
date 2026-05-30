# API de hash

As utilidades de hash estão disponíveis em `ptool.hash` e `p.hash`.

Todas as utilidades de hash seguem o mesmo formato:

- `input` (string, obrigatório): A string Lua de entrada. O digest ou checksum é calculado a partir dos bytes brutos da string.
- Retorna: `string` (saída hexadecimal em minúsculas).

## Digests criptográficos

- `ptool.hash.sha1(input)`: Digest SHA-1. Introduzido em `v0.2.0`.
- `ptool.hash.sha224(input)`: Digest SHA-224. Introduzido em `v0.9.0`.
- `ptool.hash.sha256(input)`: Digest SHA-256. Introduzido em `v0.2.0`.
- `ptool.hash.sha384(input)`: Digest SHA-384. Introduzido em `v0.9.0`.
- `ptool.hash.sha512(input)`: Digest SHA-512. Introduzido em `v0.9.0`.
- `ptool.hash.sha512_224(input)`: Digest SHA-512/224. Introduzido em `v0.9.0`.
- `ptool.hash.sha512_256(input)`: Digest SHA-512/256. Introduzido em `v0.9.0`.
- `ptool.hash.sha3_224(input)`: Digest SHA3-224. Introduzido em `v0.9.0`.
- `ptool.hash.sha3_256(input)`: Digest SHA3-256. Introduzido em `v0.9.0`.
- `ptool.hash.sha3_384(input)`: Digest SHA3-384. Introduzido em `v0.9.0`.
- `ptool.hash.sha3_512(input)`: Digest SHA3-512. Introduzido em `v0.9.0`.
- `ptool.hash.blake2s256(input)`: Digest BLAKE2s-256. Introduzido em `v0.9.0`.
- `ptool.hash.blake2b512(input)`: Digest BLAKE2b-512. Introduzido em `v0.9.0`.
- `ptool.hash.blake3(input)`: Digest BLAKE3. Introduzido em `v0.9.0`.
- `ptool.hash.md5(input)`: Digest MD5. Introduzido em `v0.2.0`.

## Checksums e hashes rápidos não criptográficos

- `ptool.hash.crc32(input)`: Checksum CRC-32/ISO-HDLC. Introduzido em `v0.9.0`.
- `ptool.hash.crc64(input)`: Checksum CRC-64/ECMA-182. Introduzido em `v0.9.0`.
- `ptool.hash.adler32(input)`: Checksum Adler-32. Introduzido em `v0.9.0`.
- `ptool.hash.xxh32(input)`: Digest xxHash32. Introduzido em `v0.9.0`.
- `ptool.hash.xxh64(input)`: Digest xxHash64. Introduzido em `v0.9.0`.
- `ptool.hash.xxh3_64(input)`: Digest XXH3 de 64 bits. Introduzido em `v0.9.0`.
- `ptool.hash.xxh3_128(input)`: Digest XXH3 de 128 bits. Introduzido em `v0.9.0`.
- `ptool.hash.murmur3_32(input)`: Digest MurmurHash3 de 32 bits. Introduzido em `v0.9.0`.
- `ptool.hash.murmur3_128(input)`: Digest MurmurHash3 x64 de 128 bits. Introduzido em `v0.9.0`.
- `ptool.hash.fnv1a32(input)`: Digest FNV-1a de 32 bits. Introduzido em `v0.9.0`.
- `ptool.hash.fnv1a64(input)`: Digest FNV-1a de 64 bits. Introduzido em `v0.9.0`.

## Exemplo

```lua
print(ptool.hash.sha256("hello"))
print(ptool.hash.blake3("hello"))
print(ptool.hash.crc32("hello"))
print(ptool.hash.xxh3_64("hello"))
```
