# Args API

CLI argument schema and parsing helpers are available under `ptool.args` and `p.args`.

## ptool.args.arg

> `v0.1.0` - Introduced.

`ptool.args.arg(id, kind, options)` creates an argument builder for use in
`ptool.args.parse(...).schema.args`.

- `id` (string, required): The argument identifier. It is also the key in the
  returned table.
- `kind` (string, required): The argument type. Supported values:
  - `"flag"`: A boolean flag.
  - `"string"`: A string option.
  - `"int"`: An integer option (`i64`).
  - `"positional"`: A positional argument.
- `options` (table, optional): The same optional fields supported by argument
  tables in `ptool.args.parse`, such as `long`, `short`, `help`, `required`,
  `multiple`, and `default`.

The builder supports chainable methods, all of which return itself:

- `arg:long(value)` sets the long option name. Supported only for
  non-`positional` arguments.
- `arg:short(value)` sets the short option name. Supported only for
  non-`positional` arguments.
- `arg:help(value)` sets the help text.
- `arg:required(value)` sets whether the argument is required. If `value` is
  omitted, it defaults to `true`.
- `arg:multiple(value)` sets whether the argument can be repeated. If `value` is
  omitted, it defaults to `true`.
- `arg:default(value)` sets the default value. If `value = nil`, the default is
  cleared.

Example:

```lua
local res = ptool.args.parse({
  args = {
    ptool.args.arg("name", "string"):required(),
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("paths", "positional"):multiple(),
  }
})
```

## ptool.args.parse

> `v0.1.0` - Introduced.
>
> `v0.3.0` - Added `subcommands` support.

`ptool.args.parse(schema)` parses script arguments with `clap` and returns a
table indexed by `id`.

Script arguments come from the part after `--` in `ptool run <lua_file> -- ...`.

For example:

```lua
ptool.use("v0.1.0")

local res = ptool.args.parse({
    name = "test",
    about = "The test command",
    args = {
        { id = "name", kind = "string" }
    }
})

print("Hello, " .. res.name .. "!")
```

### Schema Structure

- `name` (string, optional): The command name, used in help output. Defaults to
  the script file name.
- `about` (string, optional): Help description.
- `args` (table, optional): An array of argument definitions. Each item supports
  two forms:
  - An argument table.
  - A builder object returned by `ptool.args.arg(...)`.
- `subcommands` (table, optional): A map of subcommand name to subcommand
  schema. Each subcommand schema supports `about`, `args`, and `subcommands`
  recursively.

At least one of `args` or `subcommands` must be provided.

Argument table fields:

- `id` (string, required): The argument identifier. It is also the key in the
  returned table.
- `kind` (string, required): The argument type. Supported values:
  - `"flag"`: A boolean flag.
  - `"string"`: A string option.
  - `"int"`: An integer option (`i64`).
  - `"positional"`: A positional argument.
- `long` (string, optional): The long option name, such as `"name"` for
  `--name`. For non-`positional` arguments, the default can be derived from
  `id`.
- `short` (string, optional): The short option name, a single character such as
  `"v"` for `-v`.
- `help` (string, optional): Help text for the argument.
- `required` (boolean, optional): Whether the argument is required. Defaults to
  `false`.
- `multiple` (boolean, optional): Whether the argument can be repeated. Defaults
  to `false`.
- `default` (string/integer, optional): The default value.

When `subcommands` is present, the current command's `args` act as shared
options for that command tree, and are accepted before or after the selected
subcommand.

Example with subcommands:

```lua
local res = ptool.args.parse({
  name = "demo",
  args = {
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("config", "string"),
  },
  subcommands = {
    build = {
      args = {
        ptool.args.arg("release", "flag"),
      },
      subcommands = {
        web = {
          args = {
            ptool.args.arg("out", "string"):required(),
          },
        },
      },
    },
    clean = {
      args = {
        ptool.args.arg("all", "flag"),
      },
    },
  },
})
```

### Constraints

- The following constraints apply to both argument tables and builder syntax.
- Non-`positional` arguments may omit `long` and `short`. If `long` is omitted,
  `id` is used automatically.
- `positional` arguments cannot set `long`, `short`, or `default`.
- When `positional.multiple = true`, it must be the last argument in `args`.
- `multiple = true` is supported only for `string` and `positional`.
- `default` is supported only for `string` and `int`, and cannot be used
  together with `multiple = true`.
- When `subcommands` is present, `positional` arguments are not allowed in that
  same schema.
- When `subcommands` is present at the top level, argument ids `command_path`
  and `args` are reserved.
- Along one selected subcommand path, ancestor and descendant subcommands cannot
  reuse the same argument `id`, because their values are merged into one `args`
  table.

### Return Value

A Lua table is returned where keys are `id` and value types are as follows:

- `flag` -> `boolean`
- `string` -> `string` (or `string[]` when `multiple = true`)
- `int` -> `integer`
- `positional` -> `string` (or `string[]` when `multiple = true`)

When `subcommands` is not present, the return value stays flat as above.

When `subcommands` is present, the return value has this shape:

- Top-level `args` values are returned directly on the top-level table.
- `command_path` -> `string[]`: The matched subcommand path, for example
  `{"build", "web"}`.
- `args` -> `table`: The merged argument values from the matched subcommand path.

For example:

```lua
{
  verbose = true,
  config = "cfg.toml",
  command_path = { "build", "web" },
  args = {
    release = true,
    out = "dist",
  },
}
```
