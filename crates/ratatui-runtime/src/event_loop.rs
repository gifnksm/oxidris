use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event;

use crate::event::TuiEvent;

/// Rendering trigger mode.
#[derive(Debug, Clone, Copy, Default)]
pub enum RenderMode {
    /// Render at fixed intervals.
    Interval(Duration),
    /// Render after state changes (tick or crossterm event).
    #[default]
    OnDirty,
    /// Render after state changes, but with minimum interval between renders.
    ///
    /// If events occur faster than the interval, they are batched into one render.
    Throttled(Duration),
}

impl RenderMode {
    /// Creates `Interval` mode from frame rate (FPS).
    #[must_use]
    pub fn interval_from_rate(rate: f64) -> Self {
        Self::Interval(Duration::from_secs_f64(1.0 / rate))
    }

    /// Creates `Throttled` mode from frame rate (FPS).
    #[must_use]
    pub fn throttled_from_rate(rate: f64) -> Self {
        Self::Throttled(Duration::from_secs_f64(1.0 / rate))
    }
}

/// Event loop state management.
///
/// Manages tick/render intervals and returns the next event via `next()`.
/// If an interval is not set, that event type will not be generated.
#[derive(Debug)]
pub(super) struct EventLoop {
    tick_interval: Option<Duration>,
    render_mode: RenderMode,
    last_tick: Instant,
    last_render: Instant,
    dirty: bool,
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLoop {
    /// Creates a new `EventLoop`.
    ///
    /// Tick interval is unset, and render mode defaults to `OnDirty`.
    pub fn new() -> Self {
        let now = Instant::now();
        let past_time = now.checked_sub(Duration::from_secs(86400)).unwrap_or(now);
        Self {
            tick_interval: None,
            render_mode: RenderMode::default(),
            last_tick: past_time,
            last_render: past_time,
            dirty: true, // Initial render is required on startup
        }
    }

    /// Sets the tick interval.
    ///
    /// Pass `None` to disable tick events.
    pub(super) fn set_tick_interval(&mut self, interval: Option<Duration>) {
        self.tick_interval = interval;
    }

    /// Sets the render mode.
    pub(super) fn set_render_mode(&mut self, render_mode: RenderMode) {
        self.render_mode = render_mode;
    }

    /// Returns the next event.
    ///
    /// Blocks until a tick/render time is reached or a crossterm event occurs.
    /// If both tick and render are unset, only waits for crossterm events.
    pub(super) fn next(&mut self) -> io::Result<TuiEvent> {
        loop {
            let now = Instant::now();
            if let Some(tick_interval) = self.tick_interval
                && now.duration_since(self.last_tick) >= tick_interval
            {
                self.last_tick = now;
                self.dirty = true;
                return Ok(TuiEvent::Tick);
            }

            let do_render = match self.render_mode {
                RenderMode::Interval(interval) => now.duration_since(self.last_render) >= interval,
                RenderMode::OnDirty => self.dirty,
                RenderMode::Throttled(interval) => {
                    self.dirty && now.duration_since(self.last_render) >= interval
                }
            };
            if do_render {
                self.last_render = now;
                self.dirty = false;
                return Ok(TuiEvent::Render);
            }

            if let Some(timeout) = self.compute_timeout(now)
                && !event::poll(timeout)?
            {
                continue;
            }

            self.dirty = true;
            return Ok(event::read()?.into());
        }
    }

    fn compute_timeout(&self, now: Instant) -> Option<Duration> {
        let next_tick_at = self.tick_interval.map(|interval| self.last_tick + interval);
        let next_render_at = match self.render_mode {
            RenderMode::Interval(interval) => Some(self.last_render + interval),
            RenderMode::OnDirty => self.dirty.then_some(now),
            RenderMode::Throttled(interval) => self.dirty.then(|| self.last_render + interval),
        };
        let next_timeout_at = [next_tick_at, next_render_at].into_iter().flatten().min()?;
        Some(next_timeout_at.saturating_duration_since(now))
    }
}
