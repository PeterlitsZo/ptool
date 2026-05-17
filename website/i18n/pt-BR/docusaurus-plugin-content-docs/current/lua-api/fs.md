# API de sistema de arquivos

As utilidades de sistema de arquivos estão disponíveis em `ptool.fs` e `p.fs`.

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` lê um arquivo como bytes brutos e retorna uma string Lua.

- `path` (string, obrigatório): O caminho do arquivo.
- Retorna: `string`.

Notas:

- A string Lua retornada contém exatamente os bytes armazenados em disco.
- Arquivos de texto continuam funcionando como antes, e agora arquivos binários também são suportados.

Exemplo:

```lua
local content = ptool.fs.read("README.md")
print(content)

local png = ptool.fs.read("logo.png")
print(#png)
```

## ptool.fs.write

> `v0.1.0` - Introduced.

`ptool.fs.write(path, content)` grava uma string Lua em um arquivo como bytes brutos, sobrescrevendo o conteúdo existente.

- `path` (string, obrigatório): O caminho do arquivo.
- `content` (string, obrigatório): O conteúdo a gravar.

Notas:

- `content` é gravado byte por byte.
- Bytes NUL embutidos e bytes não UTF-8 são preservados.

Exemplo:

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
ptool.fs.write("tmp/blob.bin", "\x00\xffABC")
```

## ptool.fs.append

> `v0.8.0` - Introduzido.

`ptool.fs.append(path, content)` acrescenta uma string Lua ao final de um arquivo como bytes brutos. Se o arquivo não existir, ele será criado.

- `path` (string, obrigatório): O caminho do arquivo.
- `content` (string, obrigatório): O conteúdo a acrescentar.

Notas:

- `content` é gravado byte por byte no final do arquivo.
- Bytes NUL embutidos e bytes não UTF-8 são preservados.

Exemplo:

```lua
ptool.fs.append("tmp/log.txt", "first line\n")
ptool.fs.append("tmp/log.txt", "second line\n")
```

## ptool.fs.open

> `v0.8.0` - Introduzido.

`ptool.fs.open(path[, mode])` abre um arquivo local e retorna um objeto `File`.

Argumentos:

- `path` (string, obrigatório): O caminho do arquivo.
- `mode` (string, opcional): O modo do arquivo. O padrão é `"r"`.

Modos suportados:

- `"r"`: Abre para leitura.
- `"w"`: Abre para escrita, truncando o conteúdo existente e criando o arquivo quando necessário.
- `"a"`: Abre para anexar, criando o arquivo quando necessário.
- `"r+"`: Abre para leitura e escrita sem truncar.
- `"w+"`: Abre para leitura e escrita, truncando o conteúdo existente e criando o arquivo quando necessário.
- `"a+"`: Abre para leitura e anexação, criando o arquivo quando necessário.

Notas:

- Os modos podem incluir `b`, como `"rb"` ou `"w+b"`.
- As escritas com `a` e `a+` sempre vão para o fim do arquivo.

Exemplo:

```lua
local file = ptool.fs.open("tmp/log.txt", "a+")
file:write("hello\n")
file:flush()
file:close()
```

## File

> `v0.8.0` - Introduzido.

`File` representa um identificador de arquivo local aberto retornado por `ptool.fs.open()`.

Ele é implementado como um userdata do Lua.

Métodos:

- `file:read([n])` -> `string`
- `file:write(content)` -> `nil`
- `file:flush()` -> `nil`
- `file:seek([whence[, offset]])` -> `integer`
- `file:close()` -> `nil`

### read

> `v0.8.0` - Introduzido.

Nome canônico da API: `ptool.fs.File:read`.

`file:read([n])` lê bytes a partir da posição atual do arquivo e os retorna como uma string Lua.

- `n` (integer, opcional): O número máximo de bytes a ler. Se omitido, lê da posição atual até EOF.
- Retorna: `string`.

Comportamento:

- Retorna uma string vazia em EOF.
- Lê bytes brutos, então os dados binários são preservados exatamente.

Exemplo:

```lua
local file = ptool.fs.open("README.md")
local prefix = file:read(16)
local rest = file:read()
file:close()
```

### write

> `v0.8.0` - Introduzido.

Nome canônico da API: `ptool.fs.File:write`.

`file:write(content)` grava uma string Lua na posição atual do arquivo.

- `content` (string, obrigatório): Os bytes a gravar.

Comportamento:

- Grava bytes brutos exatamente como foram fornecidos.
- Em identificadores no modo append, as escritas são anexadas ao fim do arquivo.

### flush

> `v0.8.0` - Introduzido.

Nome canônico da API: `ptool.fs.File:flush`.

`file:flush()` descarrega para o SO as escritas em arquivo que estão em buffer.

### seek

> `v0.8.0` - Introduzido.

Nome canônico da API: `ptool.fs.File:seek`.

`file:seek([whence[, offset]])` move a posição atual do arquivo.

- `whence` (string, opcional): Um de `"set"`, `"cur"` ou `"end"`. O padrão é `"cur"`.
- `offset` (integer, opcional): O deslocamento em bytes relativo a `whence`. O padrão é `0`.
- Retorna: `integer`.

Comportamento:

- Retorna a nova posição absoluta no arquivo.
- `"set"` exige um `offset` não negativo.

Exemplo:

```lua
local file = ptool.fs.open("tmp/data.bin", "w+")
file:write("abcdef")
file:seek("set", 2)
print(file:read(2)) -- cd
file:close()
```

### close

> `v0.8.0` - Introduzido.

Nome canônico da API: `ptool.fs.File:close`.

`file:close()` fecha o identificador do arquivo.

Comportamento:

- Depois de fechado, o identificador não pode mais ser usado.

## ptool.fs.mkdir

> `v0.1.0` - Introduced.

`ptool.fs.mkdir(path)` cria um diretório. Se diretórios pais não existirem, eles são criados recursivamente.

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

## ptool.fs.is_file

> `v0.6.0` - Introduzido.

`ptool.fs.is_file(path)` verifica se um caminho existe e é um arquivo regular.

- `path` (string, obrigatório): O caminho a verificar.
- Retorna: `boolean`.

Exemplo:

```lua
if ptool.fs.is_file("tmp/hello.txt") then
  print("file")
end
```

## ptool.fs.is_dir

> `v0.6.0` - Introduzido.

`ptool.fs.is_dir(path)` verifica se um caminho existe e é um diretório.

- `path` (string, obrigatório): O caminho a verificar.
- Retorna: `boolean`.

Exemplo:

```lua
if ptool.fs.is_dir("tmp") then
  print("dir")
end
```

## ptool.fs.remove

> `v0.6.0` - Introduzido.

`ptool.fs.remove(path[, options])` remove um arquivo, link simbólico ou diretório.

- `path` (string, obrigatório): O caminho a remover.
- `options` (table, opcional): Opções de remoção. Campos suportados:
  - `recursive` (boolean, opcional): Se diretórios devem ser removidos recursivamente. O padrão é `false`.
  - `missing_ok` (boolean, opcional): Se caminhos ausentes devem ser ignorados. O padrão é `false`.

Comportamento:

- Arquivos e links simbólicos podem ser removidos sem `recursive`.
- Diretórios exigem `recursive = true` quando não estão vazios.
- Nomes de opção desconhecidos ou tipos de valor inválidos geram erro.

Exemplo:

```lua
ptool.fs.remove("tmp/hello.txt")
ptool.fs.remove("tmp/cache", { recursive = true })
ptool.fs.remove("tmp/missing.txt", { missing_ok = true })
```

## ptool.fs.glob

> `v0.2.0` - Introduced. `v0.5.0` - Added the `working_dir` option.

`ptool.fs.glob(pattern[, options])` corresponde caminhos do sistema de arquivos usando sintaxe glob no estilo Unix e retorna um array de strings com os caminhos correspondentes, ordenados lexicograficamente.

- `pattern` (string, obrigatório): Um padrão glob. Padrões relativos são resolvidos a partir do diretório de runtime atual do `ptool`, portanto seguem `ptool.cd(...)`.
- `options` (table, opcional): Opções de glob. Campos suportados:
  - `working_dir` (string, opcional): Sobrescreve o diretório base usado para resolver padrões relativos. Valores relativos de `working_dir` são resolvidos a partir do diretório de runtime atual do `ptool`.
- Retorna: `string[]`.
- Arquivos e diretórios ocultos só correspondem quando o componente de padrão correspondente começa explicitamente com `.`.

Exemplo:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
local lua_scripts = ptool.fs.glob("**/*.lua", {
  working_dir = "../scripts",
})
```
