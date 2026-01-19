use crossterm::event::Event;
use ratatui::Frame;

use crate::Runtime;

/// Trait for TUI applications.
///
/// Applications executed by `Runtime::run()` must implement this trait.
pub trait App {
    /// Initializes the application.
    ///
    /// Called at the start of `Runtime::run()`. Use this to configure `tick_rate/frame_rate`.
    fn init(&mut self, runtime: &mut Runtime);

    /// Returns whether the application should exit.
    fn should_exit(&self) -> bool;

    /// Handles terminal events (key input, mouse, resize, etc.).
    fn handle_event(&mut self, runtime: &mut Runtime, event: Event);

    /// Draws the screen (called on each `Event::Render`).
    fn draw(&self, frame: &mut Frame);

    /// Updates game logic (called on each `Event::Tick`).
    fn update(&mut self, runtime: &mut Runtime);
}
