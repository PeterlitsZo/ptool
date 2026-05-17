# Zip API

圧縮ヘルパーは `ptool.zip` と `p.zip` にあります。

`ptool.zip` は生の Lua 文字列を扱うため、テキストとバイナリの両方に利用できます。

対応する形式名:

- `gzip` と `gz`
- `zlib`
- `deflate`
- `bzip2` と `bz2`
- `xz`
- `zstd`、`zst`、`zstandard`

## ptool.zip.compress

> `v0.8.0` - Introduced.

`ptool.zip.compress(format, input)` は指定した形式で Lua 文字列を圧縮します。

- `format` (string, 必須): 圧縮形式名。
- `input` (string, 必須): 入力の Lua 文字列。圧縮では文字列の生バイト列をそのまま使います。
- 戻り値: `string` (圧縮済みバイト列を表す Lua 文字列)。

エラー時の挙動:

- `format` が対応していない形式名ならエラーになります。
- `input` が文字列でない場合はエラーになります。
- 指定した形式のエンコーダーが失敗した場合はエラーになります。

例:

```lua
local payload = p.fs.read("report.txt")
local compressed = p.zip.compress("gzip", payload)

p.fs.write("report.txt.gz", compressed)
```

## ptool.zip.decompress

> `v0.8.0` - Introduced.

`ptool.zip.decompress(format, input)` は指定した形式で Lua 文字列を展開します。

- `format` (string, 必須): 圧縮形式名。
- `input` (string, 必須): 圧縮された Lua 文字列。
- 戻り値: `string` (展開後のバイト列を表す Lua 文字列)。

エラー時の挙動:

- `format` が対応していない形式名ならエラーになります。
- `input` が文字列でない場合はエラーになります。
- `input` が指定した形式に対して正しいデータでない場合はエラーになります。

例:

```lua
local compressed = p.fs.read("report.txt.gz")
local plain = p.zip.decompress("gzip", compressed)

print(plain)
```

補足:

- `ptool.zip` はファイル名から形式を推測しません。形式は明示的に指定してください。
- `ptool.zip` は単一のバイト列を対象としており、`.zip` コンテナ内のファイル一覧のような ZIP アーカイブエントリー API は提供しません。
