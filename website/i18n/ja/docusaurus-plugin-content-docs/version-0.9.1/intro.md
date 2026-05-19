# はじめに

`ptool` は Lua スクリプトを実行し、実用的な自動化のための標準 ライブラリを注入します。

現在の主なエントリーポイントは次のとおりです。

```sh
ptool run <file>
```

`.lua` ファイルでは、短縮形も使えます。

```sh
ptool <file.lua>
```

対話的に試したい場合、`ptool` には次のコマンドもあります。

```sh
ptool repl
```

スクリプトの実行時、`ptool` はグローバルテーブル `ptool` と短い別名 `p` を通じて API を公開します。

## インストール

Linux と macOS では、リリース用インストーラーで `ptool` を インストールできます。

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash
```

このインストーラーは現在のプラットフォーム向けの最新ビルド済み リリースをダウンロードし、`ptool` を `~/.local/bin/ptool` に インストールし、必要に応じて PATH のヒントを表示します。

最新の安定版ではなく特定のリリースタグをインストールするには:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- v0.2.0
```

`~/.local/bin` ではなく独自のバイナリディレクトリにインストールする には:

```sh
curl -fsSL https://peterlits.net/ptool/install.sh | bash -s -- --bin-dir "$HOME/.cargo/bin"
```

## 最小スクリプト

```lua
ptool.use("v0.1.0")

ptool.run("echo", {"hello", "world"})
```

`ptool.use(...)` は、スクリプトに必要な `ptool` のバージョンまたは バージョン要件を宣言します。これにより期待する API バージョンを明示でき、 互換性のないランタイムでは早い段階で失敗します。`v0.1.0` のような通常の バージョンと、`^0.6.0` のような Cargo 形式の要件を受け付けます。詳しくは [コア Lua API](./lua-api/core.md) を参照してください。

実行方法:

```sh
ptool run script.lua
ptool script.lua
```

## 引数の受け渡し

スクリプトのパスの後ろに追加の CLI 引数を渡せます。

```sh
ptool run script.lua --name alice -v a.txt b.txt
ptool script.lua --name alice -v a.txt b.txt
```

これらはスクリプト内で `ptool.args.parse(...)` を使って解析できます。

## Shebang スクリプト

`ptool` は shebang ファイルをサポートしています。`.lua` 向けの短縮形を 使うと、スクリプトは次のように始められます。

```text
#!/usr/bin/env ptool
```

これにより、実行権限ビットを付けたあとでスクリプトを直接実行できます。

## 利用できるもの

- shebang ファイルを理解するスクリプトランナー。
- Lua 式や `ptool` API をその場で試せる対話型 REPL。
- サーバー、日時、パス、ファイル、TOML、正規表現、文字列、HTTP、SSH、データベース、テンプレート用の Lua ヘルパー。
- コマンド実行、引数解析、対話入力のための CLI 向けヘルパー。

## 次のステップ

- [REPL](./repl.md) を開いて、対話的な使い方、複数行入力、キーボードの 挙動を確認する。
- [Lua API 概要](./lua-api/index.md) を使って、コア API と利用可能な モジュールを確認する。
- [コア Lua API](./lua-api/core.md) から始めて、バージョン制御、 プロセス実行、設定、スクリプトのライフサイクルヘルパーを理解する。
- 特定の機能セットの詳細なリファレンスが必要な場合は、 [引数 API](./lua-api/args.md) のようなモジュールページを開く。
