# API de processos

Os utilitários de processos locais estão disponíveis em `ptool.proc` e `p.proc`.

Este módulo serve para inspecionar e gerenciar processos locais que já estão em execução. Use `ptool.run(...)` quando quiser iniciar um novo comando.

## ptool.proc.self

> `Unreleased` - Introduzido.

`ptool.proc.self()` retorna uma tabela de snapshot do processo `ptool` atual.

- Retorna: `table`.

A tabela retornada usa o mesmo formato de `ptool.proc.get(...)` e `ptool.proc.find(...)`.

## ptool.proc.get

> `Unreleased` - Introduzido.

`ptool.proc.get(pid)` retorna uma tabela de snapshot para o ID de processo informado, ou `nil` se o processo não existir.

- `pid` (integer, obrigatório): ID do processo.
- Retorna: `table|nil`.

## ptool.proc.exists

> `Unreleased` - Introduzido.

`ptool.proc.exists(pid)` informa se um ID de processo existe no momento.

- `pid` (integer, obrigatório): ID do processo.
- Retorna: `boolean`.

## ptool.proc.find

> `Unreleased` - Introduzido.

`ptool.proc.find([options])` lista processos locais e retorna um array de tabelas de snapshot.

- `options` (table, opcional): Opções de filtragem e ordenação.
- Retorna: `table`.

Campos `options` suportados:

- `pid` (integer, opcional): Corresponde a um único ID de processo exato.
- `pids` (integer[], opcional): Corresponde a um conjunto de IDs de processo.
- `ppid` (integer, opcional): Corresponde a um ID exato do processo pai.
- `name` (string, opcional): Corresponde a um nome de processo exato.
- `name_contains` (string, opcional): Corresponde a uma substring no nome do processo.
- `exe` (string, opcional): Corresponde a um caminho exato do executável.
- `exe_contains` (string, opcional): Corresponde a uma substring no caminho do executável.
- `cmdline_contains` (string, opcional): Corresponde a uma substring na linha de comando concatenada.
- `user` (string, opcional): Corresponde a um nome de usuário exato.
- `cwd` (string, opcional): Corresponde a um diretório de trabalho atual exato.
- `include_self` (boolean, opcional): Se deve incluir o processo `ptool` atual. O padrão é `false`.
- `limit` (integer, opcional): Número máximo de entradas retornadas após a filtragem e a ordenação.
- `sort_by` (string, opcional): Chave de ordenação. Valores suportados:
  - `"pid"` (padrão)
  - `"start_time"`
- `reverse` (boolean, opcional): Se deve inverter a ordem final de ordenação. O padrão é `false`.

Cada snapshot de processo retornado pode conter:

- `pid` (integer): ID do processo.
- `ppid` (integer|nil): ID do processo pai.
- `name` (string): Nome do processo.
- `exe` (string|nil): Caminho do executável, quando disponível.
- `cwd` (string|nil): Diretório de trabalho atual, quando disponível.
- `user` (string|nil): Nome do usuário proprietário, quando disponível.
- `cmdline` (string|nil): Linha de comando concatenada, quando disponível.
- `argv` (string[]): Array de argumentos da linha de comando.
- `state` (string): Rótulo do estado do processo, como `"running"` ou `"sleeping"`.
- `start_time_unix_ms` (integer): Horário de início do processo em milissegundos Unix.

Notas:

- Alguns campos podem ser `nil` quando a plataforma atual ou o nível de permissão não os expõe.
- Snapshots de processo são valores de um ponto específico no tempo. Eles não se atualizam sozinhos.

Exemplo:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
  include_self = true,
  sort_by = "start_time",
})

for _, proc in ipairs(procs) do
  print(proc.pid, proc.name, proc.cmdline)
end
```

## ptool.proc.kill

> `Unreleased` - Introduzido.

`ptool.proc.kill(targets[, options])` envia um sinal para um ou mais processos locais e retorna uma tabela de resultado estruturada.

- `targets` (integer|table, obrigatório): Um pid, uma tabela de snapshot de processo ou um array deles.
- `options` (table, opcional): Opções de sinal.
- Retorna: `table`.

Campos `options` suportados:

- `signal` (string, opcional): Nome do sinal. Valores suportados:
  - `"hup"`
  - `"term"` (padrão)
  - `"kill"`
  - `"int"`
  - `"quit"`
  - `"stop"`
  - `"cont"`
  - `"user1"`
  - `"user2"`
- `missing_ok` (boolean, opcional): Se processos ausentes contam como sucesso. O padrão é `true`.
- `allow_self` (boolean, opcional): Se o processo `ptool` atual pode receber o sinal. O padrão é `false`.
- `check` (boolean, opcional): Se deve gerar um erro imediatamente quando o resultado final não for ok. O padrão é `false`.
- `confirm` (boolean, opcional): Se deve pedir confirmação antes de enviar o sinal. O padrão é `false`.

A tabela de resultado retornada contém:

- `ok` (boolean): Se toda a operação foi bem-sucedida sob as opções atuais.
- `signal` (string): O rótulo de sinal solicitado.
- `total` (integer): Número total de alvos normalizados.
- `sent` (integer): Número de alvos para os quais o sinal foi enviado.
- `missing` (integer): Número de alvos que já não existiam.
- `failed` (integer): Número de alvos que falharam no total.
- `entries` (table): Entradas de resultado por alvo.
- `assert_ok(self)` (function): Gera um erro Lua estruturado quando `ok = false`.

Cada tabela `entries[i]` contém:

- `pid` (integer): ID do processo alvo.
- `ok` (boolean): Se este alvo foi bem-sucedido.
- `existed` (boolean): Se o processo alvo existia e ainda correspondia.
- `signal` (string): O rótulo de sinal solicitado.
- `message` (string|nil): Detalhe de status adicional.

Exemplo:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
})

local res = p.proc.kill(procs, {
  signal = "term",
  confirm = true,
})

res:assert_ok()
```

## ptool.proc.wait_gone

> `Unreleased` - Introduzido.

`ptool.proc.wait_gone(targets[, options])` espera até que um ou mais processos alvo deixem de existir e então retorna uma tabela de resultado estruturada.

- `targets` (integer|table, obrigatório): Um pid, uma tabela de snapshot de processo ou um array deles.
- `options` (table, opcional): Opções de espera.
- Retorna: `table`.

Campos `options` suportados:

- `timeout_ms` (integer, opcional): Tempo máximo de espera em milissegundos. Se omitido, a espera é indefinida.
- `interval_ms` (integer, opcional): Intervalo de polling em milissegundos. O padrão é `100`.
- `check` (boolean, opcional): Se deve gerar um erro imediatamente quando a espera atingir o tempo limite. O padrão é `false`.

A tabela de resultado retornada contém:

- `ok` (boolean): Se todos os processos alvo desapareceram antes do tempo limite.
- `timed_out` (boolean): Se o tempo limite foi atingido.
- `total` (integer): Número total de alvos normalizados.
- `gone` (integer[]): IDs de processo que já tinham desaparecido ao fim da espera.
- `remaining` (integer[]): IDs de processo ainda presentes quando a espera terminou.
- `elapsed_ms` (integer): Tempo total de espera decorrido em milissegundos.
- `assert_ok(self)` (function): Gera um erro Lua estruturado quando `ok = false`.

Exemplo:

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
})

local wait_res = p.proc.wait_gone(procs, {
  timeout_ms = 1000,
  interval_ms = 100,
})

wait_res:assert_ok()
```
