# API de hash

As utilidades de hash estão disponíveis em `ptool.hash` e `p.hash`.

## ptool.hash.sha256

> `v0.2.0` - Introduced.

`ptool.hash.sha256(input)` retorna o digest SHA-256 de uma string Lua.

- `input` (string, obrigatório): A string Lua de entrada. O digest é calculado
  a partir dos bytes brutos da string.
- Retorna: `string` (digest hexadecimal em minúsculas).

```lua
print(ptool.hash.sha256("hello"))
-- 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
```

## ptool.hash.sha1

> `v0.2.0` - Introduced.

`ptool.hash.sha1(input)` retorna o digest SHA-1 de uma string Lua.

- `input` (string, obrigatório): A string Lua de entrada. O digest é calculado
  a partir dos bytes brutos da string.
- Retorna: `string` (digest hexadecimal em minúsculas).

```lua
print(ptool.hash.sha1("hello"))
-- aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d
```

## ptool.hash.md5

> `v0.2.0` - Introduced.

`ptool.hash.md5(input)` retorna o digest MD5 de uma string Lua.

- `input` (string, obrigatório): A string Lua de entrada. O digest é calculado
  a partir dos bytes brutos da string.
- Retorna: `string` (digest hexadecimal em minúsculas).

```lua
print(ptool.hash.md5("hello"))
-- 5d41402abc4b2a76b9719d911017c592
```
