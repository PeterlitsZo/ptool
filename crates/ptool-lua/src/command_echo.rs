use jiff::Zoned;
use owo_colors::OwoColorize;
use ptool_console::Console;
use std::path::Path;

pub(crate) fn print_local_command_echo(
    console: &Console,
    user: &str,
    host: &str,
    cwd: &Path,
    command: &str,
) -> std::io::Result<()> {
    print_command_echo(
        console,
        &[format_user_host(user, host), cwd.display().to_string()],
        command,
    )
}

pub(crate) fn print_ssh_command_echo(
    console: &Console,
    user: &str,
    host: &str,
    port: u16,
    cwd: &str,
    command: &str,
) -> std::io::Result<()> {
    print_command_echo(
        console,
        &[
            format_ssh_target(user, host, port),
            if cwd.is_empty() {
                "<unknown remote cwd>".to_string()
            } else {
                cwd.to_string()
            },
        ],
        command,
    )
}

fn print_command_echo(
    console: &Console,
    segments: &[String],
    command: &str,
) -> std::io::Result<()> {
    let time = Zoned::now().strftime("%Y-%m-%d %H:%M:%S").to_string();
    let time_segment = format!("[{time}]");
    let mut rendered = format!("{} {}", "┌".dimmed(), time_segment.bright_black().bold());

    for segment in segments {
        rendered.push(' ');
        rendered.push_str(&segment.cyan().bold().to_string());
    }
    rendered.push('\n');
    rendered.push_str(&format!("{} {}", "└".dimmed(), "$".green().bold()));
    rendered.push_str(&render_command_self(command));
    console.write_stdout(&rendered)
}

fn render_command_self(command: &str) -> String {
    let mut lines = command.split('\n');
    let Some(first) = lines.next() else {
        return "\n".to_string();
    };

    let mut rendered = format!(" {}", first);
    for line in lines {
        rendered.push('\n');
        rendered.push_str(&format!("{} {}", "...".dimmed(), line.to_string().bold()));
    }
    rendered.push('\n');
    rendered
}

fn format_user_host(user: &str, host: &str) -> String {
    let host = if host.contains(':') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    format!("{user}@{host}")
}

fn format_ssh_target(user: &str, host: &str, port: u16) -> String {
    let target = format_user_host(user, host);
    if port == 22 {
        format!("ssh {target}")
    } else {
        format!("ssh {target}:{port}")
    }
}
