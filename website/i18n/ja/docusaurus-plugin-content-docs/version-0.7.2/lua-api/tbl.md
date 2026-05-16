# Table API

テーブル向けヘルパーは `ptool.tbl` と `p.tbl` で利用できます。

これらの API は、`1` から始まる連続した整数キーを持つ密なリストテーブルを 対象にしています。

## ptool.tbl.map

> `Unreleased` - Introduced.

`ptool.tbl.map(list, fn)` はリストテーブルの各要素を変換し、新しいリストを 返します。

- `list` (table, 必須): 密なリストテーブル。
- `fn` (function, 必須): `(value, index)` を受け取り、`nil` 以外を返す コールバック。
- 戻り値: `table`。

動作:

- `fn` は各要素に対して順番に 1 回ずつ呼び出されます。
- `fn` が `nil` を返すと、結果に穴を作らずエラーになります。
- 入力テーブルは変更されません。

```lua
local out = p.tbl.map({ 10, 20, 30 }, function(value, index)
  return value + index
end)

print(ptool.inspect(out)) -- { 11, 22, 33 }
```

## ptool.tbl.filter

> `Unreleased` - Introduced.

`ptool.tbl.filter(list, fn)` はコールバック結果が truthy の要素だけを残し、 新しい密なリストとして返します。

- `list` (table, 必須): 密なリストテーブル。
- `fn` (function, 必須): `(value, index)` を受け取るコールバック。
- 戻り値: `table`。

動作:

- `nil` と `false` は現在の要素を除外します。
- それ以外の Lua 値は現在の要素を保持します。
- 返されるテーブルは `1` から振り直されます。

```lua
local out = p.tbl.filter({ "a", "bb", "ccc" }, function(value)
  return #value >= 2
end)

print(ptool.inspect(out)) -- { "bb", "ccc" }
```

## ptool.tbl.concat

> `Unreleased` - Introduced.

`ptool.tbl.concat(...)` は 1 個以上の密なリストテーブルを結合して新しい リストを返します。

- `...` (table, 必須): 1 個以上の密なリストテーブル。
- 戻り値: `table`。

動作:

- 引数は左から右へ順に追加されます。
- 空のリストも渡せます。
- 入力テーブルは変更されません。

```lua
local out = p.tbl.concat({ 1, 2 }, { 3 }, {})

print(ptool.inspect(out)) -- { 1, 2, 3 }
```

## ptool.tbl.join

> `Unreleased` - Introduced.

`ptool.tbl.join(...)` は `ptool.tbl.concat(...)` の別名です。

- `...` (table, 必須): 1 個以上の密なリストテーブル。
- 戻り値: `table`。

```lua
local out = p.tbl.join({ "x" }, { "y", "z" })

print(ptool.inspect(out)) -- { "x", "y", "z" }
```

## リストのルール

`ptool.tbl` は現時点で密なリストテーブルのみをサポートします。

- キーは整数でなければなりません。
- キーは `1` から始まる必要があります。
- キーは連続しており、欠番を含められません。
- 疎なテーブルや辞書スタイルのテーブルはエラーになります。
