mod ansi;

pub use ansi::{Color, StyleOptions};

#[derive(Debug, Default)]
pub struct PtoolEngine;

impl PtoolEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn ansi_style(&self, text: String, options: StyleOptions) -> String {
        ansi::style(text, options)
    }
}
