# Path API

纯词法路径辅助能力位于 `ptool.path` 和 `p.path` 下。

## ptool.path.join

> `v0.1.0` - 引入。

`ptool.path.join(...segments)` 拼接多个路径片段，并返回规范化路径。

- `segments`（string，至少一个）：路径片段。
- 返回：`string`。

示例：

```lua
print(ptool.path.join("tmp", "a", "..", "b")) -- tmp/b
```

## ptool.path.normalize

> `v0.1.0` - 引入。

`ptool.path.normalize(path)` 对路径做纯词法规范化（处理 `.` 和 `..`）。

- `path`（string，必填）：输入路径。
- 返回：`string`。

示例：

```lua
print(ptool.path.normalize("./a/../b")) -- b
```

## ptool.path.abspath

> `v0.1.0` - 引入。

`ptool.path.abspath(path[, base])` 计算绝对路径。

- `path`（string，必填）：输入路径。
- `base`（string，可选）：基准目录。如果省略，则使用当前进程工作目录。
- 返回：`string`。
- 只接受 1 个或 2 个字符串参数。

示例：

```lua
print(ptool.path.abspath("src"))
print(ptool.path.abspath("lib", "/tmp/demo"))
```

## ptool.path.relpath

> `v0.1.0` - 引入。

`ptool.path.relpath(path[, base])` 计算从 `base` 指向 `path` 的相对路径。

- `path`（string，必填）：目标路径。
- `base`（string，可选）：起始目录。如果省略，则使用当前进程工作目录。
- 返回：`string`。
- 只接受 1 个或 2 个字符串参数。

示例：

```lua
print(ptool.path.relpath("src/main.rs", "/tmp/project"))
```

## ptool.path.isabs

> `v0.1.0` - 引入。

`ptool.path.isabs(path)` 检查路径是否为绝对路径。

- `path`（string，必填）：输入路径。
- 返回：`boolean`。

示例：

```lua
print(ptool.path.isabs("/tmp")) -- true
```

## ptool.path.dirname

> `v0.1.0` - 引入。

`ptool.path.dirname(path)` 返回目录名部分。

- `path`（string，必填）：输入路径。
- 返回：`string`。

示例：

```lua
print(ptool.path.dirname("a/b/c.txt")) -- a/b
```

## ptool.path.basename

> `v0.1.0` - 引入。

`ptool.path.basename(path)` 返回最后一个路径片段（文件名部分）。

- `path`（string，必填）：输入路径。
- 返回：`string`。

示例：

```lua
print(ptool.path.basename("a/b/c.txt")) -- c.txt
```

## ptool.path.extname

> `v0.1.0` - 引入。

`ptool.path.extname(path)` 返回扩展名（包含 `.`）。如果没有扩展名，则返回空字符串。

- `path`（string，必填）：输入路径。
- 返回：`string`。

示例：

```lua
print(ptool.path.extname("a/b/c.txt")) -- .txt
```

说明：

- `ptool.path` 的路径处理完全是词法级别的。它不会检查路径是否存在，也不会解析
  符号链接。
- 所有接口都不接受空字符串参数，传入会抛出错误。
