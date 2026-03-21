use mlua::{Lua, Table, Value, Variadic};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct RuntimeConfig {
    run: crate::exec::RunDefaults,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            run: crate::exec::RunDefaults {
                echo: true,
                check: false,
                confirm: false,
            },
        }
    }
}

pub(crate) fn install_ptool_module(
    lua: &Lua,
    script_name: &str,
    script_args: &[String],
) -> mlua::Result<()> {
    let module = create_ptool_module(lua, script_name, script_args)?;
    lua.globals().set("ptool", module.clone())?;
    lua.globals().set("p", module)?;
    Ok(())
}

fn create_ptool_module(
    lua: &Lua,
    script_name: &str,
    script_args: &[String],
) -> mlua::Result<Table> {
    let module = lua.create_table()?;
    let runtime_config = Rc::new(RefCell::new(RuntimeConfig::default()));
    let run_state = Rc::clone(&runtime_config);
    let run_fn = lua.create_function(move |lua, args: Variadic<Value>| {
        let defaults = run_state.borrow().run;
        crate::exec::run_command(lua, args, defaults)
    })?;
    let config_state = Rc::clone(&runtime_config);
    let config_fn =
        lua.create_function(move |_, options: Table| apply_runtime_config(&config_state, options))?;
    let use_fn = lua.create_function(|_, required_version: String| {
        crate::version::ensure_min_ptool_version(&required_version)
    })?;
    let unindent_fn =
        lua.create_function(|_, input: String| Ok(crate::text::unindent_text(&input)))?;
    let args_module = create_ptool_args_module(lua, script_name, script_args)?;
    let shell_module = create_ptool_shell_module(lua)?;
    let http_module = create_ptool_http_module(lua)?;
    let fs_module = create_ptool_fs_module(lua)?;
    let path_module = create_ptool_path_module(lua)?;
    let re_module = create_ptool_re_module(lua)?;
    let semver_module = create_ptool_semver_module(lua)?;
    let toml_module = create_ptool_toml_module(lua)?;
    module.set("run", run_fn)?;
    module.set("config", config_fn)?;
    module.set("use", use_fn)?;
    module.set("unindent", unindent_fn)?;
    module.set("args", args_module)?;
    module.set("sh", shell_module)?;
    module.set("http", http_module)?;
    module.set("fs", fs_module)?;
    module.set("path", path_module)?;
    module.set("re", re_module)?;
    module.set("semver", semver_module)?;
    module.set("toml", toml_module)?;
    Ok(module)
}

fn apply_runtime_config(state: &Rc<RefCell<RuntimeConfig>>, options: Table) -> mlua::Result<()> {
    let mut next_run = state.borrow().run;

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

    state.borrow_mut().run = next_run;
    Ok(())
}

fn apply_run_config(run: &mut crate::exec::RunDefaults, options: Table) -> mlua::Result<()> {
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

fn create_ptool_args_module(
    lua: &Lua,
    script_name: &str,
    script_args: &[String],
) -> mlua::Result<Table> {
    let args_module = lua.create_table()?;
    let arg_fn =
        lua.create_function(|_, (id, kind, options): (String, String, Option<Table>)| {
            crate::script_args::create_script_arg_builder(id, kind, options)
        })?;
    let script_name = script_name.to_owned();
    let script_args = script_args.to_vec();
    let parse_fn = lua.create_function(move |lua, schema: Table| {
        crate::script_args::parse_script_args(lua, schema, &script_name, &script_args)
    })?;
    args_module.set("arg", arg_fn)?;
    args_module.set("parse", parse_fn)?;
    Ok(args_module)
}

fn create_ptool_shell_module(lua: &Lua) -> mlua::Result<Table> {
    let shell_module = lua.create_table()?;
    let split_fn = lua.create_function(|lua, input: String| {
        let Some(parts) = shlex::split(&input) else {
            return Err(mlua::Error::runtime("ptool.sh.split failed to parse input"));
        };
        lua.create_sequence_from(parts)
    })?;
    shell_module.set("split", split_fn)?;
    Ok(shell_module)
}

fn create_ptool_http_module(lua: &Lua) -> mlua::Result<Table> {
    let http_module = lua.create_table()?;
    let request_fn = lua.create_function(|_, options: Table| crate::http::request(options))?;
    http_module.set("request", request_fn)?;
    Ok(http_module)
}

fn create_ptool_fs_module(lua: &Lua) -> mlua::Result<Table> {
    let fs_module = lua.create_table()?;
    let read_fn = lua.create_function(|_, path: String| crate::fs::read(path))?;
    let write_fn = lua
        .create_function(|_, (path, content): (String, String)| crate::fs::write(path, content))?;
    let mkdir_fn = lua.create_function(|_, path: String| crate::fs::mkdir(path))?;
    let exists_fn = lua.create_function(|_, path: String| Ok(crate::fs::exists(path)))?;
    fs_module.set("read", read_fn)?;
    fs_module.set("write", write_fn)?;
    fs_module.set("mkdir", mkdir_fn)?;
    fs_module.set("exists", exists_fn)?;
    Ok(fs_module)
}

fn create_ptool_toml_module(lua: &Lua) -> mlua::Result<Table> {
    let toml_module = lua.create_table()?;
    let parse_fn = lua.create_function(|lua, input: Value| crate::toml::parse(lua, input))?;
    let get_fn = lua
        .create_function(|lua, (input, path): (Value, Value)| crate::toml::get(lua, input, path))?;
    let set_fn = lua.create_function(|_, (input, path, value): (Value, Value, Value)| {
        crate::toml::set(input, path, value)
    })?;
    let remove_fn =
        lua.create_function(|_, (input, path): (Value, Value)| crate::toml::remove(input, path))?;
    toml_module.set("parse", parse_fn)?;
    toml_module.set("get", get_fn)?;
    toml_module.set("set", set_fn)?;
    toml_module.set("remove", remove_fn)?;
    Ok(toml_module)
}

fn create_ptool_path_module(lua: &Lua) -> mlua::Result<Table> {
    let path_module = lua.create_table()?;
    let join_fn =
        lua.create_function(|_, segments: Variadic<String>| crate::path::join(segments))?;
    let normalize_fn = lua.create_function(|_, path: String| crate::path::normalize(path))?;
    let abspath_fn =
        lua.create_function(|_, args: Variadic<String>| crate::path::abspath_from_args(args))?;
    let relpath_fn =
        lua.create_function(|_, args: Variadic<String>| crate::path::relpath_from_args(args))?;
    let isabs_fn = lua.create_function(|_, path: String| crate::path::isabs(path))?;
    let dirname_fn = lua.create_function(|_, path: String| crate::path::dirname(path))?;
    let basename_fn = lua.create_function(|_, path: String| crate::path::basename(path))?;
    let extname_fn = lua.create_function(|_, path: String| crate::path::extname(path))?;
    path_module.set("join", join_fn)?;
    path_module.set("normalize", normalize_fn)?;
    path_module.set("abspath", abspath_fn)?;
    path_module.set("relpath", relpath_fn)?;
    path_module.set("isabs", isabs_fn)?;
    path_module.set("dirname", dirname_fn)?;
    path_module.set("basename", basename_fn)?;
    path_module.set("extname", extname_fn)?;
    Ok(path_module)
}

fn create_ptool_re_module(lua: &Lua) -> mlua::Result<Table> {
    let re_module = lua.create_table()?;
    let compile_fn = lua.create_function(|_, args: Variadic<Value>| crate::re::compile(args))?;
    let escape_fn = lua.create_function(|_, text: String| Ok(crate::re::escape(&text)))?;
    re_module.set("compile", compile_fn)?;
    re_module.set("escape", escape_fn)?;
    Ok(re_module)
}

fn create_ptool_semver_module(lua: &Lua) -> mlua::Result<Table> {
    let semver_module = lua.create_table()?;
    let parse_fn = lua.create_function(|_, version: Value| crate::semver::parse(version))?;
    let is_valid_fn =
        lua.create_function(|_, version: Value| Ok(crate::semver::is_valid(version)))?;
    let compare_fn =
        lua.create_function(|_, (a, b): (Value, Value)| crate::semver::compare(a, b))?;
    let bump_fn = lua.create_function(|_, (v, op): (Value, String)| crate::semver::bump(v, op))?;
    semver_module.set("parse", parse_fn)?;
    semver_module.set("is_valid", is_valid_fn)?;
    semver_module.set("compare", compare_fn)?;
    semver_module.set("bump", bump_fn)?;
    Ok(semver_module)
}
