# Git API

Git 仓库辅助能力位于 `ptool.git` 和 `p.git` 下。

这个模块基于 `git2` / `libgit2`，而不是通过调用 `git` 命令行工具实现。

## ptool.git.open

> `Unreleased` - 引入。

`ptool.git.open(path?)` 直接打开一个仓库，并返回 `Repo` 对象。

参数：

- `path`（string，可选）：仓库路径。省略时使用当前 `ptool` 运行时目录。

行为说明：

- 相对路径会从当前 `ptool` 运行时目录解析，因此会跟随 `ptool.cd(...)`。
- 这个 API 不会向父目录继续搜索仓库。如果你需要仓库发现行为，请使用 `ptool.git.discover(...)`。

示例：

```lua
local repo = ptool.git.open(".")
print(repo:path())
```

## ptool.git.discover

> `Unreleased` - 引入。

`ptool.git.discover(path?)` 从 `path` 开始向上遍历父目录查找仓库，然后返回一个 `Repo` 对象。

参数：

- `path`（string，可选）：起始路径。省略时使用当前 `ptool` 运行时目录。

行为说明：

- 相对路径会从当前 `ptool` 运行时目录解析。
- 当脚本可能在 worktree 的某个子目录中运行时，这个 API 很有用。

示例：

```lua
local repo = ptool.git.discover("src")
print(repo:root())
```

## ptool.git.clone

> `Unreleased` - 引入。

`ptool.git.clone(url, path[, options])` 克隆一个仓库，并返回该克隆仓库对应的 `Repo` 对象。

参数：

- `url`（string，必填）：远端仓库 URL。
- `path`（string，必填）：目标路径。
- `options`（table，可选）：克隆选项。支持的字段：
  - `branch`（string，可选）：克隆完成后要检出的分支名。
  - `bare`（boolean，可选）：是否创建裸仓库。默认值为 `false`。
  - `auth`（table，可选）：远端认证设置。

`auth` 字段：

- `kind`（string，必填）：认证模式。支持的值：
  - `"default"`: 使用 libgit2 的默认凭据。
  - `"ssh_agent"`: 通过本地 SSH agent 进行认证。
  - `"userpass"`: 使用明文用户名和密码。
- `username`（string，可选）：用于 `"ssh_agent"` 的用户名。
- `username`（string，必填）：用于 `"userpass"` 的用户名。
- `password`（string，必填）：用于 `"userpass"` 的密码。

行为说明：

- 相对目标路径会从当前 `ptool` 运行时目录解析。
- 认证选项同样会被 `repo:fetch(...)` 和 `repo:push(...)` 使用。

示例：

```lua
local repo = ptool.git.clone(
  "git@github.com:example/project.git",
  "tmp/project",
  {
    branch = "main",
    auth = {
      kind = "ssh_agent",
    },
  }
)

print(repo:root())
```

## Repo

> `Unreleased` - 引入。

`Repo` 表示一个打开中的 Git 仓库句柄，由 `ptool.git.open()`、`ptool.git.discover()` 或 `ptool.git.clone()` 返回。

它实现为 Lua userdata。

方法：

- `repo:path()` -> `string`
- `repo:root()` -> `string|nil`
- `repo:is_bare()` -> `boolean`
- `repo:head()` -> `table`
- `repo:current_branch()` -> `string|nil`
- `repo:status([options])` -> `table`
- `repo:is_clean([options])` -> `boolean`
- `repo:add(paths[, options])` -> `nil`
- `repo:commit(message[, options])` -> `string`
- `repo:checkout(rev[, options])` -> `nil`
- `repo:switch(branch[, options])` -> `nil`
- `repo:fetch([remote[, options]])` -> `table`
- `repo:push([remote[, refspecs[, options]]])` -> `nil`

### path

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:path`。

`repo:path()` 返回仓库的 Git 目录路径。

- 返回：`string`。

说明：

- 对于非裸仓库，这通常是 `.git` 目录。
- 对于裸仓库，这就是仓库目录本身。

### root

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:root`。

`repo:root()` 返回 worktree 的根目录。

- 返回：`string|nil`。

说明：

- 对于裸仓库，这里会返回 `nil`。

### is_bare

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:is_bare`。

`repo:is_bare()` 用来报告该仓库是否为裸仓库。

- 返回：`boolean`。

### head

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:head`。

`repo:head()` 以 table 形式返回 HEAD 信息，包含：

- `oid`（string|nil）：当前提交的 OID；如果不可用则为 `nil`。
- `shorthand`（string|nil）：HEAD 的简写名称，例如分支名。
- `detached`（boolean）：HEAD 是否处于 detached 状态。
- `unborn`（boolean）：仓库是否还没有初始提交。

示例：

```lua
local head = repo:head()
print(head.oid)
print(head.detached)
```

### current_branch

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:current_branch`。

`repo:current_branch()` 返回当前本地分支名。

- 返回：`string|nil`。

说明：

- 当 HEAD 处于 detached 状态时，这里会返回 `nil`。
- 对于首次提交之前的 unborn branch，这里同样会返回 `nil`。

### status

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:status`。

`repo:status([options])` 汇总仓库状态，并返回一个包含以下内容的 table：

- `root`（string|nil）：worktree 根目录。
- `branch`（string|nil）：当前本地分支名。
- `head`（table）：与 `repo:head()` 返回的 HEAD 信息相同。
- `upstream`（string|nil）：上游分支名；仅在已配置时提供。
- `ahead`（integer）：领先上游的提交数。
- `behind`（integer）：落后上游的提交数。
- `clean`（boolean）：仓库是否没有可见的状态项。
- `entries`（table）：状态条目 table 的数组。

`entries[i]` 包含：

- `path`（string）：相对于仓库的路径。
- `index_status`（string|nil）：index 侧状态。当前支持的值包括 `"new"`、`"modified"`、`"deleted"`、`"renamed"` 和 `"typechange"`。
- `worktree_status`（string|nil）：worktree 侧状态。当前支持的值包括 `"new"`、`"modified"`、`"deleted"`、`"renamed"`、`"typechange"` 和 `"ignored"`。
- `conflicted`（boolean）：该路径是否存在冲突。
- `ignored`（boolean）：该路径是否被忽略。

`options` 字段：

- `include_untracked`（boolean，可选）：是否包含未跟踪文件。默认值为 `true`。
- `include_ignored`（boolean，可选）：是否包含已忽略文件。默认值为 `false`。
- `recurse_untracked_dirs`（boolean，可选）：是否递归进入未跟踪目录。默认值为 `true`。

示例：

```lua
local st = repo:status()
print(st.clean)
print(st.branch)

for _, entry in ipairs(st.entries) do
  print(entry.path, entry.index_status, entry.worktree_status)
end
```

### is_clean

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:is_clean`。

`repo:is_clean([options])` 返回仓库是否干净。

- `options`（table，可选）：与 `repo:status(...)` 接受的选项相同。
- 返回：`boolean`。

### add

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:add`。

`repo:add(paths[, options])` 将一个或多个路径加入 index 暂存区。

参数：

- `paths`（string|string[]，必填）：单个路径或路径数组。
- `options`（table，可选）：add 选项。支持的字段：
  - `update`（boolean，可选）：只更新 index 中已知的路径。默认值为 `false`。

行为说明：

- 路径会按仓库 worktree 的相对路径解释。

示例：

```lua
repo:add("README.md")
repo:add({"src", "Cargo.toml"})
```

### commit

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:commit`。

`repo:commit(message[, options])` 根据当前 index 创建提交，并返回新提交的 OID。

参数：

- `message`（string，必填）：提交信息。
- `options`（table，可选）：commit 选项。支持的字段：
  - `author`（table，可选）：作者签名。
  - `committer`（table，可选）：提交者签名。

签名字段：

- `name` (string, required)
- `email` (string, required)

行为说明：

- 当 `author` 和 `committer` 都省略时，`ptool` 会尝试使用 Git 配置中的仓库身份。
- 如果既没有配置身份，也没有显式提供签名，就会报错。

示例：

```lua
local oid = repo:commit("Release v0.7.0", {
  author = {
    name = "Release Bot",
    email = "bot@example.com",
  },
})

print(oid)
```

### checkout

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:checkout`。

`repo:checkout(rev[, options])` 检出一个修订版本。

参数：

- `rev`（string，必填）：修订表达式，例如分支名、标签名或提交 OID。
- `options`（table，可选）：checkout 选项。支持的字段：
  - `force`（boolean，可选）：是否强制检出。默认值为 `false`。

行为说明：

- 当 `rev` 无法解析到具名引用时，这个操作可能会让 HEAD 进入 detached 状态。

### switch

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:switch`。

`repo:switch(branch[, options])` 将 HEAD 切换到一个本地分支。

参数：

- `branch`（string，必填）：本地分支名。
- `options`（table，可选）：switch 选项。支持的字段：
  - `create`（boolean，可选）：是否先创建分支。默认值为 `false`。
  - `force`（boolean，可选）：是否强制检出。默认值为 `false`。
  - `start_point`（string，可选）：当 `create = true` 时作为建分支起点的修订。默认值为 `HEAD`。

示例：

```lua
repo:switch("release")
repo:switch("release-next", {
  create = true,
  start_point = "origin/main",
})
```

### fetch

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:fetch`。

`repo:fetch([remote[, options]])` 从远端抓取，并返回传输统计信息。

参数：

- `remote`（string，可选）：远端名称。默认值为 `"origin"`。
- `options`（table，可选）：fetch 选项。支持的字段：
  - `refspecs`（string|string[]，可选）：单个 refspec 或 refspec 数组。
  - `auth`（table，可选）：远端认证设置。结构与 `ptool.git.clone(...)` 相同。

返回：

- `received_objects` (integer)
- `indexed_objects` (integer)
- `local_objects` (integer)
- `total_objects` (integer)
- `received_bytes` (integer)

示例：

```lua
local stats = repo:fetch("origin", {
  auth = {
    kind = "ssh_agent",
  },
})

print(stats.received_objects, stats.received_bytes)
```

### push

> `Unreleased` - 引入。

规范 API 名称：`ptool.git.Repo:push`。

`repo:push([remote[, refspecs[, options]]])` 将引用推送到远端。

参数：

- `remote`（string，可选）：远端名称。默认值为 `"origin"`。
- `refspecs`（string|string[]，可选）：单个 refspec 或 refspec 数组。
- `options`（table，可选）：push 选项。支持的字段：
  - `auth`（table，可选）：远端认证设置。结构与 `ptool.git.clone(...)` 相同。

行为说明：

- 省略 `refspecs` 时，`ptool` 会尝试把当前本地分支推送到远端同名分支。
- 当 HEAD 处于 detached 状态时省略 `refspecs` 会报错。

示例：

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```
