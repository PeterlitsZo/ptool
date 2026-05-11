# API de log

`ptool.log` expone utilidades de logging por nivel de salida tanto en `ptool.log` como en `p.log`.

Cada llamada de log genera una línea con este formato:

```text
[2026-04-30 14:54:56] INFO message text
```

Comportamiento:

- La marca de tiempo usa la hora local con el formato `YYYY-MM-DD HH:MM:SS`.
- La etiqueta de nivel usa los nombres completos `TRACE`, `DEBUG`, `INFO`, `WARN` y `ERROR`.
- La salida coloreada se habilita automáticamente cuando `ptool` escribe en una terminal.
- `ptool.log.error(...)` escribe en `stderr`. Los demás niveles escriben en `stdout`.
- Varios argumentos se unen con espacios. Los valores no string se renderizan en una forma inspect de una sola línea.

## ptool.log.trace

> `v0.4.0` - Introduced.

```lua
ptool.log.trace(...)
```

Escribe una línea de log de nivel trace.

## ptool.log.debug

> `v0.4.0` - Introduced.

```lua
ptool.log.debug(...)
```

Escribe una línea de log de nivel debug.

## ptool.log.info

> `v0.4.0` - Introduced.

```lua
ptool.log.info(...)
```

Escribe una línea de log de nivel info.

## ptool.log.warn

> `v0.4.0` - Introduced.

```lua
ptool.log.warn(...)
```

Escribe una línea de log de nivel warn.

## ptool.log.error

> `v0.4.0` - Introduced.

```lua
ptool.log.error(...)
```

Escribe una línea de log de nivel error en `stderr`.

Ejemplo:

```lua
p.log.info("hello", { answer = 42 })
p.log.warn("careful")
p.log.error("boom")
```

Salida de ejemplo:

```text
[2026-04-30 14:54:56] INFO hello { answer = 42 }
[2026-04-30 14:54:56] WARN careful
[2026-04-30 14:54:56] ERROR boom
```
