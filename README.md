# ptool

`ptool` is a tool for running Lua scripts with a bundled set of utilities.

## Install

```bash
curl -fsSL https://peterlits.net/ptool/install.sh | bash
```

Install a specific release tag:

```bash
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- v0.2.0
```

## Usage

```bash
ptool run <file>
```

Minimal script:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

`ptool.use(...)` declares the minimum required `ptool` version for the script,
so the script fails early on older runtimes instead of running with a missing
API.

See [Getting Started][1] and [Lua API Overview][2] for the full documentation.

[1]: https://ptool.peterlits.net/docs/intro
[2]: https://ptool.peterlits.net/docs/lua-api/
