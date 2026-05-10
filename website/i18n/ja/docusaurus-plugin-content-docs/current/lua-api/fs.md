# ファイルシステム API

ファイルシステムヘルパーは `ptool.fs` と `p.fs` にあります。

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` はファイルを生のバイト列として読み取り、Lua 文字列を
返します。

- `path` (string, 必須): ファイルパス。
- 戻り値: `string`。

注意:

- 返される Lua 文字列には、ディスク上のファイルバイトがそのまま入ります。
- テキストファイルは従来どおり扱えますが、バイナリファイルも読み取れます。

例:

```lua
local content = ptool.fs.read("README.md")
print(content)

local png = ptool.fs.read("logo.png")
print(#png)
```

## ptool.fs.write

> `v0.1.0` - Introduced.

`ptool.fs.write(path, content)` は Lua 文字列を生のバイト列としてファイルへ
書き込み、既存の内容を上書きします。

- `path` (string, 必須): ファイルパス。
- `content` (string, 必須): 書き込む内容。

注意:

- `content` は 1 バイトずつそのまま書き込まれます。
- 埋め込み NUL バイトや非 UTF-8 バイトも保持されます。

例:

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
ptool.fs.write("tmp/blob.bin", "\x00\xffABC")
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
> `v0.5.0` - `working_dir` オプションを追加。

`ptool.fs.glob(pattern[, options])` は Unix 風 glob 構文でファイルシステムパスを
照合し、辞書順に並んだ一致パスの文字列配列を返します。

- `pattern` (string, 必須): glob パターン。相対パターンは現在の
  `ptool` ランタイムディレクトリから解決されるため、
  `ptool.cd(...)` に従います。
- `options` (table, 任意): glob オプション。サポートされるフィールド:
  - `working_dir` (string, 任意): 相対パターンの解決に使う基準
    ディレクトリを上書きします。相対 `working_dir` は現在の
    `ptool` ランタイムディレクトリから解決されます。
- 戻り値: `string[]`。
- 隠しファイルや隠しディレクトリは、対応するパターン要素が明示的に
  `.` で始まる場合にだけ一致します。

例:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
local lua_scripts = ptool.fs.glob("**/*.lua", {
  working_dir = "../scripts",
})
```
