# Git API

Git 仓库辅助能力位于 `ptool.git` 和 `p.git` 下。

这个模块基于 `git2` / `libgit2`，而不是通过调用 `git` 命令行工具实现。

修改仓库的操作可在选项表中传入 `confirm = true`，以便在执行前请求用户确认。破坏性操作还会使用独立的安全标志，例如 `force = true`；确认操作不能替代必需的安全标志。

未知选项名会直接报错，而不会被静默忽略。

## ptool.git.init

> `Unreleased` - 引入。

`ptool.git.init(path?, options?)` 初始化仓库并返回一个 `Repo` 对象。

选项：

- `bare`（boolean）：创建 bare 仓库。默认为 `false`。
- `initial_head`（string）：初始分支名，例如 `"main"`。
- `confirm`（boolean）：创建仓库前请求确认。

相对路径会从当前 `ptool` 运行时目录解析。

```lua
local repo = p.git.init("tmp/project", {
  initial_head = "main",
})
```

## ptool.git.open

> `v0.6.0` - 引入。

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

> `v0.6.0` - 引入。

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

> `v0.6.0` - 引入。

`ptool.git.clone(url, path[, options])` 克隆一个仓库，并返回该克隆仓库对应的 `Repo` 对象。

参数：

- `url`（string，必填）：远端仓库 URL。
- `path`（string，必填）：目标路径。
- `options`（table，可选）：克隆选项。支持的字段：
  - `branch`（string，可选）：克隆完成后要检出的分支名。
  - `bare`（boolean，可选）：是否创建裸仓库。默认值为 `false`。
  - `depth`（integer，可选）：浅克隆的正整数深度。
  - `checkout`（boolean，可选）：是否检出所选分支。默认为 `true`。
  - `remote`（string，可选）：分配给克隆远端的名称。默认为 `"origin"`。
  - `tags`（string，可选）：`"auto"`、`"all"` 或 `"none"`。默认为 `"auto"`。
  - `confirm`（boolean，可选）：克隆前是否请求确认。默认值为 `false`。
  - `auth`（table，可选）：远端认证设置。

`auth` 字段：

- `kind`（string，必填）：认证模式。支持的值：
  - `"default"`: 使用 libgit2 的默认凭据。
  - `"ssh_agent"`: 通过本地 SSH agent 进行认证。
  - `"ssh_key"`：使用 SSH 私钥进行认证。
  - `"userpass"`: 使用明文用户名和密码。
  - `"credential_helper"`：查询已配置的 Git 凭据助手。
- `username`（string）：对 `"ssh_agent"` 和 `"credential_helper"` 可选；对 `"ssh_key"` 和 `"userpass"` 必填。
- `private_key`（string，`"ssh_key"` 必填）：私钥路径。相对路径从当前 `ptool` 运行时目录解析。
- `public_key`（string，可选）：公钥路径。
- `passphrase`（string，可选）：SSH 私钥口令。
- `password`（string，`"userpass"` 必填）：密码。

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

> `v0.6.0` - 引入。

`Repo` 表示由 `ptool.git.init()`、`ptool.git.open()`、`ptool.git.discover()` 或 `ptool.git.clone()` 返回的已打开 Git 仓库句柄。

它实现为 Lua userdata。

以下按组介绍各方法。已有的仓库、状态、暂存、提交、checkout、switch、fetch 和 push 方法保持向后兼容。后续章节描述的工作流 API 当前尚未发布。

### path

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:path`。

`repo:path()` 返回仓库的 Git 目录路径。

- 返回：`string`。

说明：

- 对于非裸仓库，这通常是 `.git` 目录。
- 对于裸仓库，这就是仓库目录本身。

### root

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:root`。

`repo:root()` 返回 worktree 的根目录。

- 返回：`string|nil`。

说明：

- 对于裸仓库，这里会返回 `nil`。

### is_bare

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:is_bare`。

`repo:is_bare()` 用来报告该仓库是否为裸仓库。

- 返回：`boolean`。

### head

> `v0.6.0` - 引入。

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

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:current_branch`。

`repo:current_branch()` 返回当前本地分支名。

- 返回：`string|nil`。

说明：

- 当 HEAD 处于 detached 状态时，这里会返回 `nil`。
- 对于首次提交之前的 unborn branch，这里同样会返回 `nil`。

### status

> `v0.6.0` - 引入。

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

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:is_clean`。

`repo:is_clean([options])` 返回仓库是否干净。

- `options`（table，可选）：与 `repo:status(...)` 接受的选项相同。
- 返回：`boolean`。

### add

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:add`。

`repo:add(paths[, options])` 将一个或多个路径加入 index 暂存区。

参数：

- `paths`（string|string[]，必填）：单个路径或路径数组。
- `options`（table，可选）：add 选项。支持的字段：
  - `update`（boolean，可选）：只更新 index 中已知的路径。默认值为 `false`。
  - `confirm`（boolean，可选）：暂存路径前是否请求确认。默认值为 `false`。

行为说明：

- 路径会按仓库 worktree 的相对路径解释。

示例：

```lua
repo:add("README.md")
repo:add({"src", "Cargo.toml"})
```

### commit

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:commit`。

`repo:commit(message[, options])` 根据当前 index 创建提交，并返回新提交的 OID。

参数：

- `message`（string，必填）：提交信息。
- `options`（table，可选）：commit 选项。支持的字段：
  - `author`（table，可选）：作者签名。
  - `committer`（table，可选）：提交者签名。
  - `amend`（boolean，可选）：替换当前 HEAD 提交。默认为 `false`。
  - `allow_empty`（boolean，可选）：允许树未发生变化的提交。为保持向后兼容，默认为 `true`。
  - `confirm`（boolean，可选）：创建提交前是否请求确认。默认值为 `false`。

签名字段：

- `name` (string, required)
- `email` (string, required)
- `time_seconds`（integer，可选）：Unix 时间戳。
- `offset_minutes`（integer，可选）：相对于 UTC 的时区偏移分钟数。

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

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:checkout`。

`repo:checkout(rev[, options])` 检出一个修订版本。

参数：

- `rev`（string，必填）：修订表达式，例如分支名、标签名或提交 OID。
- `options`（table，可选）：checkout 选项。支持的字段：
  - `force`（boolean，可选）：是否强制检出。默认值为 `false`。
  - `confirm`（boolean，可选）：检出该修订前是否请求确认。默认值为 `false`。

行为说明：

- 当 `rev` 无法解析到具名引用时，这个操作可能会让 HEAD 进入 detached 状态。

### switch

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:switch`。

`repo:switch(branch[, options])` 将 HEAD 切换到一个本地分支。

参数：

- `branch`（string，必填）：本地分支名。
- `options`（table，可选）：switch 选项。支持的字段：
  - `create`（boolean，可选）：是否先创建分支。默认值为 `false`。
  - `force`（boolean，可选）：是否强制检出。默认值为 `false`。
  - `start_point`（string，可选）：当 `create = true` 时作为建分支起点的修订。默认值为 `HEAD`。
  - `track`（string，可选）：新分支的 upstream 引用。
  - `orphan`（boolean，可选）：创建孤儿分支。
  - `confirm`（boolean，可选）：切换分支前是否请求确认。默认值为 `false`。

示例：

```lua
repo:switch("release")
repo:switch("release-next", {
  create = true,
  start_point = "origin/main",
})
```

### fetch

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:fetch`。

`repo:fetch([remote[, options]])` 从远端抓取，并返回传输统计信息。

参数：

- `remote`（string，可选）：远端名称。默认值为 `"origin"`。
- `options`（table，可选）：fetch 选项。支持的字段：
  - `refspecs`（string|string[]，可选）：单个 refspec 或 refspec 数组。
  - `depth`（integer，可选）：浅 fetch 的正整数深度。
  - `prune`（boolean，可选）：删除过期的远端跟踪引用。
  - `tags`（string，可选）：`"auto"`、`"all"` 或 `"none"`。
  - `update_fetchhead`（boolean，可选）：更新 `FETCH_HEAD`。默认为 `true`。
  - `confirm`（boolean，可选）：抓取前是否请求确认。默认值为 `false`。
  - `auth`（table，可选）：远端认证设置。结构与 `ptool.git.clone(...)` 相同。

返回：

- `received_objects` (integer)
- `indexed_objects` (integer)
- `local_objects` (integer)
- `total_objects` (integer)
- `received_bytes` (integer)
- `updated_refs`（string[]）

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

> `v0.6.0` - 引入。

规范 API 名称：`ptool.git.Repo:push`。

`repo:push([remote[, refspecs[, options]]])` 将引用推送到远端。

参数：

- `remote`（string，可选）：远端名称。默认值为 `"origin"`。
- `refspecs`（string|string[]，可选）：单个 refspec 或 refspec 数组。
- `options`（table，可选）：push 选项。支持的字段：
  - `force`（boolean，可选）：强制推送每个 refspec。默认为 `false`。
  - `set_upstream`（boolean，可选）：将已推送的当前分支设置为跟踪目标远端分支。
  - `confirm`（boolean，可选）：推送前是否请求确认。默认值为 `false`。
  - `auth`（table，可选）：远端认证设置。结构与 `ptool.git.clone(...)` 相同。

行为说明：

- 省略 `refspecs` 时，`ptool` 会尝试把当前本地分支推送到远端同名分支。
- 当 HEAD 处于 detached 状态时省略 `refspecs` 会报错。
- 返回表包含 `ok`、`refspecs` 和 `rejected`。每个被拒绝的条目包含 `reference` 和 `message`。

示例：

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```


## Git 工作流 API

> `Unreleased` - 引入。

本节 API 覆盖仓库维护、发布、历史检查、协作和 CI 自动化。凡参数标记为 `string|string[]`，均可传入字符串或紧密字符串数组。

### 共享结果表

`commit_info()` 和 `log()` 返回的提交信息包含 `oid`、`message`、`summary`、`author`、`committer` 和 `parent_oids`。签名表包含 `name`、`email`、`time_seconds` 和 `offset_minutes`。

集成方法返回：

```lua
{
  outcome = "up_to_date" | "fast_forward" | "merged" | "conflicted",
  oid = "..." | nil,
  conflicts = {
    { path = "file", ancestor = true, ours = true, theirs = true },
  },
}
```

Rebase 结果使用 `"rebased"` 或 `"conflicted"` 状态，并额外包含 `current` 和 `total`。因冲突停止的操作可通过对应 API 继续或中止。

### 仓库与状态

```lua
repo:path() -> string
repo:root() -> string|nil
repo:is_bare() -> boolean
repo:head() -> GitHeadInfo
repo:current_branch() -> string|nil
repo:status(options?) -> GitStatusSummary
repo:is_clean(options?) -> boolean
```

`status()` 和 `is_clean()` 的选项包括 `include_untracked`（默认 `true`）、`include_ignored`、`recurse_untracked_dirs`（默认 `true`）和 `paths`。`GitStatusSummary` 包含 `root`、`branch`、`head`、`upstream`、`ahead`、`behind`、`clean` 和 `entries`。

### 历史与 diff

```lua
repo:resolve(rev) -> GitObjectInfo
repo:commit_info(rev?) -> GitCommitInfo
repo:log(options?) -> GitCommitInfo[]
repo:diff(options?) -> GitDiff
repo:describe(options?) -> string|nil
```

- `resolve()` 返回 `oid`、`kind` 和 `shorthand`。
- `commit_info()` 默认为 `HEAD`。
- `log()` 接受 `rev`、`max_count`（默认 `100`）、`skip`、`first_parent`、`reverse` 和 `paths`。
- `diff()` 接受 `from`、`to`、`cached`、`paths`、`context_lines`（默认 `3`）和 `find_renames`（默认 `true`）。未提供 `from` 或 `to` 时，它会比较适当的 worktree、index 或 HEAD 状态。
- Diff 结果包含 `patch`、`files_changed`、`insertions`、`deletions` 和 `deltas`。每个 delta 包含 `status`、`old_path`、`new_path` 和 `binary`。
- `describe()` 接受 `rev`、`pattern`、`always`、`abbrev`（默认 `7`）和 `dirty_suffix`。

```lua
local commits = repo:log({
  rev = "HEAD",
  max_count = 20,
  first_parent = true,
  paths = {"crates/ptool-engine"},
})
local changes = repo:diff({ from = "v0.10.0", to = "HEAD" })
```

### 分支

```lua
repo:branches(options?) -> GitBranchInfo[]
repo:branch_create(name, options?) -> GitBranchInfo
repo:branch_delete(name, options?) -> nil
repo:branch_rename(old_name, new_name, options?) -> GitBranchInfo
repo:branch_set_upstream(name, upstream_or_nil, options?) -> nil
```

- `branches()` 接受 `kind = "local" | "remote" | "all"`；默认为 `"local"`。
- 分支信息包含 `name`、`kind`、`oid`、`head`、`upstream`、`ahead` 和 `behind`。
- `branch_create()` 接受 `start_point`、`force`、`checkout`、`upstream` 和 `confirm`。默认起点为 `HEAD`。
- `branch_delete()` 接受 `force` 和 `confirm`。不能删除当前分支；删除未合并分支需要 `force = true`。
- `branch_rename()` 接受 `force` 和 `confirm`。
- 向 `branch_set_upstream()` 传入 `nil` 可移除 upstream。其选项仅包含 `confirm`。

### 标签

```lua
repo:tags(pattern?) -> GitTagInfo[]
repo:tag_create(name, target?, options?) -> GitTagInfo
repo:tag_delete(name, options?) -> nil
```

`tag_create()` 默认指向 `HEAD`。不提供 `message` 时创建轻量标签，提供 `message` 时创建附注标签。选项包括 `message`、`tagger`、`force` 和 `confirm`。tagger 使用共享签名字段。

标签信息包含 `name`、`oid`、`target_oid`、`target_kind`、`annotated`、`message` 和 `tagger`。`tags(pattern)` 使用 Git glob 匹配。删除标签只会修改本地仓库；请使用显式 push refspec 删除远端标签。

```lua
local tag = repo:tag_create("v1.0.0", "HEAD", {
  message = "Release v1.0.0",
})
repo:push("origin", "refs/tags/v1.0.0:refs/tags/v1.0.0")
```

### 远端与传输

```lua
repo:remotes() -> GitRemoteInfo[]
repo:remote(name) -> GitRemoteInfo
repo:remote_add(name, url, options?) -> GitRemoteInfo
repo:remote_remove(name, options?) -> nil
repo:remote_rename(name, new_name, options?) -> GitRemoteInfo
repo:remote_set_url(name, url, options?) -> GitRemoteInfo
repo:fetch(remote?, options?) -> GitFetchStats
repo:push(remote?, refspecs?, options?) -> GitPushResult
repo:pull(remote?, branch?, options?) -> GitIntegrateResult
```

远端信息包含 `name`、`url`、`push_url`、`fetch_refspecs` 和 `push_refspecs`。`remote_add()` 接受 `push_url` 和 `confirm`。`remote_set_url()` 接受 `push = true`，以修改 push URL 而不是 fetch URL。删除、重命名和设置 URL 操作均接受 `confirm`。

`fetch()` 接受 `refspecs`、`auth`、`depth`、`prune`、`tags`、`update_fetchhead` 和 `confirm`。`push()` 接受 `auth`、`force`、`set_upstream` 和 `confirm`。

`pull()` 默认使用远端 `"origin"`、当前分支和 `strategy = "ff_only"`。它还接受 `strategy = "merge" | "rebase"`、fetch 选项 `auth`、`depth`、`prune`、`tags`、`update_fetchhead`，以及 `signature`、`message` 和 `confirm`。Pull 开始前要求仓库处于 clean 状态。

### Worktree、index 与恢复

```lua
repo:add(paths, options?) -> nil
repo:restore(paths, options?) -> nil
repo:reset(rev?, options?) -> nil
repo:remove(paths, options?) -> nil
repo:clean(options?) -> string[]
```

- `restore()` 接受 `source`（默认 `"HEAD"`）、`staged`、`worktree` 和 `confirm`。仅指定 `staged = true` 时不会修改 worktree。
- `reset()` 接受 `mode = "soft" | "mixed" | "hard"`、`force` 和 `confirm`。hard reset 要求 `force = true`。
- `remove()` 接受 `cached`、`force` 和 `confirm`。
- `clean()` 接受 `dry_run`、`force`、`dirs`、`ignored`、`paths` 和 `confirm`。默认为 `dry_run = true`。实际删除要求同时设置 `dry_run = false` 和 `force = true`。除非设置 `dirs = true`，否则目录不会被修改。

```lua
local candidates = repo:clean()
repo:clean({ dry_run = false, force = true, dirs = true, confirm = true })
```

### 配置

```lua
repo:config_get(name, options?) -> string|boolean|integer|nil
repo:config_list(options?) -> GitConfigEntry[]
repo:config_set(name, value, options?) -> nil
repo:config_remove(name, options?) -> nil
```

`scope` 可以是 `"local"`、`"global"` 或 `"system"`；读取默认使用可用的最高优先级值，写入默认为 `"local"`。配置条目包含 `name`、`value` 和 `scope`。system 配置只读。global `config_set()` 和 `config_remove()` 要求 `confirm = true`，并且始终显示确认提示。

### Merge、cherry-pick 与 revert

```lua
repo:state() -> string
repo:conflicts() -> GitConflictEntry[]
repo:merge_analysis(rev) -> string
repo:merge(rev, options?) -> GitIntegrateResult
repo:merge_abort(options?) -> nil
repo:cherry_pick(rev, options?) -> GitIntegrateResult
repo:cherry_pick_abort(options?) -> nil
repo:revert(rev, options?) -> GitIntegrateResult
repo:revert_abort(options?) -> nil
```

`merge()` 接受 `ff = "allow" | "only" | "never"`、`signature`、`message` 和 `confirm`。为了让 abort 能安全恢复 `ORIG_HEAD`，merge、cherry-pick 和 revert 开始前要求仓库处于 clean 状态。

Cherry-pick 和 revert 接受 `commit`（默认 `true`）、`signature`、`message`、`mainline` 和 `confirm`。设置 `commit = false` 可在不创建提交的情况下更新 index 和 worktree。Abort 方法接受 `confirm`。

### Stash 与 rebase

```lua
repo:stash_save(message?, options?) -> string
repo:stashes() -> GitStashInfo[]
repo:stash_apply(index?, options?) -> GitIntegrateResult
repo:stash_pop(index?, options?) -> GitIntegrateResult
repo:stash_drop(index?, options?) -> nil
repo:rebase(options) -> GitRebaseResult
repo:rebase_continue(options?) -> GitRebaseResult
repo:rebase_abort(options?) -> nil
```

Stash 索引默认为 `0`。`stash_save()` 接受 `include_untracked`、`include_ignored`、`keep_index`、`signature` 和 `confirm`。Apply 和 pop 接受 `reinstate_index` 与 `confirm`；drop 接受 `confirm`。Stash 信息包含 `index`、`message` 和 `oid`。

`rebase()` 要求提供 `upstream`，并接受 `onto`、`branch`（默认 `"HEAD"`）、`signature` 和 `confirm`。第一版仅支持非交互式 pick 操作，不支持交互式 squash、fixup、reword 和 edit。Continue 接受 `signature` 和 `confirm`；abort 接受 `confirm`。

### 高级仓库

```lua
repo:worktrees() -> GitWorktreeInfo[]
repo:worktree_add(name, path, options?) -> GitWorktreeInfo
repo:worktree_lock(name, reason?, options?) -> nil
repo:worktree_unlock(name, options?) -> nil
repo:worktree_prune(name, options?) -> nil
repo:submodules() -> GitSubmoduleInfo[]
repo:submodule_init(name?, options?) -> nil
repo:submodule_update(name?, options?) -> nil
repo:submodule_sync(name?, options?) -> nil
repo:blame(path, options?) -> GitBlameHunk[]
```

- Worktree 信息包含 `name`、`path`、`locked`、`lock_reason` 和 `valid`。Add 选项包括 `reference`、`lock`、`checkout_existing` 和 `confirm`。Prune 选项包括 `valid`、`locked`、`working_tree`、`force` 和 `confirm`；lock 与 unlock 也接受 `confirm`。
- Submodule 信息包含 `name`、`path`、`url`、`branch`、`head_oid`、`index_oid` 和 `workdir_oid`。Init 选项包括 `overwrite`、`recursive` 和 `confirm`。Update 选项包括 `init`、`recursive`、`allow_fetch`、`auth` 和 `confirm`。Sync 选项包括 `recursive` 和 `confirm`。
- 除非设置 `recursive = true`，否则不会递归处理 submodule。
- `blame()` 接受 `newest`、`oldest`、`min_line`、`max_line`、`first_parent`、复制/移动跟踪标志、`ignore_whitespace` 和 `use_mailmap`。每个 hunk 包含 `final_start_line`、`original_start_line`、`line_count`、`commit_oid`、`author`、`origin_path` 和 `boundary`。

## 安全性与兼容性

所有接受 `confirm` 的修改方法都默认为 `false`。`force` 与 `confirm` 表达不同意图：`force` 启用原本会被拒绝的行为，而 `confirm` 会在执行已经有效的操作前询问用户。

现有默认行为保持兼容：默认远端为 `"origin"`，不提供 refspecs 的 push 会推送当前分支，status 包含未跟踪文件；除非传入 `allow_empty = false`，否则仍允许空提交。

远端标签删除通过 push refspec 表达：

```lua
repo:push("origin", ":refs/tags/v1.0.0", { confirm = true })
```
