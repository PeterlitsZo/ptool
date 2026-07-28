# Git API

Git リポジトリ ヘルパーは、`ptool.git` および `p.git` で利用できます。

このモジュールは、`git` コマンド ライン ツールの呼び出しではなく、`git2` / `libgit2` によってサポートされます。

リポジトリを変更する操作では、オプションテーブルに `confirm = true` を指定すると、実行前にユーザーへ確認できます。破壊的な操作では `force = true` などの独立した安全フラグも使用し、確認は必須の安全フラグの代わりにはなりません。

不明なオプション名は黙って無視されず、エラーとして拒否されます。

## ptool.git.init

> `Unreleased` - 導入されました。

`ptool.git.init(path?, options?)` はリポジトリを初期化し、`Repo` オブジェクトを返します。

オプション：

- `bare` (boolean): bare リポジトリを作成します。既定値は `false` です。
- `initial_head` (string): 初期ブランチ名です。例：`"main"`。
- `confirm` (boolean): リポジトリを作成する前に確認します。

相対パスは現在の `ptool` ランタイムディレクトリから解決されます。

```lua
local repo = p.git.init("tmp/project", {
  initial_head = "main",
})
```

## ptool.git.open

> `v0.6.0` - 導入されました。

`ptool.git.open(path?)` はリポジトリを直接開き、`Repo` オブジェクトを返します。

引数:

- `path` (文字列、オプション): リポジトリのパス。省略した場合、現在の `ptool` ランタイム ディレクトリが使用されます。

挙動:

- 相対パスは現在の `ptool` ランタイム ディレクトリから解決されるため、`ptool.cd(...)` の後に続きます。
- これは親ディレクトリを検索しません。リポジトリ検出動作が必要な場合は、`ptool.git.discover(...)` を使用します。

例:

```lua
local repo = ptool.git.open(".")
print(repo:path())
```

## ptool.git.discover

> `v0.6.0` - 導入されました。

`ptool.git.discover(path?)` は、`path` から開始して親ディレクトリをたどってリポジトリを検索し、`Repo` オブジェクトを返します。

引数:

- `path` (文字列、オプション): 開始パス。省略した場合、現在の `ptool` ランタイム ディレクトリが使用されます。

挙動:

- 相対パスは現在の `ptool` ランタイムディレクトリから解決されます。
- これは、スクリプトがワークツリー内のサブディレクトリから実行される可能性がある場合に便利です。

例:

```lua
local repo = ptool.git.discover("src")
print(repo:root())
```

## ptool.git.clone

> `v0.6.0` - 導入されました。

`ptool.git.clone(url, path[, options])` はリポジトリのクローンを作成し、クローンされたリポジトリの `Repo` オブジェクトを返します。

引数:

- `url` (文字列、必須): リモート リポジトリ URL。
- `path` (文字列、必須): 宛先パス。
- `options` (テーブル、オプション): クローン オプション。サポートされているフィールド:
  - `branch` (文字列、オプション): クローン作成後にチェックアウトするブランチ名。
  - `bare` (ブール値、オプション): ベア リポジトリを作成するかどうか。デフォルトは`false`です。
  - `depth` (integer、任意): shallow clone の正の深さです。
  - `checkout` (boolean、任意): 選択したブランチをチェックアウトするかどうかです。既定値は `true` です。
  - `remote` (string、任意): clone したリモートに付ける名前です。既定値は `"origin"` です。
  - `tags` (string、任意): `"auto"`、`"all"`、`"none"` のいずれかです。既定値は `"auto"` です。
  - `confirm` (boolean, 任意): クローン前に確認を求めるかどうか。デフォルトは `false`。
  - `auth` (テーブル、オプション): リモート認証設定。

`auth` のフィールド:

- `kind` (文字列、必須): 認証モード。サポートされている値:
  - `"default"`: libgit2 のデフォルトの資格情報を使用します。
  - `"ssh_agent"`: ローカル SSH エージェントを通じて認証します。
  - `"ssh_key"`: SSH 秘密鍵で認証します。
  - `"userpass"`: プレーンテキストのユーザー名とパスワードを使用します。
  - `"credential_helper"`: 設定済みの Git credential helper に問い合わせます。
- `username` (string): `"ssh_agent"` と `"credential_helper"` では任意、`"ssh_key"` と `"userpass"` では必須です。
- `private_key` (string、`"ssh_key"` では必須): 秘密鍵のパスです。相対パスは現在の `ptool` ランタイムディレクトリを基準に解決されます。
- `public_key` (string、任意): 公開鍵のパスです。
- `passphrase` (string、任意): SSH 秘密鍵のパスフレーズです。
- `password` (string、`"userpass"` では必須): パスワードです。

挙動:

- 相対宛先パスは、現在の `ptool` ランタイム ディレクトリから解決されます。
- 認証オプションは、`repo:fetch(...)` および `repo:push(...)` でも使用されます。

例:

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

> `v0.6.0` - 導入されました。

`Repo` は、`ptool.git.init()`、`ptool.git.open()`、`ptool.git.discover()`、または `ptool.git.clone()` が返す、開かれた Git リポジトリのハンドルです。

これは Lua userdata として実装されています。

メソッドは以下でグループ別に説明します。既存の repository、status、staging、commit、checkout、switch、fetch、push メソッドは後方互換です。以降のセクションで説明するワークフロー API は現在未リリースです。

### path

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:path`。

`repo:path()` は、リポジトリの git ディレクトリ パスを返します。

- 戻り値: `string`。

注意:

- 非ベア リポジトリの場合、これは通常、`.git` ディレクトリです。
- ベア リポジトリの場合、これはリポジトリ ディレクトリ自体です。

### root

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:root`。

`repo:root()` は、ワークツリーのルート ディレクトリを返します。

- 戻り値: `string|nil`。

注意:

- これにより、ベア リポジトリの場合は `nil` が返されます。

### is_bare

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:is_bare`。

`repo:is_bare()` は、リポジトリがベアかどうかを報告します。

- 戻り値: `boolean`。

### head

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:head`。

`repo:head()` は、HEAD 情報を以下のテーブルとして返します。

- `oid` (文字列|nil): 現在のコミット OID (使用可能な場合)。
- `shorthand` (文字列|nil): ブランチ名などの HEAD の短縮名。
- `detached` (ブール値): HEAD が切り離されているかどうか。
- `unborn` (ブール値): リポジトリに初期コミットがまだ存在しないかどうか。

例:

```lua
local head = repo:head()
print(head.oid)
print(head.detached)
```

### current_branch

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:current_branch`。

`repo:current_branch()` は、現在のローカル ブランチ名を返します。

- 戻り値: `string|nil`。

注意:

- HEAD が切り離された場合、これは `nil` を返します。
- これは、最初のコミット前の unborn ブランチでも `nil` を返します。

### status

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:status`。

`repo:status([options])` はリポジトリのステータスを要約し、以下を含むテーブルを返します。

- `root` (文字列|nil): ワークツリーのルート ディレクトリ。
- `branch` (文字列|nil): 現在のローカル ブランチ名。
- `head` (テーブル): `repo:head()` によって返される同じ HEAD 情報。
- `upstream` (文字列|nil): 構成された場合の上流ブランチ名。
- `ahead` (整数): アップストリームよりも先にあるコミットの数。
- `behind` (整数): アップストリームの背後にあるコミットの数。
- `clean` (ブール値): リポジトリに表示可能なステータス エントリがないかどうか。
- `entries` (テーブル): ステータスエントリテーブルの配列。

`entries[i]` には以下が含まれます:

- `path` (文字列): リポジトリの相対パス。
- `index_status` (文字列|nil): インデックス側のステータス。現在サポートされている値には、`"new"`、`"modified"`、`"deleted"`、`"renamed"`、`"typechange"` があります。
- `worktree_status` (文字列|nil): ワークツリー側のステータス。現在サポートされている値には、`"new"`、`"modified"`、`"deleted"`、`"renamed"`、`"typechange"`、`"ignored"` があります。
- `conflicted` (ブール値): パスが競合しているかどうか。
- `ignored` (ブール値): パスが無視されるかどうか。

`options` のフィールド:

- `include_untracked` (ブール値、オプション): 追跡されていないファイルを含めるかどうか。デフォルトは`true`です。
- `include_ignored` (ブール値、オプション): 無視されたファイルを含めるかどうか。デフォルトは`false`です。
- `recurse_untracked_dirs` (ブール値、オプション): 追跡されていないディレクトリを再帰するかどうか。デフォルトは`true`です。

例:

```lua
local st = repo:status()
print(st.clean)
print(st.branch)

for _, entry in ipairs(st.entries) do
  print(entry.path, entry.index_status, entry.worktree_status)
end
```

### is_clean

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:is_clean`。

`repo:is_clean([options])` は、リポジトリがクリーンかどうかを返します。

- `options` (テーブル、オプション): `repo:status(...)` で受け入れられるのと同じオプション。
- 戻り値: `boolean`。

### add

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:add`。

`repo:add(paths[, options])` は、インデックス内の 1 つ以上のパスをステージングします。

引数:

- `paths` (文字列|文字列[]、必須): パスまたはパスの配列。
- `options` (テーブル、オプション): 追加オプション。サポートされているフィールド:
  - `update` (ブール値、オプション): インデックスにすでに認識されているパスのみを更新します。デフォルトは`false`です。
  - `confirm` (boolean, 任意): パスをステージする前に確認を求めるかどうか。デフォルトは `false`。

挙動:

- パスはリポジトリ ワークツリーに対して相対的に解釈されます。

例:

```lua
repo:add("README.md")
repo:add({"src", "Cargo.toml"})
```

### commit

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:commit`。

`repo:commit(message[, options])` は、現在のインデックスからコミットを作成し、新しいコミット OID を返します。

引数:

- `message` (文字列、必須): コミットメッセージ。
- `options` (テーブル、オプション): コミットオプション。サポートされているフィールド:
  - `author` (テーブル、オプション): 著者の署名。
  - `committer` (テーブル、オプション): コミッターの署名。
  - `amend` (boolean、任意): 現在の HEAD commit を置き換えます。既定値は `false` です。
  - `allow_empty` (boolean、任意): ツリーが変わらない commit を許可します。後方互換性のため既定値は `true` です。
  - `confirm` (boolean, 任意): コミットを作成する前に確認を求めるかどうか。デフォルトは `false`。

署名フィールド:

- `name` (文字列、必須)
- `email` (文字列、必須)
- `time_seconds` (integer、任意): Unix タイムスタンプです。
- `offset_minutes` (integer、任意): UTC からのタイムゾーンオフセットです。

挙動:

- `author` と `committer` が省略された場合、`ptool` は構成からの Git リポジトリ ID の使用を試みます。
- ID が構成されておらず、明示的な署名も提供されていない場合は、エラーが発生します。

例:

```lua
local oid = repo:commit("Release v0.7.0", {
  author = {
    name = "Release Bot",
    email = "bot@example.com",
  },
})

print(oid)
```

### チェックアウト

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:checkout`。

`repo:checkout(rev[, options])` はリビジョンをチェックアウトします。

引数:

- `rev` (文字列、必須): ブランチ名、タグ名、コミット OID などのリビジョン式。
- `options` (テーブル、オプション): チェックアウト オプション。サポートされているフィールド:
  - `force` (ブール値、オプション): チェックアウトを強制するかどうか。デフォルトは`false`です。
  - `confirm` (boolean, 任意): リビジョンをチェックアウトする前に確認を求めるかどうか。デフォルトは `false`。

挙動:

- これにより、`rev` が名前付き参照に解決されない場合に HEAD が切り離される可能性があります。

### switch

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:switch`。

`repo:switch(branch[, options])` は HEAD をローカル ブランチに切り替えます。

引数:

- `branch` (文字列、必須): ローカルブランチ名。
- `options` (テーブル、オプション): Switch オプション。サポートされているフィールド:
  - `create` (ブール値、オプション): ブランチを最初に作成するかどうか。デフォルトは`false`です。
  - `force` (ブール値、オプション): チェックアウトを強制するかどうか。デフォルトは`false`です。
  - `start_point` (文字列、オプション): `create = true` の時点から分岐するリビジョン。デフォルトは`HEAD`です。
  - `track` (string、任意): 新しいブランチの upstream 参照です。
  - `orphan` (boolean、任意): orphan ブランチを作成します。
  - `confirm` (boolean, 任意): ブランチを切り替える前に確認を求めるかどうか。デフォルトは `false`。

例:

```lua
repo:switch("release")
repo:switch("release-next", {
  create = true,
  start_point = "origin/main",
})
```

### fetch

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:fetch`。

`repo:fetch([remote[, options]])` はリモートからフェッチし、転送統計を返します。

引数:

- `remote` (文字列、オプション): リモート名。デフォルトは`"origin"`です。
- `options` (テーブル、オプション): Fetch オプション。サポートされているフィールド:
  - `refspecs` (string|string[]、オプション): 1 つの refspec または refspec の配列。
  - `depth` (integer、任意): shallow fetch の正の深さです。
  - `prune` (boolean、任意): 古い remote-tracking 参照を削除します。
  - `tags` (string、任意): `"auto"`、`"all"`、`"none"` のいずれかです。
  - `update_fetchhead` (boolean、任意): `FETCH_HEAD` を更新します。既定値は `true` です。
  - `confirm` (boolean, 任意): fetch 前に確認を求めるかどうか。デフォルトは `false`。
  - `auth` (テーブル、オプション): リモート認証設定。 `ptool.git.clone(...)`と同じ構造を採用しています。

戻り値:

- `received_objects` (整数)
- `indexed_objects` (整数)
- `local_objects` (整数)
- `total_objects` (整数)
- `received_bytes` (整数)
- `updated_refs` (string[])

例:

```lua
local stats = repo:fetch("origin", {
  auth = {
    kind = "ssh_agent",
  },
})

print(stats.received_objects, stats.received_bytes)
```

### push

> `v0.6.0` - 導入されました。

正規 API 名: `ptool.git.Repo:push`。

`repo:push([remote[, refspecs[, options]]])` は refs をリモートにプッシュします。

引数:

- `remote` (文字列、オプション): リモート名。デフォルトは`"origin"`です。
- `refspecs` (string|string[]、オプション): 1 つの refspec または refspec の配列。
- `options` (テーブル、オプション): Push オプション。サポートされているフィールド:
  - `force` (boolean、任意): 各 push refspec を強制します。既定値は `false` です。
  - `set_upstream` (boolean、任意): push した現在のブランチが宛先のリモートブランチを追跡するよう設定します。
  - `confirm` (boolean, 任意): push 前に確認を求めるかどうか。デフォルトは `false`。
  - `auth` (テーブル、オプション): リモート認証設定。 `ptool.git.clone(...)`と同じ構造を採用しています。

挙動:

- `refspecs` が省略された場合、`ptool` は現在のローカル ブランチをリモート上の同じ名前のブランチにプッシュしようとします。
- HEAD が切り離されているときに `refspecs` を省略すると、エラーが発生します。
- 返されるテーブルには `ok`、`refspecs`、`rejected` が含まれます。拒否された各項目には `reference` と `message` が含まれます。

例:

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```


## Git ワークフロー API

> `Unreleased` - 導入されました。

このセクションの API は、リポジトリ保守、リリース、履歴調査、共同作業、CI 自動化を対象とします。`string|string[]` と記載された引数には、文字列または密な文字列配列を渡せます。

### 共通の結果テーブル

`commit_info()` と `log()` が返す commit 情報には `oid`、`message`、`summary`、`author`、`committer`、`parent_oids` が含まれます。署名テーブルには `name`、`email`、`time_seconds`、`offset_minutes` が含まれます。

統合メソッドは次を返します：

```lua
{
  outcome = "up_to_date" | "fast_forward" | "merged" | "conflicted",
  oid = "..." | nil,
  conflicts = {
    { path = "file", ancestor = true, ours = true, theirs = true },
  },
}
```

rebase の結果では `"rebased"` または `"conflicted"` の outcome を使用し、`current` と `total` が追加されます。競合で停止した操作は対応する API で続行または中止できます。

### リポジトリと状態

```lua
repo:path() -> string
repo:root() -> string|nil
repo:is_bare() -> boolean
repo:head() -> GitHeadInfo
repo:current_branch() -> string|nil
repo:status(options?) -> GitStatusSummary
repo:is_clean(options?) -> boolean
```

`status()` と `is_clean()` のオプションは、`include_untracked`（既定値 `true`）、`include_ignored`、`recurse_untracked_dirs`（既定値 `true`）、`paths` です。`GitStatusSummary` には `root`、`branch`、`head`、`upstream`、`ahead`、`behind`、`clean`、`entries` が含まれます。

### 履歴と diff

```lua
repo:resolve(rev) -> GitObjectInfo
repo:commit_info(rev?) -> GitCommitInfo
repo:log(options?) -> GitCommitInfo[]
repo:diff(options?) -> GitDiff
repo:describe(options?) -> string|nil
```

- `resolve()` は `oid`、`kind`、`shorthand` を返します。
- `commit_info()` の既定値は `HEAD` です。
- `log()` は `rev`、`max_count`（既定値 `100`）、`skip`、`first_parent`、`reverse`、`paths` を受け付けます。
- `diff()` は `from`、`to`、`cached`、`paths`、`context_lines`（既定値 `3`）、`find_renames`（既定値 `true`）を受け付けます。`from` または `to` がない場合、適切な worktree、index、HEAD の状態を比較します。
- diff の結果には `patch`、`files_changed`、`insertions`、`deletions`、`deltas` が含まれます。各 delta には `status`、`old_path`、`new_path`、`binary` が含まれます。
- `describe()` は `rev`、`pattern`、`always`、`abbrev`（既定値 `7`）、`dirty_suffix` を受け付けます。

```lua
local commits = repo:log({
  rev = "HEAD",
  max_count = 20,
  first_parent = true,
  paths = {"crates/ptool-engine"},
})
local changes = repo:diff({ from = "v0.10.0", to = "HEAD" })
```

### ブランチ

```lua
repo:branches(options?) -> GitBranchInfo[]
repo:branch_create(name, options?) -> GitBranchInfo
repo:branch_delete(name, options?) -> nil
repo:branch_rename(old_name, new_name, options?) -> GitBranchInfo
repo:branch_set_upstream(name, upstream_or_nil, options?) -> nil
```

- `branches()` は `kind = "local" | "remote" | "all"` を受け付け、既定値は `"local"` です。
- ブランチ情報には `name`、`kind`、`oid`、`head`、`upstream`、`ahead`、`behind` が含まれます。
- `branch_create()` は `start_point`、`force`、`checkout`、`upstream`、`confirm` を受け付けます。既定の開始点は `HEAD` です。
- `branch_delete()` は `force` と `confirm` を受け付けます。現在のブランチは削除できず、未マージのブランチを削除するには `force = true` が必要です。
- `branch_rename()` は `force` と `confirm` を受け付けます。
- upstream を削除するには `branch_set_upstream()` に `nil` を渡します。オプションは `confirm` のみです。

### タグ

```lua
repo:tags(pattern?) -> GitTagInfo[]
repo:tag_create(name, target?, options?) -> GitTagInfo
repo:tag_delete(name, options?) -> nil
```

`tag_create()` の既定の対象は `HEAD` です。`message` がなければ lightweight tag、あれば annotated tag を作成します。オプションは `message`、`tagger`、`force`、`confirm` です。tagger は共通の署名フィールドを使用します。

タグ情報には `name`、`oid`、`target_oid`、`target_kind`、`annotated`、`message`、`tagger` が含まれます。`tags(pattern)` は Git の glob マッチを使用します。タグの削除はローカルリポジトリだけを変更します。リモートタグは明示的な push refspec で削除してください。

```lua
local tag = repo:tag_create("v1.0.0", "HEAD", {
  message = "Release v1.0.0",
})
repo:push("origin", "refs/tags/v1.0.0:refs/tags/v1.0.0")
```

### リモートと転送

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

リモート情報には `name`、`url`、`push_url`、`fetch_refspecs`、`push_refspecs` が含まれます。`remote_add()` は `push_url` と `confirm` を受け付けます。`remote_set_url()` に `push = true` を指定すると fetch URL ではなく push URL を変更します。削除、名前変更、URL 設定操作は `confirm` を受け付けます。

`fetch()` は `refspecs`、`auth`、`depth`、`prune`、`tags`、`update_fetchhead`、`confirm` を受け付けます。`push()` は `auth`、`force`、`set_upstream`、`confirm` を受け付けます。

`pull()` の既定値はリモート `"origin"`、現在のブランチ、`strategy = "ff_only"` です。さらに `strategy = "merge" | "rebase"`、fetch オプションの `auth`、`depth`、`prune`、`tags`、`update_fetchhead`、および `signature`、`message`、`confirm` を受け付けます。Pull の開始前にはリポジトリが clean である必要があります。

### Worktree、index、復旧

```lua
repo:add(paths, options?) -> nil
repo:restore(paths, options?) -> nil
repo:reset(rev?, options?) -> nil
repo:remove(paths, options?) -> nil
repo:clean(options?) -> string[]
```

- `restore()` は `source`（既定値 `"HEAD"`）、`staged`、`worktree`、`confirm` を受け付けます。`staged = true` だけを指定した場合、worktree は変更されません。
- `reset()` は `mode = "soft" | "mixed" | "hard"`、`force`、`confirm` を受け付けます。hard reset には `force = true` が必要です。
- `remove()` は `cached`、`force`、`confirm` を受け付けます。
- `clean()` は `dry_run`、`force`、`dirs`、`ignored`、`paths`、`confirm` を受け付けます。既定値は `dry_run = true` です。実際の削除には `dry_run = false` と `force = true` の両方が必要です。`dirs = true` でない限りディレクトリは変更されません。

```lua
local candidates = repo:clean()
repo:clean({ dry_run = false, force = true, dirs = true, confirm = true })
```

### 設定

```lua
repo:config_get(name, options?) -> string|boolean|integer|nil
repo:config_list(options?) -> GitConfigEntry[]
repo:config_set(name, value, options?) -> nil
repo:config_remove(name, options?) -> nil
```

`scope` は `"local"`、`"global"`、`"system"` のいずれかです。読み取りの既定値は利用可能な最優先値、書き込みの既定値は `"local"` です。設定項目には `name`、`value`、`scope` が含まれます。system 設定は読み取り専用です。global の `config_set()` と `config_remove()` には `confirm = true` が必要で、常に確認プロンプトが表示されます。

### Merge、cherry-pick、revert

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

`merge()` は `ff = "allow" | "only" | "never"`、`signature`、`message`、`confirm` を受け付けます。安全に abort して `ORIG_HEAD` を復元できるよう、merge、cherry-pick、revert の開始前にはリポジトリが clean である必要があります。

Cherry-pick と revert は `commit`（既定値 `true`）、`signature`、`message`、`mainline`、`confirm` を受け付けます。commit を作成せず index と worktree を更新するには `commit = false` を指定します。abort メソッドは `confirm` を受け付けます。

### Stash と rebase

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

stash index の既定値は `0` です。`stash_save()` は `include_untracked`、`include_ignored`、`keep_index`、`signature`、`confirm` を受け付けます。apply と pop は `reinstate_index` と `confirm`、drop は `confirm` を受け付けます。stash 情報には `index`、`message`、`oid` が含まれます。

`rebase()` は `upstream` が必須で、`onto`、`branch`（既定値 `"HEAD"`）、`signature`、`confirm` を受け付けます。初版では非対話型の pick 操作のみをサポートし、対話型の squash、fixup、reword、edit は利用できません。continue は `signature` と `confirm`、abort は `confirm` を受け付けます。

### 高度なリポジトリ

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

- worktree 情報には `name`、`path`、`locked`、`lock_reason`、`valid` が含まれます。add オプションは `reference`、`lock`、`checkout_existing`、`confirm` です。prune オプションは `valid`、`locked`、`working_tree`、`force`、`confirm` で、lock と unlock も `confirm` を受け付けます。
- submodule 情報には `name`、`path`、`url`、`branch`、`head_oid`、`index_oid`、`workdir_oid` が含まれます。init オプションは `overwrite`、`recursive`、`confirm`、update オプションは `init`、`recursive`、`allow_fetch`、`auth`、`confirm`、sync オプションは `recursive`、`confirm` です。
- `recursive = true` でない限り、submodule の再帰処理は無効です。
- `blame()` は `newest`、`oldest`、`min_line`、`max_line`、`first_parent`、コピー／移動追跡フラグ、`ignore_whitespace`、`use_mailmap` を受け付けます。各 hunk には `final_start_line`、`original_start_line`、`line_count`、`commit_oid`、`author`、`origin_path`、`boundary` が含まれます。

## 安全性と互換性

`confirm` を受け付けるすべての変更メソッドで、既定値は `false` です。`force` と `confirm` は異なる意図を表します。`force` は通常拒否される動作を有効にし、`confirm` は有効な操作を実行する前にユーザーへ確認します。

既存の既定値は互換性を維持します。既定のリモートは `"origin"`、refspecs を省略した push は現在のブランチを送信し、status は未追跡ファイルを含み、`allow_empty = false` を指定しない限り空の commit は許可されます。

リモートタグの削除は push refspec で表します：

```lua
repo:push("origin", ":refs/tags/v1.0.0", { confirm = true })
```
