# Table API

テーブル向けヘルパーは `ptool.tbl` と `p.tbl` で利用できます。

これらの API は、`1` から始まる連続した整数キーを持つ密なリストテーブルを 対象にしています。

## ptool.tbl.map

> `v0.6.0` - 導入されました。

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

> `v0.6.0` - 導入されました。

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

## ptool.tbl.any

> `v0.10.0` - 追加。

`ptool.tbl.any(list, fn)` は、コールバックが少なくとも 1 つの要素に対して truthy な結果を返した場合に `true` を返します。

- `list` (table, 必須): 密なリストテーブル。
- `fn` (function, 必須): `(value, index)` を受け取るコールバック。
- 戻り値: `boolean`。

動作:

- `nil` と `false` は falsy として扱われます。それ以外の Lua 値はすべて truthy として扱われます。
- truthy な結果が見つかった時点で反復は停止します。
- 空のリストは `false` を返します。

```lua
local has_even = p.tbl.any({ 1, 3, 4, 5 }, function(value)
  return value % 2 == 0
end)

print(has_even) -- true
```

## ptool.tbl.all

> `v0.10.0` - 追加。

`ptool.tbl.all(list, fn)` は、コールバックがすべての要素に対して truthy な結果を返した場合にのみ `true` を返します。

- `list` (table, 必須): 密なリストテーブル。
- `fn` (function, 必須): `(value, index)` を受け取るコールバック。
- 戻り値: `boolean`。

動作:

- `nil` と `false` は falsy として扱われます。それ以外の Lua 値はすべて truthy として扱われます。
- falsy な結果が見つかった時点で反復は停止します。
- 空のリストは `true` を返します。

```lua
local all_short = p.tbl.all({ "a", "bb", "ccc" }, function(value)
  return #value <= 3
end)

print(all_short) -- true
```

## ptool.tbl.find

> `v0.10.0` - 追加。

`ptool.tbl.find(list, fn)` は、コールバック結果が truthy になる最初の元のリスト値を返します。

- `list` (table, 必須): 密なリストテーブル。
- `fn` (function, 必須): `(value, index)` を受け取るコールバック。
- 戻り値: `any | nil`。

動作:

- `nil` と `false` は falsy として扱われます。それ以外の Lua 値はすべて truthy として扱われます。
- 最初の一致で反復は停止します。
- 空のリストと一致しない場合は `nil` を返します。

```lua
local found = p.tbl.find({ 10, 15, 20 }, function(value)
  return value > 12
end)

print(found) -- 15
```

## ptool.tbl.find_index

> `v0.10.0` - 追加。

`ptool.tbl.find_index(list, fn)` は、コールバック結果が truthy になる最初の要素の 1 始まりのインデックスを返します。

- `list` (table, 必須): 密なリストテーブル。
- `fn` (function, 必須): `(value, index)` を受け取るコールバック。
- 戻り値: `integer | nil`。

動作:

- `nil` と `false` は falsy として扱われます。それ以外の Lua 値はすべて truthy として扱われます。
- 最初の一致で反復は停止します。
- 空のリストと一致しない場合は `nil` を返します。

```lua
local index = p.tbl.find_index({ "cat", "dog", "eel" }, function(value)
  return value == "dog"
end)

print(index) -- 2
```

## ptool.tbl.count

> `v0.10.0` - 追加。

`ptool.tbl.count(list, fn)` は、コールバックが truthy な結果を返した要素数を数えます。

- `list` (table, 必須): 密なリストテーブル。
- `fn` (function, 必須): `(value, index)` を受け取るコールバック。
- 戻り値: `integer`。

動作:

- `nil` と `false` は falsy として扱われます。それ以外の Lua 値はすべて truthy として扱われます。
- 空のリストは `0` を返します。

```lua
local total = p.tbl.count({ 1, 2, 3, 4, 5 }, function(value)
  return value % 2 == 1
end)

print(total) -- 3
```

## ptool.tbl.includes

> `v0.10.0` - 追加。

`ptool.tbl.includes(list, value)` は、リストが指定した値を含む場合に `true` を返します。

- `list` (table, 必須): 密なリストテーブル。
- `value` (any, 必須): 探したい値。
- 戻り値: `boolean`。

動作:

- リストは左から右へ走査されます。
- 比較には通常の Lua の `==` セマンティクスを使います。
- 空のリストは `false` を返します。

```lua
local has_blue = p.tbl.includes({ "red", "blue", "green" }, "blue")

print(has_blue) -- true
```

## ptool.tbl.reduce

> `v0.10.0` - 追加。

`ptool.tbl.reduce(list, initial, fn)` は、リストを単一の値へ畳み込みます。

- `list` (table, 必須): 密なリストテーブル。
- `initial` (any, 必須): 明示的な初期アキュムレータ値。
- `fn` (function, 必須): `(acc, value, index)` を受け取り、`nil` 以外を返すコールバック。
- 戻り値: `any`。

動作:

- `initial` は、空でないリストでも常に必須です。
- `fn` は各要素に対して順番に 1 回ずつ呼び出されます。
- 各コールバック結果が次のアキュムレータ値になります。
- `fn` が `nil` を返した場合、アキュムレータ状態を失わないようにエラーになります。
- 空のリストは `initial` をそのまま返します。

```lua
local total = p.tbl.reduce({ 5, 10, 15 }, 0, function(acc, value)
  return acc + value
end)

print(total) -- 30
```

## ptool.tbl.flat_map

> `v0.10.0` - 追加。

`ptool.tbl.flat_map(list, fn)` は、各要素を密なリストテーブルにマップし、その返されたリストを 1 つの新しい密なリストへ連結します。

- `list` (table, 必須): 密なリストテーブル。
- `fn` (function, 必須): `(value, index)` を受け取り、密なリストテーブルを返すコールバック。
- 戻り値: `table`。

動作:

- `fn` は各要素に対して順番に 1 回ずつ呼び出されます。
- 各コールバック結果は密なリストテーブルでなければなりません。
- テーブル以外、または密でないリストを返すとエラーになります。
- 空のリストは空のリストを返します。

```lua
local out = p.tbl.flat_map({ 1, 2, 3 }, function(value)
  return { value, value * 10 }
end)

print(ptool.inspect(out)) -- { 1, 10, 2, 20, 3, 30 }
```

## ptool.tbl.concat

> `v0.6.0` - 導入されました。

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

> `v0.6.0` - 導入されました。

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
