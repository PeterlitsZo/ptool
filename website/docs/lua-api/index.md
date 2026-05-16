# Lua API Overview

`ptool` exposes a broad set of helpers through `ptool` and `p`.

Modules are grouped by domain. Within each group, entries are listed in
alphabetical order.

## Runtime & Interaction

- [ANSI API](./ansi.md): Build styled terminal output with ANSI escape
  sequences.
- [Args API](./args.md): Command-line argument schema parsing for Lua scripts.
- [Core Lua API](./core.md): Script lifecycle, process execution, config, and
  terminal helpers.
- [Log API](./log.md): Write timestamped terminal logs with level-based output.
- [Shell API](./sh.md): Split shell-like command lines into argument arrays.
- [TUI API](./tui.md): Build simple terminal user interfaces with a structured
  view tree and event loop.

## Data & Text

- [DateTime API](./datetime.md): Parse, compare, format, and convert concrete
  datetimes with timezone support.
- [Hash API](./hash.md): Compute SHA-256, SHA-1, and MD5 digests.
- [JSON API](./json.md): Parse JSON text and stringify Lua values as JSON.
- [Regex API](./re.md): Compile regexes and search, capture, replace, or split
  text.
- [SemVer API](./semver.md): Parse, compare, and bump semantic versions.
- [String API](./str.md): Trim, split, join, replace, and format strings.
- [Table API](./tbl.md): Map, filter, and concatenate dense list tables.
- [Template API](./template.md): Render text templates from Lua data.
- [TOML API](./toml.md): Parse, stringify, read, update, and remove TOML values.
- [YAML API](./yaml.md): Parse YAML text, read nested values, and stringify Lua
  values as YAML.

## Filesystem & Platform

- [Filesystem API](./fs.md): Read, write, create, and glob filesystem paths.
- [OS API](./os.md): Read runtime environment variables and inspect host
  process details.
- [Path API](./path.md): Manipulate paths lexically without touching the
  filesystem.
- [Platform API](./platform.md): Detect the current OS, architecture, and target
  triple.

## Networking & Remote

- [HTTP API](./http.md): Send HTTP requests and consume response bodies.
- [Network API](./net.md): Parse URLs, IP addresses, and host-port pairs.
- [SSH API](./ssh.md): Connect to remote hosts, run commands, send HTTP
  requests from the remote host, and transfer files.

## Development & Storage

- [Database API](./db.md): Open database connections and run SQL queries.
- [Git API](./git.md): Open repositories, inspect status, and clone, fetch, or
  push through libgit2-backed handles.

Use this page as the entrypoint, then jump to the module page you need for the
full function reference.
