# API principal de Lua

`ptool` expõe estes helpers principais de runtime diretamente em `ptool` e `p`.

`ptool run <lua_file>` executa um script Lua e injeta a variável global
`ptool` (ou seu alias `p`; por exemplo, `p.run` é equivalente a `ptool.run`).

Se você quiser passar argumentos para um script Lua, pode fazer assim:

```sh
ptool run script.lua --name alice -v a.txt b.txt
```

Os argumentos então podem ser analisados com `ptool.args.parse(...)`.

Aqui está um script de exemplo:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

Shebang é suportado, então você pode adicionar isto ao topo do arquivo:

```
#!/usr/bin/env ptool run
```

## ptool.use

> `v0.1.0` - Introduced.

`ptool.use` declara a versão mínima de `ptool` exigida por um script.

```lua
ptool.use("v0.1.0")
```

- O argumento é uma string de versão semântica (SemVer) e suporta um prefixo
  `v` opcional, como `v0.1.0` ou `0.1.0`.
- Se a versão exigida for maior que a versão atual do `ptool`, o script sai
  imediatamente com um erro dizendo que a versão atual é antiga demais.

## ptool.unindent

> `v0.1.0` - Introduced.

`ptool.unindent` processa strings multilinha removendo o prefixo `| ` após a
indentação inicial em cada linha e aparando linhas em branco do começo e do fim.

```lua
local str = ptool.unindent([[
  | line 1
  | line 2
]])
```

Isso equivale a:

```lua
local str = [[line 1
line 2]]
```

## ptool.inspect

> `v0.1.0` - Introduced.

`ptool.inspect(value[, options])` renderiza um valor Lua como uma string legível
no estilo Lua. Seu objetivo principal é depuração e exibição de conteúdo de
tabelas.

- `value` (any, obrigatório): O valor Lua a inspecionar.
- `options` (table, opcional): Opções de renderização. Campos suportados:
  - `indent` (string, opcional): Indentação usada em cada nível de aninhamento.
    O padrão é dois espaços.
  - `multiline` (boolean, opcional): Se tabelas são renderizadas em várias
    linhas. O padrão é `true`.
  - `max_depth` (integer, opcional): Profundidade máxima de aninhamento a ser
    renderizada. Valores mais profundos são substituídos por `<max-depth>`.
- Retorna: `string`.

Comportamento:

- Entradas em formato array (`1..n`) são renderizadas primeiro.
- Os campos restantes da tabela são renderizados após a parte de array, em
  ordem estável por chave.
- Chaves string em formato de identificador são renderizadas como `key = value`;
  outras chaves são renderizadas como `[key] = value`.
- Referências recursivas a tabelas são renderizadas como `<cycle>`.
- Functions, threads e userdata são renderizados como valores marcador, como
  `<function>` e `<userdata>`.

Exemplo:

```lua
local value = {
  "hello",
  user = { name = "alice", tags = {"dev", "ops"} },
}
value.self = value

print(ptool.inspect(value))
print(ptool.inspect(value, { multiline = false }))
```

## ptool.ask

> `v0.1.0` - Introduced.

`ptool.ask(prompt[, options])` solicita ao usuário uma linha de texto e retorna
a resposta.

- `prompt` (string, obrigatório): O prompt mostrado ao usuário.
- `options` (table, opcional): Opções do prompt. Campos suportados:
  - `default` (string, opcional): Valor padrão usado quando o usuário envia uma
    resposta vazia.
  - `help` (string, opcional): Texto de ajuda extra exibido abaixo do prompt.
  - `placeholder` (string, opcional): Texto placeholder mostrado antes de o
    usuário começar a digitar.
- Retorna: `string`.

Comportamento:

- Requer um TTY interativo. Executá-lo em um ambiente não interativo gera erro.
- Se o usuário cancelar o prompt, o script gera erro.
- Nomes de opção desconhecidos ou tipos de valor inválidos geram erro.

Exemplo:

```lua
local name = ptool.ask("Your name?", {
  placeholder = "Alice",
  help = "Press Enter to confirm",
})

local city = ptool.ask("City?", {
  default = "Shanghai",
})

print(string.format("Hello, %s from %s!", name, city))
```

## ptool.config

> `v0.1.0` - Introduced.

`ptool.config` define a configuração de runtime do script.

Campos atualmente suportados:

- `run` (table, opcional): Configuração padrão para `ptool.run`. Campos
  suportados:
  - `echo` (boolean, opcional): Chave de echo padrão. O padrão é `true`.
  - `check` (boolean, opcional): Se falhas devem gerar erro por padrão.
    O padrão é `false`.
  - `confirm` (boolean, opcional): Se deve exigir confirmação antes da
    execução por padrão. O padrão é `false`.
  - `retry` (boolean, opcional): Se deve perguntar ao usuário se ele deseja
    tentar novamente após uma execução com falha quando `check = true`.
    O padrão é `false`.

Exemplo:

```lua
ptool.config({
  run = {
    echo = false,
    check = true,
    confirm = false,
    retry = false,
  },
})
```

## ptool.cd

> `v0.1.0` - Introduced.

`ptool.cd(path)` atualiza o diretório atual de runtime do `ptool`.

- `path` (string, obrigatório): Caminho do diretório de destino, absoluto ou
  relativo.

Comportamento:

- Caminhos relativos são resolvidos a partir do diretório de runtime atual do
  `ptool`.
- O destino precisa existir e precisa ser um diretório.
- Isso atualiza o estado de runtime do `ptool` e afeta APIs que usam o cwd de
  runtime (como `ptool.run`, `ptool.path.abspath` e `ptool.path.relpath`).

Exemplo:

```lua
ptool.cd("foobar")
local res = ptool.run({ cmd = "pwd", stdout = "capture" })
print(res.stdout)
```

## ptool.run

> `v0.1.0` - Introduced.

`ptool.run` executa comandos externos a partir de Rust.

As seguintes formas de chamada são suportadas:

```lua
ptool.run("echo hello world")
ptool.run("echo", "hello world")
ptool.run("echo", {"hello", "world"})
ptool.run("echo hello world", { echo = true })
ptool.run("echo", {"hello", "world"}, { echo = true })
ptool.run({ cmd = "echo", args = {"hello", "world"} })
ptool.run({ cmd = "echo", args = {"hello"}, stdout = "capture" })
```

Regras de argumento:

- `ptool.run(cmdline)`: `cmdline` é dividido usando regras no estilo shell
  (`shlex`). O primeiro item é tratado como comando e o restante como
  argumentos.
- `ptool.run(cmd, argsline)`: `cmd` é usado diretamente como comando e
  `argsline` é dividido em uma lista de argumentos usando regras no estilo
  shell (`shlex`).
- `ptool.run(cmd, args)`: `cmd` é uma string e `args` é um array de strings.
- `ptool.run(cmdline, options)`: `options` sobrescreve configurações desta
  invocação, como `echo`.
- `ptool.run(cmd, args, options)`: `args` pode ser string ou array de strings,
  e `options` sobrescreve configurações desta invocação, como `echo`.
- `ptool.run(options)`: `options` é uma tabela.
- Quando o segundo argumento é uma tabela: se for um array (chaves inteiras
  consecutivas `1..n`), é tratado como `args`; caso contrário, é tratado como
  `options`.

Regras de valor de retorno:

- Uma tabela é sempre retornada com os seguintes campos:
  - `ok` (boolean): Se o código de saída é `0`.
  - `code` (integer|nil): O código de saída do processo. Se o processo foi
    encerrado por sinal, isso é `nil`.
  - `stdout` (string, opcional): Presente quando `stdout = "capture"`.
  - `stderr` (string, opcional): Presente quando `stderr = "capture"`.
  - `assert_ok(self)` (function): Gera erro quando `ok = false`. A mensagem
    inclui o código de saída e, quando disponível, `stderr`.
- O valor padrão de `check` vem de
  `ptool.config({ run = { check = ... } })`. Se não estiver configurado,
  o padrão é `false`. Quando `check = false`, quem chama pode inspecionar `ok`
  diretamente ou chamar `res:assert_ok()`.
- Quando `check = true` e `retry = true`, `ptool.run` pergunta se o comando
  com falha deve ser tentado novamente antes de gerar o erro final.

Exemplo:

```lua
ptool.config({ run = { echo = false } })

ptool.run("echo from ptool")
ptool.run("echo", "from ptool")
ptool.run("echo", {"from", "ptool"})
ptool.run("echo from ptool", { echo = true })
ptool.run("echo", {"from", "ptool"}, { echo = true })
ptool.run("pwd")

local res = ptool.run({
  cmd = "sh",
  args = {"-c", "echo bad >&2; exit 7"},
  stderr = "capture",
})
print(res.ok, res.code)
res:assert_ok()
```

`ptool.run(options)` também é suportado, em que `options` é uma tabela com os
seguintes campos:

- `cmd` (string, obrigatório): O nome do comando ou caminho do executável.
- `args` (string[], opcional): A lista de argumentos.
- `cwd` (string, opcional): O diretório de trabalho do processo filho.
- `env` (table, opcional): Variáveis de ambiente adicionais, em que chaves são
  nomes de variáveis e valores são valores de variáveis.
- `echo` (boolean, opcional): Se informações do comando devem ser exibidas para
  esta execução. Se omitido, é usado o valor de
  `ptool.config({ run = { echo = ... } })`; se ele também estiver ausente, o
  padrão é `true`.
- `check` (boolean, opcional): Se deve gerar erro imediatamente quando o código
  de saída não é `0`. Se omitido, é usado o valor de
  `ptool.config({ run = { check = ... } })`; se ele também estiver ausente, o
  padrão é `false`.
- `confirm` (boolean, opcional): Se deve pedir confirmação ao usuário antes da
  execução. Se omitido, é usado o valor de
  `ptool.config({ run = { confirm = ... } })`; se ele também estiver ausente,
  o padrão é `false`.
- `retry` (boolean, opcional): Se deve perguntar ao usuário se ele deseja
  tentar novamente após uma falha quando `check = true`. Se omitido, é usado o
  valor de `ptool.config({ run = { retry = ... } })`; se ele também estiver
  ausente, o padrão é `false`.
- `stdout` (string, opcional): Estratégia de tratamento de stdout. Valores
  suportados:
  - `"inherit"`: Herda para o terminal atual (padrão).
  - `"capture"`: Captura em `res.stdout`.
  - `"null"`: Descarta a saída.
- `stderr` (string, opcional): Estratégia de tratamento de stderr. Valores
  suportados:
  - `"inherit"`: Herda para o terminal atual (padrão).
  - `"capture"`: Captura em `res.stderr`.
  - `"null"`: Descarta a saída.
- Quando `confirm = true`:
  - Se o usuário recusar a execução, um erro é gerado imediatamente.
  - Se o ambiente atual não for interativo (sem TTY), um erro é gerado
    imediatamente.
- Quando `retry = true` e `check = true`:
  - Se o comando falhar, `ptool.run` pergunta se o mesmo comando deve ser
    executado novamente.
  - Se o ambiente atual não for interativo (sem TTY), um erro é gerado
    imediatamente em vez de perguntar sobre retry.

Exemplo:

```lua
ptool.run({
  cmd = "echo",
  args = {"hello"},
  env = { FOO = "bar" },
})

local res = ptool.run({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2; exit 7"},
  stdout = "capture",
  stderr = "capture",
  check = false,
})
print(res.ok, res.code)
print(res.stdout)
print(res.stderr)
res:assert_ok()
```

## ptool.run_capture

> `Unreleased` - Introduced.

`ptool.run_capture` executa comandos externos a partir de Rust com as mesmas
formas de chamada, regras de argumento, regras de valor de retorno e opções de
`ptool.run`.

A única diferença é o tratamento padrão de streams:

- `stdout` tem padrão `"capture"`.
- `stderr` tem padrão `"capture"`.

Você ainda pode sobrescrever qualquer um dos campos explicitamente em
`options`.

Exemplo:

```lua
local res = ptool.run_capture("echo hello world")
print(res.stdout)

local res2 = ptool.run_capture({
  cmd = "sh",
  args = {"-c", "printf 'out'; printf 'err' >&2"},
})
print(res2.stdout)
print(res2.stderr)

local res3 = ptool.run_capture("echo hello", {
  stderr = "inherit",
})
print(res3.stdout)
```
