# API de shell

As utilidades de parsing de shell estão disponíveis em `ptool.sh` e `p.sh`.

Esses helpers funcionam no nível de shell words. Eles servem para dividir, citar e juntar strings de argumentos usando regras de shell no estilo POSIX, e não para fazer parse da sintaxe completa de shell, como pipelines, redirecionamentos, substituição de comandos ou expansão de variáveis.

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split(command)` faz o parse de uma string de comando usando regras em estilo shell e retorna um array de argumentos.

- `command` (string, obrigatório): A string de comando a dividir.
- Retorna: `string[]`.

Comportamento:

- Isto faz parse apenas de shell words. Não interpreta operadores de shell nem executa expansões.

Exemplo:

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

O `args` acima equivale a:

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```

## ptool.sh.quote

> `Unreleased` - Introduzido.

`ptool.sh.quote(word)` aplica quoting a uma única shell word para que ela possa ser embutida com segurança em uma string de comando de shell.

- `word` (string, obrigatório): A shell word a ser citada.
- Retorna: `string`.

Comportamento:

- A string retornada é segura para shell e semanticamente equivalente à word de entrada.
- Isto preserva o significado da shell word, não a grafia textual original.

Exemplo:

```lua
local word = ptool.sh.quote("hello world")
print(word) -- 'hello world'
```

## ptool.sh.join

> `Unreleased` - Introduzido.

`ptool.sh.join(words)` junta um array de argumentos em uma string de comando de shell, aplicando quoting às words quando necessário.

- `words` (string[], obrigatório): As shell words a juntar.
- Retorna: `string`.

Comportamento:

- Words consecutivas são unidas com um único espaço.
- A saída é adequada para ser passada a um shell no estilo POSIX.
- Isto busca round-tripping no nível de shell words, então `ptool.sh.split(ptool.sh.join(words))` é equivalente a `words`.
- `ptool.sh.join(ptool.sh.split(command))` pode normalizar quoting e espaçamento em vez de preservar o texto original do comando.

Exemplo:

```lua
local cmd = ptool.sh.join({"git", "commit", "-m", "hello world"})
print(cmd) -- git commit -m 'hello world'
```
