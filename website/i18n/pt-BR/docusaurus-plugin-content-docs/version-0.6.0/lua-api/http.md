# API HTTP

As utilidades de cliente HTTP estão disponíveis em `ptool.http` e `p.http`.

## ptool.http.request

> `v0.1.0` - Introduced.

`ptool.http.request(options)` envia uma requisição HTTP e retorna um objeto `Response`.

Campos de `options`:

- `url` (string, obrigatório): A URL da requisição.
- `method` (string, opcional): O método HTTP. O padrão é `"GET"`.
- `headers` (table, opcional): Cabeçalhos da requisição, em que chaves e valores são strings.
- `query` (table, opcional): Parâmetros de consulta adicionados à URL da requisição. As chaves devem ser strings. Os valores podem ser strings, números ou booleanos.
- `body` (string, opcional): O corpo da requisição.
- `json` (valor Lua, opcional): Um valor Lua codificado como JSON e usado como corpo da requisição. Quando `content-type` não está definido, o padrão passa a ser `application/json`.
- `form` (table, opcional): Campos codificados como `application/x-www-form-urlencoded` e usados como corpo da requisição. As chaves devem ser strings. Os valores podem ser strings, números ou booleanos.
- `timeout_ms` (integer, opcional): Tempo limite em milissegundos. O padrão é `30000`.
- `connect_timeout_ms` (integer, opcional): Tempo limite de conexão em milissegundos.
- `follow_redirects` (boolean, opcional): Se os redirecionamentos devem ser seguidos.
- `max_redirects` (integer, opcional): Número máximo de redirecionamentos a seguir.
- `user_agent` (string, opcional): Define o cabeçalho `user-agent`.
- `basic_auth` (table, opcional): Credenciais HTTP Basic com os campos string `username` e `password`.
- `bearer_token` (string, opcional): Token Bearer usado no cabeçalho `authorization`.
- `fail_on_http_error` (boolean, opcional): Quando `true`, respostas HTTP 4xx e 5xx geram erro. O padrão é `false`.

Notas:

- `body`, `json` e `form` são mutuamente exclusivos.
- `basic_auth` e `bearer_token` são mutuamente exclusivos.

Exemplo:

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

`Response` representa uma resposta HTTP retornada por `ptool.http.request(...)`.

Campos:

- `status` (integer): O código de status HTTP.
- `ok` (boolean): Se o código de status está na faixa 2xx.
- `url` (string): A URL final após redirecionamentos.
- `headers` (table): Visão simplificada dos cabeçalhos da resposta (`table<string, string>`). Cabeçalhos repetidos são unidos com `, `.

Métodos:

- `resp:text()`: Lê e retorna o corpo da resposta como texto.
- `resp:json()`: Lê o corpo da resposta, faz parse como JSON e retorna um valor Lua.
- `resp:bytes()`: Lê e retorna os bytes brutos como uma string Lua.
- `resp:header(name)`: Retorna o primeiro valor de cabeçalho correspondente, ou `nil`.
- `resp:header_values(name)`: Retorna todos os valores de cabeçalho correspondentes como um array.
- `resp:raise_for_status()`: Gera erro para respostas HTTP 4xx e 5xx.

### text

Canonical API name: `ptool.http.Response:text`.

`resp:text()` lê e retorna o corpo da resposta como texto.

- Retorna: `string`.

### json

Canonical API name: `ptool.http.Response:json`.

`resp:json()` lê o corpo da resposta, faz parse como JSON e retorna um valor Lua.

### bytes

Canonical API name: `ptool.http.Response:bytes`.

`resp:bytes()` lê e retorna o corpo da resposta como bytes brutos.

- Retorna: `string`.

### header

Canonical API name: `ptool.http.Response:header`.

`resp:header(name)` retorna o primeiro valor de cabeçalho de resposta que corresponde a `name`.

- `name` (string, obrigatório): O nome do cabeçalho a consultar.
- Retorna: `string | nil`.

### header_values

Canonical API name: `ptool.http.Response:header_values`.

`resp:header_values(name)` retorna todos os valores de cabeçalho de resposta que correspondem a `name`.

- `name` (string, obrigatório): O nome do cabeçalho a consultar.
- Retorna: `string[]`.

### raise_for_status

Canonical API name: `ptool.http.Response:raise_for_status`.

`resp:raise_for_status()` gera erro quando o código de status da resposta está na faixa 4xx ou 5xx.

Notas:

- Por padrão, status HTTP fora de 2xx não geram erro. Quem chama pode verificar `resp.ok`, definir `fail_on_http_error = true` ou chamar `resp:raise_for_status()`.
- O corpo da resposta é armazenado em cache após a primeira leitura, então `text`, `json` e `bytes` podem ser chamados várias vezes.
