# Getting Started

Make sure the `ptool` binary is available in your shell, then create a Lua
script and run it with `ptool run`.

## Minimal script

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

Run it with:

```sh
ptool run script.lua
```

## Passing arguments

You can pass extra CLI arguments after the script path:

```sh
ptool run script.lua --name alice -v a.txt b.txt
```

Parse them inside the script with `ptool.args.parse(...)`.

## Shebang scripts

`ptool` supports shebang files, so a script can start with:

```text
#!/usr/bin/env ptool run
```

This lets you execute the script directly once it has the executable bit.

## Next steps

- For an overview of the core APIs and available modules, see [Lua API Overview](./lua-api/index.md).
- For detailed API docs, open [Core Lua API](./lua-api/core.md) or the module page you need, such as [Args API](./lua-api/args.md).
