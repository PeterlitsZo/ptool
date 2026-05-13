# HTTP API

HTTP クライアントヘルパーは `ptool.http` と `p.http` にあります。

## ptool.http.request

> `v0.1.0` - Introduced.

`ptool.http.request(options)` は HTTP リクエストを送り、`Response` オブジェクトを返します。

`options` フィールド:

- `url` (string, 必須): リクエスト URL。
- `method` (string, 任意): HTTP メソッド。デフォルトは `"GET"`。
- `headers` (table, 任意): リクエストヘッダー。キーと値はいずれも文字列です。
- `query` (table, 任意): リクエスト URL に追加するクエリパラメータ。キーは 文字列でなければならず、値には文字列、数値、真偽値を指定できます。
- `body` (string, 任意): リクエスト本文。
- `json` (Lua 値, 任意): JSON にエンコードされてリクエスト本文として使われ ます。`content-type` を明示していない場合は `application/json` が設定されます。
- `form` (table, 任意): `application/x-www-form-urlencoded` として エンコードされ、リクエスト本文として使われます。キーは文字列でなければ ならず、値には文字列、数値、真偽値を指定できます。
- `timeout_ms` (integer, 任意): ミリ秒単位のタイムアウト。デフォルトは `30000`。
- `connect_timeout_ms` (integer, 任意): 接続タイムアウト（ミリ秒）。
- `follow_redirects` (boolean, 任意): リダイレクトを追跡するかどうか。
- `max_redirects` (integer, 任意): 追跡する最大リダイレクト数。
- `user_agent` (string, 任意): `user-agent` リクエストヘッダーを設定します。
- `basic_auth` (table, 任意): 文字列フィールド `username` と `password` を 持つ HTTP Basic 認証情報です。
- `bearer_token` (string, 任意): `authorization` ヘッダーに使う Bearer トークンです。
- `fail_on_http_error` (boolean, 任意): `true` の場合、4xx と 5xx の HTTP レスポンスでエラーを送出します。デフォルトは `false` です。

注意:

- `body`、`json`、`form` は相互排他的です。
- `basic_auth` と `bearer_token` は相互排他的です。

例:

```lua
local resp = ptool.http.request({
  url = "https://httpbin.org/post",
  method = "POST",
  query = {
    page = 1,
    draft = false,
  },
  json = {
    name = "alice",
    tags = {"admin", "beta"},
  },
  user_agent = "ptool-script/1.0",
  fail_on_http_error = true,
})

print(resp.status, resp.ok)
print(resp:header("content-type"))
local data = resp:json()
print(data.json.name)
```

## Response

> `v0.1.0` - Introduced.

`Response` は `ptool.http.request(...)` が返す HTTP レスポンスを表します。

フィールド:

- `status` (integer): HTTP ステータスコード。
- `ok` (boolean): ステータスコードが 2xx 範囲かどうか。
- `url` (string): リダイレクト後の最終 URL。
- `headers` (table): レスポンスヘッダーの簡易ビュー (`table<string, string>`)。重複ヘッダーは `, ` で結合されます。

メソッド:

- `resp:text()`: レスポンス本文をテキストとして読み取って返します。
- `resp:json()`: レスポンス本文を読み取り、JSON として解析し、Lua 値を 返します。
- `resp:bytes()`: 生バイト列を Lua 文字列として読み取って返します。
- `resp:header(name)`: 最初に一致したレスポンスヘッダー値を返します。 見つからない場合は `nil` を返します。
- `resp:header_values(name)`: 一致したレスポンスヘッダー値をすべて配列で 返します。
- `resp:raise_for_status()`: 4xx または 5xx の HTTP レスポンスでエラーを 送出します。

### text

Canonical API name: `ptool.http.Response:text`.

`resp:text()` はレスポンス本文をテキストとして読み取り、返します。

- 戻り値: `string`。

### json

Canonical API name: `ptool.http.Response:json`.

`resp:json()` はレスポンス本文を読み取り、JSON として解析し、Lua 値を 返します。

### bytes

Canonical API name: `ptool.http.Response:bytes`.

`resp:bytes()` はレスポンス本文を生バイト列として読み取り、返します。

- 戻り値: `string`。

### header

Canonical API name: `ptool.http.Response:header`.

`resp:header(name)` は `name` に一致する最初のレスポンスヘッダー値を返し ます。

- `name` (string, 必須): 取得したいヘッダー名。
- 戻り値: `string | nil`。

### header_values

Canonical API name: `ptool.http.Response:header_values`.

`resp:header_values(name)` は `name` に一致するレスポンスヘッダー値をすべて 返します。

- `name` (string, 必須): 取得したいヘッダー名。
- 戻り値: `string[]`。

### raise_for_status

Canonical API name: `ptool.http.Response:raise_for_status`.

`resp:raise_for_status()` はレスポンスステータスコードが 4xx または 5xx の 場合にエラーを送出します。

注意:

- デフォルトでは 2xx 以外の HTTP ステータスはエラーになりません。 呼び出し側は `resp.ok` を確認するか、 `fail_on_http_error = true` を設定するか、 `resp:raise_for_status()` を呼び出してください。
- レスポンス本文は最初の読み取り後にキャッシュされるため、`text`、`json`、 `bytes` は複数回呼び出せます。
