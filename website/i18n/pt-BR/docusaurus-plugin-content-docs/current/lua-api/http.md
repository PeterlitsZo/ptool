# API HTTP

As utilidades de cliente HTTP estão disponíveis em `ptool.http` e `p.http`.

## ptool.http.request

> `v0.1.0` - Introduced.

`ptool.http.request(options)` envia uma requisição HTTP e retorna um objeto
`Response`.

Campos de `options`:

- `url` (string, obrigatório): A URL da requisição.
- `method` (string, opcional): O método HTTP. O padrão é `"GET"`.
- `headers` (table, opcional): Cabeçalhos da requisição, em que chaves e
  valores são strings.
- `body` (string, opcional): O corpo da requisição.
- `timeout_ms` (integer, opcional): Tempo limite em milissegundos. O padrão é
  `30000`.

Exemplo:

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

`Response` representa uma resposta HTTP retornada por
`ptool.http.request(...)`.

Campos:

- `status` (integer): O código de status HTTP.
- `ok` (boolean): Se o código de status está na faixa 2xx.
- `url` (string): A URL final após redirecionamentos.
- `headers` (table): Cabeçalhos da resposta (`table<string, string>`).

Métodos:

- `resp:text()`: Lê e retorna o corpo da resposta como texto.
- `resp:json()`: Lê o corpo da resposta, faz parse como JSON e retorna um valor
  Lua.
- `resp:bytes()`: Lê e retorna os bytes brutos como uma string Lua.

### text

Canonical API name: `ptool.http.Response:text`.

`resp:text()` lê e retorna o corpo da resposta como texto.

- Retorna: `string`.

### json

Canonical API name: `ptool.http.Response:json`.

`resp:json()` lê o corpo da resposta, faz parse como JSON e retorna um valor
Lua.

### bytes

Canonical API name: `ptool.http.Response:bytes`.

`resp:bytes()` lê e retorna o corpo da resposta como bytes brutos.

- Retorna: `string`.

Notas:

- Status HTTP fora de 2xx não geram erro. Quem chama deve verificar `resp.ok`
  por conta própria.
- O corpo só pode ser consumido uma vez. Chamar `text`, `json` ou `bytes`
  mais de uma vez gera erro.
