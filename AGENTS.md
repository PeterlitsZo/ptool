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
- `website/po/docs/templates/` contains generated POT templates for docs i18n.
- `website/po/docs/<locale>/` contains locale PO catalogs for docs i18n.
- `website/i18n/` contains localized Docusaurus documentation content generated
  from the PO catalogs.
- `website/` contains the Docusaurus site project and frontend assets.

## Development Tips

- After every development task, you MUST run `cargo fmt` and `cargo clippy`.
  Do not treat this as optional cleanup.
- You MUST NOT edit dependencies in `Cargo.toml` directly. Use `cargo add` so
  dependency versions stay current and explicit.
- You MUST treat annotations such as `#[allow(...)]` and `#![allow(...)]` as a
  last resort. Do not add them unless they are genuinely necessary and
  justified by the code.
- If documentation may need to change, you MUST ask the user first. Do not
  modify documentation speculatively.
- When editing documentation under `website/docs/`, you MUST also update the
  corresponding POT/PO files under `website/po/docs/` in the same task and then
  regenerate the localized docs under `website/i18n/` using the docs i18n
  workflow (`node website/scripts/docs-i18n.mjs sync`, `compile`, or
  `refresh`) so localized docs stay in sync.
- When adding, removing, renaming, or regrouping docs pages in `website/docs/`,
  you MUST also verify whether `website/sidebars.ts` or
  `website/versioned_sidebars/` needs a matching update so the pages remain
  reachable from the docs sidebar.
- When new or changed source doc strings have not yet been translated for a
  locale, you MUST NOT regenerate that locale's files under `website/i18n/`
  from empty or fuzzy PO entries. Fill in or explicitly resolve the relevant PO
  entries first so generated localized docs do not silently fall back to mixed
  English content.
- You MUST NOT hand-edit generated localized docs under `website/i18n/` when
  the change should be represented in `website/po/docs/`. Update the source
  docs and catalogs first, then regenerate the localized Markdown.
- DO NOT add unit tests unless the user explicitly requests them, to avoid test
  bloat.
- When editing documentation, especially API version markers such as
  ``> `v0.x.y` - Introduced.``, you MUST target the exact section with enough
  surrounding context to avoid touching similarly worded entries elsewhere. Do
  not use broad search-and-replace against repeated marker text.
- When adding new non-Lua core capabilities, you MUST implement the business
  logic in `ptool-engine` by default. Keep `ptool` limited to Lua-facing
  adaptation, argument conversion, and API exposure unless there is a clear,
  defensible reason the logic cannot live in the engine.

## Behavioral Rules

- If you notice files that have already been modified, you MUST NOT revert them
  unless it is truly necessary. The user may be editing them. If you believe a
  revert may be required, or if you are unsure, stop and ask the user first. If
  no revert is needed, leave those changes alone and move on.

## When You Need to Commit

WHEN THE USER ASKS YOU TO COMMIT, you MUST follow this section.

### Before Committing

Before committing, you MUST:

- Explicitly decide whether `CHANGELOG.md` needs an update. Keep it concise.
  Add only user-visible changes such as new features, behavioral changes,
  compatibility-impacting fixes, or important bug fixes. Do not add pure
  internal refactors that do not affect functionality. If you update
  `CHANGELOG.md`, you MUST ask the user to confirm that it looks correct before
  committing, and you MUST stage it explicitly.
- Explicitly decide whether documentation needs an update. If you are unsure,
  you MUST ask the user instead of guessing. The documentation set is:

  - `README.md`
  - `website/docs/` (Docusaurus documentation content)

### Tips

- When the user asks you to commit, you MUST commit only the files that are
  already staged. Unstaged working tree files are off limits.
- You MUST ask for confirmation before running `git commit`. This remains true
  even if the user has already told you to commit.

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
documentation update). The subject MUST end with an English full stop, and its
first letter MUST be capitalized.
