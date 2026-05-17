# Git API

Git リポジトリ ヘルパーは、`ptool.git` および `p.git` で利用できます。

このモジュールは、`git` コマンド ライン ツールの呼び出しではなく、`git2` / `libgit2` によってサポートされます。

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
  - `auth` (テーブル、オプション): リモート認証設定。

`auth` のフィールド:

- `kind` (文字列、必須): 認証モード。サポートされている値:
  - `"default"`: libgit2 のデフォルトの資格情報を使用します。
  - `"ssh_agent"`: ローカル SSH エージェントを通じて認証します。
  - `"userpass"`: プレーンテキストのユーザー名とパスワードを使用します。
- `username` (文字列、オプション): `"ssh_agent"` のユーザー名。
- `username` (文字列、必須): `"userpass"` のユーザー名。
- `password` (文字列、必須): `"userpass"` のパスワード。

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

`Repo` は、`ptool.git.open()`、`ptool.git.discover()`、または `ptool.git.clone()` によって返される、開いている Git リポジトリ ハンドルを表します。

これは Lua userdata として実装されています。

メソッド:

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

署名フィールド:

- `name` (文字列、必須)
- `email` (文字列、必須)

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
  - `auth` (テーブル、オプション): リモート認証設定。 `ptool.git.clone(...)`と同じ構造を採用しています。

戻り値:

- `received_objects` (整数)
- `indexed_objects` (整数)
- `local_objects` (整数)
- `total_objects` (整数)
- `received_bytes` (整数)

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
  - `auth` (テーブル、オプション): リモート認証設定。 `ptool.git.clone(...)`と同じ構造を採用しています。

挙動:

- `refspecs` が省略された場合、`ptool` は現在のローカル ブランチをリモート上の同じ名前のブランチにプッシュしようとします。
- HEAD が切り離されているときに `refspecs` を省略すると、エラーが発生します。

例:

```lua
repo:push("origin", nil, {
  auth = {
    kind = "ssh_agent",
  },
})

repo:push("origin", "refs/heads/main:refs/heads/main")
```
