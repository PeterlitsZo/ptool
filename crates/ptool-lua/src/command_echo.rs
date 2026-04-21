use jiff::Zoned;
use owo_colors::OwoColorize;
use std::path::Path;

pub(crate) fn print_local_command_echo(user: &str, host: &str, cwd: &Path, command: &str) {
    print_command_echo(
        &[format_user_host(user, host), cwd.display().to_string()],
        command,
    );
}

pub(crate) fn print_ssh_command_echo(user: &str, host: &str, port: u16, cwd: &str, command: &str) {
    print_command_echo(
        &[
            format_ssh_target(user, host, port),
            if cwd.is_empty() {
                "<unknown remote cwd>".to_string()
            } else {
                cwd.to_string()
            },
        ],
        command,
    );
}

fn print_command_echo(segments: &[String], command: &str) {
    let time = Zoned::now().strftime("%Y-%m-%d %H:%M:%S").to_string();
    let time_segment = format!("[{time}]");

    print!("{} {}", "┌".dimmed(), time_segment.bright_black().bold(),);
    for segment in segments {
        print!(" {}", segment.cyan().bold());
    }
    print!("\n{} {}", "└".dimmed(), "$".green().bold());
    print_command_self(command);
}

fn print_command_self(command: &str) {
    let mut lines = command.split('\n');
    let Some(first) = lines.next() else {
        println!();
        return;
    };

    print!(" {}", first);
    for line in lines {
        print!("\n{} {}", "...".dimmed(), line.to_string().bold());
    }
    println!();
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
