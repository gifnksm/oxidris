use crossterm::event::Event;
use ratatui::Frame;

use crate::tui::Tui;

/// Trait for TUI applications.
///
/// Applications executed by `Tui::run()` must implement this trait.
pub trait App {
    /// Initializes the application.
    ///
    /// Called at the start of `Tui::run()`. Use this to configure `tick_rate/frame_rate`.
    fn init(&mut self, tui: &mut Tui);

    /// Returns whether the application should exit.
    fn should_exit(&self) -> bool;

    /// Handles terminal events (key input, mouse, resize, etc.).
    fn handle_event(&mut self, tui: &mut Tui, event: Event);

    /// Draws the screen (called on each `Event::Render`).
    fn draw(&self, frame: &mut Frame);

    /// Updates game logic (called on each `Event::Tick`).
    fn update(&mut self, tui: &mut Tui);
}
