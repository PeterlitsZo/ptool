# API de hash

Las utilidades de hash están disponibles bajo `ptool.hash` y `p.hash`.

## ptool.hash.sha256

> `v0.2.0` - Introduced.

`ptool.hash.sha256(input)` devuelve el digest SHA-256 de una cadena Lua.

- `input` (string, obligatorio): La cadena Lua de entrada. El digest se calcula a partir de los bytes sin procesar de la cadena.
- Devuelve: `string` (digest hexadecimal en minúsculas).

```lua
print(ptool.hash.sha256("hello"))
-- 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

## ptool.hash.sha1

> `v0.2.0` - Introduced.

`ptool.hash.sha1(input)` devuelve el digest SHA-1 de una cadena Lua.

- `input` (string, obligatorio): La cadena Lua de entrada. El digest se calcula a partir de los bytes sin procesar de la cadena.
- Devuelve: `string` (digest hexadecimal en minúsculas).

```lua
print(ptool.hash.sha1("hello"))
-- aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
```

## ptool.hash.md5

> `v0.2.0` - Introduced.

`ptool.hash.md5(input)` devuelve el digest MD5 de una cadena Lua.

- `input` (string, obligatorio): La cadena Lua de entrada. El digest se calcula a partir de los bytes sin procesar de la cadena.
- Devuelve: `string` (digest hexadecimal en minúsculas).

```lua
print(ptool.hash.md5("hello"))
-- 5d41402abc4b2a76b9719d911017c592
```
