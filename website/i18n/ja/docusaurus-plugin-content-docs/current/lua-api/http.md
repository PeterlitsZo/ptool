# HTTP API

HTTP クライアントヘルパーは `ptool.http` と `p.http` にあります。

## ptool.http.request

> `v0.1.0` - Introduced.

`ptool.http.request(options)` は HTTP リクエストを送り、`Response`
オブジェクトを返します。

`options` フィールド:

- `url` (string, 必須): リクエスト URL。
- `method` (string, 任意): HTTP メソッド。デフォルトは `"GET"`。
- `headers` (table, 任意): リクエストヘッダー。キーと値はいずれも文字列です。
- `body` (string, 任意): リクエスト本文。
- `timeout_ms` (integer, 任意): ミリ秒単位のタイムアウト。デフォルトは
  `30000`。

例:

```lua
local resp = ptool.http.request({
  url = "https://httpbin.org/post",
  method = "POST",
  headers = {
    ["content-type"] = "application/json",
  },
  body = [[{"name":"alice"}]],
})

print(resp.status, resp.ok)
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
- `headers` (table): レスポンスヘッダー (`table<string, string>`)。

メソッド:

- `resp:text()`: レスポンス本文をテキストとして読み取って返します。
- `resp:json()`: レスポンス本文を読み取り、JSON として解析し、Lua 値を
  返します。
- `resp:bytes()`: 生バイト列を Lua 文字列として読み取って返します。

### text

Canonical API name: `ptool.http.Response:text`.

`resp:text()` はレスポンス本文をテキストとして読み取り、返します。

- 戻り値: `string`。

### json

Canonical API name: `ptool.http.Response:json`.

`resp:json()` はレスポンス本文を読み取り、JSON として解析し、Lua 値を
返します。

### bytes

Canonical API name: `ptool.http.Response:bytes`.

`resp:bytes()` はレスポンス本文を生バイト列として読み取り、返します。

- 戻り値: `string`。

注意:

- 2xx 以外の HTTP ステータスはエラーになりません。呼び出し側が `resp.ok`
  を自分で確認する必要があります。
- 本文は一度しか消費できません。`text`, `json`, `bytes` のいずれかを
  複数回呼ぶとエラーになります。
