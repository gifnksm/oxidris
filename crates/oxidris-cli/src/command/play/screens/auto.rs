use std::{
    sync::mpsc::{self, RecvError, TryRecvError},
    thread,
};

use crossterm::event::{Event, KeyCode};
use oxidris_engine::{GameSession, SessionState};
use oxidris_evaluator::{
    placement_analysis::PlacementAnalysis,
    turn_evaluator::{TurnEvaluator, TurnPlan},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Text,
};

use crate::ui::widgets::SessionDisplay;

#[derive(Debug)]
pub struct AutoPlayScreen {
    session: GameSession,
    turbo: bool,
    is_exiting: bool,
    tx: mpsc::Sender<Request>,
    rx: mpsc::Receiver<GameSession>,
}

impl AutoPlayScreen {
    pub fn new(fps: u64, turn_evaluator: TurnEvaluator<'static>, turbo: bool) -> Self {
        let session = GameSession::new(fps);
        let auto_play = AutoPlay::new(session.clone(), turn_evaluator);
        let (tx_request, mut rx_request) = mpsc::channel();
        let (mut tx_session, rx_session) = mpsc::channel();
        thread::spawn(move || ai_thread(auto_play, &mut tx_session, &mut rx_request));
        Self {
            session,
            turbo,
            is_exiting: false,
            tx: tx_request,
            rx: rx_session,
        }
    }

    pub fn is_playing(&self) -> bool {
        !self.is_exiting && self.session.session_state().is_playing()
    }

    pub fn is_exiting(&self) -> bool {
        self.is_exiting
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        let session_display = SessionDisplay::new(&self.session, false).turbo(self.turbo);
        let turbo_text = if self.turbo {
            "T (Turbo: OFF)"
        } else {
            "T (Turbo: ON)"
        };
        let help_text = match self.session.session_state() {
            SessionState::Playing => format!("Controls: {turbo_text} | P (Pause) | Q (Quit)"),
            SessionState::Paused => "Controls: P (Resume) | Q (Quit)".to_owned(),
            SessionState::GameOver => "Controls: Q (Quit)".to_owned(),
        };
        let help_text = Text::from(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .centered();

        let [main_area, help_area] =
            Layout::vertical([Constraint::Length(25), Constraint::Length(1)])
                .areas::<2>(frame.area());
        frame.render_widget(session_display, main_area);
        frame.render_widget(help_text, help_area);
    }

    pub fn handle_event(&mut self, event: &Event) {
        let is_playing = self.session.session_state().is_playing();
        let is_paused = self.session.session_state().is_paused();
        let can_toggle_pause = is_playing || is_paused;

        if let Some(event) = event.as_key_event() {
            match event.code {
                KeyCode::Char('t') if is_playing => self.turbo = !self.turbo,
                KeyCode::Char('p') if can_toggle_pause => self.session.toggle_pause(),
                KeyCode::Char('q') => self.is_exiting = true,
                _ => {}
            }
        }
    }

    pub fn update_game(&mut self) {
        let req = {
            if self.session.session_state().is_paused() {
                Request::TogglePause
            } else if self.turbo {
                Request::TurboRun
            } else {
                Request::Run
            }
        };
        self.tx.send(req).unwrap();
        self.session = self.rx.recv().unwrap();
    }
}

#[derive(Debug, Clone, Copy)]
enum Request {
    TogglePause,
    Run,
    TurboRun,
}

fn ai_thread(
    auto_play: AutoPlay,
    tx: &mut mpsc::Sender<GameSession>,
    rx: &mut mpsc::Receiver<Request>,
) {
    let mut auto_play = auto_play;
    let Ok(mut req) = rx.recv() else {
        return;
    };

    loop {
        match req {
            Request::TogglePause => {
                auto_play.session.toggle_pause();
            }
            Request::Run | Request::TurboRun => {
                auto_play.session.increment_frame();
            }
        }
        tx.send(auto_play.session.clone()).unwrap();

        req = match req {
            Request::TogglePause | Request::Run => match rx.recv() {
                Ok(r) => r,
                Err(RecvError) => return,
            },
            Request::TurboRun => loop {
                match rx.try_recv() {
                    Ok(r) => break r,
                    Err(TryRecvError::Disconnected) => return,
                    Err(TryRecvError::Empty) => auto_play.increment_frame(),
                }
            },
        };
    }
}

#[derive(Debug)]
struct AutoPlay {
    session: GameSession,
    turn_evaluator: TurnEvaluator<'static>,
    best_turn: Option<(TurnPlan, PlacementAnalysis)>,
}

impl AutoPlay {
    fn new(session: GameSession, turn_evaluator: TurnEvaluator<'static>) -> Self {
        Self {
            session,
            turn_evaluator,
            best_turn: None,
        }
    }

    fn increment_frame(&mut self) {
        if !self.session.session_state().is_playing() {
            return;
        }

        self.session.increment_frame();
        if self.best_turn.is_none() {
            self.best_turn = self.turn_evaluator.select_best_turn(self.session.field());
        }

        if self.operate_game() {
            self.best_turn = None;
        }
    }

    pub fn operate_game(&mut self) -> bool {
        let Some((target, _)) = self.best_turn else {
            return true;
        };

        assert!(target.use_hold() || !self.session.hold_used());
        if target.use_hold() && !self.session.hold_used() {
            return self.session.try_hold().is_err();
        }

        let falling_piece = self.session.field().falling_piece();
        assert_eq!(target.placement().kind(), falling_piece.kind());
        if falling_piece.rotation() != target.placement().rotation() {
            return self.session.try_rotate_right().is_err();
        }

        if falling_piece.position().x() < target.placement().position().x() {
            return self.session.try_move_right().is_err();
        } else if falling_piece.position().x() > target.placement().position().x() {
            return self.session.try_move_left().is_err();
        }
        assert_eq!(
            falling_piece.position().x(),
            target.placement().position().x()
        );
        self.session.hard_drop_and_complete();
        true
    }
}
