# v0.9.0

- [ ] Enhance the `p.semvar`:
  - Support `Version:is_release` and `Version:is_prerelease`.
  - Support `Version:bump` method.
- [ ] Let ptool support more hash functions.
- [ ] Support `p.s3`.
- [ ] Support `ptool.fs.copy`?
- [ ] Support `p.redis`.
- [ ] Support the pipe -- `p.run("foo | bar")`.
- [ ] Let `p.git` module support `confirm` option.
- [ ] How about `p.run_shell`?

# v0.8.0

- [ ] Enhance `p.log` and `p.run`, `p.ask` to have better style?
  - The custom `prompt_prefix`? The background color?
- [x] Let the `ssh:upload`'s `remote_path` can be a directory.
- [x] Support to get the ptool's version.
- [ ] Support `p.zip.<algo-like-gzip>`.
- [ ] Let `p.run` and `p.exec` support redirecting stdin, stdout and stderr to a
      file.
- [ ] Let `p.fs.write` support appending to a file (or support `p.fs.append`?).
- [ ] Support `p.fs.open`? It returns a file userdata and support many methods.
- [ ] Support `p.proc` module. Like (?):
      ```lua
      local pids = p.proc.find({ cmdline_contains = "user-service" })
      p.proc.kill(pids)
      p.proc.wait_gone(pids, { timeout = "1s" })
      ```

# v0.7.0

- [x] Support `p.git` module.
- [x] Let `p.run` support `stdin`?
- [x] Enhance `p.use` and semvar module. Like `p.use("<= 0.10.0")`, etc?
- [x] Add a helper for SSH connection to send HTTP request?
- [x] Let `ptool --help` print the website's URL.
- [x] How about let `run_capture` have a `trim` option?
- [x] Support `p.exec`.
- [x] Let the API document have groups.

# v0.6.0

- [x] Update CI (to avoid "Node.js 20 actions are deprecated." warnings.)
- [x] Make the document website better.
- [x] Make `ptool -V` render more infomations.
- [x] Support a TUI toolkit.
- [x] Support `p.tbl` module for better Lua table functions.
  - `p.tbl.map`.
  - `p.tbl.filter`.
  - `p.tbl.join`.
  - etc.
- [x] Let document use better i18n solution.
- [x] Add `p.fs.is_file` and `p.fs.is_dir`, etc.
- [x] Bugfix: "response read failed: error decoding response body" when download
      with p.http.request.
- [x] Add `p.datetime` module?

# v0.5.0

- [x] Remove non-essential built-in Lua modules.
- [x] Add a function to get script itself's path.
- [x] Support just `#!/usr/bin/env ptool`.
- [x] Support `p.os.getenv` etc.
- [x] Make the `p.ask` more powerful.
- [x] Let the `glob` function support working_dir optioon.
- [x] Support `p.log` module.
- [x] Support `p.ask.select`, like:
  - `p.ask.select("Select bump type", { "patch", "minor", "major", "skip" })`
- [x] Support YAML.

# v0.4.0

- [x] More i18n support.
- [x] Support show the user and host in `p.run`, ssh connection's `run`, etc.
- [x] Enhance `p.http.request`.
- [x] Let `p.fs.write` and `p.fs.read` support bytes.
- [x] Better error handling.
- [x] Fix the local variable not working in REPL.
- [x] Make REPL support readline-like features.
- [x] Support `split_lines` function.
- [x] Change the avatar icon of the document website.
- [x] Move all core logic from `ptool-lua` into `ptool-engine`.
- [x] Support JSON API.
- [x] Support more TOML API.

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
- [x] Remove `russh` and just use `ssh` command.

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
