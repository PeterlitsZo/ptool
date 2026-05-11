# Log API

`ptool.log` は `ptool.log` と `p.log` の両方で、レベル別の端末ログ
ヘルパーを公開します。

各ログ呼び出しは次の形式で 1 行を出力します。

```text
[2026-04-30 14:54:56] INFO message text
```

動作:

- タイムスタンプはローカル時刻の `YYYY-MM-DD HH:MM:SS` 形式です。
- レベル名は `TRACE`、`DEBUG`、`INFO`、`WARN`、`ERROR` の完全表記を
  使います。
- `ptool` が端末へ書き込んでいる場合は、自動的に色付き出力になります。
- `ptool.log.error(...)` は `stderr` に書き込み、それ以外のレベルは
  `stdout` に書き込みます。
- 複数引数は空白で連結されます。文字列以外の値は単一行の inspect 形式で
  レンダリングされます。

## ptool.log.trace

> `v0.4.0` - Introduced.

```lua
ptool.log.trace(...)
```

trace レベルのログ行を書き出します。

## ptool.log.debug

> `v0.4.0` - Introduced.

```lua
ptool.log.debug(...)
```

debug レベルのログ行を書き出します。

## ptool.log.info

> `v0.4.0` - Introduced.

```lua
ptool.log.info(...)
```

info レベルのログ行を書き出します。

## ptool.log.warn

> `v0.4.0` - Introduced.

```lua
ptool.log.warn(...)
```

warn レベルのログ行を書き出します。

## ptool.log.error

> `v0.4.0` - Introduced.

```lua
ptool.log.error(...)
```

error レベルのログ行を `stderr` に書き出します。

例:

```lua
p.log.info("hello", { answer = 42 })
p.log.warn("careful")
p.log.error("boom")
```

出力例:

```text
[2026-04-30 14:54:56] INFO hello { answer = 42 }
[2026-04-30 14:54:56] WARN careful
[2026-04-30 14:54:56] ERROR boom
```
