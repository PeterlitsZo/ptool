# API de banco de dados

As utilidades de conexão e consulta a banco de dados estão disponíveis em `ptool.db` e `p.db`.

## ptool.db.connect

> `v0.1.0` - Introduced.

`ptool.db.connect(url_or_options)` abre uma conexão de banco de dados e retorna um objeto `Connection`.

Bancos de dados suportados:

- SQLite
- PostgreSQL
- MySQL

Argumentos:

- `url_or_options` (string|table, obrigatório):
  - Quando uma string é fornecida, ela é tratada como a URL do banco de dados.
  - Quando uma tabela é fornecida, atualmente ela suporta:
    - `url` (string, obrigatório): A URL do banco de dados.

Exemplos de URL suportadas:

```lua
local sqlite_db = ptool.db.connect("sqlite:test.db")
local pg_db = ptool.db.connect("postgres://user:pass@localhost/app")
local mysql_db = ptool.db.connect("mysql://user:pass@localhost/app")
```

Notas sobre SQLite:

- `sqlite:test.db` e `sqlite://test.db` são suportadas.
- Caminhos SQLite relativos são resolvidos a partir do diretório de runtime atual do `ptool`, então seguem `ptool.cd(...)`.
- Se nenhum parâmetro de query `mode=` for fornecido, conexões SQLite usam `mode=rwc` por padrão, o que permite criar o arquivo do banco automaticamente.

Exemplo:

```lua
ptool.cd("workdir")
local db = ptool.db.connect({
  url = "sqlite:data/app.db",
})
```

## Connection

> `v0.1.0` - Introduced.

`Connection` representa uma conexão de banco de dados aberta retornada por `ptool.db.connect()`.

Ela é implementada como userdata Lua.

Métodos:

- `db:query(sql, params?)` -> `table`
- `db:query_one(sql, params?)` -> `table|nil`
- `db:scalar(sql, params?)` -> `boolean|integer|number|string|nil`
- `db:execute(sql, params?)` -> `table`
- `db:transaction(fn)` -> `any`
- `db:close()` -> `nil`

Binding de parâmetros:

- `params` é opcional.
- Quando `params` é uma tabela array, ela é tratada como parâmetros posicionais e placeholders SQL devem usar `?`.
- Quando `params` é uma tabela chave-valor, ela é tratada como parâmetros nomeados e placeholders SQL devem usar `:name`.
- Parâmetros posicionais e nomeados não podem ser misturados na mesma chamada.
- Os tipos de valor de parâmetro suportados são:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil` não é suportado como parâmetro vinculado em `v0.1.0`.

Regras de valor de resultado:

- Resultados de consulta garantem apenas estes tipos de valor Lua:
  - `boolean`
  - `integer`
  - `number`
  - `string`
  - `nil` (para SQL `NULL`)
- Colunas de texto são retornadas como strings Lua.
- Colunas binárias/blob também são retornadas como strings Lua.
- Se um resultado de consulta contiver nomes de coluna duplicados, um erro é gerado. Use aliases SQL como `AS` para desambiguá-los.

### query

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:query`.

`db:query(sql, params?)` executa uma consulta e retorna uma tabela com:

- `rows` (table): Um array de tabelas de linha.
- `columns` (table): Um array com nomes de coluna.
- `row_count` (integer): O número de linhas retornadas.

Exemplo:

```lua
local db = ptool.db.connect("sqlite:test.db")

db:execute("create table users (id integer primary key, name text)")
db:execute("insert into users(name) values (?)", {"alice"})
db:execute("insert into users(name) values (:name)", { name = "bob" })

local res = db:query("select id, name from users order by id")
print(res.row_count)
print(res.columns[1], res.columns[2])
print(res.rows[1].name)
print(res.rows[2].name)
```

### query_one

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:query_one`.

`db:query_one(sql, params?)` retorna a primeira linha como tabela, ou `nil` se a consulta não retornar linhas.

Exemplo:

```lua
local row = db:query_one("select id, name from users where name = ?", {"alice"})
if row then
  print(row.id, row.name)
end
```

### scalar

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:scalar`.

`db:scalar(sql, params?)` retorna a primeira coluna da primeira linha, ou `nil` se a consulta não retornar linhas.

Exemplo:

```lua
local count = db:scalar("select count(*) from users")
print(count)
```

### execute

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:execute`.

`db:execute(sql, params?)` executa uma instrução e retorna uma tabela com:

- `rows_affected` (integer): O número de linhas afetadas.

Exemplo:

```lua
local res = db:execute("update users set name = ? where id = ?", {"alice-2", 1})
print(res.rows_affected)
```

### transaction

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:transaction`.

`db:transaction(fn)` executa `fn(tx)` dentro de uma transação de banco de dados.

Comportamento:

- Se `fn(tx)` retornar normalmente, a transação é confirmada.
- Se `fn(tx)` gerar um erro, a transação é revertida e o erro é relançado.
- Transações aninhadas não são suportadas.
- Enquanto o callback estiver ativo, o objeto de conexão externo não deve ser usado; use o objeto `tx` fornecido em vez disso.

O objeto `tx` suporta os mesmos métodos de consulta que `Connection`:

- `tx:query(sql, params?)`
- `tx:query_one(sql, params?)`
- `tx:scalar(sql, params?)`
- `tx:execute(sql, params?)`

Exemplo:

```lua
db:transaction(function(tx)
  tx:execute("insert into users(name) values (?)", {"charlie"})
  tx:execute("insert into users(name) values (?)", {"dora"})
end)

local ok, err = pcall(function()
  db:transaction(function(tx)
    tx:execute("insert into users(name) values (?)", {"eve"})
    error("stop")
  end)
end)
print(ok) -- false
print(tostring(err))
```

### close

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:close`.

`db:close()` fecha a conexão.

Comportamento:

- Depois de fechada, a conexão não pode mais ser usada.
- Fechar durante um callback de transação ativa gera erro.

Exemplo:

```lua
local db = ptool.db.connect("sqlite:test.db")
db:close()
```
