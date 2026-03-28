# ptool

`ptool` runs Lua scripts and injects a standard library for practical
automation.

The main entrypoint today is:

```sh
ptool run <file>
```

When a script runs, `ptool` exposes its API through the global `ptool` table
and the shorter alias `p`.

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

## What you get

- A script runner that understands shebang files.
- Lua helpers for semver, paths, files, TOML, regexes, strings, HTTP, SSH,
  databases, and templates.
- CLI-oriented helpers for running commands, parsing arguments, and asking for
  interactive input.

## Start here

- Read [Getting Started](./getting-started.md) for the basic scripting flow.
- Use [Lua API Overview](./lua-api/index.md) to see the available modules.
- Use [Lua API Reference](./lua-api/reference.md) when you need the full
  function reference.
