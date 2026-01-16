use crossterm::event::{Event, KeyCode, KeyEvent};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayingAction {
    MoveLeft,
    MoveRight,
    SoftDrop,
    HardDrop,
    RotateLeft,
    RotateRight,
    Hold,
    Pause,
    Quit,
}

impl PlayingAction {
    fn from_key_event(event: &KeyEvent) -> Option<Self> {
        match event.code {
            KeyCode::Left => Some(Self::MoveLeft),
            KeyCode::Right => Some(Self::MoveRight),
            KeyCode::Down => Some(Self::SoftDrop),
            KeyCode::Up => Some(Self::HardDrop),
            KeyCode::Char('z') => Some(Self::RotateLeft),
            KeyCode::Char('x') => Some(Self::RotateRight),
            KeyCode::Char(' ') => Some(Self::Hold),
            KeyCode::Char('p') => Some(Self::Pause),
            KeyCode::Char('q') | KeyCode::Esc => Some(Self::Quit),
            _ => None,
        }
    }

    fn bindings() -> &'static [KeyBinding<'static>] {
        &[
            (&["←", "→"], "Move"),
            (&["↓"], "Soft Drop"),
            (&["↑"], "Hard Drop"),
            (&["z", "x"], "Rotate"),
            (&["Space"], "Hold"),
            (&["p"], "Pause"),
            (&["q", "Esc"], "Quit"),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PausedAction {
    Resume,
    Quit,
}

impl PausedAction {
    fn from_key_event(event: &KeyEvent) -> Option<Self> {
        match event.code {
            KeyCode::Char('p') => Some(Self::Resume),
            KeyCode::Char('q') | KeyCode::Esc => Some(Self::Quit),
            _ => None,
        }
    }

    fn bindings() -> &'static [KeyBinding<'static>] {
        &[(&["p"], "Resume"), (&["q", "Esc"], "Quit")]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameOverAction {
    Quit,
}

impl GameOverAction {
    fn from_key_event(event: &KeyEvent) -> Option<Self> {
        match event.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Self::Quit),
            _ => None,
        }
    }

    fn bindings() -> &'static [KeyBinding<'static>] {
        &[(&["q", "Esc"], "Quit")]
    }
}

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

        let bindings = match self.session.session_state() {
            SessionState::Playing => PlayingAction::bindings(),
            SessionState::Paused => PausedAction::bindings(),
            SessionState::GameOver => GameOverAction::bindings(),
        };
        let help_text = KeyBindingDisplay::new(bindings);

        let [main_area, help_area] =
            Layout::vertical([Constraint::Length(25), Constraint::Length(1)])
                .areas::<2>(frame.area());
        frame.render_widget(session_display, main_area);
        frame.render_widget(help_text, help_area);
    }

    pub fn handle_event(&mut self, event: &Event) {
        if let Some(event) = event.as_key_event() {
            match self.session.session_state() {
                SessionState::Playing => {
                    if let Some(action) = PlayingAction::from_key_event(&event) {
                        match action {
                            PlayingAction::MoveLeft => _ = self.session.try_move_left(),
                            PlayingAction::MoveRight => _ = self.session.try_move_right(),
                            PlayingAction::SoftDrop => _ = self.session.try_soft_drop(),
                            PlayingAction::HardDrop => self.session.hard_drop_and_complete(),
                            PlayingAction::RotateLeft => _ = self.session.try_rotate_left(),
                            PlayingAction::RotateRight => _ = self.session.try_rotate_right(),
                            PlayingAction::Hold => _ = self.session.try_hold(),
                            PlayingAction::Pause => self.session.toggle_pause(),
                            PlayingAction::Quit => self.should_exit = true,
                        }
                    }
                }
                SessionState::Paused => {
                    if let Some(action) = PausedAction::from_key_event(&event) {
                        match action {
                            PausedAction::Resume => self.session.toggle_pause(),
                            PausedAction::Quit => self.should_exit = true,
                        }
                    }
                }
                SessionState::GameOver => {
                    if let Some(action) = GameOverAction::from_key_event(&event) {
                        match action {
                            GameOverAction::Quit => self.should_exit = true,
                        }
                    }
                }
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
