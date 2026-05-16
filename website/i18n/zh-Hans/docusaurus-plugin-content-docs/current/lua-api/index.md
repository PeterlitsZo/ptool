# Lua API 概览

`ptool` 通过 `ptool` 和 `p` 暴露了一整套功能广泛的辅助能力。

模块按领域分组。每组内的条目按字母顺序排列。

## 运行时与交互

- [ANSI API](./ansi.md)：用 ANSI 转义序列构造带样式的终端输出。
- [Args API](./args.md)：为 Lua 脚本提供命令行参数模式定义与解析。
- [Core Lua API](./core.md)：脚本生命周期、进程执行、配置以及终端辅助能力。
- [Log API](./log.md)：按 level 输出带时间戳的终端日志。
- [Shell API](./sh.md)：把 shell 风格命令行拆分成参数数组。
- [TUI API](./tui.md)：使用结构化视图树和事件循环构建简单的终端界面。

## 数据与文本

- [DateTime API](./datetime.md)：解析、比较、格式化并转换带时区支持的具体时间点。
- [Hash API](./hash.md)：计算 SHA-256、SHA-1 和 MD5 摘要。
- [JSON API](./json.md)：解析 JSON 文本，并把 Lua 值序列化为 JSON。
- [Regex API](./re.md)：编译正则，并对文本进行搜索、捕获、替换或切分。
- [SemVer API](./semver.md)：解析、比较并提升语义化版本。
- [String API](./str.md)：裁剪、拆分、拼接、替换和格式化字符串。
- [Table API](./tbl.md)：对致密顺序表执行映射、过滤和拼接。
- [Template API](./template.md)：基于 Lua 数据渲染文本模板。
- [TOML API](./toml.md)：解析、序列化、读取、更新并删除 TOML 值。
- [YAML API](./yaml.md)：解析 YAML 文本、读取嵌套值，并把 Lua 值序列化为 YAML。
- [Zip API](./zip.md)：用常见压缩格式压缩和解压 Lua 字符串。

## 文件系统与平台

- [Filesystem API](./fs.md)：读取、写入、创建和 glob 文件系统路径。
- [OS API](./os.md)：读取运行时环境变量，并查看宿主进程信息。
- [Path API](./path.md)：以纯词法方式处理路径，不触碰真实文件系统。
- [Platform API](./platform.md)：检测当前操作系统、CPU 架构和目标三元组。

## 网络与远程

- [HTTP API](./http.md)：发送 HTTP 请求并读取响应体。
- [Network API](./net.md)：解析 URL、IP 地址以及 host-port 组合。
- [SSH API](./ssh.md)：连接远程主机、运行命令、从远程主机发出 HTTP 请求并传输文件。

## 开发与存储

- [Database API](./db.md)：打开数据库连接并执行 SQL 查询。
- [Git API](./git.md)：打开仓库、查看状态，并通过 libgit2 驱动的句柄执行 clone、fetch 或 push。

把这页当作入口，然后跳转到你需要的模块页面查看完整函数参考。
