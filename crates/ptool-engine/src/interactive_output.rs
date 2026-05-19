use crossterm::QueueableCommand;
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::terminal::{self, Clear, ClearType};
use std::collections::VecDeque;
use std::io::{self, IsTerminal, Read, Write};
use std::sync::mpsc::{self, Sender};
use std::thread;

const INITIAL_VISIBLE_LINES: usize = 25;
const MAX_VISIBLE_LINES: usize = 50;
const VISIBLE_LINE_EXPAND_AT: f64 = 475.0;
const DEFAULT_TERMINAL_COLUMNS: usize = 80;

#[derive(Clone)]
pub(crate) struct InteractiveOutputSink {
    sender: Sender<Vec<u8>>,
}

pub(crate) struct InteractiveOutputViewport {
    sender: Option<Sender<Vec<u8>>>,
    handle: Option<thread::JoinHandle<io::Result<()>>>,
}

#[derive(Clone, Copy)]
enum WriterTarget {
    Stdout,
    Stderr,
}

struct ViewportRenderer {
    target: WriterTarget,
    lines: VecDeque<Vec<u8>>,
    total_lines: usize,
    current_line: Vec<u8>,
    pending_carriage_return: bool,
    rendered_rows: usize,
}

impl InteractiveOutputViewport {
    pub(crate) fn maybe_new(stdout_inherit: bool, stderr_inherit: bool) -> Option<Self> {
        if !stdout_inherit && !stderr_inherit {
            return None;
        }
        if !io::stdin().is_terminal() {
            return None;
        }
        if stdout_inherit && !io::stdout().is_terminal() {
            return None;
        }
        if stderr_inherit && !io::stderr().is_terminal() {
            return None;
        }

        let target = if stdout_inherit {
            WriterTarget::Stdout
        } else {
            WriterTarget::Stderr
        };
        let (sender, receiver) = mpsc::channel::<Vec<u8>>();
        let handle = thread::spawn(move || {
            let mut renderer = ViewportRenderer::new(target);
            while let Ok(chunk) = receiver.recv() {
                renderer.push_chunk(&chunk)?;
            }
            renderer.finish()
        });

        Some(Self {
            sender: Some(sender),
            handle: Some(handle),
        })
    }

    pub(crate) fn sink(&self) -> InteractiveOutputSink {
        InteractiveOutputSink {
            sender: self
                .sender
                .as_ref()
                .expect("interactive output viewport sender must exist while active")
                .clone(),
        }
    }

    pub(crate) fn finish(mut self) -> io::Result<()> {
        drop(self.sender.take());
        match self
            .handle
            .take()
            .expect("interactive output viewport handle must exist while active")
            .join()
        {
            Ok(result) => result,
            Err(_) => Err(io::Error::other("interactive output renderer panicked")),
        }
    }
}

impl InteractiveOutputSink {
    fn write_chunk(&self, chunk: Vec<u8>) -> io::Result<()> {
        self.sender.send(chunk).map_err(|_| {
            io::Error::new(
                io::ErrorKind::BrokenPipe,
                "interactive output renderer stopped before command output completed",
            )
        })
    }
}

pub(crate) fn spawn_interactive_output_thread<R>(
    mut reader: R,
    sink: InteractiveOutputSink,
) -> thread::JoinHandle<io::Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                return Ok(());
            }
            sink.write_chunk(buffer[..read].to_vec())?;
        }
    })
}

impl ViewportRenderer {
    fn new(target: WriterTarget) -> Self {
        Self {
            target,
            lines: VecDeque::new(),
            total_lines: 0,
            current_line: Vec::new(),
            pending_carriage_return: false,
            rendered_rows: 0,
        }
    }

    fn push_chunk(&mut self, chunk: &[u8]) -> io::Result<()> {
        for &byte in chunk {
            if self.pending_carriage_return {
                if byte == b'\n' {
                    self.complete_current_line();
                    self.pending_carriage_return = false;
                    continue;
                }
                self.current_line.clear();
                self.pending_carriage_return = false;
            }

            match byte {
                b'\n' => self.complete_current_line(),
                b'\r' => self.pending_carriage_return = true,
                _ => self.current_line.push(byte),
            }
        }
        self.redraw()
    }

    fn finish(&mut self) -> io::Result<()> {
        if self.pending_carriage_return {
            self.current_line.clear();
            self.pending_carriage_return = false;
            self.redraw()?;
        }
        match self.target {
            WriterTarget::Stdout => io::stdout().lock().flush(),
            WriterTarget::Stderr => io::stderr().lock().flush(),
        }
    }

    fn complete_current_line(&mut self) {
        self.total_lines += 1;
        self.lines.push_back(std::mem::take(&mut self.current_line));
        while self.lines.len() > MAX_VISIBLE_LINES {
            self.lines.pop_front();
        }
    }

    fn visible_lines(&self) -> Vec<&[u8]> {
        let mut visible: Vec<&[u8]> = self.lines.iter().map(Vec::as_slice).collect();
        let current_line_count =
            usize::from(!self.current_line.is_empty() || self.pending_carriage_return);
        if current_line_count > 0 {
            visible.push(self.current_line.as_slice());
        }
        let max_visible_lines = dynamic_visible_line_limit(self.total_lines + current_line_count);
        let start = visible.len().saturating_sub(max_visible_lines);
        visible.into_iter().skip(start).collect()
    }

    fn redraw(&mut self) -> io::Result<()> {
        let visible_lines = self.visible_lines();
        let columns = current_terminal_columns();
        let rendered_rows = visible_lines
            .iter()
            .map(|line| display_rows_for_line(line, columns))
            .sum();

        match self.target {
            WriterTarget::Stdout => {
                let stdout = io::stdout();
                let mut writer = stdout.lock();
                redraw_writer(&mut writer, self.rendered_rows, &visible_lines)?;
            }
            WriterTarget::Stderr => {
                let stderr = io::stderr();
                let mut writer = stderr.lock();
                redraw_writer(&mut writer, self.rendered_rows, &visible_lines)?;
            }
        }

        self.rendered_rows = rendered_rows;
        Ok(())
    }
}

fn current_terminal_columns() -> usize {
    terminal::size()
        .map(|(width, _)| usize::from(width.max(1)))
        .unwrap_or(DEFAULT_TERMINAL_COLUMNS)
}

fn display_rows_for_line(line: &[u8], columns: usize) -> usize {
    let width = crate::DISPLAY_STREAM_PREFIX.chars().count()
        + String::from_utf8_lossy(line).chars().count();
    width.max(1).div_ceil(columns.max(1))
}

fn dynamic_visible_line_limit(total_lines: usize) -> usize {
    let overflow = total_lines.saturating_sub(INITIAL_VISIBLE_LINES) as f64;
    let progress = (overflow / VISIBLE_LINE_EXPAND_AT).clamp(0.0, 1.0);
    let visible_lines = (INITIAL_VISIBLE_LINES as f64
        + (MAX_VISIBLE_LINES - INITIAL_VISIBLE_LINES) as f64 * progress.powf(0.75))
    .round() as usize;
    visible_lines.clamp(INITIAL_VISIBLE_LINES, MAX_VISIBLE_LINES)
}

fn redraw_writer<W: Write>(
    writer: &mut W,
    rendered_rows: usize,
    visible_lines: &[&[u8]],
) -> io::Result<()> {
    if rendered_rows > 0 {
        writer.queue(MoveToColumn(0))?;
        writer.queue(MoveUp(u16::try_from(rendered_rows).unwrap_or(u16::MAX)))?;
    }
    writer.queue(Clear(ClearType::FromCursorDown))?;
    for line in visible_lines {
        writer.write_all(crate::DISPLAY_STREAM_PREFIX.as_bytes())?;
        writer.write_all(line)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()
}
