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

## ptool.fs.append

> `Unreleased` - 引入。

`ptool.fs.append(path, content)` 将 Lua 字符串按原始字节追加到文件末尾。如果文件不存在，则会创建该文件。

- `path`（string，必填）：文件路径。
- `content`（string，必填）：要追加的内容。

说明：

- `content` 会逐字节写入到文件末尾。
- 内嵌 NUL 字节和非 UTF-8 字节都会被保留。

示例：

```lua
ptool.fs.append("tmp/log.txt", "first line\n")
ptool.fs.append("tmp/log.txt", "second line\n")
```

## ptool.fs.open

> `Unreleased` - 引入。

`ptool.fs.open(path[, mode])` 打开一个本地文件，并返回一个 `File` 对象。

参数：

- `path`（string，必填）：文件路径。
- `mode`（string，选填）：文件模式。默认为 `"r"`。

支持的模式：

- `"r"`：以读取方式打开。
- `"w"`：以写入方式打开，截断已有内容，并在需要时创建文件。
- `"a"`：以追加方式打开，并在需要时创建文件。
- `"r+"`：以读写方式打开，不截断内容。
- `"w+"`：以读写方式打开，截断已有内容，并在需要时创建文件。
- `"a+"`：以读取和追加方式打开，并在需要时创建文件。

说明：

- 模式中可以包含 `b`，例如 `"rb"` 或 `"w+b"`。
- `a` 和 `a+` 的写入始终会落到文件末尾。

示例：

```lua
local file = ptool.fs.open("tmp/log.txt", "a+")
file:write("hello\n")
file:flush()
file:close()
```

## File

> `Unreleased` - 引入。

`File` 表示由 `ptool.fs.open()` 返回的一个已打开的本地文件句柄。

它被实现为一个 Lua userdata。

方法：

- `file:read([n])` -> `string`
- `file:write(content)` -> `nil`
- `file:flush()` -> `nil`
- `file:seek([whence[, offset]])` -> `integer`
- `file:close()` -> `nil`

### read

> `Unreleased` - 引入。

规范 API 名称：`ptool.fs.File:read`。

`file:read([n])` 从当前文件位置读取字节，并将其作为 Lua 字符串返回。

- `n`（integer，选填）：最多读取的字节数。省略时会从当前位置一直读取到 EOF。
- 返回：`string`。

行为说明：

- 在 EOF 处会返回空字符串。
- 读取的是原始字节，因此二进制数据会被原样保留。

示例：

```lua
local file = ptool.fs.open("README.md")
local prefix = file:read(16)
local rest = file:read()
file:close()
```

### write

> `Unreleased` - 引入。

规范 API 名称：`ptool.fs.File:write`。

`file:write(content)` 会在当前文件位置写入一个 Lua 字符串。

- `content`（string，必填）：要写入的字节。

行为说明：

- 会按提供的内容精确写入原始字节。
- 对于 append 模式的句柄，写入总是追加到文件末尾。

### flush

> `Unreleased` - 引入。

规范 API 名称：`ptool.fs.File:flush`。

`file:flush()` 会将缓冲的文件写入刷新到操作系统。

### seek

> `Unreleased` - 引入。

规范 API 名称：`ptool.fs.File:seek`。

`file:seek([whence[, offset]])` 会移动当前文件位置。

- `whence`（string，选填）：`"set"`、`"cur"` 或 `"end"` 之一。默认为 `"cur"`。
- `offset`（integer，选填）：相对于 `whence` 的字节偏移量。默认为 `0`。
- 返回：`integer`。

行为说明：

- 返回新的绝对文件位置。
- `"set"` 要求 `offset` 必须为非负数。

示例：

```lua
local file = ptool.fs.open("tmp/data.bin", "w+")
file:write("abcdef")
file:seek("set", 2)
print(file:read(2)) -- cd
file:close()
```

### close

> `Unreleased` - 引入。

规范 API 名称：`ptool.fs.File:close`。

`file:close()` 会关闭该文件句柄。

行为说明：

- 关闭后，这个文件句柄将不能再继续使用。

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

> `v0.2.0` - 引入。 `v0.5.0` - 新增 `working_dir` 选项。

`ptool.fs.glob(pattern[, options])` 使用 Unix 风格 glob 语法匹配文件系统 路径，并按字典序返回命中的路径字符串数组。

- `pattern`（string，必填）：glob 模式。相对模式会从当前 `ptool` 运行时目录 解析，因此会受到 `ptool.cd(...)` 的影响。
- `options`（table，可选）：glob 选项。支持：
  - `working_dir`（string，可选）：覆盖相对模式解析时使用的基准目录。 相对 `working_dir` 会从当前 `ptool` 运行时目录解析。
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
