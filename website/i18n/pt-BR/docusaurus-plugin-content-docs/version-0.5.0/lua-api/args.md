# API de argumentos

As utilidades de esquema e parse de argumentos de CLI estão disponíveis em
`ptool.args` e `p.args`.

## ptool.args.arg

> `v0.1.0` - Introduced.

`ptool.args.arg(id, kind, options)` cria um builder de argumento para uso em
`ptool.args.parse(...).schema.args`.

- `id` (string, obrigatório): O identificador do argumento. Ele também é a
  chave na tabela retornada.
- `kind` (string, obrigatório): O tipo do argumento. Valores suportados:
  - `"flag"`: Um flag booleano.
  - `"string"`: Uma opção string.
  - `"int"`: Uma opção inteira (`i64`).
  - `"positional"`: Um argumento posicional.
- `options` (table, opcional): Os mesmos campos opcionais suportados por tabelas
  de argumento em `ptool.args.parse`, como `long`, `short`, `help`,
  `required`, `multiple` e `default`.

O builder suporta métodos encadeáveis, todos retornando ele mesmo:

- `arg:long(value)` define o nome da opção longa. Suportado apenas para
  argumentos não `positional`.
- `arg:short(value)` define o nome da opção curta. Suportado apenas para
  argumentos não `positional`.
- `arg:help(value)` define o texto de ajuda.
- `arg:required(value)` define se o argumento é obrigatório. Se `value` for
  omitido, o padrão é `true`.
- `arg:multiple(value)` define se o argumento pode se repetir. Se `value` for
  omitido, o padrão é `true`.
- `arg:default(value)` define o valor padrão. Se `value = nil`, o padrão é
  limpo.

Exemplo:

```lua
local res = ptool.args.parse({
  args = {
    ptool.args.arg("name", "string"):required(),
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("paths", "positional"):multiple(),
  }
})
```

## ptool.args.parse

> `v0.1.0` - Introduced.
>
> `v0.3.0` - Added `subcommands` support.

`ptool.args.parse(schema)` faz o parse dos argumentos do script com `clap` e
retorna uma tabela indexada por `id`.

Os argumentos do script vêm da parte após `--` em `ptool run <lua_file> -- ...`.

Por exemplo:

```lua
ptool.use("v0.1.0")

local res = ptool.args.parse({
    name = "test",
    about = "The test command",
    args = {
        { id = "name", kind = "string" }
    }
})

print("Hello, " .. res.name .. "!")
```

### Estrutura do esquema

- `name` (string, opcional): O nome do comando, usado na saída de ajuda. O
  padrão é o nome do arquivo do script.
- `about` (string, opcional): Descrição de ajuda.
- `args` (table, opcional): Um array de definições de argumento. Cada item
  suporta duas formas:
  - Uma tabela de argumento.
  - Um objeto builder retornado por `ptool.args.arg(...)`.
- `subcommands` (table, opcional): Um mapa de nome de subcomando para esquema
  de subcomando. Cada esquema de subcomando suporta `about`, `args` e
  `subcommands` recursivamente.

Pelo menos um de `args` ou `subcommands` precisa ser fornecido.

Campos da tabela de argumento:

- `id` (string, obrigatório): O identificador do argumento. Ele também é a
  chave na tabela retornada.
- `kind` (string, obrigatório): O tipo do argumento. Valores suportados:
  - `"flag"`: Um flag booleano.
  - `"string"`: Uma opção string.
  - `"int"`: Uma opção inteira (`i64`).
  - `"positional"`: Um argumento posicional.
- `long` (string, opcional): O nome da opção longa, como `"name"` para
  `--name`. Para argumentos não `positional`, o padrão pode ser derivado de
  `id`.
- `short` (string, opcional): O nome da opção curta, um único caractere como
  `"v"` para `-v`.
- `help` (string, opcional): Texto de ajuda do argumento.
- `required` (boolean, opcional): Se o argumento é obrigatório. O padrão é
  `false`.
- `multiple` (boolean, opcional): Se o argumento pode se repetir. O padrão é
  `false`.
- `default` (string/integer, opcional): O valor padrão.

Quando `subcommands` está presente, `args` do comando atual age como opções
compartilhadas para aquela árvore de comandos e é aceito antes ou depois do
subcomando selecionado.

Exemplo com subcomandos:

```lua
local res = ptool.args.parse({
  name = "demo",
  args = {
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("config", "string"),
  },
  subcommands = {
    build = {
      args = {
        ptool.args.arg("release", "flag"),
      },
      subcommands = {
        web = {
          args = {
            ptool.args.arg("out", "string"):required(),
          },
        },
      },
    },
    clean = {
      args = {
        ptool.args.arg("all", "flag"),
      },
    },
  },
})
```

### Restrições

- As seguintes restrições se aplicam tanto a tabelas de argumento quanto à
  sintaxe de builder.
- Argumentos não `positional` podem omitir `long` e `short`. Se `long` for
  omitido, `id` é usado automaticamente.
- Argumentos `positional` não podem definir `long`, `short` ou `default`.
- Quando `positional.multiple = true`, ele precisa ser o último argumento em
  `args`.
- `multiple = true` só é suportado para `string` e `positional`.
- `default` só é suportado para `string` e `int`, e não pode ser usado junto
  com `multiple = true`.
- Quando `subcommands` está presente, argumentos `positional` não são
  permitidos nesse mesmo esquema.
- Quando `subcommands` está presente no nível superior, os ids de argumento
  `command_path` e `args` ficam reservados.
- Ao longo de um mesmo caminho de subcomando selecionado, subcomandos
  ancestrais e descendentes não podem reutilizar o mesmo `id` de argumento,
  porque seus valores são mesclados em uma única tabela `args`.

### Valor de retorno

Uma tabela Lua é retornada, em que as chaves são `id` e os tipos de valor são:

- `flag` -> `boolean`
- `string` -> `string` (ou `string[]` quando `multiple = true`)
- `int` -> `integer`
- `positional` -> `string` (ou `string[]` quando `multiple = true`)

Quando `subcommands` não está presente, o valor de retorno permanece plano como
acima.

Quando `subcommands` está presente, o valor de retorno tem este formato:

- Valores de `args` do nível superior são retornados diretamente na tabela de
  nível superior.
- `command_path` -> `string[]`: O caminho de subcomando correspondente, por
  exemplo `{"build", "web"}`.
- `args` -> `table`: Os valores de argumento mesclados do caminho de
  subcomando correspondente.

Por exemplo:

```lua
{
  verbose = true,
  config = "cfg.toml",
  command_path = { "build", "web" },
  args = {
    release = true,
    out = "dist",
  },
}
```
