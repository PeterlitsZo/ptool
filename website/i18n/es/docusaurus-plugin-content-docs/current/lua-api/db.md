# API de base de datos

Las utilidades de conexión y consulta de bases de datos están disponibles bajo
`ptool.db` y `p.db`.

## ptool.db.connect

> `v0.1.0` - Introduced.

`ptool.db.connect(url_or_options)` abre una conexión de base de datos y
devuelve un objeto `Connection`.

Bases de datos admitidas:

- SQLite
- PostgreSQL
- MySQL

Argumentos:

- `url_or_options` (string|table, obligatorio):
  - Cuando se proporciona una cadena, se trata como la URL de base de datos.
  - Cuando se proporciona una tabla, actualmente admite:
    - `url` (string, obligatorio): La URL de base de datos.

Ejemplos de URL admitidos:

```lua
local sqlite_db = ptool.db.connect("sqlite:test.db")
local pg_db = ptool.db.connect("postgres://user:pass@localhost/app")
local mysql_db = ptool.db.connect("mysql://user:pass@localhost/app")
```

Notas sobre SQLite:

- Se admiten `sqlite:test.db` y `sqlite://test.db`.
- Las rutas SQLite relativas se resuelven desde el directorio de ejecución
  actual de `ptool`, por lo que siguen a `ptool.cd(...)`.
- Si no se proporciona el parámetro de consulta `mode=`, las conexiones SQLite
  usan por defecto `mode=rwc`, lo que permite crear el archivo de base de datos
  automáticamente.

Ejemplo:

```lua
ptool.cd("workdir")
local db = ptool.db.connect({
  url = "sqlite:data/app.db",
})
```

## Connection

> `v0.1.0` - Introduced.

`Connection` representa una conexión de base de datos abierta devuelta por
`ptool.db.connect()`.

Está implementada como userdata de Lua.

Métodos:

- `db:query(sql, params?)` -> `table`
- `db:query_one(sql, params?)` -> `table|nil`
- `db:scalar(sql, params?)` -> `boolean|integer|number|string|nil`
- `db:execute(sql, params?)` -> `table`
- `db:transaction(fn)` -> `any`
- `db:close()` -> `nil`

Enlace de parámetros:

- `params` es opcional.
- Cuando `params` es una tabla tipo arreglo, se trata como parámetros
  posicionales y los placeholders SQL deben usar `?`.
- Cuando `params` es una tabla clave-valor, se trata como parámetros con nombre
  y los placeholders SQL deben usar `:name`.
- No se pueden mezclar parámetros posicionales y con nombre en la misma
  llamada.
- Los tipos de valor de parámetro admitidos son:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil` no se admite como parámetro enlazado en `v0.1.0`.

Reglas de valores devueltos:

- Los resultados de consulta solo garantizan estos tipos de valor Lua:
  - `boolean`
  - `integer`
  - `number`
  - `string`
  - `nil` (para SQL `NULL`)
- Las columnas de texto se devuelven como cadenas Lua.
- Las columnas binarias/blob también se devuelven como cadenas Lua.
- Si un resultado de consulta contiene nombres de columna duplicados, se
  produce un error. Usa alias SQL como `AS` para desambiguarlos.

### query

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:query`.

`db:query(sql, params?)` ejecuta una consulta y devuelve una tabla con:

- `rows` (table): Un arreglo de tablas de fila.
- `columns` (table): Un arreglo de nombres de columna.
- `row_count` (integer): El número de filas devueltas.

Ejemplo:

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

`db:query_one(sql, params?)` devuelve la primera fila como tabla, o `nil` si la
consulta no devuelve filas.

Ejemplo:

```lua
local row = db:query_one("select id, name from users where name = ?", {"alice"})
if row then
  print(row.id, row.name)
end
```

### scalar

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:scalar`.

`db:scalar(sql, params?)` devuelve la primera columna de la primera fila, o
`nil` si la consulta no devuelve filas.

Ejemplo:

```lua
local count = db:scalar("select count(*) from users")
print(count)
```

### execute

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:execute`.

`db:execute(sql, params?)` ejecuta una sentencia y devuelve una tabla con:

- `rows_affected` (integer): El número de filas afectadas.

Ejemplo:

```lua
local res = db:execute("update users set name = ? where id = ?", {"alice-2", 1})
print(res.rows_affected)
```

### transaction

> `v0.1.0` - Introduced.

Canonical API name: `ptool.db.Connection:transaction`.

`db:transaction(fn)` ejecuta `fn(tx)` dentro de una transacción de base de
datos.

Comportamiento:

- Si `fn(tx)` devuelve normalmente, la transacción se confirma.
- Si `fn(tx)` produce un error, la transacción se revierte y el error se vuelve
  a lanzar.
- No se admiten transacciones anidadas.
- Mientras la devolución de llamada está activa, no debe usarse el objeto de
  conexión exterior; usa en su lugar el objeto `tx` proporcionado.

El objeto `tx` admite los mismos métodos de consulta que `Connection`:

- `tx:query(sql, params?)`
- `tx:query_one(sql, params?)`
- `tx:scalar(sql, params?)`
- `tx:execute(sql, params?)`

Ejemplo:

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

`db:close()` cierra la conexión.

Comportamiento:

- Después de cerrar, la conexión ya no puede usarse.
- Cerrar durante una devolución de llamada de transacción activa produce un
  error.

Ejemplo:

```lua
local db = ptool.db.connect("sqlite:test.db")
db:close()
```
