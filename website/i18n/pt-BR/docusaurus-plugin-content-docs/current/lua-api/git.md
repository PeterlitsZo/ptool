# API Git

Os auxiliares do repositório Git estão disponíveis em `ptool.git` e `p.git`.

Este módulo é baseado em `git2` / `libgit2`, não na invocação da ferramenta de linha de comando `git`.

## ptool.git.open

> `v0.6.0` - Introduzido.

`ptool.git.open(path?)` abre um repositório diretamente e retorna um objeto `Repo`.

Argumentos:

- `path` (string, opcional): Caminho do repositório. Se omitido, o diretório de tempo de execução atual do `ptool` será usado.

Comportamento:

- Os caminhos relativos são resolvidos a partir do diretório de tempo de execução `ptool` atual, portanto, seguem `ptool.cd(...)`.
- Isso não pesquisa diretórios pais. Use `ptool.git.discover(...)` quando desejar um comportamento de descoberta de repositório.

Exemplo:

```lua
local repo = ptool.git.open(".")
print(repo:path())
```

## ptool.git.discover

> `v0.6.0` - Introduzido.

`ptool.git.discover(path?)` encontra um repositório começando em `path` e subindo nos diretórios pais e, em seguida, retorna um objeto `Repo`.

Argumentos:

- `path` (string, opcional): Caminho inicial. Se omitido, o diretório de tempo de execução atual do `ptool` será usado.

Comportamento:

- Caminhos relativos são resolvidos a partir do diretório de runtime atual do `ptool`.
- Isto é útil quando um script pode ser executado a partir de um subdiretório dentro de uma árvore de trabalho.

Exemplo:

```lua
local repo = ptool.git.discover("src")
print(repo:root())
```

## ptool.git.clone

> `v0.6.0` - Introduzido.

`ptool.git.clone(url, path[, options])` clona um repositório e retorna um objeto `Repo` para o repositório clonado.

Argumentos:

- `url` (string, obrigatório): URL do repositório remoto.
- `path` (string, obrigatório): Caminho de destino.
- `options` (tabela, opcional): Opções de clonagem. Campos suportados:
  - `branch` (string, opcional): Nome da ramificação a ser verificada após a clonagem.
  - `bare` (booleano, opcional): Se deseja criar um repositório bare. O padrão é `false`.
  - `auth` (tabela, opcional): Configurações de autenticação remota.

Campos de `auth`:

- `kind` (string, obrigatório): Modo de autenticação. Valores suportados:
  - `"default"`: Use credenciais padrão da libgit2.
  - `"ssh_agent"`: Autentique através do agente SSH local.
  - `"userpass"`: Use um nome de usuário e senha em texto simples.
- `username` (string, opcional): Nome de usuário para `"ssh_agent"`.
- `username` (string, obrigatório): Nome de usuário para `"userpass"`.
- `password` (string, obrigatório): Senha para `"userpass"`.

Comportamento:

- Os caminhos de destino relativos são resolvidos a partir do diretório de tempo de execução atual do `ptool`.
- As opções de autenticação também são usadas por `repo:fetch(...)` e `repo:push(...)`.

Exemplo:

```lua
local repo = ptool.git.clone(
  "git@github.com:example/project.git",
  "tmp/project",
  {
    branch = "main",
    auth = {
      kind = "ssh_agent",
    },
  }
)

print(repo:root())
```

## Repo

> `v0.6.0` - Introduzido.

`Repo` representa um identificador de repositório Git aberto retornado por `ptool.git.open()`, `ptool.git.discover()` ou `ptool.git.clone()`.

Ele é implementado como um userdata de Lua.

Métodos:

- `repo:path()` -> `string`
- `repo:root()` -> `string|nil`
- `repo:is_bare()` -> `boolean`
- `repo:head()` -> `table`
- `repo:current_branch()` -> `string|nil`
- `repo:status([options])` -> `table`
- `repo:is_clean([options])` -> `boolean`
- `repo:add(paths[, options])` -> `nil`
- `repo:commit(message[, options])` -> `string`
- `repo:checkout(rev[, options])` -> `nil`
- `repo:switch(branch[, options])` -> `nil`
- `repo:fetch([remote[, options]])` -> `table`
- `repo:push([remote[, refspecs[, options]]])` -> `nil`

### path

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:path`.

`repo:path()` retorna o caminho do diretório git do repositório.

- Retorna: `string`.

Notas:

- Para um repositório não bare, normalmente este é o diretório `.git`.
- Para um repositório bare, este é o próprio diretório do repositório.

### root

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:root`.

`repo:root()` retorna o diretório raiz da árvore de trabalho.

- Retorna: `string|nil`.

Notas:

- Isso retorna `nil` para repositórios bare.

### is_bare

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:is_bare`.

`repo:is_bare()` informa se o repositório é bare.

- Retorna: `boolean`.

### head

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:head`.

`repo:head()` retorna informações HEAD como uma tabela com:

- `oid` (string|nil): O OID do commit atual, se disponível.
- `shorthand` (string|nil): Um nome curto para HEAD, como o nome de uma branch.
- `detached` (booleano): Se HEAD está desanexado.
- `unborn` (booleano): Se o repositório ainda não possui um commit inicial.

Exemplo:

```lua
local head = repo:head()
print(head.oid)
print(head.detached)
```

### current_branch

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:current_branch`.

`repo:current_branch()` retorna o nome da branch local atual.

- Retorna: `string|nil`.

Notas:

- Isso retorna `nil` quando HEAD é desconectado.
- Isso também retorna `nil` para uma branch unborn antes do primeiro commit.

### status

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:status`.

`repo:status([options])` resume o status do repositório e retorna uma tabela com:

- `root` (string|nil): O diretório raiz da árvore de trabalho.
- `branch` (string|nil): O nome da branch local atual.
- `head` (tabela): A mesma informação HEAD retornada por `repo:head()`.
- `upstream` (string|nil): O nome da branch upstream, quando configurada.
- `ahead` (inteiro): Número de commits à frente do upstream.
- `behind` (inteiro): Número de commits atrás do upstream.
- `clean` (booleano): Se o repositório não possui entradas de status visíveis.
- `entries` (tabela): Uma matriz de tabelas de entrada de status.

`entries[i]` contém:

- `path` (string): Caminho relativo ao repositório.
- `index_status` (string|nil): Status do lado do índice. Os valores suportados atualmente incluem `"new"`, `"modified"`, `"deleted"`, `"renamed"` e `"typechange"`.
- `worktree_status` (string|nil): Status do lado da árvore de trabalho. Os valores suportados atualmente incluem `"new"`, `"modified"`, `"deleted"`, `"renamed"`, `"typechange"` e `"ignored"`.
- `conflicted` (booleano): Se o caminho está em conflito.
- `ignored` (booleano): Se o caminho é ignorado.

Campos de `options`:

- `include_untracked` (booleano, opcional): Se deseja incluir arquivos não rastreados. O padrão é `true`.
- `include_ignored` (booleano, opcional): Se deseja incluir arquivos ignorados. O padrão é `false`.
- `recurse_untracked_dirs` (booleano, opcional): se deve recorrer a diretórios não rastreados. O padrão é `true`.

Exemplo:

```lua
local st = repo:status()
print(st.clean)
print(st.branch)

for _, entry in ipairs(st.entries) do
  print(entry.path, entry.index_status, entry.worktree_status)
end
```

### is_clean

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:is_clean`.

`repo:is_clean([options])` retorna se o repositório está limpo.

- `options` (tabela, opcional): Mesmas opções aceitas por `repo:status(...)`.
- Retorna: `boolean`.

### add

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:add`.

`repo:add(paths[, options])` adiciona um ou mais caminhos ao índice.

Argumentos:

- `paths` (string|string[], obrigatório): Um caminho ou uma matriz de caminhos.
- `options` (tabela, opcional): Adicionar opções. Campos suportados:
  - `update` (booleano, opcional): Atualiza apenas caminhos já conhecidos no índice. O padrão é `false`.

Comportamento:

- Os caminhos são interpretados em relação à árvore de trabalho do repositório.

Exemplo:

```lua
repo:add("README.md")
repo:add({"src", "Cargo.toml"})
```

### commit

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:commit`.

`repo:commit(message[, options])` cria um commit a partir do índice atual e retorna o novo OID do commit.

Argumentos:

- `message` (string, obrigatório): Mensagem do commit.
- `options` (tabela, opcional): Opções de confirmação. Campos suportados:
  - `author` (tabela, opcional): Assinatura do autor.
  - `committer` (tabela, opcional): Assinatura do committer.

Campos de assinatura:

- `name` (string, obrigatório)
- `email` (string, obrigatório)

Comportamento:

- Quando `author` e `committer` são omitidos, `ptool` tenta usar a identidade do repositório Git da configuração.
- Se nenhuma identidade for configurada e nenhuma assinatura explícita for fornecida, um erro será gerado.

Exemplo:

```lua
local oid = repo:commit("Release v0.7.0", {
  author = {
    name = "Release Bot",
    email = "bot@example.com",
  },
})

print(oid)
```

### checkout

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:checkout`.

`repo:checkout(rev[, options])` faz checkout de uma revisão.

Argumentos:

- `rev` (string, obrigatório): expressão de revisão, como nome de ramificação, nome de tag ou OID de commit.
- `options` (tabela, opcional): Opções de checkout. Campos suportados:
  - `force` (booleano, opcional): se deve forçar o checkout. O padrão é `false`.

Comportamento:

- Isso pode desanexar HEAD quando `rev` não resolve para uma referência nomeada.

### switch

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:switch`.

`repo:switch(branch[, options])` alterna HEAD para uma branch local.

Argumentos:

- `branch` (string, obrigatório): Nome da branch local.
- `options` (tabela, opcional): Alternar opções. Campos suportados:
  - `create` (booleano, opcional): Se a ramificação deve ser criada primeiro. O padrão é `false`.
  - `force` (booleano, opcional): Se deve forçar o checkout. O padrão é `false`.
  - `start_point` (string, opcional): Revisão para ramificação a partir de `create = true`. O padrão é `HEAD`.

Exemplo:

```lua
repo:switch("release")
repo:switch("release-next", {
  create = true,
  start_point = "origin/main",
})
```

### fetch

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:fetch`.

`repo:fetch([remote[, options]])` faz fetch de um remoto e retorna estatísticas de transferência.

Argumentos:

- `remote` (string, opcional): Nome remoto. O padrão é `"origin"`.
- `options` (tabela, opcional): Opções de busca. Campos suportados:
  - `refspecs` (string|string[], opcional): Um refspec ou uma matriz de refspecs.
  - `auth` (tabela, opcional): Configurações de autenticação remota. Usa a mesma estrutura do `ptool.git.clone(...)`.

Retorna:

- `received_objects` (inteiro)
- `indexed_objects` (inteiro)
- `local_objects` (inteiro)
- `total_objects` (inteiro)
- `received_bytes` (inteiro)

Exemplo:

```lua
local stats = repo:fetch("origin", {
  auth = {
    kind = "ssh_agent",
  },
})

print(stats.received_objects, stats.received_bytes)
```

### push

> `v0.6.0` - Introduzido.

Nome canônico da API: `ptool.git.Repo:push`.

`repo:push([remote[, refspecs[, options]]])` faz push de refs para um remoto.

Argumentos:

- `remote` (string, opcional): Nome remoto. O padrão é `"origin"`.
- `refspecs` (string|string[], opcional): Um refspec ou uma matriz de refspecs.
- `options` (tabela, opcional): Opções push. Campos suportados:
  - `auth` (tabela, opcional): Configurações de autenticação remota. Usa a mesma estrutura do `ptool.git.clone(...)`.

Comportamento:

- Quando `refspecs` é omitido, `ptool` tenta fazer push da branch local atual para a branch de mesmo nome no remoto.
- Omitir `refspecs` enquanto HEAD está desconectado gera um erro.

Exemplo:

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```
