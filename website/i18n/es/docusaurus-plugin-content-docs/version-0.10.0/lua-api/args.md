# API de argumentos

Las utilidades de esquema y análisis de argumentos CLI están disponibles bajo `ptool.args` y `p.args`.

## ptool.args.arg

> `v0.1.0` - Introducido.

`ptool.args.arg(id, kind, options)` crea un builder de argumentos para usar en `ptool.args.parse(...).schema.args`.

- `id` (string, obligatorio): El identificador del argumento. También es la clave en la tabla devuelta.
- `kind` (string, obligatorio): El tipo de argumento. Valores admitidos:
  - `"flag"`: Un flag booleano.
  - `"string"`: Una opción de cadena.
  - `"int"`: Una opción entera (`i64`).
  - `"positional"`: Un argumento posicional.
- `options` (table, opcional): Los mismos campos opcionales admitidos por las tablas de argumentos en `ptool.args.parse`, como `long`, `short`, `help`, `required`, `multiple` y `default`.

El builder admite métodos encadenables; todos devuelven el propio builder:

- `arg:long(value)` establece el nombre de opción larga. Solo se admite en argumentos no `positional`.
- `arg:short(value)` establece el nombre de opción corta. Solo se admite en argumentos no `positional`.
- `arg:help(value)` establece el texto de ayuda.
- `arg:required(value)` establece si el argumento es obligatorio. Si se omite `value`, el valor por defecto es `true`.
- `arg:multiple(value)` establece si el argumento puede repetirse. Si se omite `value`, el valor por defecto es `true`.
- `arg:default(value)` establece el valor por defecto. Si `value = nil`, se limpia el valor por defecto.

Ejemplo:

```lua
local res = ptool.args.parse({
  args = {
    ptool.args.arg("name", "string"):required(),
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("paths", "positional"):multiple(),
  }
})
```

## ptool.args.parse

> `v0.1.0` - Introducido.
> 
> `v0.3.0`: se agregó compatibilidad con `subcommands`.

`ptool.args.parse(schema)` analiza los argumentos del script con `clap` y devuelve una tabla indexada por `id`.

Los argumentos del script provienen de la parte posterior a `--` en `ptool run <lua_file> -- ...`.

Por ejemplo:

```lua
ptool.use("v0.1.0")

local res = ptool.args.parse({
    name = "test",
    about = "The test command",
    args = {
        { id = "name", kind = "string" }
    }
})

print("Hello, " .. res.name .. "!")
```

### Estructura del esquema

- `name` (string, opcional): El nombre del comando, usado en la salida de ayuda. Por defecto es el nombre del archivo del script.
- `about` (string, opcional): Descripción de ayuda.
- `args` (table, opcional): Un arreglo de definiciones de argumentos. Cada elemento admite dos formas:
  - Una tabla de argumento.
  - Un objeto builder devuelto por `ptool.args.arg(...)`.
- `subcommands` (table, opcional): Un mapa desde nombre de subcomando hasta esquema de subcomando. Cada esquema de subcomando admite `about`, `args` y `subcommands` de forma recursiva.

Debe proporcionarse al menos uno entre `args` o `subcommands`.

Campos de la tabla de argumento:

- `id` (string, obligatorio): El identificador del argumento. También es la clave en la tabla devuelta.
- `kind` (string, obligatorio): El tipo de argumento. Valores admitidos:
  - `"flag"`: Un flag booleano.
  - `"string"`: Una opción de cadena.
  - `"int"`: Una opción entera (`i64`).
  - `"positional"`: Un argumento posicional.
- `long` (string, opcional): El nombre de la opción larga, como `"name"` para `--name`. En argumentos no `positional`, el valor por defecto puede derivarse de `id`.
- `short` (string, opcional): El nombre de la opción corta, un solo carácter como `"v"` para `-v`.
- `help` (string, opcional): Texto de ayuda del argumento.
- `required` (boolean, opcional): Si el argumento es obligatorio. Por defecto es `false`.
- `multiple` (boolean, opcional): Si el argumento puede repetirse. Por defecto es `false`.
- `default` (string/integer, opcional): El valor por defecto.

Cuando `subcommands` está presente, `args` del comando actual actúa como opciones compartidas para ese árbol de comandos, y se aceptan antes o después del subcomando seleccionado.

Ejemplo con subcomandos:

```lua
local res = ptool.args.parse({
  name = "demo",
  args = {
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("config", "string"),
  },
  subcommands = {
    build = {
      args = {
        ptool.args.arg("release", "flag"),
      },
      subcommands = {
        web = {
          args = {
            ptool.args.arg("out", "string"):required(),
          },
        },
      },
    },
    clean = {
      args = {
        ptool.args.arg("all", "flag"),
      },
    },
  },
})
```

### Restricciones

- Las siguientes restricciones se aplican tanto a tablas de argumentos como a la sintaxis builder.
- Los argumentos no `positional` pueden omitir `long` y `short`. Si se omite `long`, se usa `id` automáticamente.
- Los argumentos `positional` no pueden definir `long`, `short` ni `default`.
- Cuando `positional.multiple = true`, debe ser el último argumento en `args`.
- `multiple = true` solo se admite para `string` y `positional`.
- `default` solo se admite para `string` e `int`, y no puede usarse junto con `multiple = true`.
- Cuando `subcommands` está presente, no se permiten argumentos `positional` en ese mismo esquema.
- Cuando `subcommands` está presente en el nivel superior, los ids de argumento `command_path` y `args` quedan reservados.
- A lo largo de una misma ruta de subcomando seleccionada, los subcomandos ancestros y descendientes no pueden reutilizar el mismo `id` de argumento, porque sus valores se fusionan en una sola tabla `args`.

### Valor devuelto

Se devuelve una tabla Lua donde las claves son `id` y los tipos de valor son:

- `flag` -> `boolean`
- `string` -> `string` (o `string[]` cuando `multiple = true`)
- `int` -> `integer`
- `positional` -> `string` (o `string[]` cuando `multiple = true`)

Cuando `subcommands` no está presente, el valor devuelto permanece plano como arriba.

Cuando `subcommands` está presente, el valor devuelto tiene esta forma:

- Los valores de `args` del nivel superior se devuelven directamente en la tabla de nivel superior.
- `command_path` -> `string[]`: La ruta de subcomando coincidente, por ejemplo `{"build", "web"}`.
- `args` -> `table`: Los valores de argumentos fusionados de la ruta de subcomando coincidente.

Por ejemplo:

```lua
{
  verbose = true,
  config = "cfg.toml",
  command_path = { "build", "web" },
  args = {
    release = true,
    out = "dist",
  },
}
```
