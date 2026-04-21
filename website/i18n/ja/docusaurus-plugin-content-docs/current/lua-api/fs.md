# ファイルシステム API

ファイルシステムヘルパーは `ptool.fs` と `p.fs` にあります。

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` は UTF-8 テキストファイルを読み取り、文字列を
返します。

- `path` (string, 必須): ファイルパス。
- 戻り値: `string`。

例:

```lua
local content = ptool.fs.read("README.md")
print(content)
```

## ptool.fs.write

> `v0.1.0` - Introduced.

`ptool.fs.write(path, content)` は文字列をファイルへ書き込み、既存の内容を
上書きします。

- `path` (string, 必須): ファイルパス。
- `content` (string, 必須): 書き込む内容。

例:

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
```

## ptool.fs.mkdir

> `v0.1.0` - Introduced.

`ptool.fs.mkdir(path)` はディレクトリを作成します。親ディレクトリが存在
しない場合は再帰的に作成されます。

- `path` (string, 必須): ディレクトリパス。

例:

```lua
ptool.fs.mkdir("tmp/a/b")
```

## ptool.fs.exists

> `v0.1.0` - Introduced.

`ptool.fs.exists(path)` はパスが存在するか確認します。

- `path` (string, 必須): ファイルまたはディレクトリのパス。
- 戻り値: `boolean`。

例:

```lua
if ptool.fs.exists("tmp/hello.txt") then
  print("exists")
end
```

## ptool.fs.glob

> `v0.2.0` - Introduced.

`ptool.fs.glob(pattern)` は Unix 風 glob 構文でファイルシステムパスを
照合し、辞書順に並んだ一致パスの文字列配列を返します。

- `pattern` (string, 必須): glob パターン。相対パターンは現在の
  `ptool` ランタイムディレクトリから解決されるため、
  `ptool.cd(...)` に従います。
- 戻り値: `string[]`。
- 隠しファイルや隠しディレクトリは、対応するパターン要素が明示的に
  `.` で始まる場合にだけ一致します。

例:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
```
