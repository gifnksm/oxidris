use std::io;

use crossterm::{
    event::{self, KeyCode},
    terminal,
};

pub(crate) struct Input {
    disabled: bool,
}

impl Input {
    pub(crate) fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self { disabled: false })
    }

    pub(crate) fn cleanup(mut self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        self.disabled = true;
        Ok(())
    }

    #[expect(clippy::unused_self)]
    pub(crate) fn read(&mut self) -> io::Result<KeyCode> {
        loop {
            if let Some(event) = event::read()?.as_key_event() {
                return Ok(event.code);
            }
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        if !self.disabled {
            let _ = terminal::disable_raw_mode();
        }
    }
}
