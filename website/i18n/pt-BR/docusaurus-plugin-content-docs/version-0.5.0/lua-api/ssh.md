# API de SSH

Os helpers para conexão SSH, execução remota e transferência de arquivos estão
disponíveis em `ptool.ssh` e `p.ssh`.

## ptool.ssh.connect

> `v0.1.0` - Introduced.

`ptool.ssh.connect(target_or_options)` prepara um handle de conexão SSH apoiado
no comando `ssh` do sistema e retorna um objeto `Connection`.

`ssh` precisa estar disponível em `PATH`.

Argumentos:

- `target_or_options` (string|table, obrigatório):
  - Quando uma string é fornecida, ela é tratada como um destino SSH.
  - Quando uma tabela é fornecida, atualmente ela suporta:
    - `target` (string, opcional): String de destino SSH, como
      `"deploy@example.com"` ou `"deploy@example.com:2222"`.
    - `host` (string, opcional): Hostname ou endereço IP.
    - `user` (string, opcional): Nome de usuário SSH. O padrão é `$USER`, ou
      `"root"` se `$USER` não estiver disponível.
    - `port` (integer, opcional): Porta SSH. O padrão é `22`.
    - `auth` (table, opcional): Configuração de autenticação.
    - `host_key` (table, opcional): Configuração de verificação de chave de
      host.
    - `connect_timeout_ms` (integer, opcional): Timeout em milissegundos.
      O padrão é `10000`.
    - `keepalive_interval_ms` (integer, opcional): Intervalo de keepalive em
      milissegundos.

Exemplos de strings de destino suportadas:

```lua
local a = ptool.ssh.connect("deploy@example.com")
local b = ptool.ssh.connect("deploy@example.com:2222")
local c = ptool.ssh.connect("[2001:db8::10]:2222")
```

Campos de `auth`:

- `private_key_file` (string, opcional): Caminho para um arquivo de chave
  privada.
- `private_key_passphrase` (string, opcional): Senha da chave privada.
  Atualmente não é suportada.
- `password` (string, opcional): Autenticação por senha. Atualmente não é
  suportada.

Comportamento de autenticação:

- Se `auth.private_key_file` for fornecido, `ptool` invoca `ssh` com essa
  chave via `-i` e também define `IdentitiesOnly=yes`.
- Se `auth.private_key_passphrase` ou `auth.password` for fornecido,
  `ptool.ssh.connect(...)` falha porque esta API não passa esses segredos para
  o comando `ssh` do sistema.
- Caso contrário, a autenticação é delegada à configuração local do OpenSSH,
  incluindo definições e mecanismos como `IdentityFile`, `ProxyJump`,
  `ProxyCommand`, `ssh-agent` e certificados.
- Caminhos de chave relativos são resolvidos a partir do diretório de runtime
  atual do `ptool`, então eles seguem `ptool.cd(...)`.
- `~` e `~/...` são expandidos em caminhos de chave.

Campos de `host_key`:

- `verify` (string, opcional): Modo de verificação de chave de host. Valores
  suportados:
  - `"known_hosts"`: Verifica contra um arquivo `known_hosts` (padrão).
  - `"ignore"`: Ignora a verificação de chave de host.
- `known_hosts_file` (string, opcional): Caminho para um arquivo
  `known_hosts`. Usado apenas quando `verify = "known_hosts"`.

Comportamento de chave de host:

- Se `verify = "ignore"`, `ptool` invoca `ssh` com `StrictHostKeyChecking=no`
  e `UserKnownHostsFile=/dev/null`.
- Se `verify = "known_hosts"` e `known_hosts_file` for fornecido, `ptool`
  invoca `ssh` com `StrictHostKeyChecking=yes` e esse `UserKnownHostsFile`.
- Se `verify = "known_hosts"` e `known_hosts_file` for omitido, ou quando
  `host_key` é omitido por completo, o tratamento da chave de host é delegado
  à configuração local do OpenSSH e aos padrões dele.
- Caminhos relativos de `known_hosts_file` são resolvidos a partir do diretório
  de runtime atual do `ptool`.
- `~` e `~/...` são expandidos em `known_hosts_file`.
- Quando `known_hosts_file` é fornecido explicitamente, ele substitui o
  `UserKnownHostsFile` padrão usado pelo comando `ssh` local para esta conexão.

Exemplo:

```lua
local ssh = ptool.ssh.connect({
  host = "example.com",
  user = "deploy",
  port = 22,
  auth = {
    private_key_file = "~/.ssh/id_ed25519",
  },
  host_key = {
    verify = "known_hosts",
  },
})
```

## Connection

> `v0.1.0` - Introduced.

`Connection` representa um handle de conexão apoiado em OpenSSH retornado por
`ptool.ssh.connect()`.

Ele é implementado como um userdata de Lua.

Campos e métodos:

- Campos:
  - `conn.host` (string)
  - `conn.user` (string)
  - `conn.port` (integer)
  - `conn.target` (string)
- Métodos:
  - `conn:run(...)` -> `table`
  - `conn:path(path)` -> `RemotePath`
  - `conn:exists(path)` -> `boolean`
  - `conn:is_file(path)` -> `boolean`
  - `conn:is_dir(path)` -> `boolean`
  - `conn:upload(local_path, remote_path[, options])` -> `table`
  - `conn:download(remote_path, local_path[, options])` -> `table`
  - `conn:close()` -> `nil`

### run

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:run`.

`conn:run(...)` executa um comando remoto através da conexão SSH atual.

As seguintes formas de chamada são suportadas:

```lua
conn:run("hostname")
conn:run("echo", "hello world")
conn:run("echo", {"hello", "world"})
conn:run("hostname", { stdout = "capture" })
conn:run("echo", {"hello", "world"}, { stdout = "capture" })
conn:run({ cmd = "git", args = {"rev-parse", "HEAD"} })
```

Regras de argumento:

- `conn:run(cmdline)`: `cmdline` é enviado como a string de comando remoto.
- `conn:run(cmd, argsline)`: `cmd` é tratado como o comando, e `argsline` é
  dividido usando regras de estilo shell (`shlex`).
- `conn:run(cmd, args)`: `cmd` é uma string e `args` é um array de strings.
  Os argumentos passam por quoting de shell antes da execução remota.
- `conn:run(cmdline, options)`: `options` sobrescreve esta invocação.
- `conn:run(cmd, args, options)`: `options` sobrescreve esta invocação.
- `conn:run(options)`: `options` é uma tabela.
- Quando o segundo argumento é uma tabela: se ela for um array (chaves inteiras
  consecutivas `1..n`), ela é tratada como `args`; caso contrário, é tratada
  como `options`.

Quando `conn:run(options)` é usado, `options` atualmente suporta:

- `cmd` (string, obrigatório): O nome do comando ou caminho do executável.
- `args` (string[], opcional): A lista de argumentos.
- `cwd` (string, opcional): Diretório de trabalho remoto. Isso é aplicado ao
  prefixar `cd ... &&` ao comando shell remoto gerado.
- `env` (table, opcional): Variáveis de ambiente remotas, onde chaves e
  valores são strings. Isso é aplicado ao prefixar `export ... &&` ao comando
  shell remoto gerado.
- `stdin` (string, opcional): String enviada ao stdin do processo remoto.
- `echo` (boolean, opcional): Se deve ecoar o comando remoto antes da
  execução. O padrão é `true`.
- `check` (boolean, opcional): Se deve gerar erro imediatamente quando o status
  de saída não for `0`. O padrão é `false`.
- `stdout` (string, opcional): Estratégia de tratamento de stdout. Valores
  suportados:
  - `"inherit"`: Herda para o terminal atual (padrão).
  - `"capture"`: Captura em `res.stdout`.
  - `"null"`: Descarta a saída.
- `stderr` (string, opcional): Estratégia de tratamento de stderr. Valores
  suportados:
  - `"inherit"`: Herda para o terminal atual (padrão).
  - `"capture"`: Captura em `res.stderr`.
  - `"null"`: Descarta a saída.

Quando as formas abreviadas são usadas, a tabela `options` suporta apenas:

- `stdin` (string, opcional): String enviada ao stdin do processo remoto.
- `echo` (boolean, opcional): Se deve ecoar o comando remoto antes da
  execução. O padrão é `true`.
- `check` (boolean, opcional): Se deve gerar erro imediatamente quando o status
  de saída não for `0`. O padrão é `false`.
- `stdout` (string, opcional): Estratégia de tratamento de stdout. Valores
  suportados:
  - `"inherit"`: Herda para o terminal atual (padrão).
  - `"capture"`: Captura em `res.stdout`.
  - `"null"`: Descarta a saída.
- `stderr` (string, opcional): Estratégia de tratamento de stderr. Valores
  suportados:
  - `"inherit"`: Herda para o terminal atual (padrão).
  - `"capture"`: Captura em `res.stderr`.
  - `"null"`: Descarta a saída.

Regras do valor de retorno:

- Uma tabela sempre é retornada com os seguintes campos:
  - `ok` (boolean): Se o status de saída remoto é `0`.
  - `code` (integer|nil): O status de saída remoto. Se o processo remoto sair
    por sinal, este valor será `nil`.
  - `target` (string): A string de destino SSH no formato `user@host:port`.
  - `stdout` (string, opcional): Presente quando `stdout = "capture"`.
  - `stderr` (string, opcional): Presente quando `stderr = "capture"`.
  - `assert_ok(self)` (function): Gera erro quando `ok = false`.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:run("uname -a", { stdout = "capture" })
print(res.target)
print(res.stdout)
local res2 = ssh:run({
  cmd = "git",
  args = {"rev-parse", "HEAD"},
  cwd = "/srv/app",
  env = {
    FOO = "bar",
  },
  stdout = "capture",
  check = true,
})

print(res2.stdout)
```

### path

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:path`.

`conn:path(path)` cria um objeto `RemotePath` reutilizável vinculado à conexão
SSH atual.

- `path` (string, obrigatório): O caminho remoto.
- Retorna: Um objeto `RemotePath` que pode ser passado para
  `conn:upload(...)`, `conn:download(...)` e `ptool.fs.copy(...)`.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

ssh:download(remote_release, "./tmp/current.tar.gz")
```

### exists

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:exists`.

`conn:exists(path)` verifica se um caminho remoto existe.

- `path` (string|remote path, obrigatório): O caminho remoto a verificar.
  Ele pode ser uma string ou um valor criado por `conn:path(...)`.
- Retorna: `true` quando o caminho remoto existe; caso contrário, `false`.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

print(ssh:exists("/srv/app"))
print(ssh:path("/srv/app/releases/current.tar.gz"):exists())
```

### is_file

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:is_file`.

`conn:is_file(path)` verifica se um caminho remoto existe e é um arquivo
regular.

- `path` (string|remote path, obrigatório): O caminho remoto a verificar.
  Ele pode ser uma string ou um valor criado por `conn:path(...)`.
- Retorna: `true` quando o caminho remoto é um arquivo; caso contrário,
  `false`.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if ssh:is_file(remote_tarball) then
  print("release tarball exists")
end
```

### is_dir

> `v0.2.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:is_dir`.

`conn:is_dir(path)` verifica se um caminho remoto existe e é um diretório.

- `path` (string|remote path, obrigatório): O caminho remoto a verificar.
  Ele pode ser uma string ou um valor criado por `conn:path(...)`.
- Retorna: `true` quando o caminho remoto é um diretório; caso contrário,
  `false`.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```

### upload

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:upload`.

`conn:upload(local_path, remote_path[, options])` envia um arquivo ou diretório
local para o host remoto.

- `local_path` (string, obrigatório): O arquivo ou diretório local a enviar.
- `remote_path` (string|remote path, obrigatório): O caminho de destino no host
  remoto. Ele pode ser uma string ou um valor criado por `conn:path(...)`.
- `options` (table, opcional): Opções de transferência.
- Retorna: Uma tabela com os seguintes campos:
  - `bytes` (integer): O número de bytes de arquivos regulares enviados.
    Quando um diretório é enviado, este valor é a soma dos tamanhos dos
    arquivos enviados.
  - `from` (string): O caminho de origem local.
  - `to` (string): O caminho de destino remoto.

Opções de transferência suportadas:

- `parents` (boolean, opcional): Cria o diretório pai de `remote_path` antes do
  envio. O padrão é `false`.
- `overwrite` (boolean, opcional): Se um arquivo de destino existente pode ser
  substituído. O padrão é `true`.
- `echo` (boolean, opcional): Se deve imprimir a transferência antes de
  executá-la. O padrão é `false`.

Comportamento de diretórios:

- Quando `local_path` é um arquivo, o comportamento não muda.
- Quando `local_path` é um diretório e `remote_path` não existe,
  `remote_path` se torna a raiz do diretório de destino.
- Quando `local_path` é um diretório e `remote_path` já existe como diretório,
  o diretório de origem é criado dentro dele usando o basename do diretório de
  origem.
- `overwrite = false` rejeita um diretório de destino já existente para a raiz
  final do diretório.
- Envios de diretório exigem que `tar` esteja disponível no host remoto.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

local res = ssh:upload("./dist/app.tar.gz", remote_tarball, {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.to)
```

Exemplo de diretório:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:upload("./dist/assets", "/srv/app/releases", {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.to) -- deploy@example.com:22:/srv/app/releases
```

### download

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:download`.

`conn:download(remote_path, local_path[, options])` baixa um arquivo ou
diretório remoto para um caminho local.

- `remote_path` (string|remote path, obrigatório): O caminho de origem no host
  remoto. Ele pode ser uma string ou um valor criado por `conn:path(...)`.
- `local_path` (string, obrigatório): O caminho de destino local.
- `options` (table, opcional): Opções de transferência.
- Retorna: Uma tabela com os seguintes campos:
  - `bytes` (integer): O número de bytes de arquivos regulares baixados.
    Quando um diretório é baixado, este valor é a soma dos tamanhos dos
    arquivos baixados.
  - `from` (string): O caminho de origem remoto.
  - `to` (string): O caminho de destino local.

Opções de transferência suportadas:

- `parents` (boolean, opcional): Cria o diretório pai de `local_path` antes do
  download. O padrão é `false`.
- `overwrite` (boolean, opcional): Se um arquivo de destino existente pode ser
  substituído. O padrão é `true`.
- `echo` (boolean, opcional): Se deve imprimir a transferência antes de
  executá-la. O padrão é `false`.

Comportamento de diretórios:

- Quando `remote_path` é um arquivo, o comportamento não muda.
- Quando `remote_path` é um diretório e `local_path` não existe,
  `local_path` se torna a raiz do diretório de destino.
- Quando `remote_path` é um diretório e `local_path` já existe como diretório,
  o diretório remoto de origem é criado dentro dele usando o basename do
  diretório remoto.
- `overwrite = false` rejeita um diretório de destino já existente para a raiz
  final do diretório.
- Downloads de diretório exigem que `tar` esteja disponível no host remoto.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:download("/srv/app/logs/app.log", "./tmp/app.log", {
  parents = true,
  overwrite = false,
  echo = true,
})

print(res.bytes)
print(res.from)
```

Exemplo de diretório:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")

local res = ssh:download("/srv/app/releases/assets", "./tmp/releases", {
  parents = true,
  overwrite = true,
  echo = true,
})

print(res.bytes)
print(res.from)
```

### close

> `v0.1.0` - Introduced.

Canonical API name: `ptool.ssh.Connection:close`.

`conn:close()` fecha o handle de conexão SSH.

Comportamento:

- Depois de fechada, a conexão não pode mais ser usada.
- Fechar uma conexão que já está fechada é permitido e não tem efeito.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
ssh:close()
```

## RemotePath

> `v0.1.0` - Introduced.

`RemotePath` representa um caminho remoto vinculado a um `Connection` e
retornado por `conn:path(path)`.

Ele é implementado como um userdata de Lua.

Métodos:

- `remote:exists()` -> `boolean`
- `remote:is_file()` -> `boolean`
- `remote:is_dir()` -> `boolean`

### exists

`remote:exists()` verifica se o caminho remoto existe.

- Retorna: `true` quando o caminho remoto existe; caso contrário, `false`.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_release = ssh:path("/srv/app/releases/current.tar.gz")

print(remote_release:exists())
```

### is_file

`remote:is_file()` verifica se o caminho remoto existe e é um arquivo regular.

- Retorna: `true` quando o caminho remoto é um arquivo; caso contrário,
  `false`.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local remote_tarball = ssh:path("/srv/app/releases/current.tar.gz")

if remote_tarball:is_file() then
  print("release tarball exists")
end
```

### is_dir

`remote:is_dir()` verifica se o caminho remoto existe e é um diretório.

- Retorna: `true` quando o caminho remoto é um diretório; caso contrário,
  `false`.

Exemplo:

```lua
local ssh = ptool.ssh.connect("deploy@example.com")
local releases = ssh:path("/srv/app/releases")

if releases:is_dir() then
  print("releases directory is ready")
end
```
