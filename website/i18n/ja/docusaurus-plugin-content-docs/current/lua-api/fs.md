# ファイルシステム API

ファイルシステムヘルパーは `ptool.fs` と `p.fs` にあります。

## ptool.fs.read

> `v0.1.0` - Introduced.

`ptool.fs.read(path)` はファイルを生のバイト列として読み取り、Lua 文字列を 返します。

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

`ptool.fs.write(path, content)` は Lua 文字列を生のバイト列としてファイルへ 書き込み、既存の内容を上書きします。

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

## ptool.fs.append

> `v0.8.0` - 導入されました。

`ptool.fs.append(path, content)` は Lua 文字列を生のバイト列としてファイルの末尾に追加します。ファイルが存在しない場合は作成されます。

- `path` (string, 必須): ファイルパス。
- `content` (string, 必須): 追加する内容。

注意:

- `content` はファイルの末尾に 1 バイトずつそのまま書き込まれます。
- 埋め込み NUL バイトや非 UTF-8 バイトも保持されます。

例:

```lua
ptool.fs.append("tmp/log.txt", "first line\n")
ptool.fs.append("tmp/log.txt", "second line\n")
```

## ptool.fs.open

> `v0.8.0` - 導入されました。

`ptool.fs.open(path[, mode])` はローカルファイルを開き、`File` オブジェクトを返します。

引数:

- `path` (string, 必須): ファイルパス。
- `mode` (string, 任意): ファイルモード。デフォルトは `"r"` です。

サポートされるモード:

- `"r"`: 読み込み用に開きます。
- `"w"`: 書き込み用に開き、既存内容を切り詰め、必要ならファイルを作成します。
- `"a"`: 追記用に開き、必要ならファイルを作成します。
- `"r+"`: 切り詰めずに読み書き用で開きます。
- `"w+"`: 読み書き用に開き、既存内容を切り詰め、必要ならファイルを作成します。
- `"a+"`: 読み込みと追記用に開き、必要ならファイルを作成します。

注意:

- モードには `"rb"` や `"w+b"` のように `b` を含められます。
- `a` と `a+` の書き込みは常にファイル末尾に行われます。

例:

```lua
local file = ptool.fs.open("tmp/log.txt", "a+")
file:write("hello\n")
file:flush()
file:close()
```

## File

> `v0.8.0` - 導入されました。

`File` は `ptool.fs.open()` が返す、開かれたローカルファイルハンドルです。

Lua userdata として実装されています。

メソッド:

- `file:read([n])` -> `string`
- `file:write(content)` -> `nil`
- `file:flush()` -> `nil`
- `file:seek([whence[, offset]])` -> `integer`
- `file:close()` -> `nil`

### read

> `v0.8.0` - 導入されました。

正規 API 名: `ptool.fs.File:read`。

`file:read([n])` は現在のファイル位置からバイト列を読み取り、Lua 文字列として返します。

- `n` (integer, 任意): 読み取る最大バイト数。省略時は現在位置から EOF まで読み取ります。
- 戻り値: `string`。

動作:

- EOF では空文字列を返します。
- 生のバイト列を読むため、バイナリデータはそのまま保持されます。

例:

```lua
local file = ptool.fs.open("README.md")
local prefix = file:read(16)
local rest = file:read()
file:close()
```

### write

> `v0.8.0` - 導入されました。

正規 API 名: `ptool.fs.File:write`。

`file:write(content)` は現在のファイル位置に Lua 文字列を書き込みます。

- `content` (string, 必須): 書き込むバイト列。

動作:

- 指定したとおりの生バイト列を書き込みます。
- append モードのハンドルでは、書き込みはファイル末尾に追加されます。

### flush

> `v0.8.0` - 導入されました。

正規 API 名: `ptool.fs.File:flush`。

`file:flush()` はバッファされたファイル書き込みを OS にフラッシュします。

### seek

> `v0.8.0` - 導入されました。

正規 API 名: `ptool.fs.File:seek`。

`file:seek([whence[, offset]])` は現在のファイル位置を移動します。

- `whence` (string, 任意): `"set"`、`"cur"`、`"end"` のいずれか。デフォルトは `"cur"` です。
- `offset` (integer, 任意): `whence` からの相対バイトオフセット。デフォルトは `0` です。
- 戻り値: `integer`。

動作:

- 新しい絶対ファイル位置を返します。
- `"set"` では `offset` に 0 以上の値が必要です。

例:

```lua
local file = ptool.fs.open("tmp/data.bin", "w+")
file:write("abcdef")
file:seek("set", 2)
print(file:read(2)) -- cd
file:close()
```

### close

> `v0.8.0` - 導入されました。

正規 API 名: `ptool.fs.File:close`。

`file:close()` はファイルハンドルを閉じます。

動作:

- 閉じた後は、そのファイルハンドルは使えなくなります。

## ptool.fs.mkdir

> `v0.1.0` - Introduced.

`ptool.fs.mkdir(path)` はディレクトリを作成します。親ディレクトリが存在 しない場合は再帰的に作成されます。

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

## ptool.fs.is_file

> `v0.6.0` - 導入されました。

`ptool.fs.is_file(path)` は、パスが存在し通常ファイルかどうかを確認します。

- `path` (string, 必須): 確認するパス。
- 戻り値: `boolean`。

例:

```lua
if ptool.fs.is_file("tmp/hello.txt") then
  print("file")
end
```

## ptool.fs.is_dir

> `v0.6.0` - 導入されました。

`ptool.fs.is_dir(path)` は、パスが存在しディレクトリかどうかを確認します。

- `path` (string, 必須): 確認するパス。
- 戻り値: `boolean`。

例:

```lua
if ptool.fs.is_dir("tmp") then
  print("dir")
end
```

## ptool.fs.remove

> `v0.6.0` - 導入されました。

`ptool.fs.remove(path[, options])` は、ファイル、シンボリックリンク、 ディレクトリを削除します。

- `path` (string, 必須): 削除するパス。
- `options` (table, 任意): 削除オプション。サポートされるフィールド:
  - `recursive` (boolean, 任意): ディレクトリを再帰的に削除するかどうか。 デフォルトは `false`。
  - `missing_ok` (boolean, 任意): パスが存在しない場合に無視するかどうか。 デフォルトは `false`。

動作:

- ファイルとシンボリックリンクは `recursive` なしで削除できます。
- 空でないディレクトリは `recursive = true` が必要です。
- 不明なオプション名や不正な値型はエラーになります。

例:

```lua
ptool.fs.remove("tmp/hello.txt")
ptool.fs.remove("tmp/cache", { recursive = true })
ptool.fs.remove("tmp/missing.txt", { missing_ok = true })
```

## ptool.fs.glob

> `v0.2.0` - Introduced. `v0.5.0` - `working_dir` オプションを追加。

`ptool.fs.glob(pattern[, options])` は Unix 風 glob 構文でファイルシステムパスを 照合し、辞書順に並んだ一致パスの文字列配列を返します。

- `pattern` (string, 必須): glob パターン。相対パターンは現在の `ptool` ランタイムディレクトリから解決されるため、 `ptool.cd(...)` に従います。
- `options` (table, 任意): glob オプション。サポートされるフィールド:
  - `working_dir` (string, 任意): 相対パターンの解決に使う基準 ディレクトリを上書きします。相対 `working_dir` は現在の `ptool` ランタイムディレクトリから解決されます。
- 戻り値: `string[]`。
- 隠しファイルや隠しディレクトリは、対応するパターン要素が明示的に `.` で始まる場合にだけ一致します。

例:

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
local lua_scripts = ptool.fs.glob("**/*.lua", {
  working_dir = "../scripts",
})
```
