use crossterm::event::{Event, KeyCode};
use oxidris_engine::SessionState;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    record::{RecordingSession, SessionHistory},
    schema::record::PlayerInfo,
    view::widgets::{KeyBinding, KeyBindingDisplay, SessionDisplay},
};

#[derive(Debug)]
pub struct ManualPlayScreen {
    session: RecordingSession,
    should_exit: bool,
}

impl ManualPlayScreen {
    pub fn new(fps: u64, history_size: usize) -> Self {
        Self {
            session: RecordingSession::new(fps, PlayerInfo::Manual, history_size),
            should_exit: false,
        }
    }

    pub fn is_playing(&self) -> bool {
        !self.should_exit && self.session.session_state().is_playing()
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        let session_display = SessionDisplay::new(&self.session, true);

        let bindings: &[KeyBinding] = {
            let quit: KeyBinding = (&["q", "Esc"], "Quit");
            match self.session.session_state() {
                SessionState::Playing => &[
                    (&["←", "→"], "Move"),
                    (&["↓"], "Soft Drop"),
                    (&["↑"], "Hard Drop"),
                    (&["z", "x"], "Rotate"),
                    (&["Space"], "Hold"),
                    (&["p"], "Pause"),
                    quit,
                ],
                SessionState::Paused => &[(&["p"], "Resume"), quit],
                SessionState::GameOver => &[quit],
            }
        };
        let help_text = KeyBindingDisplay::new(bindings);

        let [main_area, help_area] =
            Layout::vertical([Constraint::Length(25), Constraint::Length(1)])
                .areas::<2>(frame.area());
        frame.render_widget(session_display, main_area);
        frame.render_widget(help_text, help_area);
    }

    pub fn handle_event(&mut self, event: &Event) {
        let is_playing = self.is_playing();
        let is_paused = self.session.session_state().is_paused();
        let can_toggle_pause = is_playing || is_paused;

        if let Some(event) = event.as_key_event() {
            match event.code {
                KeyCode::Left if is_playing => _ = self.session.try_move_left(),
                KeyCode::Right if is_playing => _ = self.session.try_move_right(),
                KeyCode::Down if is_playing => _ = self.session.try_soft_drop(),
                KeyCode::Up if is_playing => self.session.hard_drop_and_complete(),
                KeyCode::Char('z') if is_playing => _ = self.session.try_rotate_left(),
                KeyCode::Char('x') if is_playing => _ = self.session.try_rotate_right(),
                KeyCode::Char(' ') if is_playing => _ = self.session.try_hold(),
                KeyCode::Char('p') if can_toggle_pause => self.session.toggle_pause(),
                KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                _ => {}
            }
        }
    }

    pub fn update(&mut self) {
        self.session.increment_frame();
    }

    pub fn into_history(self) -> SessionHistory {
        self.session.into_history()
    }
}
