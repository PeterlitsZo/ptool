# Template API

模板渲染辅助能力位于 `ptool.template` 和 `p.template` 下。

## ptool.template.render

> `v0.1.0` - 引入。

`ptool.template.render(template, context)` 渲染 Jinja 风格的模板字符串，并返回 渲染结果。

- `template`（string，必填）：模板源文本。
- `context`（任意可序列化 Lua 值，必填）：模板上下文。
- 返回：渲染后的字符串。

示例：

```lua
local template = ptool.unindent([[
  | {% if user.active %}
  | Hello, {{ user.name }}!
  | {% else %}
  | Inactive user: {{ user.name }}
  | {% endif %}
  | Items:
  | {% for item in items %}
  | - {{ item }}
  | {% endfor %}
]])
local result = ptool.template.render(template, {
  user = { name = "alice", active = true },
  items = { "one", "two", "three" },
})

print(result)
```

说明：

- `context` 必须能被序列化成数据值。
- `function`、`thread` 和不受支持的 `userdata` 等 Lua 值，不能作为模板上下文值。
- 缺失值采用可链式的 undefined 语义。这意味着像 `foo.bar.baz` 这样的嵌套访问， 可以安全传给 `default(...)` 之类的过滤器而不会报错。若直接渲染且没有后备值， undefined 会变成空字符串。

```lua
local template = ptool.unindent([[
  | {{ foo.bar.baz | default("N/A") }}
]])

print(ptool.template.render(template, {})) -- N/A
```

## ptool.template.write

> `v0.12.0` - 引入。

`ptool.template.write(path, template, context)` 会渲染 Jinja 风格的模板字符串，并将渲染结果直接写入文件。

- `path`（string，必填）：目标文件路径。
- `template`（string，必填）：模板源文本。
- `context`（任意可序列化 Lua 值，必填）：模板上下文。
- 返回：无。

示例：

```lua
local template = ptool.unindent([[
  | server_name = {{ server.name }}
  | port = {{ server.port }}
]])

ptool.template.write("server.conf", template, {
  server = { name = "example.com", port = 8080 },
})
```

说明：

- 渲染时使用与 `ptool.template.render(...)` 相同的上下文转换和模板语义。
- 目标文件不存在时会创建；已存在时会被截断，这与 `ptool.fs.write(...)` 的行为一致。
- 不会自动创建父目录。
