# API de log

`ptool.log` expõe utilitários de logging por nível tanto em `ptool.log` quanto em `p.log`.

Cada chamada de log gera uma linha neste formato:

```text
[2026-04-30 14:54:56] INFO message text
```

Comportamento:

- O timestamp usa a hora local no formato `YYYY-MM-DD HH:MM:SS`.
- O rótulo de nível usa os nomes completos `TRACE`, `DEBUG`, `INFO`, `WARN` e `ERROR`.
- A saída colorida é habilitada automaticamente quando o `ptool` está escrevendo em um terminal.
- `ptool.log.error(...)` escreve em `stderr`. Os demais níveis escrevem em `stdout`.
- Vários argumentos são unidos com espaços. Valores que não são string são renderizados em uma forma inspect de linha única.

## ptool.log.trace

> `v0.4.0` - Introduced.

```lua
ptool.log.trace(...)
```

Escreve uma linha de log no nível trace.

## ptool.log.debug

> `v0.4.0` - Introduced.

```lua
ptool.log.debug(...)
```

Escreve uma linha de log no nível debug.

## ptool.log.info

> `v0.4.0` - Introduced.

```lua
ptool.log.info(...)
```

Escreve uma linha de log no nível info.

## ptool.log.warn

> `v0.4.0` - Introduced.

```lua
ptool.log.warn(...)
```

Escreve uma linha de log no nível warn.

## ptool.log.error

> `v0.4.0` - Introduced.

```lua
ptool.log.error(...)
```

Escreve uma linha de log no nível error em `stderr`.

Exemplo:

```lua
p.log.info("hello", { answer = 42 })
p.log.warn("careful")
p.log.error("boom")
```

Saída de exemplo:

```text
[2026-04-30 14:54:56] INFO hello { answer = 42 }
[2026-04-30 14:54:56] WARN careful
[2026-04-30 14:54:56] ERROR boom
```
