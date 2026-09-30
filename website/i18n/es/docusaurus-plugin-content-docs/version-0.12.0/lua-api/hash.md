# API de hash

Las utilidades de hash están disponibles bajo `ptool.hash` y `p.hash`.

Todas las utilidades de hash comparten la misma forma:

- `input` (string, obligatorio): La cadena Lua de entrada. El digest o la suma de comprobación se calcula a partir de los bytes sin procesar de la cadena.
- Devuelve: `string` (salida hexadecimal en minúsculas).

## Digests criptográficos

- `ptool.hash.sha1(input)`: Digest SHA-1. Introducido en `v0.2.0`.
- `ptool.hash.sha224(input)`: Digest SHA-224. Introducido en `v0.9.0`.
- `ptool.hash.sha256(input)`: Digest SHA-256. Introducido en `v0.2.0`.
- `ptool.hash.sha384(input)`: Digest SHA-384. Introducido en `v0.9.0`.
- `ptool.hash.sha512(input)`: Digest SHA-512. Introducido en `v0.9.0`.
- `ptool.hash.sha512_224(input)`: Digest SHA-512/224. Introducido en `v0.9.0`.
- `ptool.hash.sha512_256(input)`: Digest SHA-512/256. Introducido en `v0.9.0`.
- `ptool.hash.sha3_224(input)`: Digest SHA3-224. Introducido en `v0.9.0`.
- `ptool.hash.sha3_256(input)`: Digest SHA3-256. Introducido en `v0.9.0`.
- `ptool.hash.sha3_384(input)`: Digest SHA3-384. Introducido en `v0.9.0`.
- `ptool.hash.sha3_512(input)`: Digest SHA3-512. Introducido en `v0.9.0`.
- `ptool.hash.blake2s256(input)`: Digest BLAKE2s-256. Introducido en `v0.9.0`.
- `ptool.hash.blake2b512(input)`: Digest BLAKE2b-512. Introducido en `v0.9.0`.
- `ptool.hash.blake3(input)`: Digest BLAKE3. Introducido en `v0.9.0`.
- `ptool.hash.md5(input)`: Digest MD5. Introducido en `v0.2.0`.

## Sumas de comprobación y hashes rápidos no criptográficos

- `ptool.hash.crc32(input)`: Suma de comprobación CRC-32/ISO-HDLC. Introducido en `v0.9.0`.
- `ptool.hash.crc64(input)`: Suma de comprobación CRC-64/ECMA-182. Introducido en `v0.9.0`.
- `ptool.hash.adler32(input)`: Suma de comprobación Adler-32. Introducido en `v0.9.0`.
- `ptool.hash.xxh32(input)`: Digest xxHash32. Introducido en `v0.9.0`.
- `ptool.hash.xxh64(input)`: Digest xxHash64. Introducido en `v0.9.0`.
- `ptool.hash.xxh3_64(input)`: Digest XXH3 de 64 bits. Introducido en `v0.9.0`.
- `ptool.hash.xxh3_128(input)`: Digest XXH3 de 128 bits. Introducido en `v0.9.0`.
- `ptool.hash.murmur3_32(input)`: Digest MurmurHash3 de 32 bits. Introducido en `v0.9.0`.
- `ptool.hash.murmur3_128(input)`: Digest MurmurHash3 x64 de 128 bits. Introducido en `v0.9.0`.
- `ptool.hash.fnv1a32(input)`: Digest FNV-1a de 32 bits. Introducido en `v0.9.0`.
- `ptool.hash.fnv1a64(input)`: Digest FNV-1a de 64 bits. Introducido en `v0.9.0`.

## Ejemplo

```lua
print(ptool.hash.sha256("hello"))
print(ptool.hash.blake3("hello"))
print(ptool.hash.crc32("hello"))
print(ptool.hash.xxh3_64("hello"))
```
