# API de sistema de archivos

Las utilidades de sistema de archivos están disponibles bajo `ptool.fs` y `p.fs`.

## ptool.fs.read

> `v0.1.0` - Introducido.

`ptool.fs.read(path)` lee un archivo como bytes sin procesar y devuelve una cadena Lua.

- `path` (string, obligatorio): La ruta del archivo.
- Devuelve: `string`.

Notas:

- La cadena Lua devuelta contiene exactamente los bytes almacenados en disco.
- Los archivos de texto siguen funcionando como antes, y ahora también se admiten archivos binarios.

Ejemplo:

```lua
local content = ptool.fs.read("README.md")
print(content)

local png = ptool.fs.read("logo.png")
print(#png)
```

## ptool.fs.write

> `v0.1.0` - Introducido.

`ptool.fs.write(path, content)` escribe una cadena Lua en un archivo como bytes sin procesar, sobrescribiendo el contenido existente.

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

## ptool.fs.append

> `Unreleased` - Introducido.

`ptool.fs.append(path, content)` añade una cadena Lua a un archivo como bytes sin procesar. Si el archivo no existe, se crea.

- `path` (string, obligatorio): La ruta del archivo.
- `content` (string, obligatorio): El contenido que se va a añadir.

Notas:

- `content` se escribe byte por byte al final del archivo.
- Los bytes NUL embebidos y los bytes no UTF-8 se conservan.

Ejemplo:

```lua
ptool.fs.append("tmp/log.txt", "first line\n")
ptool.fs.append("tmp/log.txt", "second line\n")
```

## ptool.fs.open

> `Unreleased` - Introducido.

`ptool.fs.open(path[, mode])` abre un archivo local y devuelve un objeto `File`.

Argumentos:

- `path` (string, obligatorio): La ruta del archivo.
- `mode` (string, opcional): El modo del archivo. Valor predeterminado: `"r"`.

Modos admitidos:

- `"r"`: Abre para lectura.
- `"w"`: Abre para escritura, trunca el contenido existente y crea el archivo cuando hace falta.
- `"a"`: Abre para anexar, creando el archivo cuando hace falta.
- `"r+"`: Abre para lectura y escritura sin truncar.
- `"w+"`: Abre para lectura y escritura, trunca el contenido existente y crea el archivo cuando hace falta.
- `"a+"`: Abre para lectura y anexado, creando el archivo cuando hace falta.

Notas:

- Los modos pueden incluir `b`, como `"rb"` o `"w+b"`.
- Las escrituras con `a` y `a+` siempre van al final del archivo.

Ejemplo:

```lua
local file = ptool.fs.open("tmp/log.txt", "a+")
file:write("hello\n")
file:flush()
file:close()
```

## File

> `Unreleased` - Introducido.

`File` representa un manejador de archivo local abierto devuelto por `ptool.fs.open()`.

Está implementado como un userdata de Lua.

Métodos:

- `file:read([n])` -> `string`
- `file:write(content)` -> `nil`
- `file:flush()` -> `nil`
- `file:seek([whence[, offset]])` -> `integer`
- `file:close()` -> `nil`

### read

> `Unreleased` - Introducido.

Nombre canónico de la API: `ptool.fs.File:read`.

`file:read([n])` lee bytes desde la posición actual del archivo y los devuelve como una cadena Lua.

- `n` (integer, opcional): El número máximo de bytes que se leen. Si se omite, lee desde la posición actual hasta EOF.
- Devuelve: `string`.

Comportamiento:

- Devuelve una cadena vacía en EOF.
- Lee bytes sin procesar, por lo que los datos binarios se conservan exactamente.

Ejemplo:

```lua
local file = ptool.fs.open("README.md")
local prefix = file:read(16)
local rest = file:read()
file:close()
```

### write

> `Unreleased` - Introducido.

Nombre canónico de la API: `ptool.fs.File:write`.

`file:write(content)` escribe una cadena Lua en la posición actual del archivo.

- `content` (string, obligatorio): Los bytes que se van a escribir.

Comportamiento:

- Escribe bytes sin procesar exactamente como se proporcionan.
- En los manejadores en modo append, las escrituras se agregan al final del archivo.

### flush

> `Unreleased` - Introducido.

Nombre canónico de la API: `ptool.fs.File:flush`.

`file:flush()` vacía en el SO las escrituras del archivo almacenadas en búfer.

### seek

> `Unreleased` - Introducido.

Nombre canónico de la API: `ptool.fs.File:seek`.

`file:seek([whence[, offset]])` mueve la posición actual del archivo.

- `whence` (string, opcional): Uno de `"set"`, `"cur"` o `"end"`. Valor predeterminado: `"cur"`.
- `offset` (integer, opcional): El desplazamiento en bytes relativo a `whence`. Valor predeterminado: `0`.
- Devuelve: `integer`.

Comportamiento:

- Devuelve la nueva posición absoluta del archivo.
- `"set"` requiere un `offset` no negativo.

Ejemplo:

```lua
local file = ptool.fs.open("tmp/data.bin", "w+")
file:write("abcdef")
file:seek("set", 2)
print(file:read(2)) -- cd
file:close()
```

### close

> `Unreleased` - Introducido.

Nombre canónico de la API: `ptool.fs.File:close`.

`file:close()` cierra el manejador del archivo.

Comportamiento:

- Después de cerrarlo, el manejador ya no puede usarse.

## ptool.fs.mkdir

> `v0.1.0` - Introducido.

`ptool.fs.mkdir(path)` crea un directorio. Si los directorios padre no existen, se crean recursivamente.

- `path` (string, obligatorio): La ruta del directorio.

Ejemplo:

```lua
ptool.fs.mkdir("tmp/a/b")
```

## ptool.fs.exists

> `v0.1.0` - Introducido.

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

> `Unreleased` - Introducido.

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

> `Unreleased` - Introducido.

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

> `Unreleased` - Introducido.

`ptool.fs.remove(path[, options])` elimina un archivo, enlace simbólico o directorio.

- `path` (string, obligatorio): La ruta que se va a eliminar.
- `options` (table, opcional): Opciones de eliminación. Campos admitidos:
  - `recursive` (boolean, opcional): Indica si los directorios deben eliminarse recursivamente. Valor predeterminado: `false`.
  - `missing_ok` (boolean, opcional): Indica si deben ignorarse las rutas ausentes. Valor predeterminado: `false`.

Comportamiento:

- Los archivos y enlaces simbólicos pueden eliminarse sin `recursive`.
- Los directorios necesitan `recursive = true` cuando no están vacíos.
- Los nombres de opción desconocidos o los tipos de valor inválidos producen un error.

Ejemplo:

```lua
ptool.fs.remove("tmp/hello.txt")
ptool.fs.remove("tmp/cache", { recursive = true })
ptool.fs.remove("tmp/missing.txt", { missing_ok = true })
```

## ptool.fs.glob

> `v0.2.0` - Introducido. `v0.5.0` - Se ha añadido la opción `working_dir`.

`ptool.fs.glob(pattern[, options])` encuentra rutas del sistema de archivos usando sintaxis glob de estilo Unix y devuelve un arreglo de cadenas ordenado lexicográficamente con las rutas coincidentes.

- `pattern` (string, obligatorio): Un patrón glob. Los patrones relativos se resuelven desde el directorio de ejecución actual de `ptool`, por lo que siguen a `ptool.cd(...)`.
- `options` (table, opcional): Opciones de glob. Campos admitidos:
  - `working_dir` (string, opcional): Sobrescribe el directorio base usado para resolver patrones relativos. Los valores relativos de `working_dir` se resuelven desde el directorio de ejecución actual de `ptool`.
- Devuelve: `string[]`.
- Los archivos y directorios ocultos solo coinciden cuando el componente del patrón correspondiente empieza explícitamente por `.`.

Ejemplo:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
local lua_scripts = ptool.fs.glob("**/*.lua", {
  working_dir = "../scripts",
})
```
