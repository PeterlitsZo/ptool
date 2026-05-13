# API de sistema operacional

`ptool.os` expõe utilitários para ler o ambiente atual do runtime e consultar detalhes básicos do processo hospedeiro.

## ptool.os.getenv

> `v0.4.0` - Introduced.

`ptool.os.getenv(name)` retorna o valor atual de uma variável de ambiente.

- `name` (string, obrigatório): Nome da variável de ambiente.
- Returns: `string|nil`.

Behavior:

- Retorna: `string|nil`.
- Reads the current `ptool` runtime environment, including values changed by `ptool.os.setenv(...)` and `ptool.os.unsetenv(...)`.
- Raises an error when `name` is empty or contains invalid characters such as `=`.

Example:

```lua
local home = p.os.getenv("HOME")
print(home)
```

## ptool.os.env

> `v0.4.0` - Introduced.

`ptool.os.env()` retorna uma tabela instantânea do ambiente atual do runtime.

- Retorna: `table`.

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

`ptool.os.setenv(name, value)` define uma variável de ambiente no runtime atual do `ptool`.

- `name` (string, obrigatório): Nome da variável de ambiente.
- `value` (string, obrigatório): Valor da variável.

Behavior:

- Isso atualiza o ambiente do runtime atual do `ptool`, não o shell pai.
- Values set here are visible to `ptool.os.getenv(...)`, `ptool.os.env()`, and child processes launched later through `ptool.run(...)`.
- Os valores definidos aqui ficam visíveis para `ptool.os.getenv(...)`, `ptool.os.env()` e processos filhos iniciados depois com `ptool.run(...)`.

Example:

```lua
p.os.setenv("APP_ENV", "dev")
print(p.os.getenv("APP_ENV"))
```

## ptool.os.unsetenv

> `v0.4.0` - Introduced.

`ptool.os.unsetenv(name)` remove uma variável de ambiente do runtime atual do `ptool`.

- `name` (string, obrigatório): Nome da variável de ambiente.

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

`ptool.os.homedir()` retorna o diretório pessoal do usuário atual.

- Returns: `string|nil`.

Example:

```lua
local home = p.os.homedir()
```

## ptool.os.tmpdir

> `v0.4.0` - Introduced.

`ptool.os.tmpdir()` retorna o diretório temporário do sistema.

- Retorna: `string`.

Example:

```lua
local tmp = p.os.tmpdir()
```

## ptool.os.hostname

> `v0.4.0` - Introduced.

`ptool.os.hostname()` retorna o nome do host atual.

- Returns: `string|nil`.

## ptool.os.username

> `v0.4.0` - Introduced.

`ptool.os.username()` retorna o nome do usuário atual.

- Returns: `string|nil`.

## ptool.os.pid

> `v0.4.0` - Introduced.

`ptool.os.pid()` retorna o PID do processo atual do `ptool`.

- Retorna: `integer`.

## ptool.os.exepath

> `v0.4.0` - Introduced.

`ptool.os.exepath()` retorna o caminho resolvido do executável `ptool` em execução.

- Returns: `string|nil`.

Example:

```lua
print(p.os.hostname(), p.os.username(), p.os.pid())
print(p.os.exepath())
```
