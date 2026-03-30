# Shell API

Shell parsing helpers are available under `ptool.sh` and `p.sh`.

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split` parses a command string using shell-style rules and returns an
argument array.

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

The `args` above is equivalent to:

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```
