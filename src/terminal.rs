use std::{
    fmt,
    io::{self, BufWriter, Write},
};

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl Color {
    // Common colors as associated constants
    pub(crate) const CYAN: Color = Color::new(0, 255, 255);
    pub(crate) const YELLOW: Color = Color::new(255, 255, 0);
    pub(crate) const GREEN: Color = Color::new(0, 255, 0);
    pub(crate) const RED: Color = Color::new(255, 0, 0);
    pub(crate) const BLUE: Color = Color::new(0, 0, 255);
    pub(crate) const ORANGE: Color = Color::new(255, 127, 0);
    pub(crate) const MAGENTA: Color = Color::new(255, 0, 255);
    pub(crate) const GRAY: Color = Color::new(127, 127, 127);
    pub(crate) const BLACK: Color = Color::new(0, 0, 0);
    pub(crate) const WHITE: Color = Color::new(255, 255, 255);

    /// Create a new color with RGB values
    pub(crate) const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Terminal control with console-style API
pub(crate) struct Terminal {
    writer: BufWriter<Box<dyn Write + Send>>,
}

impl Terminal {
    /// Create a new terminal instance writing to stdout
    pub(crate) fn stdout() -> Self {
        Self {
            writer: BufWriter::new(Box::new(io::stdout())),
        }
    }

    /// Clear the entire screen and move cursor to home
    pub(crate) fn clear_screen(&mut self) -> io::Result<&mut Self> {
        write!(self.writer, "\x1b[2J\x1b[H")?;
        Ok(self)
    }

    /// Hide the cursor
    pub(crate) fn hide_cursor(&mut self) -> io::Result<&mut Self> {
        write!(self.writer, "\x1b[?25l")?;
        Ok(self)
    }

    /// Show the cursor
    pub(crate) fn show_cursor(&mut self) -> io::Result<&mut Self> {
        write!(self.writer, "\x1b[?25h")?;
        Ok(self)
    }

    /// Move cursor to specific position (1-indexed)
    pub(crate) fn move_to(&mut self, row: usize, col: usize) -> io::Result<&mut Self> {
        write!(self.writer, "\x1b[{};{}H", row, col)?;
        Ok(self)
    }

    /// Set foreground color for subsequent writes
    pub(crate) fn set_fg(&mut self, color: Color) -> io::Result<&mut Self> {
        let Color { r, g, b } = color;
        write!(self.writer, "\x1b[38;2;{};{};{}m", r, g, b)?;
        Ok(self)
    }

    /// Set background color for subsequent writes
    pub(crate) fn set_bg(&mut self, color: Color) -> io::Result<&mut Self> {
        let Color { r, g, b } = color;
        write!(self.writer, "\x1b[48;2;{};{};{}m", r, g, b)?;
        Ok(self)
    }

    /// Enable bold text for subsequent writes
    pub(crate) fn set_bold(&mut self) -> io::Result<&mut Self> {
        write!(self.writer, "\x1b[1m")?;
        Ok(self)
    }

    /// Reset styles to terminal default
    pub(crate) fn reset_styles(&mut self) -> io::Result<&mut Self> {
        write!(self.writer, "\x1b[0m")?;
        Ok(self)
    }

    /// Write text without moving cursor or resetting color
    pub(crate) fn write(&mut self, text: impl fmt::Display) -> io::Result<&mut Self> {
        write!(self.writer, "{text}")?;
        Ok(self)
    }

    /// Flush the output
    pub(crate) fn flush(&mut self) -> io::Result<&mut Self> {
        self.writer.flush()?;
        Ok(self)
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::stdout()
    }
}
