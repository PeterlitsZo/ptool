# Lua API Overview

`ptool` exposes a broad set of helpers through `ptool` and `p`.

## Core APIs

- [Core Lua API](./core.md): Script lifecycle, process execution, config, and
  terminal helpers.

## Modules

- [Args API](./args.md): Command-line argument schema parsing for Lua scripts.
- [SemVer API](./semver.md): Parse, compare, and bump semantic versions.
- [Hash API](./hash.md): Compute SHA-256, SHA-1, and MD5 digests.
- [Network API](./net.md): Parse URLs, IP addresses, and host-port pairs.
- [Platform API](./platform.md): Detect the current OS, architecture, and target
  triple.
- [ANSI API](./ansi.md): Build styled terminal output with ANSI escape
  sequences.
- [HTTP API](./http.md): Send HTTP requests and consume response bodies.
- [Database API](./db.md): Open database connections and run SQL queries.
- [SSH API](./ssh.md): Connect to remote hosts, run commands, and transfer
  files.
- [Path API](./path.md): Manipulate paths lexically without touching the
  filesystem.
- [TOML API](./toml.md): Parse, read, update, and remove TOML values.
- [Regex API](./re.md): Compile regexes and search, capture, replace, or split
  text.
- [String API](./str.md): Trim, split, join, replace, and format strings.
- [Filesystem API](./fs.md): Read, write, create, and glob filesystem paths.
- [Shell API](./sh.md): Split shell-like command lines into argument arrays.
- [Template API](./template.md): Render text templates from Lua data.

Use this page as the entrypoint, then jump to the module page you need for the
full function reference.
