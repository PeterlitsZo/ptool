# パス API

字句的なパスヘルパーは `ptool.path` と `p.path` にあります。

## ptool.path.join

> `v0.1.0` - Introduced.

`ptool.path.join(...segments)` は複数のパスセグメントを連結し、
正規化されたパスを返します。

- `segments` (string, 1 つ以上): パスセグメント。
- 戻り値: `string`。

例:

```lua
print(ptool.path.join("tmp", "a", "..", "b")) -- tmp/b
```

## ptool.path.normalize

> `v0.1.0` - Introduced.

`ptool.path.normalize(path)` は字句的なパス正規化 (`.` と `..` の処理) を
行います。

- `path` (string, 必須): 入力パス。
- 戻り値: `string`。

例:

```lua
print(ptool.path.normalize("./a/../b")) -- b
```

## ptool.path.abspath

> `v0.1.0` - Introduced.

`ptool.path.abspath(path[, base])` は絶対パスを計算します。

- `path` (string, 必須): 入力パス。
- `base` (string, 任意): ベースディレクトリ。省略時は現在のプロセスの
  作業ディレクトリが使われます。
- 戻り値: `string`。
- 受け付ける文字列引数は 1 個または 2 個のみです。

例:

```lua
print(ptool.path.abspath("src"))
print(ptool.path.abspath("lib", "/tmp/demo"))
```

## ptool.path.relpath

> `v0.1.0` - Introduced.

`ptool.path.relpath(path[, base])` は `base` から `path` への相対パスを
計算します。

- `path` (string, 必須): 対象パス。
- `base` (string, 任意): 開始ディレクトリ。省略時は現在のプロセスの
  作業ディレクトリが使われます。
- 戻り値: `string`。
- 受け付ける文字列引数は 1 個または 2 個のみです。

例:

```lua
print(ptool.path.relpath("src/main.rs", "/tmp/project"))
```

## ptool.path.isabs

> `v0.1.0` - Introduced.

`ptool.path.isabs(path)` はパスが絶対パスかどうかを確認します。

- `path` (string, 必須): 入力パス。
- 戻り値: `boolean`。

例:

```lua
print(ptool.path.isabs("/tmp")) -- true
```

## ptool.path.dirname

> `v0.1.0` - Introduced.

`ptool.path.dirname(path)` はディレクトリ名部分を返します。

- `path` (string, 必須): 入力パス。
- 戻り値: `string`。

例:

```lua
print(ptool.path.dirname("a/b/c.txt")) -- a/b
```

## ptool.path.basename

> `v0.1.0` - Introduced.

`ptool.path.basename(path)` は最後のパスセグメント
(ファイル名部分) を返します。

- `path` (string, 必須): 入力パス。
- 戻り値: `string`。

例:

```lua
print(ptool.path.basename("a/b/c.txt")) -- c.txt
```

## ptool.path.extname

> `v0.1.0` - Introduced.

`ptool.path.extname(path)` は拡張子 (`.` を含む) を返します。拡張子がない
場合は空文字列を返します。

- `path` (string, 必須): 入力パス。
- 戻り値: `string`。

例:

```lua
print(ptool.path.extname("a/b/c.txt")) -- .txt
```

注意:

- `ptool.path` のパス処理は完全に字句的です。パスの存在確認や
  シンボリックリンクの解決は行いません。
- どのインターフェースも空文字列引数を受け付けません。渡すとエラーに
  なります。
