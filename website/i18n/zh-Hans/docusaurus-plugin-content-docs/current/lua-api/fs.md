# Filesystem API

文件系统辅助能力位于 `ptool.fs` 和 `p.fs` 下。

## ptool.fs.read

> `v0.1.0` - 引入。

`ptool.fs.read(path)` 读取 UTF-8 文本文件并返回字符串。

- `path`（string，必填）：文件路径。
- 返回：`string`。

示例：

```lua
local content = ptool.fs.read("README.md")
print(content)
```

## ptool.fs.write

> `v0.1.0` - 引入。

`ptool.fs.write(path, content)` 向文件写入字符串，并覆盖已有内容。

- `path`（string，必填）：文件路径。
- `content`（string，必填）：要写入的内容。

示例：

```lua
ptool.fs.write("tmp/hello.txt", "hello\n")
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

## ptool.fs.glob

> `v0.2.0` - 引入。

`ptool.fs.glob(pattern)` 使用 Unix 风格 glob 语法匹配文件系统路径，并按字典序
返回命中的路径字符串数组。

- `pattern`（string，必填）：glob 模式。相对模式会从当前 `ptool` 运行时目录
  解析，因此会受到 `ptool.cd(...)` 的影响。
- 返回：`string[]`。
- 隐藏文件和目录只有在对应模式片段显式以 `.` 开头时才会被匹配。

示例：

```lua
ptool.cd("src")

local rust_files = ptool.fs.glob("**/*.rs")
local hidden = ptool.fs.glob("**/.secret/*.txt")
```
