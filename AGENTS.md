# AGENTS.md

ptool is a tool. At the moment, its primary command is:

```
ptool run <file>
```

Here, `<file>` is a Lua script. `ptool` executes it with the embedded Lua
interpreter and injects a large set of utility functions (registered under the
`ptool` table and also accessible through `p`).

## Project Structure

- `ptool` is responsible for Lua-facing functionality. It handles Lua runtime
  integration and Lua API exposure, and should delegate shared core logic to
  `ptool-engine` when appropriate.
- `ptool-engine` is responsible for core logic that is not specific to the Lua
  integration layer.
- `ptool-cli` is responsible for CLI behavior and should delegate library
  functionality to `ptool`.

## Documentation

- `website/docs/` contains the current Docusaurus documentation content.
- `website/` contains the Docusaurus site project and frontend assets.
- `docs-backup/` stores the previous Markdown documentation for reference.

## Development Tips

- After each development task is finished, run `cargo fmt` to format the code
  and `cargo clippy` to check code quality.
- Do not edit dependencies in `Cargo.toml` directly. Use `cargo add` to add
  dependencies so versions do not become outdated.
- Use annotations such as `#[allow(...)]` and `#![allow(...)]` cautiously unless
  they are truly necessary.
- If documentation needs to be changed, ask the user first and only modify it
  after confirming that a doc change is indeed needed.
- DO NOT add unit tests unless the user explicitly requests them, to avoid test
  bloat.
- When editing documentation, especially API version markers such as
  ``> `v0.x.y` - Introduced.``, always target the exact section with enough
  surrounding context to avoid changing similarly worded entries elsewhere. Do
  not use broad search-and-replace for repeated version marker text without
  first narrowing the target section.
- When adding new non-Lua core capabilities, implement the core business logic
  in `ptool-engine` by default. Keep `ptool` focused on Lua-facing adaptation,
  argument conversion, and API exposure unless there is a clear reason the logic
  must stay in the Lua integration layer.

## Behavioral Rules

- If you notice files that have already been modified, do not revert them unless
  it is truly necessary, because the user may be editing them. If, after review,
  you believe a revert is required or you are unsure, talk to the user first and
  confirm whether the revert is needed. If you determine that reverting is
  unnecessary, just ignore the changes.

## When You Need to Commit

WHEN THE USER ASKS YOU TO COMMIT, you MUST follow this section.

### Before Committing

Before committing, you MUST:

- Consider whether `CHANGELOG.md` needs an update and keep it concise. Prefer
  adding only user-visible changes such as new features, behavioral changes,
  compatibility-impacting fixes, or important bug fixes. Pure internal refactors
  that do not affect functionality should generally not be added to
  `CHANGELOG.md`. If you do update `CHANGELOG.md`, ask the user to confirm that
  it looks correct, then add `CHANGELOG.md` to the staging area before
  committing.
- Consider whether the documentation needs an update. If you are unsure, ask the
  user to confirm whether a doc update is needed. The documentations list are:

  - `README.md`
  - `website/docs/` (Docusaurus documentation content)

### Tips

- When the user asks you to commit, you MUST commit only the files that are
  already staged and leave unstaged working tree files untouched.
- If you want to run `git commit` command, you MUST ask the user to confirm. (If
  the user ask you to commit, you still MUST ask the user to confirm before you
  REALLY run the `git commit` command)

### Git Commit Message

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
