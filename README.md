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

Install into a custom binary directory:

```bash
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- --bin-dir "$HOME/.cargo/bin"
```

## Usage

```bash
ptool run <file>
```

For `.lua` files, you can also omit `run`:

```bash
ptool <file.lua>
```

Minimal script:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

`ptool.use(...)` declares the required `ptool` version or version requirement
for the script, so it fails early on incompatible runtimes instead of running
with a missing API. It accepts plain versions such as `v0.1.0` and Cargo-style
requirements such as `^0.6.0` or `>= v0.6.0, < 0.7.0`.

Useful runtime helpers include `p.os.getenv(...)`, `p.os.setenv(...)`,
`p.os.unsetenv(...)`, `p.os.homedir()`, and `p.os.tmpdir()`.

See [Getting Started][1] and [Lua API Overview][2] for the full documentation.

[1]: https://ptool.peterlits.net/docs/intro
[2]: https://ptool.peterlits.net/docs/lua-api/
