use crossterm::event::Event as CrosstermEvent;

/// Events processed by TUI applications.
#[derive(Debug, Clone, derive_more::IsVariant, derive_more::From)]
pub(super) enum TuiEvent {
    /// Game logic update timing (based on `tick_interval`).
    Tick,
    /// Screen render timing (based on `render_interval`).
    Render,
    /// Terminal events such as key input, mouse, and resize.
    Crossterm(CrosstermEvent),
}
