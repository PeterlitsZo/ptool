use mlua::{Lua, Table, Value, Variadic};
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
    let ask_state = Rc::clone(&world);
    let ask_fn = lua.create_function(move |_, (prompt, options): (String, Option<Table>)| {
        ask_state.borrow().ask(prompt, options)
    })?;
    let ansi_module = create_ptool_ansi_module(lua, Rc::clone(&world))?;
    let args_module = create_ptool_args_module(lua, Rc::clone(&world), script_name, script_args)?;
    let shell_module = create_ptool_shell_module(lua, Rc::clone(&world))?;
    let http_module = create_ptool_http_module(lua, Rc::clone(&world))?;
    let db_module = create_ptool_db_module(lua, Rc::clone(&world))?;
    let ssh_module = create_ptool_ssh_module(lua, Rc::clone(&world))?;
    let fs_module = create_ptool_fs_module(lua, Rc::clone(&world))?;
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
    module.set("ask", ask_fn)?;
    module.set("ansi", ansi_module)?;
    module.set("args", args_module)?;
    module.set("sh", shell_module)?;
    module.set("http", http_module)?;
    module.set("db", db_module)?;
    module.set("ssh", ssh_module)?;
    module.set("fs", fs_module)?;
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
        lua.create_function(move |_, options: Table| world.borrow().http_request(options))?;
    http_module.set("request", request_fn)?;
    Ok(http_module)
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
    let read_fn = lua.create_function(move |_, path: String| read_state.borrow().fs_read(path))?;
    let write_state = Rc::clone(&world);
    let write_fn = lua.create_function(move |_, (path, content): (String, String)| {
        write_state.borrow().fs_write(path, content)
    })?;
    let mkdir_state = Rc::clone(&world);
    let mkdir_fn = lua.create_function(move |_, (path, options): (String, Option<Table>)| {
        mkdir_state.borrow().fs_mkdir(path, options)
    })?;
    let exists_state = Rc::clone(&world);
    let exists_fn =
        lua.create_function(move |_, path: String| Ok(exists_state.borrow().fs_exists(path)))?;
    let copy_fn =
        lua.create_function(move |lua, args: Variadic<Value>| world.borrow().fs_copy(lua, args))?;
    fs_module.set("read", read_fn)?;
    fs_module.set("write", write_fn)?;
    fs_module.set("mkdir", mkdir_fn)?;
    fs_module.set("exists", exists_fn)?;
    fs_module.set("copy", copy_fn)?;
    Ok(fs_module)
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
    let remove_fn = lua.create_function(move |_, (input, path): (Value, Value)| {
        world.borrow().toml_remove(input, path)
    })?;
    toml_module.set("parse", parse_fn)?;
    toml_module.set("get", get_fn)?;
    toml_module.set("set", set_fn)?;
    toml_module.set("remove", remove_fn)?;
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
    let bump_fn =
        lua.create_function(move |_, (v, op): (Value, String)| world.borrow().semver_bump(v, op))?;
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
