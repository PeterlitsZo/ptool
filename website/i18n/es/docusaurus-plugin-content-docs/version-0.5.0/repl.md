# REPL

`ptool repl` inicia una sesión interactiva de Lua con la API estándar de
`ptool` ya cargada.

## Iniciar el REPL

```sh
ptool repl
```

Cuando el REPL se inicia, `ptool` muestra un banner y espera entrada de Lua.

## Qué ofrece

- La tabla global `ptool` y el alias corto `p`.
- Las mismas utilidades integradas que puedes usar desde `ptool run <file>`.
- Evaluación interactiva de expresiones y sentencias Lua.
- Edición de estilo readline, incluido movimiento del cursor con las flechas e
  historial dentro de la sesión.

## Uso básico

Introduce una expresión para evaluarla de inmediato:

```lua
1 + 2
```

El REPL imprime los valores devueltos usando el mismo inspector que se usa en
otras partes de `ptool`.

También puedes llamar directamente a las APIs de `ptool`:

```lua
p.str.trim("  hello  ")
```

## Entrada multilínea

Si la entrada actual está incompleta, el prompt cambia de `>>> ` a `... `.
Esto te permite seguir escribiendo un bloque como una función o una sentencia
de control de flujo:

```lua
for i = 1, 3 do
  print(i)
end
```

Cuando la entrada está completa, `ptool` evalúa el bloque entero.

## Comportamiento del teclado

- `Up` y `Down` recorren los comandos introducidos antes en la misma sesión del
  REPL.
- `Left` y `Right` mueven el cursor dentro de la línea de entrada actual.
- `Ctrl-C` limpia la entrada actual. Si estás en medio de un bloque multilínea,
  descarta el bloque en búfer y vuelve al prompt principal.
- `Ctrl-D` sale del REPL.

## Notas

- `ptool repl` requiere un TTY interactivo.
- El historial del REPL por ahora solo vive durante la sesión actual y no se
  escribe en un archivo de historial.
