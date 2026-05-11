# API de shell

As utilidades de parsing de shell estão disponíveis em `ptool.sh` e `p.sh`.

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split(command)` faz o parse de uma string de comando usando regras em
estilo shell e retorna um array de argumentos.

- `command` (string, obrigatório): A string de comando a dividir.
- Retorna: `string[]`.

Exemplo:

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

O `args` acima equivale a:

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```
