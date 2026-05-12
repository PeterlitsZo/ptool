use crate::{Error, ErrorKind, Result};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph};
use std::io::{self, IsTerminal, Stdout};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TuiSessionOptions {
    pub tick_rate: Duration,
}

impl Default for TuiSessionOptions {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(100),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TuiEvent {
    Tick,
    Resize { width: u16, height: u16 },
    Key(TuiKeyEvent),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuiKeyEvent {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TuiNodeBlock {
    pub title: Option<String>,
    pub border: bool,
    pub padding: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuiContainerDirection {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuiTextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TuiHighlight {
    pub style: TuiStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TuiStyle {
    pub fg: Option<TuiColor>,
    pub bg: Option<TuiColor>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underlined: bool,
    pub reversed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Gray,
    DarkGray,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuiNode {
    pub block: TuiNodeBlock,
    pub style: TuiStyle,
    pub grow: u16,
    pub kind: TuiNodeKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TuiNodeKind {
    Text {
        text: String,
        align: TuiTextAlign,
    },
    List {
        items: Vec<String>,
        selected: Option<usize>,
        highlight: TuiHighlight,
    },
    Container {
        direction: TuiContainerDirection,
        children: Vec<TuiNode>,
    },
}

type Backend = CrosstermBackend<Stdout>;

pub struct TuiSession {
    terminal: Option<Terminal<Backend>>,
    tick_rate: Duration,
    last_tick_at: Instant,
}

impl TuiSession {
    pub fn new(options: TuiSessionOptions) -> Result<Self> {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return Err(Error::new(
                ErrorKind::NotInteractive,
                "ptool.tui.run(options) requires an interactive TTY",
            )
            .with_op("ptool.tui.run"));
        }
        if options.tick_rate.is_zero() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                "ptool.tui.run(options) `tick_ms` must be > 0",
            )
            .with_op("ptool.tui.run")
            .with_detail("`tick_ms` must be > 0"));
        }

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|err| io_error("ptool.tui.run", err))?;
        enable_raw_mode().map_err(|err| {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            io_error("ptool.tui.run", err)
        })?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|err| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            io_error("ptool.tui.run", err)
        })?;

        Ok(Self {
            terminal: Some(terminal),
            tick_rate: options.tick_rate,
            last_tick_at: Instant::now(),
        })
    }

    pub fn draw(&mut self, root: &TuiNode) -> Result<()> {
        let terminal = self
            .terminal
            .as_mut()
            .expect("ptool-engine TuiSession terminal must exist while active");

        terminal
            .draw(|frame| {
                render_node(frame, frame.area(), root);
            })
            .map_err(|err| io_error("ptool.tui.run", err))?;
        Ok(())
    }

    pub fn next_event(&mut self) -> Result<TuiEvent> {
        loop {
            let timeout = self
                .tick_rate
                .checked_sub(self.last_tick_at.elapsed())
                .unwrap_or(Duration::ZERO);

            if event::poll(timeout).map_err(|err| io_error("ptool.tui.run", err))? {
                match event::read().map_err(|err| io_error("ptool.tui.run", err))? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        return Ok(TuiEvent::Key(map_key_event(key)));
                    }
                    Event::Resize(width, height) => {
                        return Ok(TuiEvent::Resize { width, height });
                    }
                    Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::ScrollUp) => {
                        return Ok(TuiEvent::Key(TuiKeyEvent {
                            key: "scroll_up".to_string(),
                            ctrl: false,
                            alt: false,
                            shift: false,
                        }));
                    }
                    Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::ScrollDown) => {
                        return Ok(TuiEvent::Key(TuiKeyEvent {
                            key: "scroll_down".to_string(),
                            ctrl: false,
                            alt: false,
                            shift: false,
                        }));
                    }
                    _ => {}
                }
            } else {
                self.last_tick_at = Instant::now();
                return Ok(TuiEvent::Tick);
            }
        }
    }
}

impl Drop for TuiSession {
    fn drop(&mut self) {
        if let Some(terminal) = self.terminal.as_mut() {
            let _ = terminal.show_cursor();
        }
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

impl TuiColor {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "black" => Some(Self::Black),
            "red" => Some(Self::Red),
            "green" => Some(Self::Green),
            "yellow" => Some(Self::Yellow),
            "blue" => Some(Self::Blue),
            "magenta" => Some(Self::Magenta),
            "cyan" => Some(Self::Cyan),
            "white" => Some(Self::White),
            "gray" | "grey" => Some(Self::Gray),
            "dark_gray" | "dark_grey" => Some(Self::DarkGray),
            _ => None,
        }
    }
}

fn render_node(frame: &mut ratatui::Frame<'_>, area: Rect, node: &TuiNode) {
    let style = to_ratatui_style(node.style);
    let block = build_block(&node.block, style);
    let inner = match block.as_ref() {
        Some(block) => {
            let inner = block.inner(area);
            frame.render_widget(block.clone(), area);
            inner
        }
        None => area,
    };

    match &node.kind {
        TuiNodeKind::Text { text, align } => {
            let paragraph = Paragraph::new(text.as_str())
                .style(style)
                .alignment(to_alignment(*align));
            frame.render_widget(paragraph, inner);
        }
        TuiNodeKind::List {
            items,
            selected,
            highlight,
        } => {
            let list_items = items
                .iter()
                .map(|item| ListItem::new(item.as_str()))
                .collect::<Vec<_>>();
            let list = List::new(list_items.clone())
                .style(style)
                .highlight_style(to_ratatui_style(highlight.style));
            let mut state = ListState::default();
            if let Some(selected) = selected.filter(|selected| *selected < list_items.len()) {
                state.select(Some(selected));
            }
            frame.render_stateful_widget(list, inner, &mut state);
        }
        TuiNodeKind::Container {
            direction,
            children,
        } => {
            if children.is_empty() {
                return;
            }

            let total = total_grow(children);
            let constraints = children
                .iter()
                .map(|child| Constraint::Ratio(u32::from(child.grow.max(1)), total))
                .collect::<Vec<_>>();
            let areas = Layout::default()
                .direction(match direction {
                    TuiContainerDirection::Row => Direction::Horizontal,
                    TuiContainerDirection::Column => Direction::Vertical,
                })
                .constraints(constraints)
                .split(inner);
            for (child, child_area) in children.iter().zip(areas.iter().copied()) {
                render_node(frame, child_area, child);
            }
        }
    }
}

fn build_block(block: &TuiNodeBlock, style: Style) -> Option<Block<'static>> {
    if !block.border && block.title.is_none() && block.padding == 0 {
        return None;
    }

    let mut widget = Block::default().style(style);
    if block.border {
        widget = widget.borders(Borders::ALL);
    }
    if let Some(title) = block.title.as_deref() {
        widget = widget.title(title.to_string());
    }
    if block.padding > 0 {
        widget = widget.padding(Padding::uniform(block.padding));
    }
    Some(widget)
}

fn total_grow(children: &[TuiNode]) -> u32 {
    children
        .iter()
        .map(|child| u32::from(child.grow.max(1)))
        .sum::<u32>()
        .max(1)
}

fn to_alignment(align: TuiTextAlign) -> Alignment {
    match align {
        TuiTextAlign::Left => Alignment::Left,
        TuiTextAlign::Center => Alignment::Center,
        TuiTextAlign::Right => Alignment::Right,
    }
}

fn to_ratatui_style(style: TuiStyle) -> Style {
    let mut rendered = Style::default();
    if let Some(fg) = style.fg {
        rendered = rendered.fg(to_ratatui_color(fg));
    }
    if let Some(bg) = style.bg {
        rendered = rendered.bg(to_ratatui_color(bg));
    }
    if style.bold {
        rendered = rendered.add_modifier(Modifier::BOLD);
    }
    if style.dim {
        rendered = rendered.add_modifier(Modifier::DIM);
    }
    if style.italic {
        rendered = rendered.add_modifier(Modifier::ITALIC);
    }
    if style.underlined {
        rendered = rendered.add_modifier(Modifier::UNDERLINED);
    }
    if style.reversed {
        rendered = rendered.add_modifier(Modifier::REVERSED);
    }
    rendered
}

fn to_ratatui_color(color: TuiColor) -> Color {
    match color {
        TuiColor::Black => Color::Black,
        TuiColor::Red => Color::Red,
        TuiColor::Green => Color::Green,
        TuiColor::Yellow => Color::Yellow,
        TuiColor::Blue => Color::Blue,
        TuiColor::Magenta => Color::Magenta,
        TuiColor::Cyan => Color::Cyan,
        TuiColor::White => Color::White,
        TuiColor::Gray => Color::Gray,
        TuiColor::DarkGray => Color::DarkGray,
    }
}

fn map_key_event(event: KeyEvent) -> TuiKeyEvent {
    TuiKeyEvent {
        key: match event.code {
            KeyCode::Backspace => "backspace".to_string(),
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Left => "left".to_string(),
            KeyCode::Right => "right".to_string(),
            KeyCode::Up => "up".to_string(),
            KeyCode::Down => "down".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::PageUp => "pageup".to_string(),
            KeyCode::PageDown => "pagedown".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::BackTab => "backtab".to_string(),
            KeyCode::Delete => "delete".to_string(),
            KeyCode::Insert => "insert".to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::Char(ch) => ch.to_string(),
            KeyCode::F(number) => format!("f{number}"),
            KeyCode::Null => "null".to_string(),
            KeyCode::CapsLock => "caps_lock".to_string(),
            KeyCode::ScrollLock => "scroll_lock".to_string(),
            KeyCode::NumLock => "num_lock".to_string(),
            KeyCode::PrintScreen => "print_screen".to_string(),
            KeyCode::Pause => "pause".to_string(),
            KeyCode::Menu => "menu".to_string(),
            KeyCode::KeypadBegin => "keypad_begin".to_string(),
            KeyCode::Media(_) => "media".to_string(),
            KeyCode::Modifier(modifier) => format!("modifier:{modifier:?}").to_lowercase(),
        },
        ctrl: event.modifiers.contains(KeyModifiers::CONTROL),
        alt: event.modifiers.contains(KeyModifiers::ALT),
        shift: event.modifiers.contains(KeyModifiers::SHIFT),
    }
}

fn io_error(op: &str, err: impl std::fmt::Display) -> Error {
    Error::new(ErrorKind::Io, format!("{op} failed: {err}")).with_op(op)
}
