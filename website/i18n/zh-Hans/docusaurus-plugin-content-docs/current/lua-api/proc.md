# 进程 API

`ptool.proc` 和 `p.proc` 下提供了一组本地进程辅助能力。

这个模块用于检查和管理已经在本机运行的进程。要启动新命令时，请使用 `ptool.run(...)`。

## ptool.proc.self

> `Unreleased` - 引入。

`ptool.proc.self()` 返回当前 `ptool` 进程的快照表。

- 返回：`table`。

返回的 table 与 `ptool.proc.get(...)` 和 `ptool.proc.find(...)` 的返回结构相同。

## ptool.proc.get

> `Unreleased` - 引入。

`ptool.proc.get(pid)` 返回指定进程 ID 的快照表；如果进程不存在，则返回 `nil`。

- `pid`（integer，必填）：进程 ID。
- 返回：`table|nil`。

## ptool.proc.exists

> `Unreleased` - 引入。

`ptool.proc.exists(pid)` 用于报告某个进程 ID 当前是否存在。

- `pid`（integer，必填）：进程 ID。
- 返回：`boolean`。

## ptool.proc.find

> `Unreleased` - 引入。

`ptool.proc.find([options])` 列出本地进程，并返回一个由快照表组成的数组。

- `options`（table，可选）：过滤与排序选项。
- 返回：`table`。

支持的 `options` 字段：

- `pid`（integer，可选）：匹配一个精确的进程 ID。
- `pids`（integer[]，可选）：匹配一组进程 ID。
- `ppid`（integer，可选）：匹配一个精确的父进程 ID。
- `name`（string，可选）：匹配一个精确的进程名。
- `name_contains`（string，可选）：匹配进程名中的子串。
- `exe`（string，可选）：匹配一个精确的可执行文件路径。
- `exe_contains`（string，可选）：匹配可执行文件路径中的子串。
- `cmdline_contains`（string，可选）：匹配拼接后命令行中的子串。
- `user`（string，可选）：匹配一个精确的用户名。
- `cwd`（string，可选）：匹配一个精确的当前工作目录。
- `include_self`（boolean，可选）：是否包含当前 `ptool` 进程。默认值为 `false`。
- `limit`（integer，可选）：过滤和排序完成后，返回条目的最大数量。
- `sort_by`（string，可选）：排序键。支持的值：
  - `"pid"`（默认）
  - `"start_time"`
- `reverse`（boolean，可选）：是否反转最终排序顺序。默认值为 `false`。

每个返回的进程快照可能包含：

- `pid`（integer）：进程 ID。
- `ppid`（integer|nil）：父进程 ID。
- `name`（string）：进程名。
- `exe`（string|nil）：可执行文件路径（如果可用）。
- `cwd`（string|nil）：当前工作目录（如果可用）。
- `user`（string|nil）：所属用户名（如果可用）。
- `cmdline`（string|nil）：拼接后的命令行（如果可用）。
- `argv`（string[]）：命令行参数数组。
- `state`（string）：进程状态标签，例如 `"running"` 或 `"sleeping"`。
- `start_time_unix_ms`（integer）：进程启动时间的 Unix 毫秒时间戳。

说明：

- 某些字段在当前平台或权限级别未暴露时，可能为 `nil`。
- 进程快照是某一时刻的值，不会自行更新。

示例：

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
  include_self = true,
  sort_by = "start_time",
})

for _, proc in ipairs(procs) do
  print(proc.pid, proc.name, proc.cmdline)
end
```

## ptool.proc.kill

> `Unreleased` - 引入。

`ptool.proc.kill(targets[, options])` 向一个或多个本地进程发送信号，并返回结构化结果表。

- `targets`（integer|table，必填）：可以是一个 pid、一个进程快照表，或它们组成的数组。
- `options`（table，可选）：终止选项。
- 返回：`table`。

支持的 `options` 字段：

- `signal`（string，可选）：信号名。支持的值：
  - `"hup"`
  - `"term"`（默认）
  - `"kill"`
  - `"int"`
  - `"quit"`
  - `"stop"`
  - `"cont"`
  - `"user1"`
  - `"user2"`
- `missing_ok`（boolean，可选）：缺失的进程是否也算成功。默认值为 `true`。
- `allow_self`（boolean，可选）：当前 `ptool` 进程是否允许被发送信号。默认值为 `false`。
- `check`（boolean，可选）：当最终结果不是 ok 时，是否立即抛出错误。默认值为 `false`。
- `confirm`（boolean，可选）：发送信号前是否请求确认。默认值为 `false`。

返回的结果表包含：

- `ok`（boolean）：整个操作在当前选项下是否成功。
- `signal`（string）：请求发送的信号标签。
- `total`（integer）：归一化后的目标总数。
- `sent`（integer）：已发送信号的目标数量。
- `missing`（integer）：已经不存在的目标数量。
- `failed`（integer）：最终失败的目标数量。
- `entries`（table）：每个目标对应的结果条目。
- `assert_ok(self)`（function）：当 `ok = false` 时，抛出结构化 Lua 错误。

每个 `entries[i]` table 包含：

- `pid`（integer）：目标进程 ID。
- `ok`（boolean）：该目标是否成功。
- `existed`（boolean）：目标进程是否存在且仍然匹配。
- `signal`（string）：请求发送的信号标签。
- `message`（string|nil）：额外的状态说明。

示例：

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
})

local res = p.proc.kill(procs, {
  signal = "term",
  confirm = true,
})

res:assert_ok()
```

## ptool.proc.wait_gone

> `Unreleased` - 引入。

`ptool.proc.wait_gone(targets[, options])` 会等待一个或多个目标进程不再存在，然后返回结构化结果表。

- `targets`（integer|table，必填）：可以是一个 pid、一个进程快照表，或它们组成的数组。
- `options`（table，可选）：等待选项。
- 返回：`table`。

支持的 `options` 字段：

- `timeout_ms`（integer，可选）：最长等待时间，单位毫秒。省略时会一直等待。
- `interval_ms`（integer，可选）：轮询间隔，单位毫秒。默认值为 `100`。
- `check`（boolean，可选）：等待超时时，是否立即抛出错误。默认值为 `false`。

返回的结果表包含：

- `ok`（boolean）：是否所有目标进程都在超时前消失。
- `timed_out`（boolean）：是否触达超时。
- `total`（integer）：归一化后的目标总数。
- `gone`（integer[]）：等待结束时已经消失的进程 ID。
- `remaining`（integer[]）：等待结束时仍然存在的进程 ID。
- `elapsed_ms`（integer）：总等待耗时，单位毫秒。
- `assert_ok(self)`（function）：当 `ok = false` 时，抛出结构化 Lua 错误。

示例：

```lua
local procs = p.proc.find({
  cmdline_contains = "user-service",
})

local wait_res = p.proc.wait_gone(procs, {
  timeout_ms = 1000,
  interval_ms = 100,
})

wait_res:assert_ok()
```
