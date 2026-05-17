# Database API

数据库连接与查询辅助能力位于 `ptool.db` 和 `p.db` 下。

## ptool.db.connect

> `v0.1.0` - 引入。

`ptool.db.connect(url_or_options)` 打开数据库连接，并返回一个 `Connection` 对象。

支持的数据库：

- SQLite
- PostgreSQL
- MySQL

参数：

- `url_or_options`（string|table，必填）：
  - 传入字符串时，会被视为数据库 URL。
  - 传入 table 时，目前支持：
    - `url`（string，必填）：数据库 URL。

支持的 URL 示例：

```lua
local sqlite_db = ptool.db.connect("sqlite:test.db")
local pg_db = ptool.db.connect("postgres://user:pass@localhost/app")
local mysql_db = ptool.db.connect("mysql://user:pass@localhost/app")
```

SQLite 说明：

- 支持 `sqlite:test.db` 和 `sqlite://test.db`。
- 相对 SQLite 路径会从当前 `ptool` 运行时目录解析，因此会受到 `ptool.cd(...)` 的影响。
- 如果没有提供 `mode=` 查询参数，SQLite 连接默认使用 `mode=rwc`，从而允许自动 创建数据库文件。

示例：

```lua
ptool.cd("workdir")
local db = ptool.db.connect({
  url = "sqlite:data/app.db",
})
```

## Connection

> `v0.1.0` - 引入。

`Connection` 表示由 `ptool.db.connect()` 返回的已打开数据库连接。

它实现为 Lua userdata。

方法：

- `db:query(sql, params?)` -> `table`
- `db:query_one(sql, params?)` -> `table|nil`
- `db:scalar(sql, params?)` -> `boolean|integer|number|string|nil`
- `db:execute(sql, params?)` -> `table`
- `db:transaction(fn)` -> `any`
- `db:close()` -> `nil`

参数绑定：

- `params` 可选。
- 当 `params` 是数组表时，会视为位置参数，SQL 占位符应使用 `?`。
- 当 `params` 是键值表时，会视为命名参数，SQL 占位符应使用 `:name`。
- 同一次调用中不能混用位置参数和命名参数。
- 支持的绑定值类型包括：
  - `boolean`
  - `integer`
  - `number`
  - `string`
- 在 `v0.1.0` 中，不支持将 `nil` 作为绑定参数。

结果值规则：

- 查询结果仅保证返回以下 Lua 值类型：
  - `boolean`
  - `integer`
  - `number`
  - `string`
  - `nil`（对应 SQL `NULL`）
- 文本列会作为 Lua 字符串返回。
- 二进制 / blob 列也会作为 Lua 字符串返回。
- 如果查询结果中出现重复列名，会抛出错误。请使用 `AS` 等 SQL 别名消除歧义。

### query

> `v0.1.0` - 引入。

规范 API 名称：`ptool.db.Connection:query`。

`db:query(sql, params?)` 执行查询，并返回一个包含以下字段的表：

- `rows`（table）：行表数组。
- `columns`（table）：列名数组。
- `row_count`（integer）：返回的行数。

示例：

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

> `v0.1.0` - 引入。

规范 API 名称：`ptool.db.Connection:query_one`。

`db:query_one(sql, params?)` 返回第一行记录（table），如果查询没有结果则返回 `nil`。

示例：

```lua
local row = db:query_one("select id, name from users where name = ?", {"alice"})
if row then
  print(row.id, row.name)
end
```

### scalar

> `v0.1.0` - 引入。

规范 API 名称：`ptool.db.Connection:scalar`。

`db:scalar(sql, params?)` 返回第一行第一列的值，如果查询没有结果则返回 `nil`。

示例：

```lua
local count = db:scalar("select count(*) from users")
print(count)
```

### execute

> `v0.1.0` - 引入。

规范 API 名称：`ptool.db.Connection:execute`。

`db:execute(sql, params?)` 执行语句，并返回一个包含以下字段的表：

- `rows_affected`（integer）：受影响的行数。

示例：

```lua
local res = db:execute("update users set name = ? where id = ?", {"alice-2", 1})
print(res.rows_affected)
```

### transaction

> `v0.1.0` - 引入。

规范 API 名称：`ptool.db.Connection:transaction`。

`db:transaction(fn)` 在数据库事务中执行 `fn(tx)`。

行为说明：

- 如果 `fn(tx)` 正常返回，则提交事务。
- 如果 `fn(tx)` 抛错，则回滚事务并重新抛出错误。
- 不支持嵌套事务。
- 在回调执行期间，不得使用外层连接对象；应改用传入的 `tx` 对象。

`tx` 对象支持与 `Connection` 相同的查询方法：

- `tx:query(sql, params?)`
- `tx:query_one(sql, params?)`
- `tx:scalar(sql, params?)`
- `tx:execute(sql, params?)`

示例：

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

> `v0.1.0` - 引入。

规范 API 名称：`ptool.db.Connection:close`。

`db:close()` 关闭连接。

行为说明：

- 关闭后，连接不能再继续使用。
- 如果在活动事务回调期间关闭连接，会抛出错误。

示例：

```lua
local db = ptool.db.connect("sqlite:test.db")
db:close()
```
