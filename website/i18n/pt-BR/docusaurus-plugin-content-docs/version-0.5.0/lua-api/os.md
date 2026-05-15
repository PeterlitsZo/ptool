# API de sistema operacional

`ptool.os` expõe utilitários para ler o ambiente atual do runtime e consultar detalhes básicos do processo hospedeiro.

## ptool.os.getenv

> `v0.4.0` - Introduced.

`ptool.os.getenv(name)` retorna o valor atual de uma variável de ambiente.

- `name` (string, obrigatório): Nome da variável.
- Retorna: `string|nil`.

Comportamento:

- Retorna: `string|nil`.
- Lê o ambiente de execução atual do `ptool`, incluindo valores alterados por `ptool.os.setenv(...)` e `ptool.os.unsetenv(...)`.
- Gera um erro quando `name` está vazio ou contém caracteres inválidos, como `=`.

Exemplo:

```lua
local home = p.os.getenv("HOME")
print(home)
```

## ptool.os.env

> `v0.4.0` - Introduced.

`ptool.os.env()` retorna uma tabela instantânea do ambiente atual do runtime.

- Retorna: `table`.

Comportamento:

- A tabela retornada mapeia nomes de variáveis ​​para valores de string.
- Os valores alterados por meio de `ptool.os.setenv(...)` e `ptool.os.unsetenv(...)` são refletidos na captura instantânea.

Exemplo:

```lua
local env = p.os.env()
print(env.HOME)
```

## ptool.os.setenv

> `v0.4.0` - Introduced.

`ptool.os.setenv(name, value)` define uma variável de ambiente no runtime atual do `ptool`.

- `name` (string, obrigatório): Nome da variável.
- `value` (string, obrigatório): Valor da variável.

Comportamento:

- Isso atualiza o ambiente do runtime atual do `ptool`, não o shell pai.
- Os valores definidos aqui são visíveis para `ptool.os.getenv(...)`, `ptool.os.env()` e processos filhos iniciados posteriormente por meio de `ptool.run(...)`.
- Os valores definidos aqui ficam visíveis para `ptool.os.getenv(...)`, `ptool.os.env()` e processos filhos iniciados depois com `ptool.run(...)`.

Exemplo:

```lua
p.os.setenv("APP_ENV", "dev")
print(p.os.getenv("APP_ENV"))
```

## ptool.os.unsetenv

> `v0.4.0` - Introduced.

`ptool.os.unsetenv(name)` remove uma variável de ambiente do runtime atual do `ptool`.

- `name` (string, obrigatório): Nome da variável.

Comportamento:

- Isso afeta chamadas posteriores para `ptool.os.getenv(...)`, `ptool.os.env()` e processos filhos iniciados por `ptool.run(...)`.
- Gera um erro quando `name` está vazio ou contém caracteres inválidos, como `=`.

Exemplo:

```lua
p.os.unsetenv("APP_ENV")
assert(p.os.getenv("APP_ENV") == nil)
```

## ptool.os.homedir

> `v0.4.0` - Introduced.

`ptool.os.homedir()` retorna o diretório pessoal do usuário atual.

- Retorna: `string|nil`.

Exemplo:

```lua
local home = p.os.homedir()
```

## ptool.os.tmpdir

> `v0.4.0` - Introduced.

`ptool.os.tmpdir()` retorna o diretório temporário do sistema.

- Retorna: `string`.

Exemplo:

```lua
local tmp = p.os.tmpdir()
```

## ptool.os.hostname

> `v0.4.0` - Introduced.

`ptool.os.hostname()` retorna o nome do host atual.

- Retorna: `string|nil`.

## ptool.os.username

> `v0.4.0` - Introduced.

`ptool.os.username()` retorna o nome do usuário atual.

- Retorna: `string|nil`.

## ptool.os.pid

> `v0.4.0` - Introduced.

`ptool.os.pid()` retorna o PID do processo atual do `ptool`.

- Retorna: `integer`.

## ptool.os.exepath

> `v0.4.0` - Introduced.

`ptool.os.exepath()` retorna o caminho resolvido do executável `ptool` em execução.

- Retorna: `string|nil`.

Exemplo:

```lua
print(p.os.hostname(), p.os.username(), p.os.pid())
print(p.os.exepath())
```
