mod ansi;
mod command_echo;
mod datetime;
mod db;
mod exec;
mod fs;
mod git;
mod http;
mod inspect;
mod json;
mod lua_api;
mod lua_error;
mod lua_world;
mod net;
mod path;
mod proc;
mod prompt;
mod re;
mod redis;
mod runner;
mod script_args;
mod semver;
mod sh;
mod ssh;
mod strings;
mod tbl;
mod template;
mod toml;
mod tui;
mod version;
mod yaml;
mod zip;

pub use lua_world::{LuaWorld, LuaWorldConfig, RunConfig};
pub use runner::{run_repl, run_repl_with_console, run_script, run_script_with_console};

pub fn format_error_report(err: &(dyn std::error::Error + 'static)) -> String {
    lua_error::render_any_error(err)
}
