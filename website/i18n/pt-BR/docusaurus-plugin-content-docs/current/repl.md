# REPL

`ptool repl` inicia uma sessão interativa de Lua com a API padrão de `ptool`
já carregada.

## Iniciar o REPL

```sh
ptool repl
```

Quando o REPL inicia, `ptool` mostra um banner e aguarda entrada Lua.

## O que ele oferece

- A tabela global `ptool` e o alias curto `p`.
- Os mesmos utilitários embarcados que você pode usar em `ptool run <file>`.
- Avaliação interativa de expressões e instruções Lua.
- Edição estilo readline, incluindo movimento de cursor com as setas e navegação
  pelo histórico dentro da sessão.

## Uso básico

Digite uma expressão para avaliá-la imediatamente:

```lua
1 + 2
```

O REPL imprime os valores retornados usando o mesmo inspetor usado em outras
partes do `ptool`.

Você também pode chamar APIs do `ptool` diretamente:

```lua
p.str.trim("  hello  ")
```

## Entrada multilinha

Se a entrada atual estiver incompleta, o prompt muda de `>>> ` para `... `.
Isso permite continuar digitando um bloco como uma função ou uma estrutura de
controle de fluxo:

```lua
for i = 1, 3 do
  print(i)
end
```

Quando a entrada fica completa, `ptool` avalia todo o bloco.

## Comportamento do teclado

- `Up` e `Down` percorrem comandos digitados anteriormente na mesma sessão do
  REPL.
- `Left` e `Right` movem o cursor dentro da linha de entrada atual.
- `Ctrl-C` limpa a entrada atual. Se você estiver no meio de um bloco
  multilinha, ele descarta o bloco em buffer e volta ao prompt principal.
- `Ctrl-D` sai do REPL.

## Notas

- `ptool repl` exige um TTY interativo.
- O histórico do REPL atualmente existe apenas durante a sessão atual e não é
  gravado em um arquivo de histórico.
