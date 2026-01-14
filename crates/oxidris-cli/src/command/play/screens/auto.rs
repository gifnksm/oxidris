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
    turn_evaluator: TurnEvaluator<'static>,
    best_turn: Option<(TurnPlan, PlacementAnalysis)>,
    is_exiting: bool,
}

impl AutoPlayScreen {
    pub fn new(fps: u64, turn_evaluator: TurnEvaluator<'static>) -> Self {
        Self {
            session: GameSession::new(fps),
            turn_evaluator,
            best_turn: None,
            is_exiting: false,
        }
    }

    pub fn is_playing(&self) -> bool {
        !self.is_exiting && self.session.session_state().is_playing()
    }

    pub fn is_exiting(&self) -> bool {
        self.is_exiting
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        let session_display = SessionDisplay::new(&self.session, false);
        let help_text = match self.session.session_state() {
            SessionState::Playing => "Controls: P (Pause) | Q (Quit)",
            SessionState::Paused => "Controls: P (Resume) | Q (Quit)",
            SessionState::GameOver => "Controls: Q (Quit)",
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
                KeyCode::Char('p') if can_toggle_pause => self.session.toggle_pause(),
                KeyCode::Char('q') => self.is_exiting = true,
                _ => {}
            }
        }
    }

    pub fn update_game(&mut self) {
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
