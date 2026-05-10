# API principal de Lua

`ptool` expõe estes helpers principais de runtime diretamente em `ptool` e `p`.

`ptool run <lua_file>` executa um script Lua e injeta a variável global
`ptool` (ou seu alias `p`; por exemplo, `p.run` é equivalente a `ptool.run`).
Para arquivos que terminam em `.lua`, `ptool <lua_file>` é um atalho de CLI
com o mesmo comportamento.

O runtime Lua embutido mantém os globais básicos do Lua e, por padrão, expõe
apenas estas bibliotecas padrão:

- `table`
- `string`
- `math`
- `utf8`

Módulos embutidos voltados ao host, como `io`, `os` e `package`, ficam
intencionalmente indisponíveis. Use APIs do `ptool` como `ptool.fs`,
`ptool.os`, `ptool.path` e `ptool.run` para operações de sistema de arquivos,
ambiente, processos, rede e demais tarefas de runtime.

Se você quiser passar argumentos para um script Lua, pode fazer assim:

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

Os argumentos então podem ser analisados com `ptool.args.parse(...)`.

Aqui está um script de exemplo:

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

Shebang é suportado, então você pode adicionar isto ao topo do arquivo:

```
#!/usr/bin/env ptool
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
> `v0.5.0` - Added validation options and prompt subcommands.

`ptool.ask` oferece prompts interativos. Você pode chamá-lo diretamente para
ler texto, ou usar seus subprompts para confirmação, seleção simples, seleção
múltipla e entrada secreta.

Comportamento comum:

- Todos os prompts de `ptool.ask` exigem um TTY interativo. Executá-los em um
  ambiente não interativo gera erro.
- Se o usuário cancelar um prompt, o script gera erro.
- Nomes de opção desconhecidos ou tipos de valor inválidos geram erro.

### ptool.ask

`ptool.ask(prompt[, options])` solicita ao usuário uma linha de texto e retorna
a resposta.

- `prompt` (string, obrigatório): O prompt mostrado ao usuário.
- `options` (table, opcional): Opções do prompt. Campos suportados:
  - `default` (string, opcional): Valor padrão usado quando o usuário envia uma
    resposta vazia.
  - `help` (string, opcional): Texto de ajuda extra exibido abaixo do prompt.
  - `placeholder` (string, opcional): Texto placeholder mostrado antes de o
    usuário começar a digitar.
  - `required` (boolean, opcional): Se a resposta deve ser não vazia.
  - `allow_empty` (boolean, opcional): Se uma resposta vazia é aceita.
    O padrão é `true`.
  - `trim` (boolean, opcional): Se os espaços no início e no fim devem ser
    removidos antes de retornar a resposta.
  - `min_length` (integer, opcional): Comprimento mínimo aceito.
  - `max_length` (integer, opcional): Comprimento máximo aceito.
  - `pattern` (string, opcional): Expressão regular que a resposta deve
    corresponder.
- Retorna: `string`.

Exemplo:

```lua
local project = ptool.ask("Project name?", {
  placeholder = "my-tool",
  help = "Lowercase letters, digits, and dashes only",
  required = true,
  trim = true,
  pattern = "^[a-z0-9-]+$",
})
```

### ptool.ask.confirm

> `v0.5.0` - Introduced.

`ptool.ask.confirm(prompt[, options])` solicita ao usuário uma resposta de
sim/não.

- `prompt` (string, obrigatório): O prompt mostrado ao usuário.
- `options` (table, opcional): Opções do prompt. Campos suportados:
  - `default` (boolean, opcional): Resposta padrão quando o usuário pressiona
    Enter sem digitar.
  - `help` (string, opcional): Texto de ajuda extra exibido abaixo do prompt.
- Retorna: `boolean`.

Exemplo:

```lua
local confirmed = ptool.ask.confirm("Continue?", {
  default = true,
})
```

### ptool.ask.select

> `v0.5.0` - Introduced.

`ptool.ask.select(prompt, items[, options])` solicita ao usuário que escolha um
item de uma lista.

- `prompt` (string, obrigatório): O prompt mostrado ao usuário.
- `items` (table, obrigatório): Itens candidatos. Cada entrada pode ser:
  - Um string, usado tanto como rótulo exibido quanto como valor retornado.
  - Um table como `{ label = "Patch", value = "patch" }`.
- `options` (table, opcional): Opções do prompt. Campos suportados:
  - `help` (string, opcional): Texto de ajuda extra exibido abaixo do prompt.
  - `page_size` (integer, opcional): Número máximo de linhas mostradas por vez.
  - `default_index` (integer, opcional): Índice 1-based do item inicialmente
    selecionado.
- Retorna: `string`.

Exemplo:

```lua
local bump = ptool.ask.select("Select bump type", {
  { label = "Patch", value = "patch" },
  { label = "Minor", value = "minor" },
  { label = "Major", value = "major" },
}, {
  default_index = 2,
})
```

### ptool.ask.multiselect

> `v0.5.0` - Introduced.

`ptool.ask.multiselect(prompt, items[, options])` solicita ao usuário que
escolha zero ou mais itens de uma lista.

- `prompt` (string, obrigatório): O prompt mostrado ao usuário.
- `items` (table, obrigatório): Itens candidatos. O formato é o mesmo de
  `ptool.ask.select`.
- `options` (table, opcional): Opções do prompt. Campos suportados:
  - `help` (string, opcional): Texto de ajuda extra exibido abaixo do prompt.
  - `page_size` (integer, opcional): Número máximo de linhas mostradas por vez.
  - `default_indexes` (table, opcional): Índices 1-based selecionados por
    padrão.
  - `min_selected` (integer, opcional): Quantidade mínima de itens que devem
    ser selecionados.
  - `max_selected` (integer, opcional): Quantidade máxima de itens que podem
    ser selecionados.
- Retorna: `table`.

Exemplo:

```lua
local targets = ptool.ask.multiselect("Select targets", {
  "linux",
  "macos",
  "windows",
}, {
  default_indexes = { 1, 2 },
  min_selected = 1,
})
```

### ptool.ask.secret

> `v0.5.0` - Introduced.

`ptool.ask.secret(prompt[, options])` solicita ao usuário uma entrada secreta,
como um token ou senha.

- `prompt` (string, obrigatório): O prompt mostrado ao usuário.
- `options` (table, opcional): Opções do prompt. Campos suportados:
  - `help` (string, opcional): Texto de ajuda extra exibido abaixo do prompt.
  - `required` (boolean, opcional): Se a resposta deve ser não vazia.
  - `allow_empty` (boolean, opcional): Se uma resposta vazia é aceita.
    O padrão é `false`.
  - `confirm` (boolean, opcional): Se deve pedir que o usuário digite o segredo
    duas vezes. O padrão é `false`.
  - `confirm_prompt` (string, opcional): Prompt personalizado para a etapa de
    confirmação.
  - `mismatch_message` (string, opcional): Mensagem de erro personalizada
    mostrada quando as duas respostas não coincidem.
  - `display_toggle` (boolean, opcional): Se deve permitir mostrar
    temporariamente o segredo digitado.
  - `min_length` (integer, opcional): Comprimento mínimo aceito.
  - `max_length` (integer, opcional): Comprimento máximo aceito.
  - `pattern` (string, opcional): Expressão regular que a resposta deve
    corresponder.
- Retorna: `string`.

Exemplo:

```lua
local token = ptool.ask.secret("API token?", {
  confirm = true,
  min_length = 20,
})
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

## ptool.script_path

> `v0.4.0` - Introduced.

`ptool.script_path()` retorna o caminho absoluto do script de entrada atual.

- Retorna: `string|nil`.

Comportamento:

- Ao executar com `ptool run <file>`, retorna o caminho do script de entrada
  como um caminho absoluto e normalizado.
- O caminho retornado é fixado quando o runtime inicia e não muda após
  `ptool.cd(...)`.
- Em `ptool repl`, retorna `nil`.

Exemplo:

```lua
local script_path = ptool.script_path()
local script_dir = ptool.path.dirname(script_path)
local project_root = ptool.path.dirname(script_dir)
```

## ptool.try

> `v0.4.0` - Introduced.

`ptool.try(fn)` executa `fn` e converte erros lançados em valores de retorno.

- `fn` (function, obrigatório): Callback a ser executado.
- Retorna: `ok, value, err`.

Regras do valor de retorno:

- Em caso de sucesso, `ok = true`, `err = nil` e `value` contém o resultado do
  callback.
- Se o callback não retornar valores, `value` será `nil`.
- Se o callback retornar um valor, `value` será esse valor.
- Se o callback retornar vários valores, `value` será uma tabela do tipo array.
- Em caso de falha, `ok = false`, `value = nil` e `err` será uma tabela.

Campos de erro estruturado:

- `kind` (string): Categoria estável do erro, como `io_error`,
  `command_failed`, `invalid_argument`, `http_error` ou `lua_error`.
- `message` (string): Mensagem de erro legível.
- `op` (string, opcional): Nome da API ou da operação, como `ptool.fs.read`.
- `detail` (string, opcional): Detalhe adicional da falha.
- `path` (string, opcional): Caminho envolvido em uma falha de sistema de
  arquivos.
- `input` (string, opcional): Entrada original que falhou na análise ou
  validação.
- `cmd` (string, opcional): Nome do comando em falhas de comando.
- `status` (integer, opcional): Código de saída ou código HTTP quando
  disponível.
- `stderr` (string, opcional): stderr capturado em falhas de comando.
- `url` (string, opcional): URL envolvida em uma falha HTTP.
- `cwd` (string, opcional): Diretório de trabalho efetivo usado em falhas de
  comando.
- `target` (string, opcional): Alvo SSH em falhas de comando relacionadas a
  SSH.
- `retryable` (boolean): Se faz sentido tentar novamente. O padrão é `false`.

Comportamento:

- As APIs de `ptool` lançam erros estruturados. `ptool.try` os converte na
  tabela `err` acima para que quem chama possa fazer branching por `err.kind`
  e campos relacionados.
- Erros Lua comuns também são capturados. Nesse caso, `err.kind` será
  `lua_error` e apenas `message` é garantido.
- `ptool.try` é a forma recomendada de tratar erros de APIs como
  `ptool.fs.read`, `ptool.http.request`, `ptool.run(..., { check = true })` e
  `res:assert_ok()`.

Exemplo:

```lua
local ok, content, err = ptool.try(function()
  return ptool.fs.read("missing.txt")
end)

if not ok and err.kind == "io_error" then
  print(err.op, err.path)
end

local ok2, _, err2 = ptool.try(function()
  local res = ptool.run({
    cmd = "sh",
    args = {"-c", "echo bad >&2; exit 7"},
    stderr = "capture",
  })
  res:assert_ok()
end)

if not ok2 and err2.kind == "command_failed" then
  print(err2.cmd, err2.status, err2.stderr)
end
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
  - `cmd` (string): Nome do comando usado na execução.
  - `cwd` (string): Diretório de trabalho efetivo usado na execução.
  - `stdout` (string, opcional): Presente quando `stdout = "capture"`.
  - `stderr` (string, opcional): Presente quando `stderr = "capture"`.
  - `assert_ok(self)` (function): Gera um erro estruturado quando `ok = false`.
    O tipo de erro é `command_failed` e ele pode incluir `cmd`, `status`,
    `stderr` e `cwd`.
- O valor padrão de `check` vem de
  `ptool.config({ run = { check = ... } })`. Se não estiver configurado,
  o padrão é `false`. Quando `check = false`, quem chama pode inspecionar `ok`
  diretamente ou chamar `res:assert_ok()`.
- Quando `check = true` e `retry = true`, `ptool.run` pergunta se o comando
  com falha deve ser tentado novamente antes de gerar o erro final.
- Quando `check = true`, `ptool.run` gera o mesmo erro estruturado
  `command_failed` que `res:assert_ok()` gera. Use `ptool.try(...)` se quiser
  capturá-lo e inspecioná-lo em Lua.

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
