# Shell API

Shell parsing helpers are available under `ptool.sh` and `p.sh`.

These helpers work at the shell-word level. They are intended for splitting,
quoting, and joining argument strings using POSIX shell-style rules, not for
parsing full shell syntax such as pipelines, redirections, command
substitution, or variable expansion.

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split(command)` parses a command string using shell-style rules and
returns an argument array.

- `command` (string, required): The command string to split.
- Returns: `string[]`.

Behavior:

- This parses shell words only. It does not interpret shell operators or
  execute expansions.

Example:

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

The `args` above is equivalent to:

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```

## ptool.sh.quote

> `Unreleased` - Introduced.

`ptool.sh.quote(word)` quotes a single shell word so it can be embedded safely
into a shell command string.

- `word` (string, required): The shell word to quote.
- Returns: `string`.

Behavior:

- The returned string is shell-safe and semantically equivalent to the input
  word.
- This preserves shell-word meaning, not the original textual spelling.

Example:

```lua
local word = ptool.sh.quote("hello world")
print(word) -- 'hello world'
```

## ptool.sh.join

> `Unreleased` - Introduced.

`ptool.sh.join(words)` joins an argument array into a shell command string,
quoting words when needed.

- `words` (string[], required): The shell words to join.
- Returns: `string`.

Behavior:

- Consecutive words are joined with a single space.
- The output is suitable for passing to a POSIX-style shell.
- This aims for shell-word round-tripping, so `ptool.sh.split(ptool.sh.join(words))`
  is equivalent to `words`.
- `ptool.sh.join(ptool.sh.split(command))` may normalize quoting and spacing
  instead of preserving the original command text.

Example:

```lua
local cmd = ptool.sh.join({"git", "commit", "-m", "hello world"})
print(cmd) -- git commit -m 'hello world'
```
