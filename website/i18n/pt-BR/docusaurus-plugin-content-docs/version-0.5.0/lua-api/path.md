# API de caminhos

As utilidades léxicas de caminho estão disponíveis em `ptool.path` e `p.path`.

## ptool.path.join

> `v0.1.0` - Introduced.

`ptool.path.join(...segments)` junta vários segmentos de caminho e retorna o
caminho normalizado.

- `segments` (string, pelo menos um): Segmentos de caminho.
- Retorna: `string`.

Exemplo:

```lua
print(ptool.path.join("tmp", "a", "..", "b")) -- tmp/b
```

## ptool.path.normalize

> `v0.1.0` - Introduced.

`ptool.path.normalize(path)` faz a normalização léxica de caminhos
(processando `.` e `..`).

- `path` (string, obrigatório): O caminho de entrada.
- Retorna: `string`.

Exemplo:

```lua
print(ptool.path.normalize("./a/../b")) -- b
```

## ptool.path.abspath

> `v0.1.0` - Introduced.

`ptool.path.abspath(path[, base])` calcula um caminho absoluto.

- `path` (string, obrigatório): O caminho de entrada.
- `base` (string, opcional): O diretório base. Se omitido, é usado o diretório
  de trabalho do processo atual.
- Retorna: `string`.
- Aceita apenas 1 ou 2 argumentos string.

Exemplo:

```lua
print(ptool.path.abspath("src"))
print(ptool.path.abspath("lib", "/tmp/demo"))
```

## ptool.path.relpath

> `v0.1.0` - Introduced.

`ptool.path.relpath(path[, base])` calcula um caminho relativo de `base` até
`path`.

- `path` (string, obrigatório): O caminho de destino.
- `base` (string, opcional): O diretório inicial. Se omitido, é usado o
  diretório de trabalho do processo atual.
- Retorna: `string`.
- Aceita apenas 1 ou 2 argumentos string.

Exemplo:

```lua
print(ptool.path.relpath("src/main.rs", "/tmp/project"))
```

## ptool.path.isabs

> `v0.1.0` - Introduced.

`ptool.path.isabs(path)` verifica se um caminho é absoluto.

- `path` (string, obrigatório): O caminho de entrada.
- Retorna: `boolean`.

Exemplo:

```lua
print(ptool.path.isabs("/tmp")) -- true
```

## ptool.path.dirname

> `v0.1.0` - Introduced.

`ptool.path.dirname(path)` retorna a parte de nome do diretório.

- `path` (string, obrigatório): O caminho de entrada.
- Retorna: `string`.

Exemplo:

```lua
print(ptool.path.dirname("a/b/c.txt")) -- a/b
```

## ptool.path.basename

> `v0.1.0` - Introduced.

`ptool.path.basename(path)` retorna o último segmento do caminho
(a parte do nome do arquivo).

- `path` (string, obrigatório): O caminho de entrada.
- Retorna: `string`.

Exemplo:

```lua
print(ptool.path.basename("a/b/c.txt")) -- c.txt
```

## ptool.path.extname

> `v0.1.0` - Introduced.

`ptool.path.extname(path)` retorna a extensão, incluindo o `.`. Se não houver
extensão, retorna uma string vazia.

- `path` (string, obrigatório): O caminho de entrada.
- Retorna: `string`.

Exemplo:

```lua
print(ptool.path.extname("a/b/c.txt")) -- .txt
```

Notas:

- O tratamento de caminhos em `ptool.path` é puramente léxico. Ele não verifica
  se os caminhos existem nem resolve links simbólicos.
- Nenhuma das interfaces aceita argumentos de string vazia. Passar um deles
  gera erro.
