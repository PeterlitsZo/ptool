# Changelog

## Unreleased

- Fixed `ptool repl` so top-level `local` bindings now remain available across
  later REPL inputs within the same session.
- Added `ptool.try(...)` for structured Lua-side error handling, and changed
  command and runtime failures across `ptool.run`, `assert_ok()`, REPL, and the
  CLI to carry richer error metadata such as `kind`, `op`, `path`, `cmd`, and
  `status`.
- Changed `ptool.fs.read(...)` and `ptool.fs.write(...)` to operate on raw byte
  strings, so Lua scripts can read and write binary files without separate
  byte-specific APIs, and synchronized the filesystem docs across all supported
  locales.
- Enhanced `p.http.request(...)` with query/form/JSON request helpers, redirect
  and auth options, richer response header APIs, cached response bodies, and
  synchronized HTTP documentation across all supported locales.
- Added Spanish, Brazilian Portuguese, and Japanese i18n support for the
  Docusaurus docs site, including localized theme strings, homepage text, and
  translated documentation pages.
- Added `ptool.json.parse(...)` and `ptool.json.stringify(...)` for JSON parsing
  and serialization in Lua scripts, with English and zh-Hans API docs.
- Added readline-style line editing to `ptool repl`, including arrow-key cursor
  movement, in-session history navigation, multi-line input reset on `Ctrl-C`,
  and cleaner interactive Lua experimentation.
- Changed `ptool.run`, `ptool.run_capture`, and SSH command echo output to show
  the current `user@host` together with the relevant working directory.

## v0.3.0 (2026-04-15)

### Added

- Added zh-Hans i18n support for the Docusaurus docs site, including a locale
  switcher, a translated homepage, and localized Chinese documentation pages.
- Added `ptool.ssh.Connection:run_capture`, which mirrors
  `Connection:run(...)` while defaulting both `stdout` and `stderr` to
  captured output, and documented the new SSH Lua API.
- Added subcommand support to `ptool.args.parse(...)`, including nested
  subcommands, shared command-tree options, and documented Lua schema/return
  value support.
- Added recursive directory transfers to `ptool.ssh.Connection:upload(...)`
  and `Connection:download(...)`, and extended `ptool.fs.copy(...)` so
  local/remote copies can move directories through the same SSH-backed path
  transfer API.

### Changed

- Expanded platform detection, Lua `ptool.platform` arch reporting, release
  artifacts, and the install script to cover more Linux, macOS, and Windows
  targets, including `x86`, `arm`, and `riscv64` support where applicable.
- Changed `ptool.ssh.Connection:run(...)` and `run_capture(...)` to echo remote
  commands by default, including the SSH target and resolved remote working
  directory in the echoed output when available.
- Changed `scripts/install.sh` to accept an optional full release tag such as
  `v0.2.0`, so users can install a specific version instead of only the latest
  stable release.
- Changed `scripts/install.sh` to accept `--bin-dir`, so users can install the
  `ptool` binary into a custom directory instead of only `~/.local/bin`.
- Expanded `ptool.semver.bump` with `prepatch`, `preminor`, and `premajor`
  operations plus an optional prerelease channel, and updated
  `scripts/release.lua` to support channel-aware prerelease bumps.
- Changed `ptool.ssh` to execute through the system OpenSSH client instead of
  the embedded SSH implementation, so SSH config, jump hosts, agent-based
  authentication, and other OpenSSH behaviors now follow the local `ssh`
  command more closely.

## v0.2.0 (2026-03-30)

### Added

- Added the `ptool.net` module with `parse_url`, `parse_ip`, and
  `parse_host_port` helpers for parsing network identifiers in Lua scripts.
- Added engine-backed database connection support in `ptool-engine` via
  `PtoolEngine::db_connect`, with a public `DbConnection` type for shared DB
  operations outside the Lua integration layer.
- Added `ptool.fs.glob(pattern)` for Unix-style filesystem globbing in Lua
  scripts, with support for recursive `**` matches, `ptool.cd(...)`-relative
  pattern resolution, stable sorted results, and shell-style hidden-file
  matching.
- Added remote path predicate helpers to `ptool.ssh`, including
  `Connection:exists`, `Connection:is_file`, `Connection:is_dir`, and matching
  methods on reusable `conn:path(...)` remote path values.

## v0.2.0-alpha.1 (2026-03-28)

### Added

- Added a Docusaurus documentation site under `website/`, with a new homepage,
  structured getting-started pages, and a dedicated API reference section.
- Added a simple `scripts/install.sh` installer that downloads the latest
  platform-specific `ptool-<os>-<arch>.tar.gz` archive and installs `ptool`
  into `~/.local/bin`.
- Added a top-level `README.md` with a one-line install command that downloads
  `https://peterlits.net/ptool/install.sh` and runs it.
- Added `ptool.run_capture`, which mirrors `ptool.run` while defaulting both
  `stdout` and `stderr` to captured output, and documented the new Lua API.
- Added Lua scripting documentation for `ptool.ssh.Connection:path`,
  `Connection:upload`, and `Connection:download`, including reusable remote
  path values, transfer options, and usage examples.

### Changed

- Changed the release workflow so stable releases also publish the installer at
  `ptool/install.sh` in Cloudflare R2.
- Changed `ptool.ssh.connect` to honor `ssh -G` identity ordering and to keep
  trying later private keys when earlier auto-discovered keys cannot be loaded.
- Changed the release workflow so stable releases also publish fixed download
  aliases under `ptool/release/latest/` plus a `ptool/release/LATEST` marker,
  while prereleases keep using only versioned release paths.

## v0.1.0 (2026-03-27)

### Added

- Added MiniJinja JSON and URL-encoding support for `ptool.template.render`
  templates by enabling the upstream `json` and `urlencode` features.
- Added the `ptool.str` module for common string operations in Lua scripts,
  including trimming, prefix/suffix checks, splitting, joining, replacing,
  repeating, prefix/suffix cutting, and line indentation helpers.
- Added `retry` support for `ptool.run`.
- Added `ptool.ask(prompt[, options])` for interactive text prompts in Lua
  scripts, with support for `default`, `help`, and `placeholder` options.

### Changed

- Changed `ptool.semver.bump` to support the `release` operation, allowing
  prerelease versions such as `1.2.3-rc.2` to be converted to the stable
  `1.2.3` form.
- Changed `ptool.template.render` to use chainable undefined semantics so
  nested missing values such as `foo.bar.baz` can still be handled by filters
  like `default(...)` instead of failing early.

## v0.1.0-alpha.4 (2026-03-26)

### Changed

- Changed the release workflow to upload packaged `*.tar.gz` artifacts and
  `SHA256SUMS` to the Cloudflare R2 release bucket for each Git tag.
- Changed shebang preprocessing so `ptool run` preserves Lua source line
  numbers by replacing the shebang line with a blank line instead of removing
  it entirely.

### Added

- Added file transfer support across `ptool.fs.copy` and `ptool.ssh`, including
  local-to-local copies, local uploads, remote downloads, reusable
  `conn:path(...)` remote path values, and transfer options such as `parents`,
  `overwrite`, and `echo`.
- Added the `ptool.ansi` module for styling terminal text, including
  `ptool.ansi.style(text[, options])`, foreground-color shortcuts such as
  `ptool.ansi.red(...)`, TTY-aware default enablement, and support for bold,
  dimmed, italic, and underline text attributes.

## v0.1.0-alpha.3 (2026-03-24)

### Added

- Added the `ptool version` subcommand to print the current CLI version, with
  dedicated help output and argument validation.
- Added `ptool.inspect(value[, options])` to render Lua values as readable
  Lua-style strings, including table formatting with nested structures, stable
  key ordering, and cycle-safe output.

### Changed

- Switched the GitHub release workflow to build Linux artifacts for
  `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, using Zig via
  `cargo-zigbuild` for cross-platform release packaging.

## v0.1.0-alpha.2 (2026-03-23)

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
