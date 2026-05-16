# Git API

Git repository helpers are available under `ptool.git` and `p.git`.

This module is backed by `git2` / `libgit2`, not by invoking the `git`
command-line tool.

## ptool.git.open

> `Unreleased` - Introduced.

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

> `Unreleased` - Introduced.

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

> `Unreleased` - Introduced.

`ptool.git.clone(url, path[, options])` clones a repository and returns a
`Repo` object for the cloned repository.

Arguments:

- `url` (string, required): Remote repository URL.
- `path` (string, required): Destination path.
- `options` (table, optional): Clone options. Supported fields:
  - `branch` (string, optional): Branch name to check out after cloning.
  - `bare` (boolean, optional): Whether to create a bare repository. Defaults
    to `false`.
  - `auth` (table, optional): Remote authentication settings.

`auth` fields:

- `kind` (string, required): Authentication mode. Supported values:
  - `"default"`: Use libgit2 default credentials.
  - `"ssh_agent"`: Authenticate through the local SSH agent.
  - `"userpass"`: Use a plaintext username and password.
- `username` (string, optional): Username for `"ssh_agent"`.
- `username` (string, required): Username for `"userpass"`.
- `password` (string, required): Password for `"userpass"`.

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

> `Unreleased` - Introduced.

`Repo` represents an open Git repository handle returned by `ptool.git.open()`,
`ptool.git.discover()`, or `ptool.git.clone()`.

It is implemented as a Lua userdata.

Methods:

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

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:path`.

`repo:path()` returns the repository git directory path.

- Returns: `string`.

Notes:

- For a non-bare repository this is typically the `.git` directory.
- For a bare repository this is the repository directory itself.

### root

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:root`.

`repo:root()` returns the worktree root directory.

- Returns: `string|nil`.

Notes:

- This returns `nil` for bare repositories.

### is_bare

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:is_bare`.

`repo:is_bare()` reports whether the repository is bare.

- Returns: `boolean`.

### head

> `Unreleased` - Introduced.

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

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:current_branch`.

`repo:current_branch()` returns the current local branch name.

- Returns: `string|nil`.

Notes:

- This returns `nil` when HEAD is detached.
- This also returns `nil` for an unborn branch before the first commit.

### status

> `Unreleased` - Introduced.

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

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:is_clean`.

`repo:is_clean([options])` returns whether the repository is clean.

- `options` (table, optional): The same options accepted by `repo:status(...)`.
- Returns: `boolean`.

### add

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:add`.

`repo:add(paths[, options])` stages one or more paths in the index.

Arguments:

- `paths` (string|string[], required): A path or an array of paths.
- `options` (table, optional): Add options. Supported fields:
  - `update` (boolean, optional): Update only paths already known to the
    index. Defaults to `false`.

Behavior:

- Paths are interpreted relative to the repository worktree.

Example:

```lua
repo:add("README.md")
repo:add({"src", "Cargo.toml"})
```

### commit

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:commit`.

`repo:commit(message[, options])` creates a commit from the current index and
returns the new commit OID.

Arguments:

- `message` (string, required): Commit message.
- `options` (table, optional): Commit options. Supported fields:
  - `author` (table, optional): Author signature.
  - `committer` (table, optional): Committer signature.

Signature fields:

- `name` (string, required)
- `email` (string, required)

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

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:checkout`.

`repo:checkout(rev[, options])` checks out a revision.

Arguments:

- `rev` (string, required): Revision expression such as a branch name, tag
  name, or commit OID.
- `options` (table, optional): Checkout options. Supported fields:
  - `force` (boolean, optional): Whether to force checkout. Defaults to
    `false`.

Behavior:

- This can detach HEAD when `rev` does not resolve to a named reference.

### switch

> `Unreleased` - Introduced.

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

Example:

```lua
repo:switch("release")
repo:switch("release-next", {
  create = true,
  start_point = "origin/main",
})
```

### fetch

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:fetch`.

`repo:fetch([remote[, options]])` fetches from a remote and returns transfer
statistics.

Arguments:

- `remote` (string, optional): Remote name. Defaults to `"origin"`.
- `options` (table, optional): Fetch options. Supported fields:
  - `refspecs` (string|string[], optional): One refspec or an array of
    refspecs.
  - `auth` (table, optional): Remote authentication settings. Uses the same
    structure as `ptool.git.clone(...)`.

Returns:

- `received_objects` (integer)
- `indexed_objects` (integer)
- `local_objects` (integer)
- `total_objects` (integer)
- `received_bytes` (integer)

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

> `Unreleased` - Introduced.

Canonical API name: `ptool.git.Repo:push`.

`repo:push([remote[, refspecs[, options]]])` pushes refs to a remote.

Arguments:

- `remote` (string, optional): Remote name. Defaults to `"origin"`.
- `refspecs` (string|string[], optional): One refspec or an array of refspecs.
- `options` (table, optional): Push options. Supported fields:
  - `auth` (table, optional): Remote authentication settings. Uses the same
    structure as `ptool.git.clone(...)`.

Behavior:

- When `refspecs` is omitted, `ptool` tries to push the current local branch to
  the branch of the same name on the remote.
- Omitting `refspecs` while HEAD is detached raises an error.

Example:

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```
