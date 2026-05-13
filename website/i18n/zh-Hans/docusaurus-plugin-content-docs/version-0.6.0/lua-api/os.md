# OS API

`ptool.os` exposes helpers for reading the current runtime environment and querying basic host-process details.

## ptool.os.getenv

> `v0.4.0` - Introduced.

`ptool.os.getenv(name)` returns the current value of an environment variable.

- `name` (string, required): Environment variable name.
- Returns: `string|nil`.

Behavior:

- Returns `nil` when the variable is not set.
- Reads the current `ptool` runtime environment, including values changed by `ptool.os.setenv(...)` and `ptool.os.unsetenv(...)`.
- Raises an error when `name` is empty or contains invalid characters such as `=`.

Example:

```lua
local home = p.os.getenv("HOME")
print(home)
```

## ptool.os.env

> `v0.4.0` - Introduced.

`ptool.os.env()` returns a snapshot table of the current runtime environment.

- Returns: `table`.

Behavior:

- The returned table maps variable names to string values.
- Values changed through `ptool.os.setenv(...)` and `ptool.os.unsetenv(...)` are reflected in the snapshot.

Example:

```lua
local env = p.os.env()
print(env.HOME)
```

## ptool.os.setenv

> `v0.4.0` - Introduced.

`ptool.os.setenv(name, value)` sets an environment variable in the current `ptool` runtime.

- `name` (string, required): Environment variable name.
- `value` (string, required): Environment variable value.

Behavior:

- This updates the current `ptool` runtime environment, not the parent shell.
- Values set here are visible to `ptool.os.getenv(...)`, `ptool.os.env()`, and child processes launched later through `ptool.run(...)`.
- Raises an error when `name` is empty, contains `=`, or when `value` contains NUL.

Example:

```lua
p.os.setenv("APP_ENV", "dev")
print(p.os.getenv("APP_ENV"))
```

## ptool.os.unsetenv

> `v0.4.0` - Introduced.

`ptool.os.unsetenv(name)` removes an environment variable from the current `ptool` runtime.

- `name` (string, required): Environment variable name.

Behavior:

- This affects later calls to `ptool.os.getenv(...)`, `ptool.os.env()`, and child processes launched by `ptool.run(...)`.
- Raises an error when `name` is empty or contains invalid characters such as `=`.

Example:

```lua
p.os.unsetenv("APP_ENV")
assert(p.os.getenv("APP_ENV") == nil)
```

## ptool.os.homedir

> `v0.4.0` - Introduced.

`ptool.os.homedir()` returns the current user's home directory.

- Returns: `string|nil`.

Example:

```lua
local home = p.os.homedir()
```

## ptool.os.tmpdir

> `v0.4.0` - Introduced.

`ptool.os.tmpdir()` returns the system temporary directory.

- Returns: `string`.

Example:

```lua
local tmp = p.os.tmpdir()
```

## ptool.os.hostname

> `v0.4.0` - Introduced.

`ptool.os.hostname()` returns the current host name.

- Returns: `string|nil`.

## ptool.os.username

> `v0.4.0` - Introduced.

`ptool.os.username()` returns the current user name.

- Returns: `string|nil`.

## ptool.os.pid

> `v0.4.0` - Introduced.

`ptool.os.pid()` returns the current `ptool` process ID.

- Returns: `integer`.

## ptool.os.exepath

> `v0.4.0` - Introduced.

`ptool.os.exepath()` returns the resolved path of the running `ptool` executable.

- Returns: `string|nil`.

Example:

```lua
print(p.os.hostname(), p.os.username(), p.os.pid())
print(p.os.exepath())
```
