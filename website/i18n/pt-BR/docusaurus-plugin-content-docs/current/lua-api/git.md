# API Git

Os auxiliares do repositório Git estão disponíveis em `ptool.git` e `p.git`.

Este módulo é baseado em `git2` / `libgit2`, não na invocação da ferramenta de linha de comando `git`.

As operações que alteram o repositório aceitam `confirm = true` na tabela de opções para pedir confirmação antes da execução. Operações destrutivas também usam sinalizadores de segurança separados, como `force = true`; a confirmação nunca substitui o sinalizador de segurança exigido.

Nomes de opções desconhecidos são rejeitados em vez de ignorados silenciosamente.

## ptool.git.init

> `Unreleased` - Introduzido.

`ptool.git.init(path?, options?)` inicializa um repositório e retorna um objeto `Repo`.

Opções:

- `bare` (boolean): Cria um repositório bare. O padrão é `false`.
- `initial_head` (string): Nome do branch inicial, por exemplo `"main"`.
- `confirm` (boolean): Pede confirmação antes de criar o repositório.

Caminhos relativos são resolvidos a partir do diretório de runtime atual do `ptool`.

```lua
local repo = p.git.init("tmp/project", {
  initial_head = "main",
})
```

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
  - `depth` (integer, opcional): Profundidade positiva para um clone raso.
  - `checkout` (boolean, opcional): Define se o branch selecionado será extraído. O padrão é `true`.
  - `remote` (string, opcional): Nome atribuído ao remoto clonado. O padrão é `"origin"`.
  - `tags` (string, opcional): `"auto"`, `"all"` ou `"none"`. O padrão é `"auto"`.
  - `confirm` (boolean, opcional): Se deve pedir confirmação antes de clonar. O padrão é `false`.
  - `auth` (tabela, opcional): Configurações de autenticação remota.

Campos de `auth`:

- `kind` (string, obrigatório): Modo de autenticação. Valores suportados:
  - `"default"`: Use credenciais padrão da libgit2.
  - `"ssh_agent"`: Autentique através do agente SSH local.
  - `"ssh_key"`: Autentica com uma chave SSH privada.
  - `"userpass"`: Use um nome de usuário e senha em texto simples.
  - `"credential_helper"`: Consulta o helper de credenciais Git configurado.
- `username` (string): Opcional para `"ssh_agent"` e `"credential_helper"`; obrigatório para `"ssh_key"` e `"userpass"`.
- `private_key` (string, obrigatório para `"ssh_key"`): Caminho da chave privada. Caminhos relativos são resolvidos a partir do diretório de execução atual do `ptool`.
- `public_key` (string, opcional): Caminho da chave pública.
- `passphrase` (string, opcional): Frase secreta da chave SSH privada.
- `password` (string, obrigatório para `"userpass"`): Senha.

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

`Repo` representa um identificador de repositório Git aberto retornado por `ptool.git.init()`, `ptool.git.open()`, `ptool.git.discover()` ou `ptool.git.clone()`.

Ele é implementado como um userdata de Lua.

Os métodos estão agrupados abaixo. Os métodos existentes de repositório, status, staging, commit, checkout, switch, fetch e push permanecem compatíveis. As APIs de fluxo de trabalho descritas nas seções seguintes ainda não foram lançadas.

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
  - `confirm` (boolean, opcional): Se deve pedir confirmação antes de adicionar caminhos ao índice. O padrão é `false`.

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
  - `amend` (boolean, opcional): Substitui o commit HEAD atual. O padrão é `false`.
  - `allow_empty` (boolean, opcional): Permite um commit cuja árvore não foi alterada. O padrão é `true` para manter a compatibilidade.
  - `confirm` (boolean, opcional): Se deve pedir confirmação antes de criar o commit. O padrão é `false`.

Campos de assinatura:

- `name` (string, obrigatório)
- `email` (string, obrigatório)
- `time_seconds` (integer, opcional): Timestamp Unix.
- `offset_minutes` (integer, opcional): Deslocamento de fuso horário em relação ao UTC.

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
  - `confirm` (boolean, opcional): Se deve pedir confirmação antes de fazer checkout da revisão. O padrão é `false`.

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
  - `track` (string, opcional): Referência upstream do novo branch.
  - `orphan` (boolean, opcional): Cria um branch órfão.
  - `confirm` (boolean, opcional): Se deve pedir confirmação antes de trocar de branch. O padrão é `false`.

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
  - `depth` (integer, opcional): Profundidade positiva para um fetch raso.
  - `prune` (boolean, opcional): Remove referências de rastreamento remoto obsoletas.
  - `tags` (string, opcional): `"auto"`, `"all"` ou `"none"`.
  - `update_fetchhead` (boolean, opcional): Atualiza `FETCH_HEAD`. O padrão é `true`.
  - `confirm` (boolean, opcional): Se deve pedir confirmação antes de fazer fetch. O padrão é `false`.
  - `auth` (tabela, opcional): Configurações de autenticação remota. Usa a mesma estrutura do `ptool.git.clone(...)`.

Retorna:

- `received_objects` (inteiro)
- `indexed_objects` (inteiro)
- `local_objects` (inteiro)
- `total_objects` (inteiro)
- `received_bytes` (inteiro)
- `updated_refs` (string[])

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
  - `force` (boolean, opcional): Força cada refspec do push. O padrão é `false`.
  - `set_upstream` (boolean, opcional): Configura o branch atual enviado para rastrear o branch remoto de destino.
  - `confirm` (boolean, opcional): Se deve pedir confirmação antes de fazer push. O padrão é `false`.
  - `auth` (tabela, opcional): Configurações de autenticação remota. Usa a mesma estrutura do `ptool.git.clone(...)`.

Comportamento:

- Quando `refspecs` é omitido, `ptool` tenta fazer push da branch local atual para a branch de mesmo nome no remoto.
- Omitir `refspecs` enquanto HEAD está desconectado gera um erro.
- A tabela retornada contém `ok`, `refspecs` e `rejected`. Cada item rejeitado contém `reference` e `message`.

Exemplo:

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```


## APIs de fluxo de trabalho do Git

> `Unreleased` - Introduzido.

As APIs desta seção cobrem manutenção de repositórios, releases, inspeção de histórico, colaboração e automação de CI. Uma string ou um array denso de strings é aceito quando um parâmetro é documentado como `string|string[]`.

### Tabelas de resultados compartilhadas

As informações de commit retornadas por `commit_info()` e `log()` contêm `oid`, `message`, `summary`, `author`, `committer` e `parent_oids`. Uma tabela de assinatura contém `name`, `email`, `time_seconds` e `offset_minutes`.

Os métodos de integração retornam:

```lua
{
  outcome = "up_to_date" | "fast_forward" | "merged" | "conflicted",
  oid = "..." | nil,
  conflicts = {
    { path = "file", ancestor = true, ours = true, theirs = true },
  },
}
```

Os resultados de rebase usam os estados `"rebased"` ou `"conflicted"` e adicionam `current` e `total`. Operações interrompidas por conflitos podem ser continuadas ou abortadas com a API correspondente.

### Repositório e status

```lua
repo:path() -> string
repo:root() -> string|nil
repo:is_bare() -> boolean
repo:head() -> GitHeadInfo
repo:current_branch() -> string|nil
repo:status(options?) -> GitStatusSummary
repo:is_clean(options?) -> boolean
```

As opções de `status()` e `is_clean()` são `include_untracked` (padrão `true`), `include_ignored`, `recurse_untracked_dirs` (padrão `true`) e `paths`. `GitStatusSummary` contém `root`, `branch`, `head`, `upstream`, `ahead`, `behind`, `clean` e `entries`.

### Histórico e diff

```lua
repo:resolve(rev) -> GitObjectInfo
repo:commit_info(rev?) -> GitCommitInfo
repo:log(options?) -> GitCommitInfo[]
repo:diff(options?) -> GitDiff
repo:describe(options?) -> string|nil
```

- `resolve()` retorna `oid`, `kind` e `shorthand`.
- `commit_info()` usa `HEAD` por padrão.
- `log()` aceita `rev`, `max_count` (padrão `100`), `skip`, `first_parent`, `reverse` e `paths`.
- `diff()` aceita `from`, `to`, `cached`, `paths`, `context_lines` (padrão `3`) e `find_renames` (padrão `true`). Sem `from` ou `to`, compara o estado apropriado da worktree, do índice ou de HEAD.
- Um resultado de diff contém `patch`, `files_changed`, `insertions`, `deletions` e `deltas`. Cada delta contém `status`, `old_path`, `new_path` e `binary`.
- `describe()` aceita `rev`, `pattern`, `always`, `abbrev` (padrão `7`) e `dirty_suffix`.

```lua
local commits = repo:log({
  rev = "HEAD",
  max_count = 20,
  first_parent = true,
  paths = {"crates/ptool-engine"},
})
local changes = repo:diff({ from = "v0.10.0", to = "HEAD" })
```

### Branches

```lua
repo:branches(options?) -> GitBranchInfo[]
repo:branch_create(name, options?) -> GitBranchInfo
repo:branch_delete(name, options?) -> nil
repo:branch_rename(old_name, new_name, options?) -> GitBranchInfo
repo:branch_set_upstream(name, upstream_or_nil, options?) -> nil
```

- `branches()` aceita `kind = "local" | "remote" | "all"`; o padrão é `"local"`.
- As informações de branch contêm `name`, `kind`, `oid`, `head`, `upstream`, `ahead` e `behind`.
- `branch_create()` aceita `start_point`, `force`, `checkout`, `upstream` e `confirm`. O ponto inicial padrão é `HEAD`.
- `branch_delete()` aceita `force` e `confirm`. A exclusão do branch atual falha, e excluir um branch não mesclado exige `force = true`.
- `branch_rename()` aceita `force` e `confirm`.
- Passe `nil` para `branch_set_upstream()` para remover o upstream. Suas opções contêm apenas `confirm`.

### Tags

```lua
repo:tags(pattern?) -> GitTagInfo[]
repo:tag_create(name, target?, options?) -> GitTagInfo
repo:tag_delete(name, options?) -> nil
```

`tag_create()` aponta para `HEAD` por padrão. Sem `message`, cria uma tag leve; com `message`, cria uma tag anotada. As opções são `message`, `tagger`, `force` e `confirm`. O tagger usa os campos de assinatura compartilhados.

As informações de tag contêm `name`, `oid`, `target_oid`, `target_kind`, `annotated`, `message` e `tagger`. `tags(pattern)` usa correspondência glob do Git. Excluir uma tag altera apenas o repositório local; exclua uma tag remota com um refspec de push explícito.

```lua
local tag = repo:tag_create("v1.0.0", "HEAD", {
  message = "Release v1.0.0",
})
repo:push("origin", "refs/tags/v1.0.0:refs/tags/v1.0.0")
```

### Remotos e transferência

```lua
repo:remotes() -> GitRemoteInfo[]
repo:remote(name) -> GitRemoteInfo
repo:remote_add(name, url, options?) -> GitRemoteInfo
repo:remote_remove(name, options?) -> nil
repo:remote_rename(name, new_name, options?) -> GitRemoteInfo
repo:remote_set_url(name, url, options?) -> GitRemoteInfo
repo:fetch(remote?, options?) -> GitFetchStats
repo:push(remote?, refspecs?, options?) -> GitPushResult
repo:pull(remote?, branch?, options?) -> GitIntegrateResult
```

As informações de remoto contêm `name`, `url`, `push_url`, `fetch_refspecs` e `push_refspecs`. `remote_add()` aceita `push_url` e `confirm`. `remote_set_url()` aceita `push = true` para alterar a URL de push em vez da URL de fetch. As operações de remoção, renomeação e alteração de URL aceitam `confirm`.

`fetch()` aceita `refspecs`, `auth`, `depth`, `prune`, `tags`, `update_fetchhead` e `confirm`. `push()` aceita `auth`, `force`, `set_upstream` e `confirm`.

`pull()` usa por padrão o remoto `"origin"`, o branch atual e `strategy = "ff_only"`. Também aceita `strategy = "merge" | "rebase"`, as opções de fetch `auth`, `depth`, `prune`, `tags` e `update_fetchhead`, além de `signature`, `message` e `confirm`. Pull exige um repositório limpo antes de começar.

### Worktree, índice e recuperação

```lua
repo:add(paths, options?) -> nil
repo:restore(paths, options?) -> nil
repo:reset(rev?, options?) -> nil
repo:remove(paths, options?) -> nil
repo:clean(options?) -> string[]
```

- `restore()` aceita `source` (padrão `"HEAD"`), `staged`, `worktree` e `confirm`. Quando apenas `staged = true` é informado, a worktree não é alterada.
- `reset()` aceita `mode = "soft" | "mixed" | "hard"`, `force` e `confirm`. Um reset hard exige `force = true`.
- `remove()` aceita `cached`, `force` e `confirm`.
- `clean()` aceita `dry_run`, `force`, `dirs`, `ignored`, `paths` e `confirm`. O padrão é `dry_run = true`. A exclusão real exige `dry_run = false` e `force = true`. Diretórios não são alterados a menos que `dirs = true`.

```lua
local candidates = repo:clean()
repo:clean({ dry_run = false, force = true, dirs = true, confirm = true })
```

### Configuração

```lua
repo:config_get(name, options?) -> string|boolean|integer|nil
repo:config_list(options?) -> GitConfigEntry[]
repo:config_set(name, value, options?) -> nil
repo:config_remove(name, options?) -> nil
```

`scope` é `"local"`, `"global"` ou `"system"`; o padrão para leitura é o valor disponível de maior prioridade e, para escrita, `"local"`. Entradas de configuração contêm `name`, `value` e `scope`. A configuração do sistema é somente leitura. `config_set()` e `config_remove()` globais exigem `confirm = true` e sempre mostram uma confirmação.

### Merge, cherry-pick e revert

```lua
repo:state() -> string
repo:conflicts() -> GitConflictEntry[]
repo:merge_analysis(rev) -> string
repo:merge(rev, options?) -> GitIntegrateResult
repo:merge_abort(options?) -> nil
repo:cherry_pick(rev, options?) -> GitIntegrateResult
repo:cherry_pick_abort(options?) -> nil
repo:revert(rev, options?) -> GitIntegrateResult
repo:revert_abort(options?) -> nil
```

`merge()` aceita `ff = "allow" | "only" | "never"`, `signature`, `message` e `confirm`. Merge, cherry-pick e revert exigem um repositório limpo antes de começar para que abort possa restaurar `ORIG_HEAD` com segurança.

Cherry-pick e revert aceitam `commit` (padrão `true`), `signature`, `message`, `mainline` e `confirm`. Use `commit = false` para atualizar o índice e a worktree sem criar um commit. Os métodos abort aceitam `confirm`.

### Stash e rebase

```lua
repo:stash_save(message?, options?) -> string
repo:stashes() -> GitStashInfo[]
repo:stash_apply(index?, options?) -> GitIntegrateResult
repo:stash_pop(index?, options?) -> GitIntegrateResult
repo:stash_drop(index?, options?) -> nil
repo:rebase(options) -> GitRebaseResult
repo:rebase_continue(options?) -> GitRebaseResult
repo:rebase_abort(options?) -> nil
```

Os índices de stash usam `0` por padrão. `stash_save()` aceita `include_untracked`, `include_ignored`, `keep_index`, `signature` e `confirm`. Apply e pop aceitam `reinstate_index` e `confirm`; drop aceita `confirm`. As informações de stash contêm `index`, `message` e `oid`.

`rebase()` exige `upstream` e aceita `onto`, `branch` (padrão `"HEAD"`), `signature` e `confirm`. A primeira versão oferece operações pick não interativas; squash, fixup, reword e edit interativos não estão disponíveis. Continue aceita `signature` e `confirm`; abort aceita `confirm`.

### Repositórios avançados

```lua
repo:worktrees() -> GitWorktreeInfo[]
repo:worktree_add(name, path, options?) -> GitWorktreeInfo
repo:worktree_lock(name, reason?, options?) -> nil
repo:worktree_unlock(name, options?) -> nil
repo:worktree_prune(name, options?) -> nil
repo:submodules() -> GitSubmoduleInfo[]
repo:submodule_init(name?, options?) -> nil
repo:submodule_update(name?, options?) -> nil
repo:submodule_sync(name?, options?) -> nil
repo:blame(path, options?) -> GitBlameHunk[]
```

- As informações de worktree contêm `name`, `path`, `locked`, `lock_reason` e `valid`. As opções de add são `reference`, `lock`, `checkout_existing` e `confirm`. As opções de prune são `valid`, `locked`, `working_tree`, `force` e `confirm`; lock e unlock também aceitam `confirm`.
- As informações de submódulo contêm `name`, `path`, `url`, `branch`, `head_oid`, `index_oid` e `workdir_oid`. As opções de init são `overwrite`, `recursive` e `confirm`. As opções de update são `init`, `recursive`, `allow_fetch`, `auth` e `confirm`. As opções de sync são `recursive` e `confirm`.
- O processamento recursivo de submódulos fica desativado, a menos que `recursive = true`.
- `blame()` aceita `newest`, `oldest`, `min_line`, `max_line`, `first_parent`, sinalizadores de rastreamento de cópias/movimentos, `ignore_whitespace` e `use_mailmap`. Cada hunk contém `final_start_line`, `original_start_line`, `line_count`, `commit_oid`, `author`, `origin_path` e `boundary`.

## Segurança e compatibilidade

Todos os métodos que alteram o repositório e aceitam `confirm` usam `false` por padrão. `force` e `confirm` expressam intenções diferentes: `force` habilita um comportamento que seria rejeitado, enquanto `confirm` consulta o usuário antes de executar uma ação já válida.

Os padrões existentes permanecem compatíveis: o remoto padrão é `"origin"`, push sem refspecs envia o branch atual, status inclui arquivos não rastreados e commits vazios continuam permitidos, a menos que `allow_empty = false` seja informado.

A exclusão de tags remotas é expressa como um refspec de push:

```lua
repo:push("origin", ":refs/tags/v1.0.0", { confirm = true })
```
