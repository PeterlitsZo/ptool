# API de sistema de archivos

Las utilidades de sistema de archivos están disponibles bajo `ptool.fs` y
`p.fs`.

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` lee un archivo de texto UTF-8 y devuelve una cadena.

- `path` (string, obligatorio): La ruta del archivo.
- Devuelve: `string`.

Ejemplo:

```lua
local content = ptool.fs.read("README.md")
print(content)
```

## ptool.fs.write

> `v0.1.0` - Introduced.

`ptool.fs.write(path, content)` escribe una cadena en un archivo, sobrescribiendo
el contenido existente.

- `path` (string, obligatorio): La ruta del archivo.
- `content` (string, obligatorio): El contenido que se va a escribir.

Ejemplo:

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
```

## ptool.fs.mkdir

> `v0.1.0` - Introduced.

`ptool.fs.mkdir(path)` crea un directorio. Si los directorios padre no existen,
se crean recursivamente.

- `path` (string, obligatorio): La ruta del directorio.

Ejemplo:

```lua
ptool.fs.mkdir("tmp/a/b")
```

## ptool.fs.exists

> `v0.1.0` - Introduced.

`ptool.fs.exists(path)` comprueba si una ruta existe.

- `path` (string, obligatorio): Una ruta de archivo o directorio.
- Devuelve: `boolean`.

Ejemplo:

```lua
if ptool.fs.exists("tmp/hello.txt") then
  print("exists")
end
```

## ptool.fs.glob

> `v0.2.0` - Introduced.

`ptool.fs.glob(pattern)` encuentra rutas del sistema de archivos usando sintaxis
glob de estilo Unix y devuelve un arreglo de cadenas ordenado
lexicográficamente con las rutas coincidentes.

- `pattern` (string, obligatorio): Un patrón glob. Los patrones relativos se
  resuelven desde el directorio de ejecución actual de `ptool`, por lo que
  siguen a `ptool.cd(...)`.
- Devuelve: `string[]`.
- Los archivos y directorios ocultos solo coinciden cuando el componente del
  patrón correspondiente empieza explícitamente por `.`.

Ejemplo:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
```
