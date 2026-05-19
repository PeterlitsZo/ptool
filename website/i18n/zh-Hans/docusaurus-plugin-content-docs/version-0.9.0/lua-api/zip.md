# Zip API

压缩辅助能力位于 `ptool.zip` 和 `p.zip` 下。

`ptool.zip` 直接处理原始 Lua 字符串，因此既可用于文本，也可用于二进制载荷。

支持的格式名：

- `gzip` 和 `gz`
- `zlib`
- `deflate`
- `bzip2` 和 `bz2`
- `xz`
- `zstd`、`zst` 和 `zstandard`

## ptool.zip.compress

> `v0.8.0` - 引入。

`ptool.zip.compress(format, input)` 使用指定格式压缩 Lua 字符串。

- `format`（string，必填）：压缩格式名。
- `input`（string，必填）：输入的 Lua 字符串。压缩时会原样使用字符串的原始字节。
- 返回：`string`（以 Lua 字符串表示的压缩后字节）。

错误行为：

- 如果 `format` 不是受支持的格式名，会抛出错误。
- 如果 `input` 不是字符串，会抛出错误。
- 如果请求格式对应的编码器失败，会抛出错误。

示例：

```lua
local payload = p.fs.read("report.txt")
local compressed = p.zip.compress("gzip", payload)

p.fs.write("report.txt.gz", compressed)
```

## ptool.zip.decompress

> `v0.8.0` - 引入。

`ptool.zip.decompress(format, input)` 使用指定格式解压 Lua 字符串。

- `format`（string，必填）：压缩格式名。
- `input`（string，必填）：压缩后的 Lua 字符串。
- 返回：`string`（以 Lua 字符串表示的解压后字节）。

错误行为：

- 如果 `format` 不是受支持的格式名，会抛出错误。
- 如果 `input` 不是字符串，会抛出错误。
- 如果 `input` 不是该格式的有效数据，会抛出错误。

示例：

```lua
local compressed = p.fs.read("report.txt.gz")
local plain = p.zip.decompress("gzip", compressed)

print(plain)
```

说明：

- `ptool.zip` 不会从文件名推断格式。请显式传入格式。
- `ptool.zip` 只处理单个字节串，不提供 ZIP 档案条目 API，例如列出 `.zip` 容器里的文件。
