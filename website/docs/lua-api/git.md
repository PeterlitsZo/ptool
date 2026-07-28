# Git API

Git repository helpers are available under `ptool.git` and `p.git`.

This module is backed by `git2` / `libgit2`, not by invoking the `git`
command-line tool.

Mutating operations accept `confirm = true` in their options table to ask the
user for confirmation before the action runs. Destructive operations also use
separate safety flags such as `force = true`; confirmation never replaces the
required safety flag.

Unknown option names are rejected instead of being silently ignored.

## ptool.git.init

> `Unreleased` - Introduced.

`ptool.git.init(path?, options?)` initializes a repository and returns a `Repo`
object.

Options:

- `bare` (boolean): Create a bare repository. Defaults to `false`.
- `initial_head` (string): Initial branch name, for example `"main"`.
- `confirm` (boolean): Ask before creating the repository.

Relative paths are resolved from the current `ptool` runtime directory.

```lua
local repo = p.git.init("tmp/project", {
  initial_head = "main",
})
```

## ptool.git.open

> `v0.6.0` - Introduced.

`ptool.git.open(path?)` opens a repository directly and returns a `Repo`
object.

Arguments:

- `path` (string, optional): Repository path. If omitted, the current `ptool`
  runtime directory is used.

Behavior:

- Relative paths are resolved from the current `ptool` runtime directory, so
  they follow `ptool.cd(...)`.
- This does not search parent directories. Use `ptool.git.discover(...)` when
  you want repository discovery behavior.

Example:

```lua
local repo = ptool.git.open(".")
print(repo:path())
```

## ptool.git.discover

> `v0.6.0` - Introduced.

`ptool.git.discover(path?)` finds a repository starting from `path` and walking
up parent directories, then returns a `Repo` object.

Arguments:

- `path` (string, optional): Starting path. If omitted, the current `ptool`
  runtime directory is used.

Behavior:

- Relative paths are resolved from the current `ptool` runtime directory.
- This is useful when a script may run from a subdirectory inside a worktree.

Example:

```lua
local repo = ptool.git.discover("src")
print(repo:root())
```

## ptool.git.clone

> `v0.6.0` - Introduced.

`ptool.git.clone(url, path[, options])` clones a repository and returns a
`Repo` object for the cloned repository.

Arguments:

- `url` (string, required): Remote repository URL.
- `path` (string, required): Destination path.
- `options` (table, optional): Clone options. Supported fields:
  - `branch` (string, optional): Branch name to check out after cloning.
  - `bare` (boolean, optional): Whether to create a bare repository. Defaults
    to `false`.
  - `depth` (integer, optional): Positive shallow-clone depth.
  - `checkout` (boolean, optional): Whether to check out the selected branch.
    Defaults to `true`.
  - `remote` (string, optional): Name assigned to the cloned remote. Defaults to
    `"origin"`.
  - `tags` (string, optional): `"auto"`, `"all"`, or `"none"`. Defaults to
    `"auto"`.
  - `confirm` (boolean, optional): Whether to ask for confirmation before
    cloning. Defaults to `false`.
  - `auth` (table, optional): Remote authentication settings.

`auth` fields:

- `kind` (string, required): Authentication mode. Supported values:
  - `"default"`: Use libgit2 default credentials.
  - `"ssh_agent"`: Authenticate through the local SSH agent.
  - `"ssh_key"`: Authenticate with an SSH private key.
  - `"userpass"`: Use a plaintext username and password.
  - `"credential_helper"`: Ask the configured Git credential helper.
- `username` (string): Optional for `"ssh_agent"` and `"credential_helper"`;
  required for `"ssh_key"` and `"userpass"`.
- `private_key` (string, required for `"ssh_key"`): Private key path. Relative
  paths are resolved from the current `ptool` runtime directory.
- `public_key` (string, optional): Public key path.
- `passphrase` (string, optional): SSH private-key passphrase.
- `password` (string, required for `"userpass"`): Password.

Behavior:

- Relative destination paths are resolved from the current `ptool` runtime
  directory.
- Authentication options are also used by `repo:fetch(...)` and
  `repo:push(...)`.

Example:

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

> `v0.6.0` - Introduced.

`Repo` represents an open Git repository handle returned by `ptool.git.init()`, `ptool.git.open()`,
`ptool.git.discover()`, or `ptool.git.clone()`.

It is implemented as a Lua userdata.

Methods are grouped below. Existing repository, status, staging, commit, checkout,
switch, fetch, and push methods remain backward compatible. The workflow APIs
described in the following sections are currently unreleased.

### path

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:path`.

`repo:path()` returns the repository git directory path.

- Returns: `string`.

Notes:

- For a non-bare repository this is typically the `.git` directory.
- For a bare repository this is the repository directory itself.

### root

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:root`.

`repo:root()` returns the worktree root directory.

- Returns: `string|nil`.

Notes:

- This returns `nil` for bare repositories.

### is_bare

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:is_bare`.

`repo:is_bare()` reports whether the repository is bare.

- Returns: `boolean`.

### head

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:head`.

`repo:head()` returns HEAD information as a table with:

- `oid` (string|nil): The current commit OID if available.
- `shorthand` (string|nil): A short name for HEAD, such as a branch name.
- `detached` (boolean): Whether HEAD is detached.
- `unborn` (boolean): Whether the repository does not yet have an initial
  commit.

Example:

```lua
local head = repo:head()
print(head.oid)
print(head.detached)
```

### current_branch

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:current_branch`.

`repo:current_branch()` returns the current local branch name.

- Returns: `string|nil`.

Notes:

- This returns `nil` when HEAD is detached.
- This also returns `nil` for an unborn branch before the first commit.

### status

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:status`.

`repo:status([options])` summarizes repository status and returns a table with:

- `root` (string|nil): The worktree root directory.
- `branch` (string|nil): The current local branch name.
- `head` (table): The same HEAD information returned by `repo:head()`.
- `upstream` (string|nil): The upstream branch name, when configured.
- `ahead` (integer): Number of commits ahead of upstream.
- `behind` (integer): Number of commits behind upstream.
- `clean` (boolean): Whether the repository has no visible status entries.
- `entries` (table): An array of status entry tables.

`entries[i]` contains:

- `path` (string): Repository-relative path.
- `index_status` (string|nil): Index-side status. Supported values currently
  include `"new"`, `"modified"`, `"deleted"`, `"renamed"`, and `"typechange"`.
- `worktree_status` (string|nil): Worktree-side status. Supported values
  currently include `"new"`, `"modified"`, `"deleted"`, `"renamed"`,
  `"typechange"`, and `"ignored"`.
- `conflicted` (boolean): Whether the path is conflicted.
- `ignored` (boolean): Whether the path is ignored.

`options` fields:

- `include_untracked` (boolean, optional): Whether to include untracked files.
  Defaults to `true`.
- `include_ignored` (boolean, optional): Whether to include ignored files.
  Defaults to `false`.
- `recurse_untracked_dirs` (boolean, optional): Whether to recurse into
  untracked directories. Defaults to `true`.

Example:

```lua
local st = repo:status()
print(st.clean)
print(st.branch)

for _, entry in ipairs(st.entries) do
  print(entry.path, entry.index_status, entry.worktree_status)
end
```

### is_clean

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:is_clean`.

`repo:is_clean([options])` returns whether the repository is clean.

- `options` (table, optional): The same options accepted by `repo:status(...)`.
- Returns: `boolean`.

### add

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:add`.

`repo:add(paths[, options])` stages one or more paths in the index.

Arguments:

- `paths` (string|string[], required): A path or an array of paths.
- `options` (table, optional): Add options. Supported fields:
  - `update` (boolean, optional): Update only paths already known to the
    index. Defaults to `false`.
  - `confirm` (boolean, optional): Whether to ask for confirmation before
    staging paths. Defaults to `false`.

Behavior:

- Paths are interpreted relative to the repository worktree.

Example:

```lua
repo:add("README.md")
repo:add({"src", "Cargo.toml"})
```

### commit

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:commit`.

`repo:commit(message[, options])` creates a commit from the current index and
returns the new commit OID.

Arguments:

- `message` (string, required): Commit message.
- `options` (table, optional): Commit options. Supported fields:
  - `author` (table, optional): Author signature.
  - `committer` (table, optional): Committer signature.
  - `amend` (boolean, optional): Replace the current HEAD commit. Defaults to
    `false`.
  - `allow_empty` (boolean, optional): Permit a commit whose tree is unchanged.
    Defaults to `true` for backward compatibility.
  - `confirm` (boolean, optional): Whether to ask for confirmation before
    creating the commit. Defaults to `false`.

Signature fields:

- `name` (string, required)
- `email` (string, required)
- `time_seconds` (integer, optional): Unix timestamp.
- `offset_minutes` (integer, optional): Time-zone offset from UTC.

Behavior:

- When `author` and `committer` are omitted, `ptool` tries to use the Git
  repository identity from configuration.
- If no identity is configured and no explicit signature is provided, an error
  is raised.

Example:

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

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:checkout`.

`repo:checkout(rev[, options])` checks out a revision.

Arguments:

- `rev` (string, required): Revision expression such as a branch name, tag
  name, or commit OID.
- `options` (table, optional): Checkout options. Supported fields:
  - `force` (boolean, optional): Whether to force checkout. Defaults to
    `false`.
  - `confirm` (boolean, optional): Whether to ask for confirmation before
    checking out the revision. Defaults to `false`.

Behavior:

- This can detach HEAD when `rev` does not resolve to a named reference.

### switch

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:switch`.

`repo:switch(branch[, options])` switches HEAD to a local branch.

Arguments:

- `branch` (string, required): Local branch name.
- `options` (table, optional): Switch options. Supported fields:
  - `create` (boolean, optional): Whether to create the branch first. Defaults
    to `false`.
  - `force` (boolean, optional): Whether to force the checkout. Defaults to
    `false`.
  - `start_point` (string, optional): Revision to branch from when
    `create = true`. Defaults to `HEAD`.
  - `track` (string, optional): Upstream reference for the new branch.
  - `orphan` (boolean, optional): Create an orphan branch.
  - `confirm` (boolean, optional): Whether to ask for confirmation before
    switching branches. Defaults to `false`.

Example:

```lua
repo:switch("release")
repo:switch("release-next", {
  create = true,
  start_point = "origin/main",
})
```

### fetch

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:fetch`.

`repo:fetch([remote[, options]])` fetches from a remote and returns transfer
statistics.

Arguments:

- `remote` (string, optional): Remote name. Defaults to `"origin"`.
- `options` (table, optional): Fetch options. Supported fields:
  - `refspecs` (string|string[], optional): One refspec or an array of
    refspecs.
  - `depth` (integer, optional): Positive shallow-fetch depth.
  - `prune` (boolean, optional): Remove stale remote-tracking references.
  - `tags` (string, optional): `"auto"`, `"all"`, or `"none"`.
  - `update_fetchhead` (boolean, optional): Update `FETCH_HEAD`. Defaults to
    `true`.
  - `confirm` (boolean, optional): Whether to ask for confirmation before
    fetching. Defaults to `false`.
  - `auth` (table, optional): Remote authentication settings. Uses the same
    structure as `ptool.git.clone(...)`.

Returns:

- `received_objects` (integer)
- `indexed_objects` (integer)
- `local_objects` (integer)
- `total_objects` (integer)
- `received_bytes` (integer)
- `updated_refs` (string[])

Example:

```lua
local stats = repo:fetch("origin", {
  auth = {
    kind = "ssh_agent",
  },
})

print(stats.received_objects, stats.received_bytes)
```

### push

> `v0.6.0` - Introduced.

Canonical API name: `ptool.git.Repo:push`.

`repo:push([remote[, refspecs[, options]]])` pushes refs to a remote.

Arguments:

- `remote` (string, optional): Remote name. Defaults to `"origin"`.
- `refspecs` (string|string[], optional): One refspec or an array of refspecs.
- `options` (table, optional): Push options. Supported fields:
  - `force` (boolean, optional): Force each push refspec. Defaults to `false`.
  - `set_upstream` (boolean, optional): Configure the pushed current branch to
    track the destination remote branch.
  - `confirm` (boolean, optional): Whether to ask for confirmation before
    pushing. Defaults to `false`.
  - `auth` (table, optional): Remote authentication settings. Uses the same
    structure as `ptool.git.clone(...)`.

Behavior:

- When `refspecs` is omitted, `ptool` tries to push the current local branch to
  the branch of the same name on the remote.
- Omitting `refspecs` while HEAD is detached raises an error.
- The returned table contains `ok`, `refspecs`, and `rejected`. Each rejected
  entry contains `reference` and `message`.

Example:

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```


## Git workflow APIs

> `Unreleased` - Introduced.

The APIs in this section cover repository maintenance, releases, history
inspection, collaboration, and CI automation. A string or a dense string array
is accepted wherever a parameter is documented as `string|string[]`.

### Shared result tables

Commit information returned by `commit_info()` and `log()` contains `oid`,
`message`, `summary`, `author`, `committer`, and `parent_oids`. A signature table
contains `name`, `email`, `time_seconds`, and `offset_minutes`.

Integration methods return:

```lua
{
  outcome = "up_to_date" | "fast_forward" | "merged" | "conflicted",
  oid = "..." | nil,
  conflicts = {
    { path = "file", ancestor = true, ours = true, theirs = true },
  },
}
```

Rebase results use `"rebased"` or `"conflicted"` outcomes and add `current` and
`total`. Operations that stop on conflicts can
be continued or aborted with the corresponding API.

### Repository and status

```lua
repo:path() -> string
repo:root() -> string|nil
repo:is_bare() -> boolean
repo:head() -> GitHeadInfo
repo:current_branch() -> string|nil
repo:status(options?) -> GitStatusSummary
repo:is_clean(options?) -> boolean
```

`status()` and `is_clean()` options are `include_untracked` (default `true`),
`include_ignored`, `recurse_untracked_dirs` (default `true`), and `paths`.
`GitStatusSummary` contains `root`, `branch`, `head`, `upstream`, `ahead`,
`behind`, `clean`, and `entries`.

### History and diff

```lua
repo:resolve(rev) -> GitObjectInfo
repo:commit_info(rev?) -> GitCommitInfo
repo:log(options?) -> GitCommitInfo[]
repo:diff(options?) -> GitDiff
repo:describe(options?) -> string|nil
```

- `resolve()` returns `oid`, `kind`, and `shorthand`.
- `commit_info()` defaults to `HEAD`.
- `log()` accepts `rev`, `max_count` (default `100`), `skip`, `first_parent`,
  `reverse`, and `paths`.
- `diff()` accepts `from`, `to`, `cached`, `paths`, `context_lines` (default
  `3`), and `find_renames` (default `true`). Without `from` or `to`, it compares
  the appropriate worktree, index, or HEAD state.
- A diff result contains `patch`, `files_changed`, `insertions`, `deletions`,
  and `deltas`. Each delta contains `status`, `old_path`, `new_path`, and
  `binary`.
- `describe()` accepts `rev`, `pattern`, `always`, `abbrev` (default `7`), and
  `dirty_suffix`.

```lua
local commits = repo:log({
  rev = "HEAD",
  max_count = 20,
  first_parent = true,
  paths = {"crates/ptool-engine"},
})
local changes = repo:diff({ from = "v0.10.0", to = "HEAD" })
```

### Branches

```lua
repo:branches(options?) -> GitBranchInfo[]
repo:branch_create(name, options?) -> GitBranchInfo
repo:branch_delete(name, options?) -> nil
repo:branch_rename(old_name, new_name, options?) -> GitBranchInfo
repo:branch_set_upstream(name, upstream_or_nil, options?) -> nil
```

- `branches()` accepts `kind = "local" | "remote" | "all"`; the default is
  `"local"`.
- Branch information contains `name`, `kind`, `oid`, `head`, `upstream`,
  `ahead`, and `behind`.
- `branch_create()` accepts `start_point`, `force`, `checkout`, `upstream`, and
  `confirm`. The default start point is `HEAD`.
- `branch_delete()` accepts `force` and `confirm`. Deleting the current branch
  fails, and deleting an unmerged branch requires `force = true`.
- `branch_rename()` accepts `force` and `confirm`.
- Pass `nil` to `branch_set_upstream()` to remove an upstream. Its options only
  contain `confirm`.

### Tags

```lua
repo:tags(pattern?) -> GitTagInfo[]
repo:tag_create(name, target?, options?) -> GitTagInfo
repo:tag_delete(name, options?) -> nil
```

`tag_create()` targets `HEAD` by default. Without `message` it creates a
lightweight tag; with `message` it creates an annotated tag. Options are
`message`, `tagger`, `force`, and `confirm`. A tagger uses the shared signature
fields.

Tag information contains `name`, `oid`, `target_oid`, `target_kind`,
`annotated`, `message`, and `tagger`. `tags(pattern)` uses Git's glob matching.
Deleting a tag only changes the local repository; delete a remote tag with an
explicit push refspec.

```lua
local tag = repo:tag_create("v1.0.0", "HEAD", {
  message = "Release v1.0.0",
})
repo:push("origin", "refs/tags/v1.0.0:refs/tags/v1.0.0")
```

### Remotes and transfer

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

Remote information contains `name`, `url`, `push_url`, `fetch_refspecs`, and
`push_refspecs`. `remote_add()` accepts `push_url` and `confirm`.
`remote_set_url()` accepts `push = true` to change the push URL instead of the
fetch URL. Remove, rename, and set URL operations accept `confirm`.

`fetch()` accepts `refspecs`, `auth`, `depth`, `prune`, `tags`,
`update_fetchhead`, and `confirm`. `push()` accepts `auth`, `force`,
`set_upstream`, and `confirm`.

`pull()` defaults to remote `"origin"`, the current branch, and
`strategy = "ff_only"`. It also accepts `strategy = "merge" | "rebase"`, the
fetch options `auth`, `depth`, `prune`, `tags`, and `update_fetchhead`, plus
`signature`, `message`, and `confirm`. Pull requires a clean repository before
it starts.

### Worktree, index, and recovery

```lua
repo:add(paths, options?) -> nil
repo:restore(paths, options?) -> nil
repo:reset(rev?, options?) -> nil
repo:remove(paths, options?) -> nil
repo:clean(options?) -> string[]
```

- `restore()` accepts `source` (default `"HEAD"`), `staged`, `worktree`, and
  `confirm`. When only `staged = true` is specified, the worktree is not
  changed.
- `reset()` accepts `mode = "soft" | "mixed" | "hard"`, `force`, and
  `confirm`. A hard reset requires `force = true`.
- `remove()` accepts `cached`, `force`, and `confirm`.
- `clean()` accepts `dry_run`, `force`, `dirs`, `ignored`, `paths`, and
  `confirm`. It defaults to `dry_run = true`. Actual deletion requires both
  `dry_run = false` and `force = true`. Directories are left untouched unless
  `dirs = true`.

```lua
local candidates = repo:clean()
repo:clean({ dry_run = false, force = true, dirs = true, confirm = true })
```

### Configuration

```lua
repo:config_get(name, options?) -> string|boolean|integer|nil
repo:config_list(options?) -> GitConfigEntry[]
repo:config_set(name, value, options?) -> nil
repo:config_remove(name, options?) -> nil
```

`scope` is `"local"`, `"global"`, or `"system"`; the default for reads is the
highest-priority available value and the default for writes is `"local"`.
Configuration entries contain `name`, `value`, and `scope`. System configuration
is read-only. Global `config_set()` and `config_remove()` require
`confirm = true` and always show a confirmation prompt.

### Merge, cherry-pick, and revert

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

`merge()` accepts `ff = "allow" | "only" | "never"`, `signature`, `message`,
and `confirm`. Merge, cherry-pick, and revert require a clean repository before
they start so abort can safely restore `ORIG_HEAD`.

Cherry-pick and revert accept `commit` (default `true`), `signature`, `message`,
`mainline`, and `confirm`. Set `commit = false` to update the index and worktree
without creating a commit. Abort methods accept `confirm`.

### Stash and rebase

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

Stash indices default to `0`. `stash_save()` accepts `include_untracked`,
`include_ignored`, `keep_index`, `signature`, and `confirm`. Apply and pop accept
`reinstate_index` and `confirm`; drop accepts `confirm`. Stash information
contains `index`, `message`, and `oid`.

`rebase()` requires `upstream` and accepts `onto`, `branch` (default `"HEAD"`),
`signature`, and `confirm`. The first version supports non-interactive pick
operations; interactive squash, fixup, reword, and edit operations are not
available. Continue accepts `signature` and `confirm`; abort accepts `confirm`.

### Advanced repositories

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

- Worktree information contains `name`, `path`, `locked`, `lock_reason`, and
  `valid`. Add options are `reference`, `lock`, `checkout_existing`, and
  `confirm`. Prune options are `valid`, `locked`, `working_tree`, `force`, and
  `confirm`; lock and unlock also accept `confirm`.
- Submodule information contains `name`, `path`, `url`, `branch`, `head_oid`,
  `index_oid`, and `workdir_oid`. Init options are `overwrite`, `recursive`, and
  `confirm`. Update options are `init`, `recursive`, `allow_fetch`, `auth`, and
  `confirm`. Sync options are `recursive` and `confirm`.
- Recursive submodule processing is disabled unless `recursive = true`.
- `blame()` accepts `newest`, `oldest`, `min_line`, `max_line`, `first_parent`,
  copy/move tracking flags, `ignore_whitespace`, and `use_mailmap`. Each hunk
  contains `final_start_line`, `original_start_line`, `line_count`,
  `commit_oid`, `author`, `origin_path`, and `boundary`.

## Safety and compatibility

All mutating methods that accept `confirm` default it to `false`. `force` and
`confirm` express different intent: `force` enables behavior that is otherwise
rejected, while `confirm` asks the user before an already-valid action runs.

Existing defaults remain compatible: the default remote is `"origin"`, push
without refspecs pushes the current branch, status includes untracked files,
and empty commits remain allowed unless `allow_empty = false` is supplied.

Network tag deletion is expressed as a push refspec:

```lua
repo:push("origin", ":refs/tags/v1.0.0", { confirm = true })
```
