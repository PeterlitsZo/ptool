# Primeiros passos

`ptool` executa scripts Lua e injeta uma biblioteca padrão voltada para automação prática.

Hoje, o ponto de entrada principal é:

```sh
ptool run <file>
```

Para arquivos `.lua`, você também pode usar a forma abreviada:

```sh
ptool <file.lua>
```

Para exploração interativa, `ptool` também oferece:

```sh
ptool repl
```

Quando um script é executado, `ptool` expõe sua API por meio da tabela global `ptool` e do alias curto `p`.

## Instalação

No Linux e no macOS, instale o `ptool` com o instalador de releases:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash
```

O instalador baixa a release pré-compilada mais recente para a plataforma atual, instala o `ptool` em `~/.local/bin/ptool` e mostra uma dica de PATH se necessário.

Para instalar uma tag de release específica em vez da versão estável mais recente:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- v0.2.0
```

Para instalar em um diretório de binários personalizado em vez de `~/.local/bin`:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- --bin-dir "$HOME/.cargo/bin"
```

## Script mínimo

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

`ptool.use(...)` declara a versão mínima de `ptool` exigida pelo script. Isso deixa explícita a versão da API esperada pelo script e falha cedo em runtimes mais antigos. Veja [API principal de Lua](./lua-api/core.md) para detalhes.

Execute com:

```sh
ptool run script.lua
ptool script.lua
```

## Passando argumentos

Você pode passar argumentos extras de CLI após o caminho do script:

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

Depois, faça o parse deles dentro do script com `ptool.args.parse(...)`.

## Scripts com shebang

`ptool` oferece suporte a arquivos com shebang. Com a forma abreviada para `.lua`, um script pode começar com:

```text
#!/usr/bin/env ptool
```

Isso permite executar o script diretamente depois que ele tiver o bit de execução.

## O que você recebe

- Um executor de scripts que entende arquivos com shebang.
- Um REPL interativo para testar expressões Lua e APIs do `ptool` diretamente.
- Auxiliares Lua para semver, data e hora, caminhos, arquivos, TOML, regexes, strings, HTTP, SSH, bancos de dados e modelos.
- Utilitários voltados para CLI para executar comandos, fazer parse de argumentos e solicitar entrada interativa.

## Próximos passos

- Abra [REPL](./repl.md) para ver uso interativo, entrada multilinha e comportamento do teclado.
- Use [Visão geral da API de Lua](./lua-api/index.md) para navegar pelas APIs principais e pelos módulos disponíveis.
- Comece por [API principal de Lua](./lua-api/core.md) para entender controle de versão, execução de processos, configuração e utilitários de ciclo de vida do script.
- Abra uma página de módulo como [API de argumentos](./lua-api/args.md) quando precisar da referência detalhada de um conjunto específico de recursos.
