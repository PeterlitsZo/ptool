# REPL

`ptool repl` starts an interactive Lua session with the standard `ptool` API
already loaded.

## Start the REPL

```sh
ptool repl
```

When the REPL starts, `ptool` shows a banner and waits for Lua input.

## What it provides

- The global `ptool` table and the shorter alias `p`.
- The same bundled helpers you can use from `ptool run <file>`.
- Interactive evaluation of Lua expressions and statements.
- Readline-style editing, including arrow-key cursor movement and in-session
  history navigation.

## Basic usage

Enter an expression to evaluate it immediately:

```lua
1 + 2
```

The REPL prints returned values using the same inspector used elsewhere in
`ptool`.

You can also call `ptool` APIs directly:

```lua
p.str.trim("  hello  ")
```

## Multi-line input

If the current input is incomplete, the prompt changes from `>>> ` to `... `.
This lets you continue entering a block such as a function or control flow
statement:

```lua
for i = 1, 3 do
  print(i)
end
```

Once the input is complete, `ptool` evaluates the whole chunk.

## Keyboard behavior

- `Up` and `Down` browse commands entered earlier in the same REPL session.
- `Left` and `Right` move the cursor within the current input line.
- `Ctrl-C` clears the current input. If you are in the middle of a multi-line
  chunk, it discards the buffered chunk and returns to the primary prompt.
- `Ctrl-D` exits the REPL.

## Notes

- `ptool repl` requires an interactive TTY.
- REPL history currently lives only for the current session and is not written
  to a history file.
