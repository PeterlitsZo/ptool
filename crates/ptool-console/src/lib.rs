use jiff::Zoned;
use owo_colors::OwoColorize;
use std::io::{self, IsTerminal, Stderr, StderrLock, Stdout, StdoutLock, Write};
use std::path::Path;

const DISPLAY_PREFIX: &str = "· ";
const REPL_PROMPT: &str = ">>> ";
const REPL_CONTINUATION_PROMPT: &str = "... ";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Console;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SshTransferKind {
    Upload,
    Download,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputTarget {
    Stdout,
    Stderr,
}

impl Console {
    pub const fn new() -> Self {
        Self
    }

    pub fn stdout(&self) -> Stdout {
        io::stdout()
    }

    pub fn stderr(&self) -> Stderr {
        io::stderr()
    }

    pub fn stdout_is_terminal(&self) -> bool {
        self.stdout().is_terminal()
    }

    pub fn stderr_is_terminal(&self) -> bool {
        self.stderr().is_terminal()
    }

    pub fn with_stdout_lock<T>(
        &self,
        f: impl FnOnce(&mut StdoutLock<'_>) -> io::Result<T>,
    ) -> io::Result<T> {
        let stdout = self.stdout();
        let mut stdout = stdout.lock();
        f(&mut stdout)
    }

    pub fn with_stderr_lock<T>(
        &self,
        f: impl FnOnce(&mut StderrLock<'_>) -> io::Result<T>,
    ) -> io::Result<T> {
        let stderr = self.stderr();
        let mut stderr = stderr.lock();
        f(&mut stderr)
    }

    pub fn write_stdout(&self, text: &str) -> io::Result<()> {
        self.with_stdout_lock(|stdout| stdout.write_all(text.as_bytes()))
    }

    pub fn write_stdout_line(&self, text: &str) -> io::Result<()> {
        self.with_stdout_lock(|stdout| {
            stdout.write_all(text.as_bytes())?;
            stdout.write_all(b"\n")
        })
    }

    pub fn write_stderr(&self, text: &str) -> io::Result<()> {
        self.with_stderr_lock(|stderr| stderr.write_all(text.as_bytes()))
    }

    pub fn write_stderr_line(&self, text: &str) -> io::Result<()> {
        self.with_stderr_lock(|stderr| {
            stderr.write_all(text.as_bytes())?;
            stderr.write_all(b"\n")
        })
    }

    pub fn flush_stdout(&self) -> io::Result<()> {
        self.with_stdout_lock(|stdout| stdout.flush())
    }

    pub fn flush_stderr(&self) -> io::Result<()> {
        self.with_stderr_lock(|stderr| stderr.flush())
    }

    pub fn log(&self, level: LogLevel, message: &str) -> io::Result<()> {
        let target = level.output_target();
        let color_enabled = target.is_terminal(self);
        let timestamp = style_if(
            color_enabled,
            &format!("[{}]", Zoned::now().strftime("%Y-%m-%d %H:%M:%S")),
            |text| text.bright_black().bold().to_string(),
        );
        let level = style_if(color_enabled, level.label(), |text| {
            level.colorize_label(text)
        });
        let rendered = if message.is_empty() {
            format!("{DISPLAY_PREFIX}{timestamp} {level}")
        } else {
            format!("{DISPLAY_PREFIX}{timestamp} {level} {message}")
        };
        target.write_line(self, &rendered)
    }

    pub fn command_echo_local(
        &self,
        user: &str,
        host: &str,
        cwd: &Path,
        command: &str,
    ) -> io::Result<()> {
        self.command_echo(
            &[format_user_host(user, host), cwd.display().to_string()],
            None,
            command,
        )
    }

    pub fn command_echo_local_shell(
        &self,
        user: &str,
        host: &str,
        cwd: &Path,
        shell: &str,
        command: &str,
    ) -> io::Result<()> {
        self.command_echo(
            &[format_user_host(user, host), cwd.display().to_string()],
            Some(shell),
            command,
        )
    }

    pub fn command_echo_ssh(
        &self,
        user: &str,
        host: &str,
        port: u16,
        cwd: &str,
        command: &str,
    ) -> io::Result<()> {
        self.command_echo(
            &[
                format_ssh_target(user, host, port),
                if cwd.is_empty() {
                    "<unknown remote cwd>".to_string()
                } else {
                    cwd.to_string()
                },
            ],
            None,
            command,
        )
    }

    pub fn repl_banner(&self, version: &str) -> io::Result<()> {
        self.write_stdout_line(&format!("ptool repl ({version})"))?;
        self.write_stdout_line("Press Ctrl-D to exit.")
    }

    pub const fn repl_prompt(&self) -> &'static str {
        REPL_PROMPT
    }

    pub const fn repl_continuation_prompt(&self) -> &'static str {
        REPL_CONTINUATION_PROMPT
    }

    pub fn repl_result(&self, rendered: &str) -> io::Result<()> {
        self.write_stdout_line(rendered)
    }

    pub fn repl_runtime_error(&self, report: &str) -> io::Result<()> {
        self.write_stdout_line(&format!("error: {report}"))
    }

    pub fn repl_exit(&self) -> io::Result<()> {
        self.write_stdout("\n")
    }

    pub fn show_help(&self, text: &str) -> io::Result<()> {
        self.write_stdout_line(text)
    }

    pub fn show_parse_error(&self, message: &str, usage: &str) -> io::Result<()> {
        self.write_stderr_line(&format!("error: {message}"))?;
        self.write_stderr_line("")?;
        self.write_stderr_line(usage)
    }

    pub fn show_script_run_error(&self, filename: &str, report: &str) -> io::Result<()> {
        self.write_stderr_line(&format!("Failed to run Lua script `{filename}`:\n{report}"))
    }

    pub fn show_repl_start_error(&self, report: &str) -> io::Result<()> {
        self.write_stderr_line(&format!("Failed to start REPL:\n{report}"))
    }

    pub fn show_version_entry(&self, label: &str, width: usize, value: &str) -> io::Result<()> {
        self.write_stdout_line(&format!("{label:>width$}  {value}"))
    }

    pub fn copy_echo(&self, src: &str, dst: &str) -> io::Result<()> {
        self.write_stdout_line(&format!("[copy] {src} -> {dst}"))
    }

    pub fn ssh_transfer_echo(
        &self,
        kind: SshTransferKind,
        target: &str,
        from: &str,
        to: &str,
    ) -> io::Result<()> {
        self.write_stdout_line(&format!("[ssh {} {target}] {from} -> {to}", kind.label()))
    }

    fn command_echo(
        &self,
        segments: &[String],
        shell: Option<&str>,
        command: &str,
    ) -> io::Result<()> {
        let color_enabled = self.stdout_is_terminal();
        let time = format!("[{}]", Zoned::now().strftime("%Y-%m-%d %H:%M:%S"));
        let mut rendered = format!(
            "{} {}",
            style_if(color_enabled, "┌", |text| text.dimmed().to_string()),
            style_if(color_enabled, &time, |text| {
                text.bright_black().bold().to_string()
            }),
        );

        for segment in segments {
            rendered.push(' ');
            rendered.push_str(&style_if(color_enabled, segment, |text| {
                text.cyan().bold().to_string()
            }));
        }

        rendered.push('\n');
        rendered.push_str(&style_if(color_enabled, "└", |text| {
            text.dimmed().to_string()
        }));
        if let Some(shell) = shell {
            rendered.push(' ');
            rendered.push_str(&style_if(color_enabled, shell, |text| {
                text.bold().to_string()
            }));
        }
        rendered.push_str(&format!(
            " {}",
            style_if(color_enabled, "$", |text| text.green().bold().to_string())
        ));
        rendered.push_str(&render_command_lines(command, color_enabled));
        self.write_stdout(&rendered)
    }
}

impl LogLevel {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }

    const fn output_target(self) -> OutputTarget {
        match self {
            Self::Error | Self::Fatal => OutputTarget::Stderr,
            Self::Trace | Self::Debug | Self::Info | Self::Warn => OutputTarget::Stdout,
        }
    }

    fn colorize_label(self, text: &str) -> String {
        match self {
            Self::Trace => text.bright_black().bold().to_string(),
            Self::Debug => text.blue().bold().to_string(),
            Self::Info => text.green().bold().to_string(),
            Self::Warn => text.yellow().bold().to_string(),
            Self::Error | Self::Fatal => text.red().bold().to_string(),
        }
    }
}

impl SshTransferKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::Download => "download",
        }
    }
}

impl OutputTarget {
    fn is_terminal(self, console: &Console) -> bool {
        match self {
            Self::Stdout => console.stdout_is_terminal(),
            Self::Stderr => console.stderr_is_terminal(),
        }
    }

    fn write_line(self, console: &Console, text: &str) -> io::Result<()> {
        match self {
            Self::Stdout => console.write_stdout_line(text),
            Self::Stderr => console.write_stderr_line(text),
        }
    }
}

fn style_if(enabled: bool, text: &str, style: impl FnOnce(&str) -> String) -> String {
    if enabled {
        style(text)
    } else {
        text.to_string()
    }
}

fn render_command_lines(command: &str, color_enabled: bool) -> String {
    let mut lines = command.split('\n');
    let Some(first) = lines.next() else {
        return "\n".to_string();
    };

    let mut rendered = format!(" {first}");
    for line in lines {
        rendered.push('\n');
        rendered.push_str(&format!(
            "{} {}",
            style_if(color_enabled, "...", |text| text.dimmed().to_string()),
            style_if(color_enabled, line, |text| text.bold().to_string()),
        ));
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
