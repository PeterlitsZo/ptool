# API Zip

Las utilidades de compresión están disponibles bajo `ptool.zip` y `p.zip`.

`ptool.zip` trabaja con cadenas Lua sin procesar, por lo que puede usarse tanto con texto como con cargas binarias.

Nombres de formato admitidos:

- `gzip` y `gz`
- `zlib`
- `deflate`
- `bzip2` y `bz2`
- `xz`
- `zstd`, `zst` y `zstandard`

## ptool.zip.compress

> `v0.8.0` - Introduced.

`ptool.zip.compress(format, input)` comprime una cadena Lua con el formato solicitado.

- `format` (string, obligatorio): El nombre del formato de compresión.
- `input` (string, obligatorio): La cadena Lua de entrada. La compresión usa los bytes sin procesar de la cadena sin modificarlos.
- Devuelve: `string` (bytes comprimidos como una cadena Lua).

Comportamiento ante errores:

- Se genera un error si `format` no es un nombre de formato admitido.
- Se genera un error si `input` no es una cadena.
- Se genera un error si el codificador falla para el formato solicitado.

Ejemplo:

```lua
local payload = p.fs.read("report.txt")
local compressed = p.zip.compress("gzip", payload)

p.fs.write("report.txt.gz", compressed)
```

## ptool.zip.decompress

> `v0.8.0` - Introduced.

`ptool.zip.decompress(format, input)` descomprime una cadena Lua con el formato solicitado.

- `format` (string, obligatorio): El nombre del formato de compresión.
- `input` (string, obligatorio): La cadena Lua comprimida.
- Devuelve: `string` (bytes descomprimidos como una cadena Lua).

Comportamiento ante errores:

- Se genera un error si `format` no es un nombre de formato admitido.
- Se genera un error si `input` no es una cadena.
- Se genera un error si `input` no contiene datos válidos para el formato solicitado.

Ejemplo:

```lua
local compressed = p.fs.read("report.txt.gz")
local plain = p.zip.decompress("gzip", compressed)

print(plain)
```

Notas:

- `ptool.zip` no infiere formatos a partir de nombres de archivo. Pasa el formato explícitamente.
- `ptool.zip` opera sobre una única cadena de bytes y no expone APIs de entradas de archivos ZIP, como listar archivos dentro de un contenedor `.zip`.
