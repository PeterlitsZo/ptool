# Getting Started

`ptool` runs Lua scripts and injects a standard library for practical
automation.

The main entrypoint today is:

```sh
ptool run <file>
```

For `.lua` files, you can also use the shortcut form:

```sh
ptool <file.lua>
```

For interactive exploration, `ptool` also provides:

```sh
ptool repl
```

When a script runs, `ptool` exposes its API through the global `ptool` table
and the shorter alias `p`.

## Install

On Linux and macOS, install `ptool` with the release installer:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash
```

The installer downloads the latest prebuilt release for the current platform,
installs `ptool` to `~/.local/bin/ptool`, and prints a PATH hint if needed.

To install a specific release tag instead of the latest stable release:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- v0.2.0
```

To install into a custom binary directory instead of `~/.local/bin`:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- --bin-dir "$HOME/.cargo/bin"
```

## Minimal script

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

`ptool.use(...)` declares the minimum required `ptool` version for the script.
This keeps scripts explicit about the API version they expect and fails early
on older runtimes. See [Core Lua API](./lua-api/core.md) for details.

Run it with:

```sh
ptool run script.lua
ptool script.lua
```

## Passing arguments

You can pass extra CLI arguments after the script path:

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

Parse them inside the script with `ptool.args.parse(...)`.

## Shebang scripts

`ptool` supports shebang files. With the `.lua` CLI shortcut, a script can
start with:

```text
#!/usr/bin/env ptool
```

This lets you execute the script directly once it has the executable bit.

## What you get

- A script runner that understands shebang files.
- An interactive REPL for trying Lua expressions and `ptool` APIs directly.
- Lua helpers for semver, datetimes, paths, files, TOML, regexes, strings,
  HTTP, SSH, databases, and templates.
- CLI-oriented helpers for running commands, parsing arguments, and asking for
  interactive input.

## Next steps

- Open [REPL](./repl.md) for interactive usage, multi-line input, and keyboard
  behavior.
- Use [Lua API Overview](./lua-api/index.md) to browse the core APIs and
  available modules.
- Start with [Core Lua API](./lua-api/core.md) for version gating, process
  execution, config, and script lifecycle helpers.
- Open a module page such as [Args API](./lua-api/args.md) when you need the
  detailed reference for a specific feature set.
