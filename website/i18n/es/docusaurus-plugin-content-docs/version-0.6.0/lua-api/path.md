# API de rutas

Las utilidades léxicas de rutas están disponibles bajo `ptool.path` y `p.path`.

## ptool.path.join

> `v0.1.0` - Introduced.

`ptool.path.join(...segments)` une varios segmentos de ruta y devuelve la ruta normalizada.

- `segments` (string, al menos uno): Segmentos de ruta.
- Devuelve: `string`.

Ejemplo:

```lua
print(ptool.path.join("tmp", "a", "..", "b")) -- tmp/b
```

## ptool.path.normalize

> `v0.1.0` - Introduced.

`ptool.path.normalize(path)` realiza normalización léxica de rutas (procesamiento de `.` y `..`).

- `path` (string, obligatorio): La ruta de entrada.
- Devuelve: `string`.

Ejemplo:

```lua
print(ptool.path.normalize("./a/../b")) -- b
```

## ptool.path.abspath

> `v0.1.0` - Introduced.

`ptool.path.abspath(path[, base])` calcula una ruta absoluta.

- `path` (string, obligatorio): La ruta de entrada.
- `base` (string, opcional): El directorio base. Si se omite, se usa el directorio de trabajo del proceso actual.
- Devuelve: `string`.
- Solo acepta 1 o 2 argumentos de tipo string.

Ejemplo:

```lua
print(ptool.path.abspath("src"))
print(ptool.path.abspath("lib", "/tmp/demo"))
```

## ptool.path.relpath

> `v0.1.0` - Introduced.

`ptool.path.relpath(path[, base])` calcula una ruta relativa desde `base` hasta `path`.

- `path` (string, obligatorio): La ruta objetivo.
- `base` (string, opcional): El directorio inicial. Si se omite, se usa el directorio de trabajo del proceso actual.
- Devuelve: `string`.
- Solo acepta 1 o 2 argumentos de tipo string.

Ejemplo:

```lua
print(ptool.path.relpath("src/main.rs", "/tmp/project"))
```

## ptool.path.isabs

> `v0.1.0` - Introduced.

`ptool.path.isabs(path)` comprueba si una ruta es absoluta.

- `path` (string, obligatorio): La ruta de entrada.
- Devuelve: `boolean`.

Ejemplo:

```lua
print(ptool.path.isabs("/tmp")) -- true
```

## ptool.path.dirname

> `v0.1.0` - Introduced.

`ptool.path.dirname(path)` devuelve la parte de nombre de directorio.

- `path` (string, obligatorio): La ruta de entrada.
- Devuelve: `string`.

Ejemplo:

```lua
print(ptool.path.dirname("a/b/c.txt")) -- a/b
```

## ptool.path.basename

> `v0.1.0` - Introduced.

`ptool.path.basename(path)` devuelve el último segmento de la ruta (la parte del nombre de archivo).

- `path` (string, obligatorio): La ruta de entrada.
- Devuelve: `string`.

Ejemplo:

```lua
print(ptool.path.basename("a/b/c.txt")) -- c.txt
```

## ptool.path.extname

> `v0.1.0` - Introduced.

`ptool.path.extname(path)` devuelve la extensión, incluido el `.`. Si no hay extensión, devuelve una cadena vacía.

- `path` (string, obligatorio): La ruta de entrada.
- Devuelve: `string`.

Ejemplo:

```lua
print(ptool.path.extname("a/b/c.txt")) -- .txt
```

Notas:

- El manejo de rutas en `ptool.path` es puramente léxico. No comprueba si las rutas existen ni resuelve enlaces simbólicos.
- Ninguna de las interfaces acepta argumentos de cadena vacía. Pasar uno produce un error.
