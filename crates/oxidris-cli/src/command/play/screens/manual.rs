use crossterm::event::{Event, KeyCode, KeyEvent};
use oxidris_engine::SessionState;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    DEFAULT_FRAME_RATE,
    command::play::TICK_RATE,
    record::{RecordingSession, SessionHistory},
    schema::record::PlayerInfo,
    tui::{RenderMode, Screen, ScreenTransition, Tui},
    view::{
        screens::ReplayScreen,
        widgets::{KeyBinding, KeyBindingDisplay, SessionDisplay},
    },
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
    OpenReplay,
    Resume,
    Quit,
}

impl PausedAction {
    fn from_key_event(event: &KeyEvent) -> Option<Self> {
        match event.code {
            KeyCode::Char('p') => Some(Self::Resume),
            KeyCode::Char('R') => Some(Self::OpenReplay),
            KeyCode::Char('q') | KeyCode::Esc => Some(Self::Quit),
            _ => None,
        }
    }

    fn bindings() -> &'static [KeyBinding<'static>] {
        &[
            (&["p"], "Resume"),
            (&["R"], "Open Replay"),
            (&["q", "Esc"], "Quit"),
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameOverAction {
    OpenReplay,
    Quit,
}

impl GameOverAction {
    fn from_key_event(event: &KeyEvent) -> Option<Self> {
        match event.code {
            KeyCode::Char('R') => Some(Self::OpenReplay),
            KeyCode::Char('q') | KeyCode::Esc => Some(Self::Quit),
            _ => None,
        }
    }

    fn bindings() -> &'static [KeyBinding<'static>] {
        &[(&["R"], "Open Replay"), (&["q", "Esc"], "Quit")]
    }
}

#[derive(Debug)]
pub struct ManualPlayScreen<'a> {
    session: RecordingSession,
    session_history: &'a mut Option<SessionHistory>,
}

impl<'a> ManualPlayScreen<'a> {
    pub fn new(
        tick_rate: f64,
        max_replay_turns: usize,
        session_history: &'a mut Option<SessionHistory>,
    ) -> Self {
        Self {
            session: RecordingSession::new(tick_rate, PlayerInfo::Manual, max_replay_turns),
            session_history,
        }
    }
}

impl Screen for ManualPlayScreen<'_> {
    fn on_active(&mut self, tui: &mut Tui) {
        tui.set_render_mode(RenderMode::throttled_from_rate(DEFAULT_FRAME_RATE));
        self.update_tick_interval(tui);
    }

    fn on_inactive(&mut self, _tui: &mut Tui) {}

    fn on_close(&mut self, _tui: &mut Tui) {
        *self.session_history = Some(self.session.to_history());
    }

    fn handle_event(&mut self, tui: &mut Tui, event: &Event) -> ScreenTransition {
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
                            PlayingAction::Quit => return ScreenTransition::Pop,
                        }
                    }
                }
                SessionState::Paused => {
                    if let Some(action) = PausedAction::from_key_event(&event) {
                        match action {
                            PausedAction::OpenReplay => {
                                let session = self.session.to_history().to_recorded_session();
                                return ScreenTransition::Push(Box::new(ReplayScreen::in_game(
                                    session,
                                )));
                            }
                            PausedAction::Resume => self.session.toggle_pause(),
                            PausedAction::Quit => return ScreenTransition::Pop,
                        }
                    }
                }
                SessionState::GameOver => {
                    if let Some(action) = GameOverAction::from_key_event(&event) {
                        match action {
                            GameOverAction::OpenReplay => {
                                let session = self.session.to_history().to_recorded_session();
                                return ScreenTransition::Push(Box::new(ReplayScreen::in_game(
                                    session,
                                )));
                            }
                            GameOverAction::Quit => return ScreenTransition::Pop,
                        }
                    }
                }
            }
        }
        self.update_tick_interval(tui);
        ScreenTransition::Stay
    }

    fn update(&mut self, _tui: &mut Tui) {
        self.session.increment_frame();
    }

    fn draw(&self, frame: &mut Frame) {
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
}

impl ManualPlayScreen<'_> {
    fn is_playing(&self) -> bool {
        self.session.session_state().is_playing()
    }

    fn update_tick_interval(&mut self, tui: &mut Tui) {
        tui.set_tick_rate(self.is_playing().then_some(TICK_RATE));
    }
}
