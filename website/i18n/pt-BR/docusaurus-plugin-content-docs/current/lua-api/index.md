# Visão geral da API de Lua

`ptool` expõe um amplo conjunto de utilitários por meio de `ptool` e `p`.

## APIs principais

- [API principal de Lua](./core.md): Ciclo de vida do script, execução de
  processos, configuração e utilitários de terminal.

## Módulos

- [API de argumentos](./args.md): Parse de esquemas de argumentos de linha de
  comando para scripts Lua.
- [API de SemVer](./semver.md): Faz parse, compara e incrementa versões
  semânticas.
- [API de hash](./hash.md): Calcula digests SHA-256, SHA-1 e MD5.
- [API de rede](./net.md): Faz parse de URLs, endereços IP e pares host-porta.
- [API de sistema operacional](./os.md): Lê variáveis de ambiente do runtime e
  inspeciona detalhes do processo hospedeiro.
- [API de plataforma](./platform.md): Detecta o SO, a arquitetura e o target
  triple atuais.
- [API ANSI](./ansi.md): Monta saída de terminal com estilo usando sequências
  ANSI.
- [API de log](./log.md): Escreve logs de terminal com timestamp e nível.
- [API HTTP](./http.md): Envia requisições HTTP e consome corpos de resposta.
- [API JSON](./json.md): Faz parse de texto JSON e serializa valores Lua como
  JSON.
- [API YAML](./yaml.md): Faz parse de texto YAML, lê valores aninhados e
  serializa valores Lua como YAML.
- [API de banco de dados](./db.md): Abre conexões de banco de dados e executa
  consultas SQL.
- [API SSH](./ssh.md): Conecta-se a hosts remotos, executa comandos e transfere
  arquivos.
- [API de caminhos](./path.md): Manipula caminhos lexicalmente sem tocar no
  sistema de arquivos.
- [API TOML](./toml.md): Faz parse, serializa, lê, atualiza e remove valores TOML.
- [API de regex](./re.md): Compila regex e busca, captura, substitui ou divide
  texto.
- [API de strings](./str.md): Remove espaços, divide, junta, substitui e
  formata strings.
- [API de sistema de arquivos](./fs.md): Lê, grava, cria e faz glob em caminhos
  do sistema de arquivos.
- [API de shell](./sh.md): Divide linhas de comando em estilo shell em arrays
  de argumentos.
- [API de templates](./template.md): Renderiza templates de texto a partir de
  dados Lua.

Use esta página como ponto de entrada e depois salte para o módulo necessário
para consultar a referência completa das funções.
