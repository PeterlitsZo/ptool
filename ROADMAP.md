# v0.2.0

- [ ] Add crate ptool-engine, and let the ptool-engine manage its tokio runtime.
- [ ] Support a TUI toolkit.
- [ ] Support a document website.
- [ ] Better error handling.
- [ ] Remove non-essential built-in Lua modules.
- [ ] Support `ptool.map`, etc. for FP-style operations on tables.
- [ ] Make the `ptool.run`'s echo style more pretty (two lines).

# v0.1.0

- [x] Support disabling echo for `ptool.run`.
- [x] Support handling shebang.
- [x] Support sending network requests.
- [x] Support path handling.
- [x] Support file system I/O such as reading files.
- [x] Support the `p` alias.
- [x] Support regular expressions.
- [x] Support capturing stdout, stderr, and exit codes from `ptool.run`.
- [x] Support rendering ninja2 templates.
- [x] Extract Lua-related features into a `LuaWorld` struct that maintains state
  and provides related methods.
- [x] Support terminal colors.
- [x] Support database interaction.
- [x] Maintain the current directory inside `LuaWorld`, and support functions
  such as `p.cd`.
- [x] Support a subcommand `version`.
- [x] Support a simple function to "dump" table.
- [x] Support sending files with rsync or scp.
- [x] Do not change line-number if there is shebang.
- [ ] Add README about how to install ptool, etc.
- [ ] Support ask for user's input.
- [ ] Make `p.run` can retry if failed.
- [ ] Add `p.str` table for helpful string functions, such as `p.str.trim`,
  `p.str.split`, etc.
