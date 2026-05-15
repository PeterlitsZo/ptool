use mlua::{Function, Lua, MultiValue, String as LuaString, Table, Value, Variadic};
use ptool_engine::PtoolEngine;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct RunConfig {
    pub echo: bool,
    pub check: bool,
    pub confirm: bool,
    pub retry: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            echo: true,
            check: false,
            confirm: false,
            retry: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LuaWorldConfig {
    pub run: RunConfig,
}

#[derive(Debug)]
pub struct LuaWorld {
    current_dir: PathBuf,
    script_path: Option<String>,
    config: LuaWorldConfig,
    engine: PtoolEngine,
}

impl LuaWorld {
    pub fn new(script_name: Option<&str>) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = PtoolEngine::new();
        let script_path = script_name
            .map(|path| engine.path_runtime_abspath(path))
            .transpose()?;

        Ok(Self {
            current_dir: std::env::current_dir()?,
            script_path,
            config: LuaWorldConfig::default(),
            engine,
        })
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    pub fn script_path(&self) -> Option<&str> {
        self.script_path.as_deref()
    }

    pub fn config(&self) -> &LuaWorldConfig {
        &self.config
    }

    pub(crate) fn cd(&mut self, dir: String) -> mlua::Result<()> {
        ensure_non_empty(&dir, "ptool.cd(path)")?;
        let target = Path::new(&dir);
        let target = if target.is_absolute() {
            target.to_path_buf()
        } else {
            self.current_dir.join(target)
        };
        let target = std::fs::canonicalize(&target).map_err(|err| {
            crate::lua_error::to_mlua_error(
                crate::lua_error::LuaError::new(
                    "io_error",
                    format!("ptool.cd `{dir}` failed: {err}"),
                )
                .with_op("ptool.cd")
                .with_path(dir.clone()),
            )
        })?;
        if !target.is_dir() {
            return Err(crate::lua_error::to_mlua_error(
                crate::lua_error::LuaError::new(
                    "not_a_directory",
                    format!("ptool.cd `{dir}` failed: not a directory"),
                )
                .with_op("ptool.cd")
                .with_path(dir),
            ));
        }

        self.current_dir = target;
        Ok(())
    }

    pub(crate) fn run(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Value> {
        crate::exec::run_command(lua, args, self.current_dir(), &self.engine, self.config.run)
    }

    pub(crate) fn run_capture(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Value> {
        crate::exec::run_capture_command(
            lua,
            args,
            self.current_dir(),
            &self.engine,
            self.config.run,
        )
    }

    pub(crate) fn configure(&mut self, options: Table) -> mlua::Result<()> {
        let mut next_run = self.config.run;

        for pair in options.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let key = match key {
                Value::String(key) => key.to_str()?.to_string(),
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        "ptool.config(options)",
                        "keys must be strings",
                    ));
                }
            };

            match key.as_str() {
                "run" => {
                    let Value::Table(run_options) = value else {
                        return Err(crate::lua_error::invalid_option(
                            "ptool.config(options)",
                            "`run` must be a table",
                        ));
                    };
                    apply_run_config(&mut next_run, run_options)?;
                }
                _ => {
                    return Err(crate::lua_error::invalid_option(
                        "ptool.config(options)",
                        format!("unknown field `{key}`"),
                    ));
                }
            }
        }

        self.config.run = next_run;
        Ok(())
    }

    pub(crate) fn require_version(&self, required_version: String) -> mlua::Result<()> {
        crate::version::ensure_min_ptool_version(&self.engine, &required_version)
    }

    pub(crate) fn unindent(&self, input: String) -> String {
        self.engine.text_unindent(&input)
    }

    pub(crate) fn inspect(&self, value: Value, options: Option<Table>) -> mlua::Result<String> {
        crate::inspect::render(value, options)
    }

    pub(crate) fn ask(&self, prompt: String, options: Option<Table>) -> mlua::Result<String> {
        crate::prompt::ask(&self.engine, prompt, options)
    }

    pub(crate) fn ask_confirm(&self, prompt: String, options: Option<Table>) -> mlua::Result<bool> {
        crate::prompt::ask_confirm(&self.engine, prompt, options)
    }

    pub(crate) fn ask_select(
        &self,
        prompt: String,
        items: Table,
        options: Option<Table>,
    ) -> mlua::Result<String> {
        crate::prompt::ask_select(&self.engine, prompt, items, options)
    }

    pub(crate) fn ask_multiselect(
        &self,
        lua: &Lua,
        prompt: String,
        items: Table,
        options: Option<Table>,
    ) -> mlua::Result<Table> {
        crate::prompt::ask_multiselect(lua, &self.engine, prompt, items, options)
    }

    pub(crate) fn ask_secret(
        &self,
        prompt: String,
        options: Option<Table>,
    ) -> mlua::Result<String> {
        crate::prompt::ask_secret(&self.engine, prompt, options)
    }

    pub(crate) fn script_path_value(&self) -> Option<String> {
        self.script_path.clone()
    }

    pub(crate) fn try_call(
        &self,
        lua: &Lua,
        callback: Function,
    ) -> mlua::Result<(bool, Value, Value)> {
        match callback.call::<MultiValue>(()) {
            Ok(values) => Ok((true, pack_try_success_values(lua, values)?, Value::Nil)),
            Err(err) => Ok((
                false,
                Value::Nil,
                Value::Table(crate::lua_error::caught_error_to_table(lua, &err)?),
            )),
        }
    }

    pub(crate) fn ansi_style(&self, text: String, options: Option<Table>) -> mlua::Result<String> {
        let options = crate::ansi::style_options(options, None)?;
        Ok(self.engine.ansi_style(text, options))
    }

    pub(crate) fn ansi_color(
        &self,
        text: String,
        options: Option<Table>,
        color: &'static str,
    ) -> mlua::Result<String> {
        let options = crate::ansi::style_options(options, Some(color))?;
        Ok(self.engine.ansi_style(text, options))
    }

    pub(crate) fn log_write(
        &self,
        lua: &Lua,
        level: ptool_engine::LogLevel,
        op: &str,
        args: Variadic<Value>,
    ) -> mlua::Result<()> {
        let message = render_log_message(lua, args)?;
        self.engine
            .log(level, &message)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, op))
    }

    pub(crate) fn platform_os(&self) -> String {
        match self.engine.current_os() {
            ptool_engine::OS::Linux => "linux".to_string(),
            ptool_engine::OS::Macos => "macos".to_string(),
            ptool_engine::OS::Windows => "windows".to_string(),
        }
    }

    pub(crate) fn platform_arch(&self) -> String {
        match self.engine.current_arch() {
            ptool_engine::Arch::X86_64 => "amd64".to_string(),
            ptool_engine::Arch::Aarch64 => "arm64".to_string(),
            ptool_engine::Arch::X86 => "x86".to_string(),
            ptool_engine::Arch::Arm => "arm".to_string(),
            ptool_engine::Arch::Riscv64 => "riscv64".to_string(),
        }
    }

    pub(crate) fn platform_target(&self) -> String {
        format!("{}-{}", self.platform_os(), self.platform_arch())
    }

    pub(crate) fn os_getenv(&self, name: String) -> mlua::Result<Option<String>> {
        self.engine
            .env_get(&name)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.os.getenv"))
    }

    pub(crate) fn os_env(&self, lua: &Lua) -> mlua::Result<Table> {
        let table = lua.create_table()?;
        for (key, value) in self.engine.env_vars() {
            table.set(key, value)?;
        }
        Ok(table)
    }

    pub(crate) fn os_homedir(&self) -> Option<String> {
        self.engine.home_dir()
    }

    pub(crate) fn os_tmpdir(&self) -> String {
        self.engine.temp_dir()
    }

    pub(crate) fn os_hostname(&self) -> Option<String> {
        self.engine.current_hostname()
    }

    pub(crate) fn os_username(&self) -> Option<String> {
        self.engine.current_username()
    }

    pub(crate) fn os_pid(&self) -> i64 {
        i64::from(self.engine.current_pid())
    }

    pub(crate) fn os_exepath(&self) -> Option<String> {
        self.engine.current_exe()
    }

    pub(crate) fn os_setenv(&mut self, name: String, value: String) -> mlua::Result<()> {
        self.engine
            .env_set(&name, &value)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.os.setenv"))
    }

    pub(crate) fn os_unsetenv(&mut self, name: String) -> mlua::Result<()> {
        self.engine
            .env_unset(&name)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.os.unsetenv"))
    }

    pub(crate) fn create_script_arg_builder(
        &self,
        id: String,
        kind: String,
        options: Option<Table>,
    ) -> mlua::Result<crate::script_args::ScriptArgBuilder> {
        crate::script_args::create_script_arg_builder(id, kind, options)
    }

    pub(crate) fn parse_script_args(
        &self,
        lua: &Lua,
        schema: Table,
        script_name: &str,
        script_args: &[String],
    ) -> mlua::Result<Table> {
        crate::script_args::parse_script_args(lua, schema, script_name, script_args)
    }

    pub(crate) fn shell_split(&self, lua: &Lua, input: String) -> mlua::Result<Table> {
        let parts = self
            .engine
            .shell_split(&input)
            .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.sh.split"))?;
        lua.create_sequence_from(parts)
    }

    pub(crate) fn http_request(
        &self,
        lua: &Lua,
        options: Table,
    ) -> mlua::Result<crate::http::HttpResponse> {
        crate::http::request(lua, &self.engine, options)
    }

    pub(crate) fn json_parse(&self, lua: &Lua, input: Value) -> mlua::Result<Value> {
        crate::json::parse(lua, &self.engine, input)
    }

    pub(crate) fn json_stringify(
        &self,
        lua: &Lua,
        value: Value,
        options: Option<Table>,
    ) -> mlua::Result<String> {
        crate::json::stringify(lua, &self.engine, value, options)
    }

    pub(crate) fn yaml_parse(&self, lua: &Lua, input: Value) -> mlua::Result<Value> {
        crate::yaml::parse(lua, &self.engine, input)
    }

    pub(crate) fn yaml_get(&self, lua: &Lua, input: Value, path: Value) -> mlua::Result<Value> {
        crate::yaml::get(lua, &self.engine, input, path)
    }

    pub(crate) fn yaml_stringify(&self, lua: &Lua, value: Value) -> mlua::Result<String> {
        crate::yaml::stringify(lua, &self.engine, value)
    }

    pub(crate) fn net_parse_url(&self, lua: &Lua, input: String) -> mlua::Result<Table> {
        crate::net::parse_url(lua, &self.engine, input)
    }

    pub(crate) fn net_parse_ip(&self, lua: &Lua, input: String) -> mlua::Result<Table> {
        crate::net::parse_ip(lua, &self.engine, input)
    }

    pub(crate) fn net_parse_host_port(&self, lua: &Lua, input: String) -> mlua::Result<Table> {
        crate::net::parse_host_port(lua, &self.engine, input)
    }

    pub(crate) fn hash_sha256(&self, input: LuaString) -> String {
        self.engine.hash_sha256_hex(&input.as_bytes())
    }

    pub(crate) fn hash_sha1(&self, input: LuaString) -> String {
        self.engine.hash_sha1_hex(&input.as_bytes())
    }

    pub(crate) fn hash_md5(&self, input: LuaString) -> String {
        self.engine.hash_md5_hex(&input.as_bytes())
    }

    pub(crate) fn db_connect(&self, value: Value) -> mlua::Result<crate::db::LuaDbConnection> {
        crate::db::connect(value, self.current_dir(), &self.engine)
    }

    pub(crate) fn git_open(&self, path: Option<String>) -> mlua::Result<crate::git::LuaGitRepo> {
        crate::git::open(path, self.current_dir(), &self.engine)
    }

    pub(crate) fn git_discover(
        &self,
        path: Option<String>,
    ) -> mlua::Result<crate::git::LuaGitRepo> {
        crate::git::discover(path, self.current_dir(), &self.engine)
    }

    pub(crate) fn git_clone(
        &self,
        url: String,
        path: String,
        options: Option<Table>,
    ) -> mlua::Result<crate::git::LuaGitRepo> {
        crate::git::clone_repo(url, path, options, self.current_dir(), &self.engine)
    }

    pub(crate) fn datetime_now(
        &self,
        timezone: Option<String>,
    ) -> mlua::Result<crate::datetime::LuaDateTime> {
        crate::datetime::now(&self.engine, timezone)
    }

    pub(crate) fn datetime_parse(
        &self,
        input: String,
        options: Option<Table>,
    ) -> mlua::Result<crate::datetime::LuaDateTime> {
        crate::datetime::parse(&self.engine, input, options)
    }

    pub(crate) fn datetime_from_unix(
        &self,
        value: i64,
        options: Option<Table>,
    ) -> mlua::Result<crate::datetime::LuaDateTime> {
        crate::datetime::from_unix(&self.engine, value, options)
    }

    pub(crate) fn datetime_compare(&self, a: Value, b: Value) -> mlua::Result<i64> {
        crate::datetime::compare(&self.engine, a, b)
    }

    pub(crate) fn datetime_is_valid(
        &self,
        input: String,
        options: Option<Table>,
    ) -> mlua::Result<bool> {
        crate::datetime::is_valid(&self.engine, input, options)
    }

    pub(crate) fn ssh_connect(&self, value: Value) -> mlua::Result<crate::ssh::LuaSshConnection> {
        crate::ssh::connect(value, self.current_dir(), &self.engine)
    }

    pub(crate) fn fs_read(&self, lua: &Lua, path: String) -> mlua::Result<mlua::String> {
        crate::fs::read(lua, &self.engine, path)
    }

    pub(crate) fn fs_write(&self, path: String, content: mlua::String) -> mlua::Result<()> {
        crate::fs::write(&self.engine, path, content)
    }

    pub(crate) fn fs_mkdir(&self, path: String, options: Option<Table>) -> mlua::Result<()> {
        crate::fs::mkdir(&self.engine, path, options)
    }

    pub(crate) fn fs_exists(&self, path: String) -> bool {
        crate::fs::exists(&self.engine, path)
    }

    pub(crate) fn fs_is_file(&self, path: String) -> bool {
        crate::fs::is_file(&self.engine, path)
    }

    pub(crate) fn fs_is_dir(&self, path: String) -> bool {
        crate::fs::is_dir(&self.engine, path)
    }

    pub(crate) fn fs_remove(&self, path: String, options: Option<Table>) -> mlua::Result<()> {
        crate::fs::remove(&self.engine, path, options)
    }

    pub(crate) fn fs_glob(
        &self,
        lua: &Lua,
        pattern: String,
        options: Option<Table>,
    ) -> mlua::Result<Table> {
        crate::fs::glob(&self.engine, self.current_dir(), lua, pattern, options)
    }

    pub(crate) fn fs_copy(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        crate::fs::copy(&self.engine, lua, args)
    }

    pub(crate) fn toml_parse(&self, lua: &Lua, input: Value) -> mlua::Result<Table> {
        crate::toml::parse(lua, &self.engine, input)
    }

    pub(crate) fn toml_get(&self, lua: &Lua, input: Value, path: Value) -> mlua::Result<Value> {
        crate::toml::get(lua, &self.engine, input, path)
    }

    pub(crate) fn toml_set(&self, input: Value, path: Value, value: Value) -> mlua::Result<String> {
        crate::toml::set(&self.engine, input, path, value)
    }

    pub(crate) fn toml_remove(&self, input: Value, path: Value) -> mlua::Result<String> {
        crate::toml::remove(&self.engine, input, path)
    }

    pub(crate) fn toml_stringify(&self, value: Value) -> mlua::Result<String> {
        crate::toml::stringify(&self.engine, value)
    }

    pub(crate) fn template_render(
        &self,
        lua: &Lua,
        template: String,
        context: Value,
    ) -> mlua::Result<String> {
        crate::template::render(lua, &self.engine, template, context)
    }

    pub(crate) fn path_join(&self, segments: Variadic<String>) -> mlua::Result<String> {
        crate::path::join(&self.engine, segments)
    }

    pub(crate) fn path_normalize(&self, path: String) -> mlua::Result<String> {
        crate::path::normalize(&self.engine, path)
    }

    pub(crate) fn path_abspath(&self, args: Variadic<String>) -> mlua::Result<String> {
        crate::path::abspath_from_args(&self.engine, args, self.current_dir())
    }

    pub(crate) fn path_relpath(&self, args: Variadic<String>) -> mlua::Result<String> {
        crate::path::relpath_from_args(&self.engine, args, self.current_dir())
    }

    pub(crate) fn path_isabs(&self, path: String) -> mlua::Result<bool> {
        crate::path::isabs(&self.engine, path)
    }

    pub(crate) fn path_dirname(&self, path: String) -> mlua::Result<String> {
        crate::path::dirname(&self.engine, path)
    }

    pub(crate) fn path_basename(&self, path: String) -> mlua::Result<String> {
        crate::path::basename(&self.engine, path)
    }

    pub(crate) fn path_extname(&self, path: String) -> mlua::Result<String> {
        crate::path::extname(&self.engine, path)
    }

    pub(crate) fn str_trim(&self, input: String) -> String {
        crate::strings::trim(&self.engine, input)
    }

    pub(crate) fn str_trim_start(&self, input: String) -> String {
        crate::strings::trim_start(&self.engine, input)
    }

    pub(crate) fn str_trim_end(&self, input: String) -> String {
        crate::strings::trim_end(&self.engine, input)
    }

    pub(crate) fn str_is_blank(&self, input: String) -> bool {
        crate::strings::is_blank(&self.engine, input)
    }

    pub(crate) fn str_starts_with(&self, input: String, prefix: String) -> bool {
        crate::strings::starts_with(&self.engine, input, prefix)
    }

    pub(crate) fn str_ends_with(&self, input: String, suffix: String) -> bool {
        crate::strings::ends_with(&self.engine, input, suffix)
    }

    pub(crate) fn str_contains(&self, input: String, needle: String) -> bool {
        crate::strings::contains(&self.engine, input, needle)
    }

    pub(crate) fn str_split(
        &self,
        lua: &Lua,
        input: String,
        separator: String,
        options: Option<Table>,
    ) -> mlua::Result<Table> {
        crate::strings::split(lua, &self.engine, input, separator, options)
    }

    pub(crate) fn str_split_lines(
        &self,
        lua: &Lua,
        input: String,
        options: Option<Table>,
    ) -> mlua::Result<Table> {
        crate::strings::split_lines(lua, &self.engine, input, options)
    }

    pub(crate) fn str_join(&self, parts: Table, separator: String) -> mlua::Result<String> {
        crate::strings::join(&self.engine, parts, separator)
    }

    pub(crate) fn str_replace(&self, args: Variadic<Value>) -> mlua::Result<String> {
        crate::strings::replace(&self.engine, args)
    }

    pub(crate) fn str_repeat(&self, input: String, count: i64) -> mlua::Result<String> {
        crate::strings::repeat(&self.engine, input, count)
    }

    pub(crate) fn str_cut_prefix(&self, input: String, prefix: String) -> String {
        crate::strings::cut_prefix(&self.engine, input, prefix)
    }

    pub(crate) fn str_cut_suffix(&self, input: String, suffix: String) -> String {
        crate::strings::cut_suffix(&self.engine, input, suffix)
    }

    pub(crate) fn str_indent(
        &self,
        input: String,
        prefix: String,
        options: Option<Table>,
    ) -> mlua::Result<String> {
        crate::strings::indent(&self.engine, input, prefix, options)
    }

    pub(crate) fn tbl_map(
        &self,
        lua: &Lua,
        list: Table,
        callback: Function,
    ) -> mlua::Result<Table> {
        crate::tbl::map(lua, list, callback)
    }

    pub(crate) fn tbl_filter(
        &self,
        lua: &Lua,
        list: Table,
        callback: Function,
    ) -> mlua::Result<Table> {
        crate::tbl::filter(lua, list, callback)
    }

    pub(crate) fn tbl_concat(&self, lua: &Lua, lists: Variadic<Value>) -> mlua::Result<Table> {
        crate::tbl::concat(lua, lists)
    }

    pub(crate) fn tbl_join(&self, lua: &Lua, lists: Variadic<Value>) -> mlua::Result<Table> {
        crate::tbl::join(lua, lists)
    }

    pub(crate) fn re_compile(&self, args: Variadic<Value>) -> mlua::Result<crate::re::LuaRegex> {
        crate::re::compile(&self.engine, args)
    }

    pub(crate) fn re_escape(&self, text: String) -> String {
        crate::re::escape(&self.engine, &text)
    }

    pub(crate) fn tui_run(&self, lua: &Lua, options: Table) -> mlua::Result<Value> {
        crate::tui::run(lua, &self.engine, options)
    }

    pub(crate) fn semver_parse(&self, version: Value) -> mlua::Result<crate::semver::LuaSemVer> {
        crate::semver::parse(&self.engine, version)
    }

    pub(crate) fn semver_is_valid(&self, version: Value) -> bool {
        crate::semver::is_valid(&self.engine, version)
    }

    pub(crate) fn semver_parse_req(
        &self,
        requirement: Value,
    ) -> mlua::Result<crate::semver::LuaSemVerReq> {
        crate::semver::parse_req(&self.engine, requirement)
    }

    pub(crate) fn semver_is_valid_req(&self, requirement: Value) -> bool {
        crate::semver::is_valid_req(&self.engine, requirement)
    }

    pub(crate) fn semver_compare(&self, a: Value, b: Value) -> mlua::Result<i64> {
        crate::semver::compare(&self.engine, a, b)
    }

    pub(crate) fn semver_matches(&self, requirement: Value, version: Value) -> mlua::Result<bool> {
        crate::semver::matches(&self.engine, requirement, version)
    }

    pub(crate) fn semver_bump(
        &self,
        version: Value,
        op: String,
        channel: Option<String>,
    ) -> mlua::Result<crate::semver::LuaSemVer> {
        crate::semver::bump(&self.engine, version, op, channel)
    }
}

fn apply_run_config(run: &mut RunConfig, options: Table) -> mlua::Result<()> {
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = match key {
            Value::String(key) => key.to_str()?.to_string(),
            _ => {
                return Err(crate::lua_error::invalid_option(
                    "ptool.config(options.run)",
                    "keys must be strings",
                ));
            }
        };

        match key.as_str() {
            "echo" => run.echo = parse_config_bool(value, "ptool.config(options.run)", "echo")?,
            "check" => run.check = parse_config_bool(value, "ptool.config(options.run)", "check")?,
            "confirm" => {
                run.confirm = parse_config_bool(value, "ptool.config(options.run)", "confirm")?
            }
            "retry" => run.retry = parse_config_bool(value, "ptool.config(options.run)", "retry")?,
            _ => {
                return Err(crate::lua_error::invalid_option(
                    "ptool.config(options.run)",
                    format!("unknown field `{key}`"),
                ));
            }
        }
    }
    Ok(())
}

fn parse_config_bool(value: Value, context: &str, key: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(crate::lua_error::invalid_option(
            context,
            format!("`{key}` must be a boolean"),
        )),
    }
}

fn ensure_non_empty(input: &str, context: &str) -> mlua::Result<()> {
    if input.is_empty() {
        return Err(crate::lua_error::invalid_argument(
            context,
            "does not accept empty string",
        ));
    }
    Ok(())
}

fn pack_try_success_values(lua: &Lua, values: MultiValue) -> mlua::Result<Value> {
    match values.len() {
        0 => Ok(Value::Nil),
        1 => Ok(values.into_iter().next().unwrap_or(Value::Nil)),
        _ => lua.create_sequence_from(values).map(Value::Table),
    }
}

fn render_log_message(lua: &Lua, values: Variadic<Value>) -> mlua::Result<String> {
    let mut rendered = Vec::with_capacity(values.len());
    for value in values {
        rendered.push(render_log_value(lua, value)?);
    }
    Ok(rendered.join(" "))
}

fn render_log_value(lua: &Lua, value: Value) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(String::from_utf8_lossy(text.as_bytes().as_ref()).into_owned()),
        other => crate::inspect::render(other, Some(log_inspect_options(lua)?)),
    }
}

fn log_inspect_options(lua: &Lua) -> mlua::Result<Table> {
    let options = lua.create_table()?;
    options.set("multiline", false)?;
    Ok(options)
}
