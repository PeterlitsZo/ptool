use mlua::{Lua, String as LuaString, Table, Value, Variadic};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn install_ptool_module(
    lua: &Lua,
    world: Rc<RefCell<crate::LuaWorld>>,
    script_name: &str,
    script_args: &[String],
) -> mlua::Result<()> {
    let module = create_ptool_module(lua, world, script_name, script_args)?;
    lua.globals().set("ptool", module.clone())?;
    lua.globals().set("p", module)?;
    Ok(())
}

fn create_ptool_module(
    lua: &Lua,
    world: Rc<RefCell<crate::LuaWorld>>,
    script_name: &str,
    script_args: &[String],
) -> mlua::Result<Table> {
    let module = lua.create_table()?;
    let run_state = Rc::clone(&world);
    let run_fn =
        lua.create_function(move |lua, args: Variadic<Value>| run_state.borrow().run(lua, args))?;
    let run_capture_state = Rc::clone(&world);
    let run_capture_fn = lua.create_function(move |lua, args: Variadic<Value>| {
        run_capture_state.borrow().run_capture(lua, args)
    })?;
    let config_state = Rc::clone(&world);
    let config_fn =
        lua.create_function(move |_, options: Table| config_state.borrow_mut().configure(options))?;
    let use_state = Rc::clone(&world);
    let use_fn = lua.create_function(move |_, required_version: String| {
        use_state.borrow().require_version(required_version)
    })?;
    let cd_state = Rc::clone(&world);
    let cd_fn = lua.create_function(move |_, path: String| cd_state.borrow_mut().cd(path))?;
    let unindent_state = Rc::clone(&world);
    let unindent_fn =
        lua.create_function(move |_, input: String| Ok(unindent_state.borrow().unindent(input)))?;
    let inspect_state = Rc::clone(&world);
    let inspect_fn = lua.create_function(move |_, (value, options): (Value, Option<Table>)| {
        inspect_state.borrow().inspect(value, options)
    })?;
    let ask_module = create_ptool_ask_module(lua, Rc::clone(&world))?;
    let script_path_state = Rc::clone(&world);
    let script_path_fn =
        lua.create_function(move |_, ()| Ok(script_path_state.borrow().script_path_value()))?;
    let try_state = Rc::clone(&world);
    let try_fn = lua.create_function(move |lua, callback: mlua::Function| {
        try_state.borrow().try_call(lua, callback)
    })?;
    let ansi_module = create_ptool_ansi_module(lua, Rc::clone(&world))?;
    let args_module = create_ptool_args_module(lua, Rc::clone(&world), script_name, script_args)?;
    let shell_module = create_ptool_shell_module(lua, Rc::clone(&world))?;
    let hash_module = create_ptool_hash_module(lua, Rc::clone(&world))?;
    let http_module = create_ptool_http_module(lua, Rc::clone(&world))?;
    let json_module = create_ptool_json_module(lua, Rc::clone(&world))?;
    let net_module = create_ptool_net_module(lua, Rc::clone(&world))?;
    let db_module = create_ptool_db_module(lua, Rc::clone(&world))?;
    let ssh_module = create_ptool_ssh_module(lua, Rc::clone(&world))?;
    let fs_module = create_ptool_fs_module(lua, Rc::clone(&world))?;
    let log_module = create_ptool_log_module(lua, Rc::clone(&world))?;
    let os_module = create_ptool_os_module(lua, Rc::clone(&world))?;
    let path_module = create_ptool_path_module(lua, Rc::clone(&world))?;
    let platform_module = create_ptool_platform_module(lua, Rc::clone(&world))?;
    let re_module = create_ptool_re_module(lua, Rc::clone(&world))?;
    let str_module = create_ptool_str_module(lua, Rc::clone(&world))?;
    let semver_module = create_ptool_semver_module(lua, Rc::clone(&world))?;
    let template_module = create_ptool_template_module(lua, Rc::clone(&world))?;
    let toml_module = create_ptool_toml_module(lua, world)?;
    module.set("run", run_fn)?;
    module.set("run_capture", run_capture_fn)?;
    module.set("config", config_fn)?;
    module.set("use", use_fn)?;
    module.set("cd", cd_fn)?;
    module.set("unindent", unindent_fn)?;
    module.set("inspect", inspect_fn)?;
    module.set("ask", ask_module)?;
    module.set("script_path", script_path_fn)?;
    module.set("try", try_fn)?;
    module.set("ansi", ansi_module)?;
    module.set("args", args_module)?;
    module.set("sh", shell_module)?;
    module.set("hash", hash_module)?;
    module.set("http", http_module)?;
    module.set("json", json_module)?;
    module.set("net", net_module)?;
    module.set("db", db_module)?;
    module.set("ssh", ssh_module)?;
    module.set("fs", fs_module)?;
    module.set("log", log_module)?;
    module.set("os", os_module)?;
    module.set("path", path_module)?;
    module.set("platform", platform_module)?;
    module.set("re", re_module)?;
    module.set("str", str_module)?;
    module.set("semver", semver_module)?;
    module.set("template", template_module)?;
    module.set("toml", toml_module)?;
    Ok(module)
}

fn create_ptool_args_module(
    lua: &Lua,
    world: Rc<RefCell<crate::LuaWorld>>,
    script_name: &str,
    script_args: &[String],
) -> mlua::Result<Table> {
    let args_module = lua.create_table()?;
    let arg_state = Rc::clone(&world);
    let arg_fn = lua.create_function(
        move |_, (id, kind, options): (String, String, Option<Table>)| {
            arg_state
                .borrow()
                .create_script_arg_builder(id, kind, options)
        },
    )?;
    let script_name = script_name.to_owned();
    let script_args = script_args.to_vec();
    let parse_state = world;
    let parse_fn = lua.create_function(move |lua, schema: Table| {
        parse_state
            .borrow()
            .parse_script_args(lua, schema, &script_name, &script_args)
    })?;
    args_module.set("arg", arg_fn)?;
    args_module.set("parse", parse_fn)?;
    Ok(args_module)
}

fn create_ptool_ask_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let ask_module = lua.create_table()?;
    let mt = lua.create_table()?;

    let ask_state = Rc::clone(&world);
    let ask_fn = lua.create_function(move |_, (prompt, options): (String, Option<Table>)| {
        ask_state.borrow().ask(prompt, options)
    })?;
    mt.set("__call", ask_fn)?;
    ask_module.set_metatable(Some(mt))?;

    let confirm_state = Rc::clone(&world);
    let confirm_fn =
        lua.create_function(move |_, (prompt, options): (String, Option<Table>)| {
            confirm_state.borrow().ask_confirm(prompt, options)
        })?;

    let select_state = Rc::clone(&world);
    let select_fn = lua.create_function(
        move |_, (prompt, items, options): (String, Table, Option<Table>)| {
            select_state.borrow().ask_select(prompt, items, options)
        },
    )?;

    let multiselect_state = Rc::clone(&world);
    let multiselect_fn = lua.create_function(
        move |lua, (prompt, items, options): (String, Table, Option<Table>)| {
            multiselect_state
                .borrow()
                .ask_multiselect(lua, prompt, items, options)
        },
    )?;

    let secret_state = world;
    let secret_fn = lua.create_function(move |_, (prompt, options): (String, Option<Table>)| {
        secret_state.borrow().ask_secret(prompt, options)
    })?;

    ask_module.set("confirm", confirm_fn)?;
    ask_module.set("select", select_fn)?;
    ask_module.set("multiselect", multiselect_fn)?;
    ask_module.set("secret", secret_fn)?;
    Ok(ask_module)
}

fn create_ptool_shell_module(
    lua: &Lua,
    world: Rc<RefCell<crate::LuaWorld>>,
) -> mlua::Result<Table> {
    let shell_module = lua.create_table()?;
    let split_fn =
        lua.create_function(move |lua, input: String| world.borrow().shell_split(lua, input))?;
    shell_module.set("split", split_fn)?;
    Ok(shell_module)
}

fn create_ptool_ansi_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let ansi_module = lua.create_table()?;

    let style_state = Rc::clone(&world);
    let style_fn = lua.create_function(move |_, (text, options): (String, Option<Table>)| {
        style_state.borrow().ansi_style(text, options)
    })?;
    ansi_module.set("style", style_fn)?;

    for (name, color) in [
        ("black", "black"),
        ("red", "red"),
        ("green", "green"),
        ("yellow", "yellow"),
        ("blue", "blue"),
        ("magenta", "magenta"),
        ("cyan", "cyan"),
        ("white", "white"),
    ] {
        let color_state = Rc::clone(&world);
        let color_fn =
            lua.create_function(move |_, (text, options): (String, Option<Table>)| {
                color_state.borrow().ansi_color(text, options, color)
            })?;
        ansi_module.set(name, color_fn)?;
    }

    Ok(ansi_module)
}

fn create_ptool_http_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let http_module = lua.create_table()?;
    let request_fn =
        lua.create_function(move |lua, options: Table| world.borrow().http_request(lua, options))?;
    http_module.set("request", request_fn)?;
    Ok(http_module)
}

fn create_ptool_json_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let json_module = lua.create_table()?;
    let parse_state = Rc::clone(&world);
    let parse_fn =
        lua.create_function(move |lua, input: Value| parse_state.borrow().json_parse(lua, input))?;
    let stringify_fn =
        lua.create_function(move |lua, (value, options): (Value, Option<Table>)| {
            world.borrow().json_stringify(lua, value, options)
        })?;
    json_module.set("parse", parse_fn)?;
    json_module.set("stringify", stringify_fn)?;
    Ok(json_module)
}

fn create_ptool_net_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let net_module = lua.create_table()?;
    let parse_url_state = Rc::clone(&world);
    let parse_url_fn = lua.create_function(move |lua, input: String| {
        parse_url_state.borrow().net_parse_url(lua, input)
    })?;
    let parse_ip_state = Rc::clone(&world);
    let parse_ip_fn = lua.create_function(move |lua, input: String| {
        parse_ip_state.borrow().net_parse_ip(lua, input)
    })?;
    let parse_host_port_fn = lua.create_function(move |lua, input: String| {
        world.borrow().net_parse_host_port(lua, input)
    })?;
    net_module.set("parse_url", parse_url_fn)?;
    net_module.set("parse_ip", parse_ip_fn)?;
    net_module.set("parse_host_port", parse_host_port_fn)?;
    Ok(net_module)
}

fn create_ptool_hash_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let hash_module = lua.create_table()?;
    let sha256_state = Rc::clone(&world);
    let sha256_fn = lua
        .create_function(move |_, input: LuaString| Ok(sha256_state.borrow().hash_sha256(input)))?;
    let sha1_state = Rc::clone(&world);
    let sha1_fn =
        lua.create_function(move |_, input: LuaString| Ok(sha1_state.borrow().hash_sha1(input)))?;
    let md5_fn =
        lua.create_function(move |_, input: LuaString| Ok(world.borrow().hash_md5(input)))?;
    hash_module.set("sha256", sha256_fn)?;
    hash_module.set("sha1", sha1_fn)?;
    hash_module.set("md5", md5_fn)?;
    Ok(hash_module)
}

fn create_ptool_db_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let db_module = lua.create_table()?;
    let connect_fn =
        lua.create_function(move |_, value: Value| world.borrow().db_connect(value))?;
    db_module.set("connect", connect_fn)?;
    Ok(db_module)
}

fn create_ptool_ssh_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let ssh_module = lua.create_table()?;
    let connect_fn =
        lua.create_function(move |_, value: Value| world.borrow().ssh_connect(value))?;
    ssh_module.set("connect", connect_fn)?;
    Ok(ssh_module)
}

fn create_ptool_fs_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let fs_module = lua.create_table()?;
    let read_state = Rc::clone(&world);
    let read_fn =
        lua.create_function(move |lua, path: String| read_state.borrow().fs_read(lua, path))?;
    let write_state = Rc::clone(&world);
    let write_fn = lua.create_function(move |_, (path, content): (String, mlua::String)| {
        write_state.borrow().fs_write(path, content)
    })?;
    let mkdir_state = Rc::clone(&world);
    let mkdir_fn = lua.create_function(move |_, (path, options): (String, Option<Table>)| {
        mkdir_state.borrow().fs_mkdir(path, options)
    })?;
    let exists_state = Rc::clone(&world);
    let exists_fn =
        lua.create_function(move |_, path: String| Ok(exists_state.borrow().fs_exists(path)))?;
    let glob_state = Rc::clone(&world);
    let glob_fn =
        lua.create_function(move |lua, (pattern, options): (String, Option<Table>)| {
            glob_state.borrow().fs_glob(lua, pattern, options)
        })?;
    let copy_fn =
        lua.create_function(move |lua, args: Variadic<Value>| world.borrow().fs_copy(lua, args))?;
    fs_module.set("read", read_fn)?;
    fs_module.set("write", write_fn)?;
    fs_module.set("mkdir", mkdir_fn)?;
    fs_module.set("exists", exists_fn)?;
    fs_module.set("glob", glob_fn)?;
    fs_module.set("copy", copy_fn)?;
    Ok(fs_module)
}

fn create_ptool_log_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let log_module = lua.create_table()?;

    for (name, level, op) in [
        ("trace", ptool_engine::LogLevel::Trace, "ptool.log.trace"),
        ("debug", ptool_engine::LogLevel::Debug, "ptool.log.debug"),
        ("info", ptool_engine::LogLevel::Info, "ptool.log.info"),
        ("warn", ptool_engine::LogLevel::Warn, "ptool.log.warn"),
        ("error", ptool_engine::LogLevel::Error, "ptool.log.error"),
    ] {
        let log_state = Rc::clone(&world);
        let log_fn = lua.create_function(move |lua, args: Variadic<Value>| {
            log_state.borrow().log_write(lua, level, op, args)
        })?;
        log_module.set(name, log_fn)?;
    }

    Ok(log_module)
}

fn create_ptool_os_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let os_module = lua.create_table()?;
    let getenv_state = Rc::clone(&world);
    let getenv_fn =
        lua.create_function(move |_, name: String| getenv_state.borrow().os_getenv(name))?;
    let env_state = Rc::clone(&world);
    let env_fn = lua.create_function(move |lua, ()| env_state.borrow().os_env(lua))?;
    let homedir_state = Rc::clone(&world);
    let homedir_fn = lua.create_function(move |_, ()| Ok(homedir_state.borrow().os_homedir()))?;
    let tmpdir_state = Rc::clone(&world);
    let tmpdir_fn = lua.create_function(move |_, ()| Ok(tmpdir_state.borrow().os_tmpdir()))?;
    let hostname_state = Rc::clone(&world);
    let hostname_fn =
        lua.create_function(move |_, ()| Ok(hostname_state.borrow().os_hostname()))?;
    let username_state = Rc::clone(&world);
    let username_fn =
        lua.create_function(move |_, ()| Ok(username_state.borrow().os_username()))?;
    let pid_state = Rc::clone(&world);
    let pid_fn = lua.create_function(move |_, ()| Ok(pid_state.borrow().os_pid()))?;
    let exepath_state = Rc::clone(&world);
    let exepath_fn = lua.create_function(move |_, ()| Ok(exepath_state.borrow().os_exepath()))?;
    let setenv_state = Rc::clone(&world);
    let setenv_fn = lua.create_function(move |_, (name, value): (String, String)| {
        setenv_state.borrow_mut().os_setenv(name, value)
    })?;
    let unsetenv_fn =
        lua.create_function(move |_, name: String| world.borrow_mut().os_unsetenv(name))?;
    os_module.set("getenv", getenv_fn)?;
    os_module.set("env", env_fn)?;
    os_module.set("homedir", homedir_fn)?;
    os_module.set("tmpdir", tmpdir_fn)?;
    os_module.set("hostname", hostname_fn)?;
    os_module.set("username", username_fn)?;
    os_module.set("pid", pid_fn)?;
    os_module.set("exepath", exepath_fn)?;
    os_module.set("setenv", setenv_fn)?;
    os_module.set("unsetenv", unsetenv_fn)?;
    Ok(os_module)
}

fn create_ptool_platform_module(
    lua: &Lua,
    world: Rc<RefCell<crate::LuaWorld>>,
) -> mlua::Result<Table> {
    let platform_module = lua.create_table()?;
    let os_state = Rc::clone(&world);
    let os_fn = lua.create_function(move |_, ()| Ok(os_state.borrow().platform_os()))?;
    let arch_state = Rc::clone(&world);
    let arch_fn = lua.create_function(move |_, ()| Ok(arch_state.borrow().platform_arch()))?;
    let target_fn = lua.create_function(move |_, ()| Ok(world.borrow().platform_target()))?;
    platform_module.set("os", os_fn)?;
    platform_module.set("arch", arch_fn)?;
    platform_module.set("target", target_fn)?;
    Ok(platform_module)
}

fn create_ptool_toml_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let toml_module = lua.create_table()?;
    let parse_state = Rc::clone(&world);
    let parse_fn =
        lua.create_function(move |lua, input: Value| parse_state.borrow().toml_parse(lua, input))?;
    let get_state = Rc::clone(&world);
    let get_fn = lua.create_function(move |lua, (input, path): (Value, Value)| {
        get_state.borrow().toml_get(lua, input, path)
    })?;
    let set_state = Rc::clone(&world);
    let set_fn = lua.create_function(move |_, (input, path, value): (Value, Value, Value)| {
        set_state.borrow().toml_set(input, path, value)
    })?;
    let remove_state = Rc::clone(&world);
    let remove_fn = lua.create_function(move |_, (input, path): (Value, Value)| {
        remove_state.borrow().toml_remove(input, path)
    })?;
    let stringify_state = Rc::clone(&world);
    let stringify_fn =
        lua.create_function(move |_, value: Value| stringify_state.borrow().toml_stringify(value))?;
    toml_module.set("parse", parse_fn)?;
    toml_module.set("get", get_fn)?;
    toml_module.set("set", set_fn)?;
    toml_module.set("remove", remove_fn)?;
    toml_module.set("stringify", stringify_fn)?;
    Ok(toml_module)
}

fn create_ptool_path_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let path_module = lua.create_table()?;
    let join_state = Rc::clone(&world);
    let join_fn = lua.create_function(move |_, segments: Variadic<String>| {
        join_state.borrow().path_join(segments)
    })?;
    let normalize_state = Rc::clone(&world);
    let normalize_fn =
        lua.create_function(move |_, path: String| normalize_state.borrow().path_normalize(path))?;
    let abspath_state = Rc::clone(&world);
    let abspath_fn = lua.create_function(move |_, args: Variadic<String>| {
        abspath_state.borrow().path_abspath(args)
    })?;
    let relpath_state = Rc::clone(&world);
    let relpath_fn = lua.create_function(move |_, args: Variadic<String>| {
        relpath_state.borrow().path_relpath(args)
    })?;
    let isabs_state = Rc::clone(&world);
    let isabs_fn =
        lua.create_function(move |_, path: String| isabs_state.borrow().path_isabs(path))?;
    let dirname_state = Rc::clone(&world);
    let dirname_fn =
        lua.create_function(move |_, path: String| dirname_state.borrow().path_dirname(path))?;
    let basename_state = Rc::clone(&world);
    let basename_fn =
        lua.create_function(move |_, path: String| basename_state.borrow().path_basename(path))?;
    let extname_fn =
        lua.create_function(move |_, path: String| world.borrow().path_extname(path))?;
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

fn create_ptool_re_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let re_module = lua.create_table()?;
    let compile_state = Rc::clone(&world);
    let compile_fn = lua
        .create_function(move |_, args: Variadic<Value>| compile_state.borrow().re_compile(args))?;
    let escape_fn =
        lua.create_function(move |_, text: String| Ok(world.borrow().re_escape(text)))?;
    re_module.set("compile", compile_fn)?;
    re_module.set("escape", escape_fn)?;
    Ok(re_module)
}

fn create_ptool_str_module(lua: &Lua, world: Rc<RefCell<crate::LuaWorld>>) -> mlua::Result<Table> {
    let str_module = lua.create_table()?;

    let trim_state = Rc::clone(&world);
    let trim_fn =
        lua.create_function(move |_, input: String| Ok(trim_state.borrow().str_trim(input)))?;
    let trim_start_state = Rc::clone(&world);
    let trim_start_fn = lua.create_function(move |_, input: String| {
        Ok(trim_start_state.borrow().str_trim_start(input))
    })?;
    let trim_end_state = Rc::clone(&world);
    let trim_end_fn = lua
        .create_function(move |_, input: String| Ok(trim_end_state.borrow().str_trim_end(input)))?;
    let is_blank_state = Rc::clone(&world);
    let is_blank_fn = lua
        .create_function(move |_, input: String| Ok(is_blank_state.borrow().str_is_blank(input)))?;
    let starts_with_state = Rc::clone(&world);
    let starts_with_fn = lua.create_function(move |_, (input, prefix): (String, String)| {
        Ok(starts_with_state.borrow().str_starts_with(input, prefix))
    })?;
    let ends_with_state = Rc::clone(&world);
    let ends_with_fn = lua.create_function(move |_, (input, suffix): (String, String)| {
        Ok(ends_with_state.borrow().str_ends_with(input, suffix))
    })?;
    let contains_state = Rc::clone(&world);
    let contains_fn = lua.create_function(move |_, (input, needle): (String, String)| {
        Ok(contains_state.borrow().str_contains(input, needle))
    })?;
    let split_state = Rc::clone(&world);
    let split_fn = lua.create_function(
        move |lua, (input, separator, options): (String, String, Option<Table>)| {
            split_state
                .borrow()
                .str_split(lua, input, separator, options)
        },
    )?;
    let split_lines_state = Rc::clone(&world);
    let split_lines_fn =
        lua.create_function(move |lua, (input, options): (String, Option<Table>)| {
            split_lines_state
                .borrow()
                .str_split_lines(lua, input, options)
        })?;
    let join_state = Rc::clone(&world);
    let join_fn = lua.create_function(move |_, (parts, separator): (Table, String)| {
        join_state.borrow().str_join(parts, separator)
    })?;
    let replace_state = Rc::clone(&world);
    let replace_fn = lua.create_function(move |_, args: Variadic<Value>| {
        replace_state.borrow().str_replace(args)
    })?;
    let repeat_state = Rc::clone(&world);
    let repeat_fn = lua.create_function(move |_, (input, count): (String, i64)| {
        repeat_state.borrow().str_repeat(input, count)
    })?;
    let cut_prefix_state = Rc::clone(&world);
    let cut_prefix_fn = lua.create_function(move |_, (input, prefix): (String, String)| {
        Ok(cut_prefix_state.borrow().str_cut_prefix(input, prefix))
    })?;
    let cut_suffix_state = Rc::clone(&world);
    let cut_suffix_fn = lua.create_function(move |_, (input, suffix): (String, String)| {
        Ok(cut_suffix_state.borrow().str_cut_suffix(input, suffix))
    })?;
    let indent_fn = lua.create_function(
        move |_, (input, prefix, options): (String, String, Option<Table>)| {
            world.borrow().str_indent(input, prefix, options)
        },
    )?;

    str_module.set("trim", trim_fn)?;
    str_module.set("trim_start", trim_start_fn)?;
    str_module.set("trim_end", trim_end_fn)?;
    str_module.set("is_blank", is_blank_fn)?;
    str_module.set("starts_with", starts_with_fn)?;
    str_module.set("ends_with", ends_with_fn)?;
    str_module.set("contains", contains_fn)?;
    str_module.set("split", split_fn)?;
    str_module.set("split_lines", split_lines_fn)?;
    str_module.set("join", join_fn)?;
    str_module.set("replace", replace_fn)?;
    str_module.set("repeat", repeat_fn)?;
    str_module.set("cut_prefix", cut_prefix_fn)?;
    str_module.set("cut_suffix", cut_suffix_fn)?;
    str_module.set("indent", indent_fn)?;
    Ok(str_module)
}

fn create_ptool_semver_module(
    lua: &Lua,
    world: Rc<RefCell<crate::LuaWorld>>,
) -> mlua::Result<Table> {
    let semver_module = lua.create_table()?;
    let parse_state = Rc::clone(&world);
    let parse_fn =
        lua.create_function(move |_, version: Value| parse_state.borrow().semver_parse(version))?;
    let is_valid_state = Rc::clone(&world);
    let is_valid_fn = lua.create_function(move |_, version: Value| {
        Ok(is_valid_state.borrow().semver_is_valid(version))
    })?;
    let compare_state = Rc::clone(&world);
    let compare_fn = lua.create_function(move |_, (a, b): (Value, Value)| {
        compare_state.borrow().semver_compare(a, b)
    })?;
    let bump_fn = lua.create_function(
        move |_, (v, op, channel): (Value, String, Option<String>)| {
            world.borrow().semver_bump(v, op, channel)
        },
    )?;
    semver_module.set("parse", parse_fn)?;
    semver_module.set("is_valid", is_valid_fn)?;
    semver_module.set("compare", compare_fn)?;
    semver_module.set("bump", bump_fn)?;
    Ok(semver_module)
}

fn create_ptool_template_module(
    lua: &Lua,
    world: Rc<RefCell<crate::LuaWorld>>,
) -> mlua::Result<Table> {
    let template_module = lua.create_table()?;
    let render_fn = lua.create_function(move |lua, (template, context): (String, Value)| {
        world.borrow().template_render(lua, template, context)
    })?;
    template_module.set("render", render_fn)?;
    Ok(template_module)
}
