# Filesystem API

文件系统辅助能力位于 `ptool.fs` 和 `p.fs` 下。

## ptool.fs.read

> `v0.1.0` - 引入。

`ptool.fs.read(path)` 按原始字节读取文件，并返回一个 Lua 字符串。

- `path`（string，必填）：文件路径。
- 返回：`string`。

说明：

- 返回的 Lua 字符串会精确包含磁盘上的文件字节。
- 文本文件依然可以像以前一样读取，同时也支持二进制文件。

示例：

```lua
local content = ptool.fs.read("README.md")
print(content)

local png = ptool.fs.read("logo.png")
print(#png)
```

## ptool.fs.write

> `v0.1.0` - 引入。

`ptool.fs.write(path, content)` 将 Lua 字符串按原始字节写入文件，并覆盖已有内容。

- `path`（string，必填）：文件路径。
- `content`（string，必填）：要写入的内容。

说明：

- `content` 会逐字节写入。
- 内嵌 NUL 字节和非 UTF-8 字节都会被保留。

示例：

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
ptool.fs.write("tmp/blob.bin", "\x00\xffABC")
```

## ptool.fs.mkdir

> `v0.1.0` - 引入。

`ptool.fs.mkdir(path)` 创建目录。如果父目录不存在，会递归创建。

- `path`（string，必填）：目录路径。

示例：

```lua
ptool.fs.mkdir("tmp/a/b")
```

## ptool.fs.exists

> `v0.1.0` - 引入。

`ptool.fs.exists(path)` 检查路径是否存在。

- `path`（string，必填）：文件或目录路径。
- 返回：`boolean`。

示例：

```lua
if ptool.fs.exists("tmp/hello.txt") then
  print("exists")
end
```

## ptool.fs.is_file

> `Unreleased` - 引入。

`ptool.fs.is_file(path)` 检查路径是否存在且是否为普通文件。

- `path`（string，必填）：要检查的路径。
- 返回：`boolean`。

示例：

```lua
if ptool.fs.is_file("tmp/hello.txt") then
  print("file")
end
```

## ptool.fs.is_dir

> `Unreleased` - 引入。

`ptool.fs.is_dir(path)` 检查路径是否存在且是否为目录。

- `path`（string，必填）：要检查的路径。
- 返回：`boolean`。

示例：

```lua
if ptool.fs.is_dir("tmp") then
  print("dir")
end
```

## ptool.fs.remove

> `Unreleased` - 引入。

`ptool.fs.remove(path[, options])` 删除文件、符号链接或目录。

- `path`（string，必填）：要删除的路径。
- `options`（table，可选）：删除选项。支持：
  - `recursive`（boolean，可选）：是否递归删除目录。默认值为 `false`。
  - `missing_ok`（boolean，可选）：路径不存在时是否忽略。默认值为 `false`。

行为说明：

- 文件和符号链接无需 `recursive` 即可删除。
- 目录在非空时需要设置 `recursive = true`。
- 未知选项名或非法选项值类型都会抛出错误。

示例：

```lua
ptool.fs.remove("tmp/hello.txt")
ptool.fs.remove("tmp/cache", { recursive = true })
ptool.fs.remove("tmp/missing.txt", { missing_ok = true })
```

## ptool.fs.glob

> `v0.2.0` - 引入。
> `v0.5.0` - 新增 `working_dir` 选项。

`ptool.fs.glob(pattern[, options])` 使用 Unix 风格 glob 语法匹配文件系统
路径，并按字典序返回命中的路径字符串数组。

- `pattern`（string，必填）：glob 模式。相对模式会从当前 `ptool` 运行时目录
  解析，因此会受到 `ptool.cd(...)` 的影响。
- `options`（table，可选）：glob 选项。支持：
  - `working_dir`（string，可选）：覆盖相对模式解析时使用的基准目录。
    相对 `working_dir` 会从当前 `ptool` 运行时目录解析。
- 返回：`string[]`。
- 隐藏文件和目录只有在对应模式片段显式以 `.` 开头时才会被匹配。

示例：

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
local lua_scripts = ptool.fs.glob("**/*.lua", {
  working_dir = "../scripts",
})
```
