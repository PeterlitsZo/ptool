use mlua::{Lua, Table, Value, Variadic};
use ptool_engine::PtoolEngine;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tokio::runtime::Runtime;

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
    config: LuaWorldConfig,
    db_runtime: Rc<Runtime>,
    engine: PtoolEngine,
}

impl LuaWorld {
    pub fn new() -> std::io::Result<Self> {
        sqlx::any::install_default_drivers();
        let db_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(Self {
            current_dir: std::env::current_dir()?,
            config: LuaWorldConfig::default(),
            db_runtime: Rc::new(db_runtime),
            engine: PtoolEngine::new(),
        })
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
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
        let target = std::fs::canonicalize(&target)
            .map_err(|err| mlua::Error::runtime(format!("ptool.cd `{dir}` failed: {err}")))?;
        if !target.is_dir() {
            return Err(mlua::Error::runtime(format!(
                "ptool.cd `{dir}` failed: not a directory"
            )));
        }

        self.current_dir = target;
        Ok(())
    }

    pub(crate) fn run(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Value> {
        crate::exec::run_command(lua, args, self.current_dir(), self.config.run)
    }

    pub(crate) fn run_capture(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Value> {
        crate::exec::run_capture_command(lua, args, self.current_dir(), self.config.run)
    }

    pub(crate) fn configure(&mut self, options: Table) -> mlua::Result<()> {
        let mut next_run = self.config.run;

        for pair in options.pairs::<Value, Value>() {
            let (key, value) = pair?;
            let key = match key {
                Value::String(key) => key.to_str()?.to_string(),
                _ => {
                    return Err(mlua::Error::runtime(
                        "ptool.config(options) keys must be strings",
                    ));
                }
            };

            match key.as_str() {
                "run" => {
                    let Value::Table(run_options) = value else {
                        return Err(mlua::Error::runtime(
                            "ptool.config(options) `run` must be a table",
                        ));
                    };
                    apply_run_config(&mut next_run, run_options)?;
                }
                _ => {
                    return Err(mlua::Error::runtime(format!(
                        "ptool.config(options) unknown field `{key}`"
                    )));
                }
            }
        }

        self.config.run = next_run;
        Ok(())
    }

    pub(crate) fn require_version(&self, required_version: String) -> mlua::Result<()> {
        crate::version::ensure_min_ptool_version(&required_version)
    }

    pub(crate) fn unindent(&self, input: String) -> String {
        crate::text::unindent_text(&input)
    }

    pub(crate) fn inspect(&self, value: Value, options: Option<Table>) -> mlua::Result<String> {
        crate::inspect::render(value, options)
    }

    pub(crate) fn ask(&self, prompt: String, options: Option<Table>) -> mlua::Result<String> {
        crate::prompt::ask(prompt, options)
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
        let Some(parts) = shlex::split(&input) else {
            return Err(mlua::Error::runtime("ptool.sh.split failed to parse input"));
        };
        lua.create_sequence_from(parts)
    }

    pub(crate) fn http_request(&self, options: Table) -> mlua::Result<crate::http::HttpResponse> {
        crate::http::request(options)
    }

    pub(crate) fn db_connect(&self, value: Value) -> mlua::Result<crate::db::LuaDbConnection> {
        crate::db::connect(value, self.current_dir(), Rc::clone(&self.db_runtime))
    }

    pub(crate) fn ssh_connect(&self, value: Value) -> mlua::Result<crate::ssh::LuaSshConnection> {
        crate::ssh::connect(value, self.current_dir(), Rc::clone(&self.db_runtime))
    }

    pub(crate) fn fs_read(&self, path: String) -> mlua::Result<String> {
        crate::fs::read(path)
    }

    pub(crate) fn fs_write(&self, path: String, content: String) -> mlua::Result<()> {
        crate::fs::write(path, content)
    }

    pub(crate) fn fs_mkdir(&self, path: String) -> mlua::Result<()> {
        crate::fs::mkdir(path)
    }

    pub(crate) fn fs_exists(&self, path: String) -> bool {
        crate::fs::exists(path)
    }

    pub(crate) fn fs_copy(&self, lua: &Lua, args: Variadic<Value>) -> mlua::Result<Table> {
        crate::fs::copy(lua, args)
    }

    pub(crate) fn toml_parse(&self, lua: &Lua, input: Value) -> mlua::Result<Table> {
        crate::toml::parse(lua, input)
    }

    pub(crate) fn toml_get(&self, lua: &Lua, input: Value, path: Value) -> mlua::Result<Value> {
        crate::toml::get(lua, input, path)
    }

    pub(crate) fn toml_set(&self, input: Value, path: Value, value: Value) -> mlua::Result<String> {
        crate::toml::set(input, path, value)
    }

    pub(crate) fn toml_remove(&self, input: Value, path: Value) -> mlua::Result<String> {
        crate::toml::remove(input, path)
    }

    pub(crate) fn template_render(
        &self,
        lua: &Lua,
        template: String,
        context: Value,
    ) -> mlua::Result<String> {
        crate::template::render(lua, template, context)
    }

    pub(crate) fn path_join(&self, segments: Variadic<String>) -> mlua::Result<String> {
        crate::path::join(segments)
    }

    pub(crate) fn path_normalize(&self, path: String) -> mlua::Result<String> {
        crate::path::normalize(path)
    }

    pub(crate) fn path_abspath(&self, args: Variadic<String>) -> mlua::Result<String> {
        crate::path::abspath_from_args(args, self.current_dir())
    }

    pub(crate) fn path_relpath(&self, args: Variadic<String>) -> mlua::Result<String> {
        crate::path::relpath_from_args(args, self.current_dir())
    }

    pub(crate) fn path_isabs(&self, path: String) -> mlua::Result<bool> {
        crate::path::isabs(path)
    }

    pub(crate) fn path_dirname(&self, path: String) -> mlua::Result<String> {
        crate::path::dirname(path)
    }

    pub(crate) fn path_basename(&self, path: String) -> mlua::Result<String> {
        crate::path::basename(path)
    }

    pub(crate) fn path_extname(&self, path: String) -> mlua::Result<String> {
        crate::path::extname(path)
    }

    pub(crate) fn str_trim(&self, input: String) -> String {
        crate::strings::trim(input)
    }

    pub(crate) fn str_trim_start(&self, input: String) -> String {
        crate::strings::trim_start(input)
    }

    pub(crate) fn str_trim_end(&self, input: String) -> String {
        crate::strings::trim_end(input)
    }

    pub(crate) fn str_is_blank(&self, input: String) -> bool {
        crate::strings::is_blank(input)
    }

    pub(crate) fn str_starts_with(&self, input: String, prefix: String) -> bool {
        crate::strings::starts_with(input, prefix)
    }

    pub(crate) fn str_ends_with(&self, input: String, suffix: String) -> bool {
        crate::strings::ends_with(input, suffix)
    }

    pub(crate) fn str_contains(&self, input: String, needle: String) -> bool {
        crate::strings::contains(input, needle)
    }

    pub(crate) fn str_split(
        &self,
        lua: &Lua,
        input: String,
        separator: String,
        options: Option<Table>,
    ) -> mlua::Result<Table> {
        crate::strings::split(lua, input, separator, options)
    }

    pub(crate) fn str_split_lines(
        &self,
        lua: &Lua,
        input: String,
        options: Option<Table>,
    ) -> mlua::Result<Table> {
        crate::strings::split_lines(lua, input, options)
    }

    pub(crate) fn str_join(&self, parts: Table, separator: String) -> mlua::Result<String> {
        crate::strings::join(parts, separator)
    }

    pub(crate) fn str_replace(&self, args: Variadic<Value>) -> mlua::Result<String> {
        crate::strings::replace(args)
    }

    pub(crate) fn str_repeat(&self, input: String, count: i64) -> mlua::Result<String> {
        crate::strings::repeat(input, count)
    }

    pub(crate) fn str_cut_prefix(&self, input: String, prefix: String) -> String {
        crate::strings::cut_prefix(input, prefix)
    }

    pub(crate) fn str_cut_suffix(&self, input: String, suffix: String) -> String {
        crate::strings::cut_suffix(input, suffix)
    }

    pub(crate) fn str_indent(
        &self,
        input: String,
        prefix: String,
        options: Option<Table>,
    ) -> mlua::Result<String> {
        crate::strings::indent(input, prefix, options)
    }

    pub(crate) fn re_compile(&self, args: Variadic<Value>) -> mlua::Result<crate::re::LuaRegex> {
        crate::re::compile(args)
    }

    pub(crate) fn re_escape(&self, text: String) -> String {
        crate::re::escape(&text)
    }

    pub(crate) fn semver_parse(&self, version: Value) -> mlua::Result<crate::semver::LuaSemVer> {
        crate::semver::parse(version)
    }

    pub(crate) fn semver_is_valid(&self, version: Value) -> bool {
        crate::semver::is_valid(version)
    }

    pub(crate) fn semver_compare(&self, a: Value, b: Value) -> mlua::Result<i64> {
        crate::semver::compare(a, b)
    }

    pub(crate) fn semver_bump(
        &self,
        version: Value,
        op: String,
    ) -> mlua::Result<crate::semver::LuaSemVer> {
        crate::semver::bump(version, op)
    }
}

fn apply_run_config(run: &mut RunConfig, options: Table) -> mlua::Result<()> {
    for pair in options.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let key = match key {
            Value::String(key) => key.to_str()?.to_string(),
            _ => {
                return Err(mlua::Error::runtime(
                    "ptool.config(options.run) keys must be strings",
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
                return Err(mlua::Error::runtime(format!(
                    "ptool.config(options.run) unknown field `{key}`"
                )));
            }
        }
    }
    Ok(())
}

fn parse_config_bool(value: Value, context: &str, key: &str) -> mlua::Result<bool> {
    match value {
        Value::Boolean(value) => Ok(value),
        _ => Err(mlua::Error::runtime(format!(
            "{context} `{key}` must be a boolean"
        ))),
    }
}

fn ensure_non_empty(input: &str, context: &str) -> mlua::Result<()> {
    if input.is_empty() {
        return Err(mlua::Error::runtime(format!(
            "{context} does not accept empty string"
        )));
    }
    Ok(())
}
