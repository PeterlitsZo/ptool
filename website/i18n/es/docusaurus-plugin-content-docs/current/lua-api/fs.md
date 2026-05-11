# API de sistema de archivos

Las utilidades de sistema de archivos están disponibles bajo `ptool.fs` y
`p.fs`.

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` lee un archivo como bytes sin procesar y devuelve una
cadena Lua.

- `path` (string, obligatorio): La ruta del archivo.
- Devuelve: `string`.

Notas:

- La cadena Lua devuelta contiene exactamente los bytes almacenados en disco.
- Los archivos de texto siguen funcionando como antes, y ahora también se
  admiten archivos binarios.

Ejemplo:

```lua
local content = ptool.fs.read("README.md")
print(content)

local png = ptool.fs.read("logo.png")
print(#png)
```

## ptool.fs.write

> `v0.1.0` - Introduced.

`ptool.fs.write(path, content)` escribe una cadena Lua en un archivo como bytes
sin procesar, sobrescribiendo el contenido existente.

- `path` (string, obligatorio): La ruta del archivo.
- `content` (string, obligatorio): El contenido que se va a escribir.

Notas:

- `content` se escribe byte por byte.
- Los bytes NUL embebidos y los bytes no UTF-8 se conservan.

Ejemplo:

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
ptool.fs.write("tmp/blob.bin", "\x00\xffABC")
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

## ptool.fs.is_file

> `Unreleased` - Introduced.

`ptool.fs.is_file(path)` comprueba si una ruta existe y es un archivo normal.

- `path` (string, obligatorio): La ruta que se va a comprobar.
- Devuelve: `boolean`.

Ejemplo:

```lua
if ptool.fs.is_file("tmp/hello.txt") then
  print("file")
end
```

## ptool.fs.is_dir

> `Unreleased` - Introduced.

`ptool.fs.is_dir(path)` comprueba si una ruta existe y es un directorio.

- `path` (string, obligatorio): La ruta que se va a comprobar.
- Devuelve: `boolean`.

Ejemplo:

```lua
if ptool.fs.is_dir("tmp") then
  print("dir")
end
```

## ptool.fs.remove

> `Unreleased` - Introduced.

`ptool.fs.remove(path[, options])` elimina un archivo, enlace simbólico o
directorio.

- `path` (string, obligatorio): La ruta que se va a eliminar.
- `options` (table, opcional): Opciones de eliminación. Campos admitidos:
  - `recursive` (boolean, opcional): Indica si los directorios deben
    eliminarse recursivamente. Valor predeterminado: `false`.
  - `missing_ok` (boolean, opcional): Indica si deben ignorarse las rutas
    ausentes. Valor predeterminado: `false`.

Comportamiento:

- Los archivos y enlaces simbólicos pueden eliminarse sin `recursive`.
- Los directorios necesitan `recursive = true` cuando no están vacíos.
- Los nombres de opción desconocidos o los tipos de valor inválidos producen
  un error.

Ejemplo:

```lua
ptool.fs.remove("tmp/hello.txt")
ptool.fs.remove("tmp/cache", { recursive = true })
ptool.fs.remove("tmp/missing.txt", { missing_ok = true })
```

## ptool.fs.glob

> `v0.2.0` - Introduced.
> `v0.5.0` - Added the `working_dir` option.

`ptool.fs.glob(pattern[, options])` encuentra rutas del sistema de archivos
usando sintaxis glob de estilo Unix y devuelve un arreglo de cadenas ordenado
lexicográficamente con las rutas coincidentes.

- `pattern` (string, obligatorio): Un patrón glob. Los patrones relativos se
  resuelven desde el directorio de ejecución actual de `ptool`, por lo que
  siguen a `ptool.cd(...)`.
- `options` (table, opcional): Opciones de glob. Campos admitidos:
  - `working_dir` (string, opcional): Sobrescribe el directorio base usado
    para resolver patrones relativos. Los valores relativos de `working_dir`
    se resuelven desde el directorio de ejecución actual de `ptool`.
- Devuelve: `string[]`.
- Los archivos y directorios ocultos solo coinciden cuando el componente del
  patrón correspondiente empieza explícitamente por `.`.

Ejemplo:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
local lua_scripts = ptool.fs.glob("**/*.lua", {
  working_dir = "../scripts",
})
```
