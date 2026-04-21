# API de sistema de arquivos

As utilidades de sistema de arquivos estão disponíveis em `ptool.fs` e `p.fs`.

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` lê um arquivo de texto UTF-8 e retorna uma string.

- `path` (string, obrigatório): O caminho do arquivo.
- Retorna: `string`.

Exemplo:

```lua
local content = ptool.fs.read("README.md")
print(content)
```

## ptool.fs.write

> `v0.1.0` - Introduced.

`ptool.fs.write(path, content)` grava uma string em um arquivo, sobrescrevendo o
conteúdo existente.

- `path` (string, obrigatório): O caminho do arquivo.
- `content` (string, obrigatório): O conteúdo a gravar.

Exemplo:

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
```

## ptool.fs.mkdir

> `v0.1.0` - Introduced.

`ptool.fs.mkdir(path)` cria um diretório. Se diretórios pais não existirem,
eles são criados recursivamente.

- `path` (string, obrigatório): O caminho do diretório.

Exemplo:

```lua
ptool.fs.mkdir("tmp/a/b")
```

## ptool.fs.exists

> `v0.1.0` - Introduced.

`ptool.fs.exists(path)` verifica se um caminho existe.

- `path` (string, obrigatório): Um caminho de arquivo ou diretório.
- Retorna: `boolean`.

Exemplo:

```lua
if ptool.fs.exists("tmp/hello.txt") then
  print("exists")
end
```

## ptool.fs.glob

> `v0.2.0` - Introduced.

`ptool.fs.glob(pattern)` corresponde caminhos do sistema de arquivos usando
sintaxe glob no estilo Unix e retorna um array de strings com os caminhos
correspondentes, ordenados lexicograficamente.

- `pattern` (string, obrigatório): Um padrão glob. Padrões relativos são
  resolvidos a partir do diretório de runtime atual do `ptool`, portanto seguem
  `ptool.cd(...)`.
- Retorna: `string[]`.
- Arquivos e diretórios ocultos só correspondem quando o componente de padrão
  correspondente começa explicitamente com `.`.

Exemplo:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
```
