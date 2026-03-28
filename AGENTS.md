# AGENTS.md

ptool is a tool. At the moment, its primary command is:

```
ptool run <file>
```

Here, `<file>` is a Lua script. `ptool` executes it with the embedded Lua
interpreter and injects a large set of utility functions (registered under the
`ptool` table and also accessible through `p`).

## Documentation

- `docs/lua-script.md` is the Lua scripting documentation and describes the
  provided utility functions.

## Development Tips

- After each development task is finished, run `cargo fmt` to format the code
  and `cargo clippy` to check code quality.
- Do not edit dependencies in `Cargo.toml` directly. Use `cargo add` to add
  dependencies so versions do not become outdated.
- Use annotations such as `#[allow(...)]` and `#![allow(...)]` cautiously unless
  they are truly necessary.
- If documentation needs to be changed, ask the user first and only modify it
  after confirming that a doc change is indeed needed.
- Do not add tests unless the user explicitly requests them, to avoid test
  bloat.

## Behavioral Rules

- If you notice files that have already been modified, do not revert them unless
  it is truly necessary, because the user may be editing them. If, after review,
  you believe a revert is required or you are unsure, talk to the user first and
  confirm whether the revert is needed. If you determine that reverting is
  unnecessary, just ignore the changes.

## Git Commit and Message Conventions

Git commit messages use the following format:

```
<type>: <subject>
```

Here, `<type>` describes the nature of the change, such as `feat`, `fix`,
`docs`, `chore`, or `refactor`.

The `<subject>` should be a short description of the change in English. Keep it
concise and clear. You need to review all changed files and summarize the most
important change (for example, if the project includes both a refactor and a
documentation update, the subject should describe the refactor rather than the
documentation update). End it with a English full stop. The first letter of the
subject should be capitalized.

When the user asks you to commit, you should commit only the files that are
already staged and leave unstaged working tree files untouched.

Before committing, consider whether `CHANGELOG.md` needs an update and keep it
concise. Prefer adding only user-visible changes such as new features,
behavioral changes, compatibility-impacting fixes, or important bug fixes.
Pure internal refactors that do not affect functionality should generally not be
added to `CHANGELOG.md`. If you do update `CHANGELOG.md`, ask the user to
confirm that it looks correct, then add `CHANGELOG.md` to the staging area
before committing. You may also check whether documentation changes are needed
(the `docs/` directory) and update them together if necessary.
