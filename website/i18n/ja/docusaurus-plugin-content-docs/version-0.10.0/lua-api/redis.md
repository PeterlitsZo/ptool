# Redis API

Redis 接続ヘルパーは `ptool.redis` と `p.redis` にあります。

## ptool.redis.connect

> `v0.9.0` - Introduced.

`ptool.redis.connect(url_or_options)` は Redis 接続を開き、`Connection` オブジェクトを返します。

引数:

- `url_or_options` (string|table, 必須):
  - 文字列が渡された場合は、Redis URL として扱われます。
  - テーブルが渡された場合、現在サポートされるのは次のとおりです:
    - `url` (string, 必須): Redis URL。

一般的な URL の例:

```lua
local redis = ptool.redis.connect("redis://127.0.0.1/")
local db1 = ptool.redis.connect("redis://127.0.0.1/1")
local auth = ptool.redis.connect({
  url = "redis://:secret@cache.internal:6379/0",
})
```

補足:

- この接続は `conn:call(...)` による直接的なコマンド実行を目的としています。
- 現在この API には、専用の pub/sub、pipeline、transaction ヘルパーはありません。

## Connection

> `v0.9.0` - Introduced.

`Connection` は `ptool.redis.connect()` が返すオープンな Redis 接続を表します。

これは Lua userdata として実装されています。

メソッド:

- `conn:call(command, ...)` -> `any`
- `conn:close()` -> `nil`

コマンド引数のルール:

- `command` は空でない文字列である必要があります。
- 残りの引数はフラットな引数リストとして Redis に渡されます。
- サポートされる引数値の型:
  - `boolean`
  - `integer`
  - `number`
  - `string`
- `nil`、table、function、thread、userdata は Redis コマンド引数としてサポートされません。
- Lua 文字列は生バイト列として渡されるため、バイナリセーフな Redis 値を扱えます。

レスポンス変換ルール:

- null レスポンスは `nil` になります。
- 整数レスポンスは Lua 整数になります。
- bulk string、simple string、verbatim string は Lua 文字列になります。
- array と set のレスポンスは密な Lua 配列テーブルになります。
- map レスポンスは、キーを Lua の文字列、整数、数値、真偽値として表現できる場合に Lua テーブルになります。
- double レスポンスは Lua 数値になります。
- boolean レスポンスは Lua 真偽値になります。
- big number レスポンスは Lua 文字列になります。
- push レスポンスは `{ kind = "...", data = {...} }` の形をしたテーブルになります。

### call

> `v0.9.0` - Introduced.

正規 API 名: `ptool.redis.Connection:call`。

`conn:call(command, ...)` は Redis コマンドを送信し、変換後のレスポンスを返します。

例:

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

正規 API 名: `ptool.redis.Connection:close`。

`conn:close()` は接続を閉じます。

挙動:

- 閉じたあとは、その接続を再利用できません。

