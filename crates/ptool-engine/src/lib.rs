mod ansi;
mod platform;

pub use ansi::{Color, StyleOptions};
pub use platform::{Arch, OS};

#[derive(Debug, Default)]
pub struct PtoolEngine;

impl PtoolEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn ansi_style(&self, text: String, options: StyleOptions) -> String {
        ansi::style(text, options)
    }

    pub fn current_os(&self) -> OS {
        platform::detect_current_os()
    }

    pub fn current_arch(&self) -> Arch {
        platform::detect_current_arch()
    }
}
