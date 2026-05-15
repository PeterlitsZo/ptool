# Primeros pasos

`ptool` ejecuta scripts Lua e inyecta una biblioteca estándar orientada a la automatización práctica.

El punto de entrada principal hoy es:

```sh
ptool run <file>
```

Para archivos `.lua`, también puedes usar la forma abreviada:

```sh
ptool <file.lua>
```

Para explorar de forma interactiva, `ptool` también ofrece:

```sh
ptool repl
```

Cuando se ejecuta un script, `ptool` expone su API a través de la tabla global `ptool` y del alias corto `p`.

## Instalación

En Linux y macOS, instala `ptool` con el instalador de lanzamientos:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash
```

El instalador descarga la versión precompilada más reciente para la plataforma actual, instala `ptool` en `~/.local/bin/ptool` y muestra una sugerencia de PATH si hace falta.

Para instalar una etiqueta de lanzamiento concreta en lugar de la última versión estable:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- v0.2.0
```

Para instalarlo en un directorio binario personalizado en lugar de `~/.local/bin`:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- --bin-dir "$HOME/.cargo/bin"
```

## Script mínimo

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

`ptool.use(...)` declara la versión mínima de `ptool` requerida por el script. Así el script deja explícita la versión de la API que espera y falla pronto en runtimes antiguos. Consulta [API principal de Lua](./lua-api/core.md) para más detalles.

Ejecútalo con:

```sh
ptool run script.lua
ptool script.lua
```

## Paso de argumentos

Puedes pasar argumentos extra de CLI después de la ruta del script:

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

Después puedes analizarlos dentro del script con `ptool.args.parse(...)`.

## Scripts con shebang

`ptool` admite archivos con shebang. Con la forma abreviada para `.lua`, un script puede empezar con:

```text
#!/usr/bin/env ptool
```

Esto te permite ejecutar el script directamente una vez que tenga el bit ejecutable.

## Qué obtienes

- Un ejecutor de scripts que entiende archivos con shebang.
- Un REPL interactivo para probar expresiones de Lua y APIs de `ptool` directamente.
- Ayudantes de Lua para semver, fechas y horas, rutas, archivos, TOML, expresiones regulares, cadenas, HTTP, SSH, bases de datos y plantillas.
- Utilidades orientadas a CLI para ejecutar comandos, analizar argumentos y pedir entrada interactiva.

## Siguientes pasos

- Abre [REPL](./repl.md) para aprender el uso interactivo, la entrada multilínea y el comportamiento del teclado.
- Usa [Resumen de la API de Lua](./lua-api/index.md) para recorrer las APIs principales y los módulos disponibles.
- Empieza por [API principal de Lua](./lua-api/core.md) para conocer el control de versión, la ejecución de procesos, la configuración y las utilidades del ciclo de vida del script.
- Abre una página de módulo como [API de argumentos](./lua-api/args.md) cuando necesites la referencia detallada de una capacidad concreta.
