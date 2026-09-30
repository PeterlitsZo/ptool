# API Zip

As utilidades de compressão estão disponíveis em `ptool.zip` e `p.zip`.

`ptool.zip` trabalha com strings Lua brutas, então pode ser usado tanto com texto quanto com cargas binárias.

Nomes de formatos suportados:

- `gzip` e `gz`
- `zlib`
- `deflate`
- `bzip2` e `bz2`
- `xz`
- `zstd`, `zst` e `zstandard`

## ptool.zip.compress

> `v0.8.0` - Introduced.

`ptool.zip.compress(format, input)` comprime uma string Lua com o formato solicitado.

- `format` (string, obrigatório): O nome do formato de compressão.
- `input` (string, obrigatório): A string Lua de entrada. A compressão usa os bytes brutos da string sem modificá-los.
- Retorna: `string` (bytes comprimidos como uma string Lua).

Comportamento em caso de erro:

- Um erro é gerado se `format` não for um nome de formato suportado.
- Um erro é gerado se `input` não for uma string.
- Um erro é gerado se o codificador falhar para o formato solicitado.

Exemplo:

```lua
local payload = p.fs.read("report.txt")
local compressed = p.zip.compress("gzip", payload)

p.fs.write("report.txt.gz", compressed)
```

## ptool.zip.decompress

> `v0.8.0` - Introduced.

`ptool.zip.decompress(format, input)` descomprime uma string Lua com o formato solicitado.

- `format` (string, obrigatório): O nome do formato de compressão.
- `input` (string, obrigatório): A string Lua comprimida.
- Retorna: `string` (bytes descomprimidos como uma string Lua).

Comportamento em caso de erro:

- Um erro é gerado se `format` não for um nome de formato suportado.
- Um erro é gerado se `input` não for uma string.
- Um erro é gerado se `input` não contiver dados válidos para o formato solicitado.

Exemplo:

```lua
local compressed = p.fs.read("report.txt.gz")
local plain = p.zip.decompress("gzip", compressed)

print(plain)
```

Notas:

- `ptool.zip` não infere formatos a partir de nomes de arquivos. Passe o formato explicitamente.
- `ptool.zip` opera sobre uma única string de bytes e não expõe APIs de entrada de arquivo ZIP, como listar arquivos dentro de um contêiner `.zip`.
