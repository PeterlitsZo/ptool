use std::io::{self, IsTerminal, Stderr, StderrLock, Stdout, StdoutLock, Write};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Console;

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
}
