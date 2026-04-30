use mlua::{Lua, Table, UserData, UserDataMethods, Value, Variadic};
use ptool_engine::{PtoolEngine, RegexOptions, RegexPattern};

const COMPILE_SIGNATURE: &str = "ptool.re.compile(pattern, opts?)";

pub(crate) fn compile(engine: &PtoolEngine, args: Variadic<Value>) -> mlua::Result<LuaRegex> {
    let args_count = args.len();
    if args_count == 0 {
        return Err(mlua::Error::runtime(format!(
            "{COMPILE_SIGNATURE} requires `pattern`"
        )));
    }
    if args_count > 2 {
        return Err(mlua::Error::runtime(format!(
            "{COMPILE_SIGNATURE} accepts at most 2 arguments"
        )));
    }

    let pattern = parse_pattern_arg(args[0].clone())?;
    let options = parse_compile_options_arg(args.get(1).cloned())?;

    let regex = engine
        .re_compile(&pattern, options)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, COMPILE_SIGNATURE))?;
    Ok(LuaRegex { regex })
}

pub(crate) fn escape(engine: &PtoolEngine, text: &str) -> String {
    engine.re_escape(text)
}

#[derive(Clone)]
pub(crate) struct LuaRegex {
    regex: RegexPattern,
}

impl UserData for LuaRegex {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("is_match", |_, this, input: String| {
            Ok(this.regex.is_match(&input))
        });

        methods.add_method("find", |lua, this, (input, init): (String, Option<i64>)| {
            find(lua, this, &input, init)
        });

        methods.add_method("find_all", |lua, this, input: String| {
            let values: Vec<Table> = this
                .regex
                .find_all(&input)
                .into_iter()
                .map(|matched| matched_to_lua(lua, matched))
                .collect::<mlua::Result<Vec<_>>>()?;
            lua.create_sequence_from(values)
        });

        methods.add_method("captures", |lua, this, input: String| {
            let Some(captures) = this.regex.captures(&input) else {
                return Ok(Value::Nil);
            };
            Ok(Value::Table(captures_to_lua(lua, captures)?))
        });

        methods.add_method("captures_all", |lua, this, input: String| {
            let values: Vec<Table> = this
                .regex
                .captures_all(&input)
                .into_iter()
                .map(|captures| captures_to_lua(lua, captures))
                .collect::<mlua::Result<Vec<_>>>()?;
            lua.create_sequence_from(values)
        });

        methods.add_method(
            "replace",
            |_, this, (input, replacement): (String, String)| {
                Ok(this.regex.replace(&input, replacement.as_str()))
            },
        );

        methods.add_method(
            "replace_all",
            |_, this, (input, replacement): (String, String)| {
                Ok(this.regex.replace_all(&input, replacement.as_str()))
            },
        );

        methods.add_method(
            "split",
            |lua, this, (input, limit): (String, Option<i64>)| {
                let values: Vec<String> = match limit {
                    None => this.regex.split(&input, None),
                    Some(limit) => {
                        let limit = parse_split_limit(limit)?;
                        this.regex.split(&input, Some(limit))
                    }
                };
                lua.create_sequence_from(values)
            },
        );
    }
}

fn parse_pattern_arg(value: Value) -> mlua::Result<String> {
    match value {
        Value::String(pattern) => Ok(pattern.to_str()?.to_string()),
        _ => Err(mlua::Error::runtime(format!(
            "{COMPILE_SIGNATURE} requires string `pattern`"
        ))),
    }
}

fn parse_compile_options_arg(value: Option<Value>) -> mlua::Result<RegexOptions> {
    let Some(value) = value else {
        return Ok(RegexOptions::default());
    };

    match value {
        Value::Nil => Ok(RegexOptions::default()),
        Value::Table(options) => {
            let case_insensitive = options.get::<Option<bool>>("case_insensitive")?;
            Ok(RegexOptions {
                case_insensitive: case_insensitive.unwrap_or(false),
            })
        }
        _ => Err(mlua::Error::runtime(format!(
            "{COMPILE_SIGNATURE} `opts` must be a table"
        ))),
    }
}

fn find(lua: &Lua, regex: &LuaRegex, input: &str, init: Option<i64>) -> mlua::Result<Value> {
    let start = parse_find_start_offset(input, init)?;
    let Some(matched) = regex
        .regex
        .find(input, start)
        .map_err(|err| crate::lua_error::lua_error_from_engine(err, "ptool.re.Regex:find"))?
    else {
        return Ok(Value::Nil);
    };
    Ok(Value::Table(matched_to_lua(lua, matched)?))
}

fn parse_find_start_offset(input: &str, init: Option<i64>) -> mlua::Result<usize> {
    let Some(init) = init else {
        return Ok(0);
    };
    if init <= 0 {
        return Err(mlua::Error::runtime(
            "ptool.re.Regex:find(input, init) `init` must be >= 1",
        ));
    }

    let start = usize::try_from(init - 1).map_err(|_| {
        mlua::Error::runtime("ptool.re.Regex:find(input, init) `init` is too large")
    })?;
    if start > input.len() {
        return Err(mlua::Error::runtime(
            "ptool.re.Regex:find(input, init) `init` exceeds input length",
        ));
    }
    Ok(start)
}

fn parse_split_limit(limit: i64) -> mlua::Result<usize> {
    if limit <= 0 {
        return Err(mlua::Error::runtime(
            "ptool.re.Regex:split(input, limit) `limit` must be > 0",
        ));
    }
    usize::try_from(limit).map_err(|_| {
        mlua::Error::runtime("ptool.re.Regex:split(input, limit) `limit` is too large")
    })
}

fn matched_to_lua(lua: &Lua, matched: ptool_engine::RegexMatch) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.raw_set("start", to_lua_index_start(matched.start)?)?;
    table.raw_set("finish", to_lua_index_end(matched.end)?)?;
    table.raw_set("text", matched.text)?;
    Ok(table)
}

fn captures_to_lua(lua: &Lua, captures: ptool_engine::RegexCaptures) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.raw_set("full", captures.full)?;

    let groups = lua.create_table()?;
    for (group_index, group) in captures.groups.into_iter().enumerate() {
        match group {
            Some(group) => groups.raw_set(group_index + 1, group)?,
            None => groups.raw_set(group_index + 1, Value::Nil)?,
        };
    }
    table.raw_set("groups", groups)?;

    let named = lua.create_table()?;
    for (name, value) in captures.named {
        named.raw_set(name, value)?;
    }
    table.raw_set("named", named)?;

    Ok(table)
}

fn to_lua_index_start(index: usize) -> mlua::Result<i64> {
    let one_based = index.saturating_add(1);
    i64::try_from(one_based).map_err(|_| mlua::Error::runtime("ptool.re index is too large"))
}

fn to_lua_index_end(index: usize) -> mlua::Result<i64> {
    i64::try_from(index).map_err(|_| mlua::Error::runtime("ptool.re index is too large"))
}
