# Database API

Database connection and query helpers are available under `ptool.db` and `p.db`.

## ptool.db.connect

> `v0.1.0` - Introduced.

`ptool.db.connect(url_or_options)` opens a database connection and returns a `Connection` object.

Supported databases:

- SQLite
- PostgreSQL
- MySQL

Arguments:

- `url_or_options` (string|table, required):
  - When a string is provided, it is treated as the database URL.
  - When a table is provided, it currently supports:
    - `url` (string, required): The database URL.

Supported URL examples:

```lua
local sqlite_db = ptool.db.connect("sqlite:test.db")
local pg_db = ptool.db.connect("postgres://user:pass@localhost/app")
local mysql_db = ptool.db.connect("mysql://user:pass@localhost/app")
```

SQLite notes:

- `sqlite:test.db` and `sqlite://test.db` are supported.
- Relative SQLite paths are resolved from the current `ptool` runtime directory, so they follow `ptool.cd(...)`.
- If no `mode=` query parameter is provided, SQLite connections default to `mode=rwc`, which allows creating the database file automatically.

Example:

```lua
ptool.cd("workdir")
local db = ptool.db.connect({
  url = "sqlite:data/app.db",
})
```

## Connection

> `v0.1.0` - Introduced.

`Connection` represents an open database connection returned by `ptool.db.connect()`.

It is implemented as a Lua userdata.

Methods:

- `db:query(sql, params?)` -> `table`
- `db:query_one(sql, params?)` -> `table|nil`
- `db:scalar(sql, params?)` -> `boolean|integer|number|string|nil`
- `db:execute(sql, params?)` -> `table`
- `db:transaction(fn)` -> `any`
- `db:close()` -> `nil`

Parameter binding:

- `params` is optional.
- When `params` is an array table, it is treated as positional parameters and SQL placeholders should use `?`.
- When `params` is a key-value table, it is treated as named parameters and SQL placeholders should use `:name`.
- Positional and named parameters cannot be mixed in the same call.
- Supported parameter value types are:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil` is not supported as a bound parameter in `v0.1.0`.

Result value rules:

- Query results only guarantee these Lua value types:
  - `boolean`
  - `integer`
  - `number`
  - `string`
  - `nil` (for SQL `NULL`)
- Text columns are returned as Lua strings.
- Binary/blob columns are also returned as Lua strings.
- If a query result contains duplicate column names, an error is raised. Use SQL aliases such as `AS` to disambiguate them.

### query

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:query`.

`db:query(sql, params?)` executes a query and returns a table with:

- `rows` (table): An array of row tables.
- `columns` (table): An array of column names.
- `row_count` (integer): The number of rows returned.

Example:

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

`db:query_one(sql, params?)` returns the first row as a table, or `nil` if the query returns no rows.

Example:

```lua
local row = db:query_one("select id, name from users where name = ?", {"alice"})
if row then
  print(row.id, row.name)
end
```

### scalar

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:scalar`.

`db:scalar(sql, params?)` returns the first column of the first row, or `nil` if the query returns no rows.

Example:

```lua
local count = db:scalar("select count(*) from users")
print(count)
```

### execute

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:execute`.

`db:execute(sql, params?)` executes a statement and returns a table with:

- `rows_affected` (integer): The number of affected rows.

Example:

```lua
local res = db:execute("update users set name = ? where id = ?", {"alice-2", 1})
print(res.rows_affected)
```

### transaction

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:transaction`.

`db:transaction(fn)` runs `fn(tx)` inside a database transaction.

Behavior:

- If `fn(tx)` returns normally, the transaction is committed.
- If `fn(tx)` raises an error, the transaction is rolled back and the error is re-raised.
- Nested transactions are not supported.
- While the callback is active, the outer connection object must not be used; use the provided `tx` object instead.

The `tx` object supports the same query methods as `Connection`:

- `tx:query(sql, params?)`
- `tx:query_one(sql, params?)`
- `tx:scalar(sql, params?)`
- `tx:execute(sql, params?)`

Example:

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

`db:close()` closes the connection.

Behavior:

- After closing, the connection can no longer be used.
- Closing during an active transaction callback raises an error.

Example:

```lua
local db = ptool.db.connect("sqlite:test.db")
db:close()
```
