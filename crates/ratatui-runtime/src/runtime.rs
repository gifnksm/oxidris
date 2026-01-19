use std::{io, time::Duration};

use crate::{
    App,
    event::TuiEvent,
    event_loop::{EventLoop, RenderMode},
};

/// TUI application runtime.
///
/// Manages the event loop and executes applications that implement the `App` trait.
#[derive(Default, Debug)]
pub struct Runtime {
    events: EventLoop,
}

impl Runtime {
    /// Creates a new Runtime.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the tick rate (Hz, ticks per second).
    pub fn set_tick_rate(&mut self, rate: Option<f64>) {
        self.set_tick_interval(rate.map(|rate| Duration::from_secs_f64(1.0 / rate)));
    }

    /// Sets the tick interval.
    pub fn set_tick_interval(&mut self, interval: Option<Duration>) {
        self.events.set_tick_interval(interval);
    }

    /// Sets the render mode.
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.events.set_render_mode(mode);
    }

    /// Runs the application.
    ///
    /// 1. Calls `app.init()` for initialization
    /// 2. Runs the event loop until `app.should_exit()` returns true
    ///    - `Event::Tick`: calls `app.update()`
    ///    - `Event::Render`: calls `app.draw()`
    ///    - `Event::Crossterm`: calls `app.handle_event()`
    pub fn run<A>(mut self, app: &mut A) -> io::Result<()>
    where
        A: App,
    {
        app.init(&mut self);

        ratatui::run(|terminal| {
            while !app.should_exit() {
                match self.events.next()? {
                    TuiEvent::Tick => {
                        app.update(&mut self);
                    }
                    TuiEvent::Render => {
                        terminal.draw(|f| app.draw(f))?;
                    }
                    TuiEvent::Crossterm(event) => {
                        app.handle_event(&mut self, event);
                    }
                }
            }
            Ok(())
        })
    }
}
