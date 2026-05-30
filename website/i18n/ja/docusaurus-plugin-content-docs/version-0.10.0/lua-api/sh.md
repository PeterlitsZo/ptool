# シェル API

シェル解析ヘルパーは `ptool.sh` と `p.sh` にあります。

これらのヘルパーは shell-word レベルで動作します。POSIX 風のシェル規則で引数文字列を分割・引用・結合するためのものであり、パイプ、リダイレクト、コマンド置換、変数展開のような完全なシェル構文を解析するものではありません。

## ptool.sh.split

> `v0.1.0` - Introduced.

`ptool.sh.split(command)` はシェル風ルールでコマンド文字列を解析し、 引数配列を返します。

- `command` (string, 必須): 分割するコマンド文字列。
- 戻り値: `string[]`。

動作:

- これは shell words だけを解析します。シェル演算子を解釈したり、展開を実行したりはしません。

例:

```lua
local args = ptool.sh.split("clippy --all-targets -- -D warnings")
```

上の `args` は次と等価です。

```lua
{"clippy", "--all-targets", "--", "-D", "warnings"}
```

## ptool.sh.quote

> `v0.10.0` - 追加。

`ptool.sh.quote(word)` は単一の shell word を引用し、シェルコマンド文字列へ安全に埋め込めるようにします。

- `word` (string, 必須): 引用する shell word。
- 戻り値: `string`。

動作:

- 返される文字列はシェルに対して安全で、入力 word と意味的に等価です。
- 保持されるのは shell word としての意味であり、元のテキスト表記そのものではありません。

例:

```lua
local word = ptool.sh.quote("hello world")
print(word) -- 'hello world'
```

## ptool.sh.join

> `v0.10.0` - 追加。

`ptool.sh.join(words)` は引数配列をシェルコマンド文字列へ結合し、必要に応じて words を引用します。

- `words` (string[], 必須): 結合する shell words。
- 戻り値: `string`。

動作:

- 連続する words は単一の空白で結合されます。
- 出力は POSIX 風シェルへ渡すのに適しています。
- これは shell-word レベルの往復性を目指しているため、`ptool.sh.split(ptool.sh.join(words))` は `words` と等価になります。
- `ptool.sh.join(ptool.sh.split(command))` は、元のコマンド文字列をそのまま保つのではなく、引用や空白を正規化することがあります。

例:

```lua
local cmd = ptool.sh.join({"git", "commit", "-m", "hello world"})
print(cmd) -- git commit -m 'hello world'
```
