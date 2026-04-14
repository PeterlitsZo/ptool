# v0.4.0

- [ ] Support just `#!/usr/bin/env ptool`.
- [ ] Make the `p.ask` more powerful.
- [ ] More i18n support.
- [ ] Support show the user and host in `p.run`, ssh connection's `run`, etc.
- [ ] Let `p.fs.write` and `p.fs.read` support bytes.
- [ ] Let the `glob` function support working_dir optioon.
- [ ] Support a TUI toolkit.
- [ ] Better error handling.
- [ ] Fix the local variable not working in REPL.
- [ ] Make REPL support readline-like features.
- [ ] Support `split_lines` function.
- [ ] Make the document website better.
- [ ] Change the avatar icon of the document website.
- [ ] Remove non-essential built-in Lua modules.
- [ ] Move all core logic from `ptool-lua` into `ptool-engine`.

# v0.3.0

- [x] Rename `ptool` crate to `ptool-lua`.
- [x] Support more OS and arch.
- [x] How to bump version from `0.1.0` to `0.2.0-alpha.1`?
- [x] Let `release.lua` use `p.ask`.
- [x] Make the `install.sh` support installing a specific version, e.g.
      `install.sh 0.2.0-alpha.1`.
- [x] Make the `install.sh` support installing to a specified directory, e.g.
      `install.sh --dir /usr/local/bin`.
- [x] Support `ptool.ssh.Connection:run_capture`.
- [x] Make ssh connection's can also echo the command.
- [x] Make document support i18n.
- [x] Why sometime I need use `host_key = { verify = "ignore" }`? I need to fix
      this.
- [x] Let the SSH connection support upload directories.
- [x] Support the subcommand for `ptool.args`.
- [ ] Remove `russh` and just use `ssh` command.

# v0.2.0

- [x] Add crate ptool-engine, and let the ptool-engine manage its tokio runtime,
      and move some core logic from `ptool` into `ptool-engine`.
- [x] Support a document website.
- [ ] ~~Support `ptool.map`, etc. for FP-style operations on tables.~~
- [x] Make the `ptool.run`'s echo style more pretty (two lines).
- [x] Support the REPL mode, with `ptool repl` subcommand.
- [x] Support detect OS and arch.
- [x] Let `ptool.fs.mkdir` support `exist_ok` option.
- [x] Add the install script.
- [x] Add README about how to install ptool, etc.
- [x] Add a `ptool.run_capture` as a shortcut.
- [x] Support to calculate the SHA256, etc.
- [x] Support to parse URL, IP, IP + port, etc.
- [x] Support `p.fs.glob`, etc.
- [x] Make `p.ssh`'s connection support `:exists`, `:is_file`, `:is_dir`, etc.

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
