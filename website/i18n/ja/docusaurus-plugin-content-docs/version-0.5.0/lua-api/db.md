# データベース API

データベース接続とクエリのヘルパーは `ptool.db` と `p.db` にあります。

## ptool.db.connect

> `v0.1.0` - Introduced.

`ptool.db.connect(url_or_options)` はデータベース接続を開き、
`Connection` オブジェクトを返します。

サポートされるデータベース:

- SQLite
- PostgreSQL
- MySQL

引数:

- `url_or_options` (string|table, 必須):
  - 文字列が渡された場合は、データベース URL として扱われます。
  - テーブルが渡された場合、現在サポートされるのは次のとおりです:
    - `url` (string, 必須): データベース URL。

サポートされる URL の例:

```lua
local sqlite_db = ptool.db.connect("sqlite:test.db")
local pg_db = ptool.db.connect("postgres://user:pass@localhost/app")
local mysql_db = ptool.db.connect("mysql://user:pass@localhost/app")
```

SQLite に関する注意:

- `sqlite:test.db` と `sqlite://test.db` がサポートされます。
- 相対 SQLite パスは現在の `ptool` ランタイムディレクトリから解決される
  ため、`ptool.cd(...)` に従います。
- `mode=` クエリパラメータが指定されていない場合、SQLite 接続のデフォルト
  は `mode=rwc` で、データベースファイルの自動作成が可能です。

例:

```lua
ptool.cd("workdir")
local db = ptool.db.connect({
  url = "sqlite:data/app.db",
})
```

## Connection

> `v0.1.0` - Introduced.

`Connection` は `ptool.db.connect()` が返す開かれたデータベース接続を
表します。

これは Lua userdata として実装されています。

メソッド:

- `db:query(sql, params?)` -> `table`
- `db:query_one(sql, params?)` -> `table|nil`
- `db:scalar(sql, params?)` -> `boolean|integer|number|string|nil`
- `db:execute(sql, params?)` -> `table`
- `db:transaction(fn)` -> `any`
- `db:close()` -> `nil`

パラメータバインディング:

- `params` は任意です。
- `params` が配列テーブルの場合は位置パラメータとして扱われ、
  SQL プレースホルダーには `?` を使います。
- `params` がキーと値のテーブルの場合は名前付きパラメータとして扱われ、
  SQL プレースホルダーには `:name` を使います。
- 位置パラメータと名前付きパラメータは同じ呼び出し内で混在できません。
- サポートされるパラメータ値の型:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `v0.1.0` では、バインドパラメータとして `nil` はサポートされません。

戻り値のルール:

- クエリ結果で保証される Lua 値型は次のものだけです:
  - `boolean`
  - `integer`
  - `number`
  - `string`
  - `nil` (SQL `NULL` 用)
- テキスト列は Lua 文字列として返されます。
- バイナリ/blob 列も Lua 文字列として返されます。
- クエリ結果に重複した列名が含まれる場合はエラーになります。`AS` などの
  SQL エイリアスで区別してください。

### query

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:query`.

`db:query(sql, params?)` はクエリを実行し、次を持つテーブルを返します:

- `rows` (table): 行テーブルの配列。
- `columns` (table): 列名の配列。
- `row_count` (integer): 返された行数。

例:

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

`db:query_one(sql, params?)` は最初の 1 行をテーブルとして返します。
クエリ結果が 0 行なら `nil` を返します。

例:

```lua
local row = db:query_one("select id, name from users where name = ?", {"alice"})
if row then
  print(row.id, row.name)
end
```

### scalar

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:scalar`.

`db:scalar(sql, params?)` は最初の行の最初の列を返します。クエリ結果が
0 行なら `nil` を返します。

例:

```lua
local count = db:scalar("select count(*) from users")
print(count)
```

### execute

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:execute`.

`db:execute(sql, params?)` は文を実行し、次を持つテーブルを返します:

- `rows_affected` (integer): 影響を受けた行数。

例:

```lua
local res = db:execute("update users set name = ? where id = ?", {"alice-2", 1})
print(res.rows_affected)
```

### transaction

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:transaction`.

`db:transaction(fn)` はデータベーストランザクション内で `fn(tx)` を
実行します。

挙動:

- `fn(tx)` が正常に返れば、トランザクションはコミットされます。
- `fn(tx)` がエラーを投げると、トランザクションはロールバックされ、
  そのエラーが再送出されます。
- ネストしたトランザクションはサポートされません。
- コールバックが有効な間は外側の接続オブジェクトを使ってはいけません。
  代わりに渡された `tx` オブジェクトを使ってください。

`tx` オブジェクトは `Connection` と同じクエリメソッドをサポートします:

- `tx:query(sql, params?)`
- `tx:query_one(sql, params?)`
- `tx:scalar(sql, params?)`
- `tx:execute(sql, params?)`

例:

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

`db:close()` は接続を閉じます。

挙動:

- 閉じた後、その接続はもう使えません。
- アクティブなトランザクションコールバックの最中に閉じるとエラーに
  なります。

例:

```lua
local db = ptool.db.connect("sqlite:test.db")
db:close()
```
