# Log API

`ptool.log` exposes level-based terminal logging helpers under both
`ptool.log` and `p.log`.

Every log call renders a line in this format:

```text
[2026-04-30 14:54:56] INFO message text
```

Behavior:

- The timestamp uses local time in `YYYY-MM-DD HH:MM:SS` format.
- The level label uses the full names `TRACE`, `DEBUG`, `INFO`, `WARN`, and
  `ERROR`.
- Colored output is enabled automatically when `ptool` is writing to a
  terminal.
- `ptool.log.error(...)` writes to `stderr`. Other levels write to `stdout`.
- Multiple arguments are joined with spaces. Non-string values are rendered in
  a single-line inspected form.

## ptool.log.trace

> `v0.4.0` - Introduced.

```lua
ptool.log.trace(...)
```

Writes a trace-level log line.

## ptool.log.debug

> `v0.4.0` - Introduced.

```lua
ptool.log.debug(...)
```

Writes a debug-level log line.

## ptool.log.info

> `v0.4.0` - Introduced.

```lua
ptool.log.info(...)
```

Writes an info-level log line.

## ptool.log.warn

> `v0.4.0` - Introduced.

```lua
ptool.log.warn(...)
```

Writes a warning-level log line.

## ptool.log.error

> `v0.4.0` - Introduced.

```lua
ptool.log.error(...)
```

Writes an error-level log line to `stderr`.

Example:

```lua
p.log.info("hello", { answer = 42 })
p.log.warn("careful")
p.log.error("boom")
```

Example output:

```text
[2026-04-30 14:54:56] INFO hello { answer = 42 }
[2026-04-30 14:54:56] WARN careful
[2026-04-30 14:54:56] ERROR boom
```
