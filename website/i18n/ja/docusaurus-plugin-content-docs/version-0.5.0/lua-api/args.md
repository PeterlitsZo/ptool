# 引数 API

CLI 引数のスキーマ定義と解析ヘルパーは `ptool.args` と `p.args` に
あります。

## ptool.args.arg

> `v0.1.0` - Introduced.

`ptool.args.arg(id, kind, options)` は、
`ptool.args.parse(...).schema.args` で使う引数ビルダーを作成します。

- `id` (string, 必須): 引数識別子。返されるテーブルでもキーになります。
- `kind` (string, 必須): 引数タイプ。サポートされる値:
  - `"flag"`: 真偽値フラグ。
  - `"string"`: 文字列オプション。
  - `"int"`: 整数オプション (`i64`)。
  - `"positional"`: 位置引数。
- `options` (table, 任意): `ptool.args.parse` の引数テーブルで使える
  `long`, `short`, `help`, `required`, `multiple`, `default` などの
  同じ任意フィールド。

このビルダーはメソッドチェーンをサポートしており、すべて自身を返します:

- `arg:long(value)` は長いオプション名を設定します。`positional`
  以外の引数でのみサポートされます。
- `arg:short(value)` は短いオプション名を設定します。`positional`
  以外の引数でのみサポートされます。
- `arg:help(value)` はヘルプテキストを設定します。
- `arg:required(value)` はその引数が必須かどうかを設定します。`value`
  を省略した場合のデフォルトは `true` です。
- `arg:multiple(value)` はその引数を繰り返し指定できるかどうかを設定します。
  `value` を省略した場合のデフォルトは `true` です。
- `arg:default(value)` はデフォルト値を設定します。`value = nil` の場合は
  デフォルト値がクリアされます。

例:

```lua
local res = ptool.args.parse({
  args = {
    ptool.args.arg("name", "string"):required(),
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("paths", "positional"):multiple(),
  }
})
```

## ptool.args.parse

> `v0.1.0` - Introduced.
>
> `v0.3.0` - Added `subcommands` support.

`ptool.args.parse(schema)` は `clap` を使ってスクリプト引数を解析し、
`id` をキーとするテーブルを返します。

スクリプト引数は `ptool run <lua_file> -- ...` の `--` 以降から取得
されます。

たとえば:

```lua
ptool.use("v0.1.0")

local res = ptool.args.parse({
    name = "test",
    about = "The test command",
    args = {
        { id = "name", kind = "string" }
    }
})

print("Hello, " .. res.name .. "!")
```

### スキーマ構造

- `name` (string, 任意): ヘルプ出力に使われるコマンド名。デフォルトは
  スクリプトファイル名です。
- `about` (string, 任意): ヘルプ説明。
- `args` (table, 任意): 引数定義の配列。各要素は次の 2 形式を
  サポートします:
  - 引数テーブル。
  - `ptool.args.arg(...)` が返すビルダーオブジェクト。
- `subcommands` (table, 任意): サブコマンド名からサブコマンドスキーマへの
  マップ。各サブコマンドスキーマは `about`, `args`, `subcommands` を
  再帰的にサポートします。

`args` または `subcommands` の少なくともどちらか一方は指定する必要が
あります。

引数テーブルのフィールド:

- `id` (string, 必須): 引数識別子。返されるテーブルでもキーになります。
- `kind` (string, 必須): 引数タイプ。サポートされる値:
  - `"flag"`: 真偽値フラグ。
  - `"string"`: 文字列オプション。
  - `"int"`: 整数オプション (`i64`)。
  - `"positional"`: 位置引数。
- `long` (string, 任意): `"name"` のような長いオプション名 (`--name`)
  です。`positional` 以外の引数では、デフォルト値を `id` から導出できます。
- `short` (string, 任意): `-v` の `"v"` のような 1 文字の短いオプション名。
- `help` (string, 任意): 引数のヘルプテキスト。
- `required` (boolean, 任意): その引数が必須かどうか。デフォルトは
  `false`。
- `multiple` (boolean, 任意): その引数を繰り返し指定できるかどうか。
  デフォルトは `false`。
- `default` (string/integer, 任意): デフォルト値。

`subcommands` が存在する場合、現在のコマンドの `args` はそのコマンド
ツリー全体で共有されるオプションとして扱われ、選択したサブコマンドの前後
どちらでも受け付けられます。

サブコマンド付きの例:

```lua
local res = ptool.args.parse({
  name = "demo",
  args = {
    ptool.args.arg("verbose", "flag", { short = "v" }),
    ptool.args.arg("config", "string"),
  },
  subcommands = {
    build = {
      args = {
        ptool.args.arg("release", "flag"),
      },
      subcommands = {
        web = {
          args = {
            ptool.args.arg("out", "string"):required(),
          },
        },
      },
    },
    clean = {
      args = {
        ptool.args.arg("all", "flag"),
      },
    },
  },
})
```

### 制約

- 以下の制約は、引数テーブルとビルダー構文の両方に適用されます。
- `positional` 以外の引数では `long` と `short` を省略できます。
  `long` を省略すると `id` が自動で使われます。
- `positional` 引数では `long`, `short`, `default` を設定できません。
- `positional.multiple = true` の場合、その引数は `args` の最後である
  必要があります。
- `multiple = true` は `string` と `positional` でのみサポートされます。
- `default` は `string` と `int` でのみサポートされ、`multiple = true`
  と同時には使えません。
- `subcommands` が存在する場合、その同じスキーマ内では `positional`
  引数は許可されません。
- トップレベルに `subcommands` がある場合、引数 id `command_path` と
  `args` は予約済みです。
- ひとつの選択されたサブコマンドパス上では、祖先と子孫のサブコマンドが
  同じ引数 `id` を再利用することはできません。値が 1 つの `args`
  テーブルにマージされるためです。

### 戻り値

Lua テーブルが返され、キーは `id`、値型は次のとおりです:

- `flag` -> `boolean`
- `string` -> `string` (`multiple = true` の場合は `string[]`)
- `int` -> `integer`
- `positional` -> `string` (`multiple = true` の場合は `string[]`)

`subcommands` が存在しない場合、戻り値は上記のようなフラットな形のまま
です。

`subcommands` が存在する場合、戻り値は次の形になります:

- トップレベル `args` の値は、トップレベルテーブル上にそのまま返されます。
- `command_path` -> `string[]`: 一致したサブコマンドパス。たとえば
  `{"build", "web"}`。
- `args` -> `table`: 一致したサブコマンドパスからマージされた引数値。

たとえば:

```lua
{
  verbose = true,
  config = "cfg.toml",
  command_path = { "build", "web" },
  args = {
    release = true,
    out = "dist",
  },
}
```
