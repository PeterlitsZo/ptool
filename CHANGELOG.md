# Changelog

## Unreleased

### Added

- Added `ptool.db` powered by `sqlx`, with synchronous Lua-facing APIs for
  connecting to SQLite, PostgreSQL, and MySQL, plus query helpers such as
  `query`, `query_one`, `scalar`, `execute`, and callback-based transactions.
- Added `ptool.template.render(template, context)` powered by MiniJinja for
  Jinja-style string template rendering.
- Added `ptool.cd(path)` to change `ptool` runtime working directory for
  subsequent operations such as `ptool.run` and `ptool.path` path resolution.
- Added `ptool.ssh`, including `connect`, connection metadata fields, remote
  command execution via `Connection:run(...)`, and explicit connection teardown
  with `Connection:close()`, with scripting documentation for SSH targets,
  authentication, host-key policies, and stream handling.

### Changed

- Split the project into a Cargo workspace with `crates/ptool` for the core
  library and `crates/ptool-cli` for the CLI, while keeping the `ptool`
  executable name and `ptool run` behavior unchanged.
- Centralized Lua API registration and runtime state in `LuaWorld`, making all
  exported `ptool` modules delegate through a single runtime object while
  keeping the default working-directory behavior unchanged.

## v0.1.0-alpha.1 (2026-03-21)

### Added

- Initialized the `ptool` project and added support for running Lua scripts with
  the `run` command. It provides the following features:
  - Shebang support: if the first line of a script starts with `#!`, that line
    is automatically removed before execution.
  - Support for the `p` alias of `ptool`, so you can access the provided
    features through either name.
  - Support for `ptool.run`, allowing execution of external commands with
    configurable arguments, working directory, and environment variables.
  - Support for `ptool.use`, which declares a minimum supported version using
    SemVer and exits early with an error when the current version is too old.
  - Support for `ptool.semver`, including parsing, validation, comparison, and
    bump operations (`major`/`minor`/`patch`/`alpha`/`beta`/`rc`), plus a
    `Version` object for field access and operator comparisons.
  - Support for `ptool.unindent`, which removes the `| ` prefix from multi-line
    strings and trims leading and trailing blank lines.
  - Support for `ptool.config`, used to configure global options.
  - Support for `ptool.args.parse`, used to parse command-line arguments.
  - Support for builder-style argument definitions via `ptool.args.arg`, with
    chainable methods such as `required`.
  - Support for `ptool.sh.split`, used to split a command-line string into an
    argument list.
  - Support for `ptool.http.request`, used to send HTTP requests and consume the
    response body on demand in a fetch-like style (`text/json/bytes`).
  - Support for `ptool.fs`, providing four basic file system functions: `read`,
    `write`, `mkdir`, and `exists`.
  - Support for `ptool.path`, providing path utilities such as `join`,
    `normalize`, `abspath`, `relpath`, `isabs`, `dirname`, `basename`, and
    `extname`.
  - Support for `ptool.toml.parse`, used to parse a TOML string into a Lua
    table.
  - Support for TOML editing based on `toml_edit`, including `ptool.toml.get`,
    `ptool.toml.set`, and `ptool.toml.remove`.
  - Support for the `ptool.re` module, providing regex compilation/escaping and
    matching, capturing, replacing, and splitting.
  - Completed the `ptool.path` module implementation and added script
    documentation, providing purely lexical path handling.
