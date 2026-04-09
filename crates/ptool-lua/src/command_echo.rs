use jiff::Zoned;
use owo_colors::OwoColorize;
use std::path::Path;

pub(crate) fn print_local_command_echo(cwd: &Path, command: &str) {
    print_command_echo(&[cwd.display().to_string()], command);
}

pub(crate) fn print_ssh_command_echo(target: &str, cwd: &str, command: &str) {
    print_command_echo(
        &[
            format!("ssh {target}"),
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
