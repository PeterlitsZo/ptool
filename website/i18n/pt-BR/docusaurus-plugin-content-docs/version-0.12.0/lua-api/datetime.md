# DateTime API

Auxiliares de data e hora estão disponíveis em `ptool.datetime` e `p.datetime`.

`ptool.datetime` trabalha com instantes concretos. Cada valor `DateTime` carrega um fuso horário ou deslocamento numérico.

## ptool.datetime.now

> `v0.6.0` - Introduzido.

`ptool.datetime.now([tz])` retorna a hora atual como `DateTime`.

- `tz` (string, opcional): um fuso horário IANA, como `UTC`, `America/New_York` ou `Asia/Shanghai`. Se omitido, o fuso horário do sistema local será usado.
- Retorna: `DateTime`.

```lua
local local_now = p.datetime.now()
local utc_now = p.datetime.now("UTC")

print(local_now)
print(utc_now:format("%Y-%m-%d %H:%M:%S %Z"))
```

## ptool.datetime.parse

> `v0.6.0` - Introduzido.

`ptool.datetime.parse(input[, options])` analisa uma string de data e hora e retorna um `DateTime`.

- `input` (string, obrigatório): Uma string de data e hora.
- `options.timezone` (string, opcional): um fuso horário IANA usado somente quando a entrada ainda não inclui um fuso horário ou deslocamento.
- Retorna: `DateTime`.

Entradas aceitas:

- Entradas zoneadas como `2024-07-15T16:24:59-04:00`.
- Entradas zoneadas com anotações de fuso horário entre colchetes quando suportadas pelo analisador.
- Entradas ingênuas como `2024-07-15 16:24:59`, mas somente quando `options.timezone` é fornecido.

Comportamento:

- Strings vazias são rejeitadas.
- Se `input` já incluir um fuso horário ou deslocamento, definir `options.timezone` gerará um erro.
- Sem `options.timezone`, as entradas ingênuas são rejeitadas.

```lua
local a = p.datetime.parse("2024-07-15T16:24:59-04:00")
local b = p.datetime.parse("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})

print(a.offset)   -- -04:00
print(b.timezone) -- America/New_York
```

## ptool.datetime.from_unix

> `v0.6.0` - Introduzido.

`ptool.datetime.from_unix(value[, options])` constrói um `DateTime` a partir de um carimbo de data/hora Unix.

- `value` (inteiro, obrigatório): O carimbo de data/hora Unix.
- `options.unit` (sequência, opcional): Um de `s`, `ms` ou `ns`. O padrão é `s`.
- `options.timezone` (string, opcional): um fuso horário IANA. Se omitido, o timestamp será interpretado em `UTC`.
- Retorna: `DateTime`.

```lua
local a = p.datetime.from_unix(1721075099)
local b = p.datetime.from_unix(1721075099000, {
  unit = "ms",
  timezone = "Asia/Tokyo",
})

print(a) -- 2024-07-15T20:24:59+00:00
print(b)
```

## ptool.datetime.compare

> `v0.6.0` - Introduzido.

`ptool.datetime.compare(a, b)` compara dois instantes.

- `a` / `b` (string|DateTime, obrigatório): Uma string de data e hora ou objeto `DateTime`.
- Retorna: `-1 | 0 | 1`.

Os argumentos de string são analisados ​​usando as mesmas regras estritas de `ptool.datetime.parse(input)`, portanto, eles já devem incluir um fuso horário ou deslocamento.

```lua
print(ptool.datetime.compare(
  "2024-07-15T20:24:59+00:00",
  "2024-07-15T16:24:59-04:00"
)) -- 0
```

## ptool.datetime.is_valid

> `v0.6.0` - Introduzido.

`ptool.datetime.is_valid(input[, options])` verifica se uma sequência de data e hora pode ser analisada.

- `input` (string, obrigatório): Uma string de data e hora.
- `options.timezone` (string, opcional): um fuso horário IANA para entrada ingênua.
- Retorna: `boolean`.

```lua
print(ptool.datetime.is_valid("2024-07-15T16:24:59-04:00")) -- true
print(ptool.datetime.is_valid("2024-07-15 16:24:59")) -- false
print(ptool.datetime.is_valid("2024-07-15 16:24:59", {
  timezone = "America/New_York",
})) -- true
```

## DateTime

> `v0.6.0` - Introduzido.

`DateTime` representa um instante concreto retornado por `ptool.datetime.now(...)`, `parse(...)` ou `from_unix(...)`.

Ele é implementado como um userdata de Lua.

Campos e métodos:

- Campos:
  - `year` (inteiro)
  - `month` (inteiro)
  - `day` (inteiro)
  - `hour` (inteiro)
  - `minute` (inteiro)
  - `second` (inteiro)
  - `nanosecond` (inteiro)
  - `offset` (string)
  - `timezone` (string)
- Métodos:
  - `dt:format(fmt)` -> `string`
  - `dt:to_string()` -> `string`
  - `dt:unix([unit])` -> `integer`
  - `dt:in_tz(tz)` -> `DateTime`
  - `dt:compare(other)` -> `-1|0|1`
- Metamétodos:
  - `tostring(dt)` está disponível.
  - Comparações `==`, `<` e `<=` são suportadas.

### format

Nome canônico da API: `ptool.datetime.DateTime:format`.

`dt:format(fmt)` formata a data e hora usando diretivas no estilo `strftime`.

- `fmt` (string, obrigatório): Uma string de formato como `%Y-%m-%d %H:%M:%S %Z`.
- Retorna: `string`.

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:format("%Y-%m-%d %H:%M:%S %:z"))
```

### to_string

Nome canônico da API: `ptool.datetime.DateTime:to_string`.

`dt:to_string()` retorna o formato de string canônica com um deslocamento numérico.

- Retorna: `string`.

```lua
local dt = p.datetime.parse("2024-07-15T16:24:59-04:00")
print(dt:to_string()) -- 2024-07-15T16:24:59-04:00
```

### unix

Nome canônico da API: `ptool.datetime.DateTime:unix`.

`dt:unix([unit])` retorna o carimbo de data/hora Unix do instante.

- `unit` (string, opcional): Um de `s`, `ms` ou `ns`. O padrão é `s`.
- Retorna: `integer`.

Notas:

- `ns` pode gerar um erro se o resultado não couber no intervalo de inteiros de Lua.

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
print(dt:unix())       -- seconds
print(dt:unix("ms"))   -- milliseconds
```

### in_tz

Nome canônico da API: `ptool.datetime.DateTime:in_tz`.

`dt:in_tz(tz)` converte o mesmo instante em outro fuso horário.

- `tz` (string, obrigatório): um fuso horário da IANA.
- Retorna: `DateTime`.

```lua
local dt = p.datetime.parse("2024-07-15T20:24:59+00:00")
local tokyo = dt:in_tz("Asia/Tokyo")

print(dt)
print(tokyo)
```

### compare

Nome canônico da API: `ptool.datetime.DateTime:compare`.

`dt:compare(other)` compara o instante atual com `other`.

- `other` (string|DateTime, obrigatório): Uma string de data e hora ou outro objeto `DateTime`.
- Retorna: `-1 | 0 | 1`.

```lua
local a = p.datetime.parse("2024-07-15T20:24:59+00:00")
local b = p.datetime.parse("2024-07-15T21:24:59+00:00")

print(a:compare(b)) -- -1
print(a < b)        -- true
```

## Notas

- `ptool.datetime` não analisa frases de linguagem natural como `"tomorrow 8am"`.
- Os nomes de fuso horário devem ser identificadores da IANA, como `UTC`, `Asia/Tokyo` ou `America/New_York`.
- As comparações operam sobre o instante, não sobre os campos de relógio exibidos.
