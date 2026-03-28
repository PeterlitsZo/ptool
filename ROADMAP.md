# v0.3.0

- [ ] Rename `ptool` crate to `ptool-lua`.
- [ ] Support more OS and arch.
- [ ] Make the document website better.
- [ ] How to bump version from `0.1.0` to `0.2.0-alpha.1`?
- [ ] Let `release.lua` use `p.ask`.
- [ ] Make the `install.sh` support installing a specific version, e.g.
      `install.sh 0.2.0-alpha.1`.
- [ ] Support `ptool.ssh.Connection:run_capture`.
- [ ] Make ssh connection's can also echo the command.
- [ ] Change the avatar icon of the document website.
- [ ] Make document support i18n.
- [ ] Make REPL support readline-like features.
- [ ] Fix the local variable not working in REPL.
- [ ] Why sometime I need use `host_key = { verify = "ignore" }`? I need to fix
      this.

# v0.2.0

- [ ] Add crate ptool-engine, and let the ptool-engine manage its tokio runtime.
- [ ] Support a TUI toolkit.
- [x] Support a document website.
- [ ] Better error handling.
- [ ] Remove non-essential built-in Lua modules.
- [ ] Support `ptool.map`, etc. for FP-style operations on tables.
- [x] Make the `ptool.run`'s echo style more pretty (two lines).
- [x] Support the REPL mode, with `ptool repl` subcommand.
- [x] Support detect OS and arch.
- [x] Let `ptool.fs.mkdir` support `exist_ok` option.
- [x] Add the install script.
- [x] Add README about how to install ptool, etc.
- [x] Add a `ptool.run_capture` as a shortcut.
- [ ] Support to calculate the SHA256, etc.
- [ ] Support to parse URL, IP, IP + port, etc.
- [ ] Support `p.fs.glob`, etc.
- [ ] Make `p.ssh`'s connection support `:exists`, `:is_file`, `:is_dir`, etc.

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
- [x] Support ask for user's input.
- [x] Make `p.run` can retry if failed.
- [x] Add `p.str` table for helpful string functions, such as `p.str.trim`,
  `p.str.split`, etc.
