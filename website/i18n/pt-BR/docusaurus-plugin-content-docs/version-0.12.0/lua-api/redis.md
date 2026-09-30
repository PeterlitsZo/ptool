# API de Redis

Os utilitários de conexão Redis estão disponíveis em `ptool.redis` e `p.redis`.

## ptool.redis.connect

> `v0.9.0` - Introduced.

`ptool.redis.connect(url_or_options)` abre uma conexão Redis e retorna um objeto `Connection`.

Argumentos:

- `url_or_options` (string|table, obrigatório):
  - Quando uma string é fornecida, ela é tratada como a URL Redis.
  - Quando uma tabela é fornecida, atualmente ela suporta:
    - `url` (string, obrigatório): A URL Redis.

Exemplos comuns de URL:

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")
local db1 = ptool.redis.connect("redis://127.0.0.1/1")
local auth = ptool.redis.connect({
  url = "redis://:secret@cache.internal:6379/0",
})
```

Notas:

- A conexão foi pensada para execução direta de comandos por meio de `conn:call(...)`.
- Atualmente esta API não oferece helpers dedicados para pub/sub, pipeline ou transações.

## Connection

> `v0.9.0` - Introduced.

`Connection` representa uma conexão Redis aberta retornada por `ptool.redis.connect()`.

Ela é implementada como um userdata Lua.

Métodos:

- `conn:call(command, ...)` -> `any`
- `conn:close()` -> `nil`

Regras dos argumentos de comando:

- `command` deve ser uma string não vazia.
- Os argumentos restantes são passados ao Redis como uma lista plana de argumentos.
- Os tipos de valor suportados para argumentos são:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil`, tabelas, funções, threads e userdata não são suportados como argumentos de comandos Redis.
- Strings Lua são passadas como bytes brutos, então valores Redis binary-safe são suportados.

Regras de conversão de respostas:

- Respostas nulas se tornam `nil`.
- Respostas inteiras se tornam inteiros Lua.
- Bulk strings, simple strings e verbatim strings se tornam strings Lua.
- Respostas de array e set se tornam tabelas array densas do Lua.
- Respostas de mapa se tornam tabelas Lua quando as chaves do mapa podem ser representadas como strings, inteiros, números ou booleanos do Lua.
- Respostas double se tornam números Lua.
- Respostas booleanas se tornam booleanos Lua.
- Respostas de números grandes se tornam strings Lua.
- Respostas push se tornam tabelas no formato `{ kind = "...", data = {...} }`.

### call

> `v0.9.0` - Introduced.

Nome canônico da API: `ptool.redis.Connection:call`.

`conn:call(command, ...)` envia um comando Redis e retorna a resposta convertida.

Exemplo:

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")

redis:call("SET", "user:1:name", "alice")
print(redis:call("GET", "user:1:name")) -- alice

local count = redis:call("INCR", "counters:signup")
print(count)

local values = redis:call("MGET", "user:1:name", "missing")
print(values[1])        -- alice
print(values[2] == nil) -- true
```

### close

> `v0.9.0` - Introduced.

Nome canônico da API: `ptool.redis.Connection:close`.

`conn:close()` fecha a conexão.

Comportamento:

- Depois de fechada, a conexão não pode mais ser usada.

