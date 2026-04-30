# API de sistema operacional

`ptool.os` expõe utilitários para ler o ambiente atual do runtime e consultar
detalhes básicos do processo hospedeiro.

## ptool.os.getenv

> `v0.4.0` - Introduced.

`ptool.os.getenv(name)` retorna o valor atual de uma variável de ambiente.

- `name` (string, obrigatório): Nome da variável de ambiente.
- Retorna: `string|nil`.

## ptool.os.env

> `v0.4.0` - Introduced.

`ptool.os.env()` retorna uma tabela instantânea do ambiente atual do runtime.

- Retorna: `table`.

## ptool.os.setenv

> `v0.4.0` - Introduced.

`ptool.os.setenv(name, value)` define uma variável de ambiente no runtime atual
do `ptool`.

- `name` (string, obrigatório): Nome da variável.
- `value` (string, obrigatório): Valor da variável.

Comportamento:

- Isso atualiza o ambiente do runtime atual do `ptool`, não o shell pai.
- Os valores definidos aqui ficam visíveis para `ptool.os.getenv(...)`,
  `ptool.os.env()` e processos filhos iniciados depois com `ptool.run(...)`.

## ptool.os.unsetenv

> `v0.4.0` - Introduced.

`ptool.os.unsetenv(name)` remove uma variável de ambiente do runtime atual do
`ptool`.

- `name` (string, obrigatório): Nome da variável.

## ptool.os.homedir

> `v0.4.0` - Introduced.

`ptool.os.homedir()` retorna o diretório pessoal do usuário atual.

- Retorna: `string|nil`.

## ptool.os.tmpdir

> `v0.4.0` - Introduced.

`ptool.os.tmpdir()` retorna o diretório temporário do sistema.

- Retorna: `string`.

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

`ptool.os.exepath()` retorna o caminho resolvido do executável `ptool` em
execução.

- Retorna: `string|nil`.
