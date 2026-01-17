use std::{
    ops::ControlFlow,
    sync::mpsc::{self, RecvError, TryRecvError},
    thread,
};

use crossterm::event::{Event, KeyCode, KeyEvent};
use oxidris_engine::{GameSession, SessionState};
use oxidris_evaluator::{
    placement_analysis::PlacementAnalysis,
    placement_evaluator::FeatureBasedPlacementEvaluator,
    turn_evaluator::{TurnEvaluator, TurnPlan},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::{
    record::{RecordingSession, SessionHistory},
    schema::{ai_model::AiModel, record::PlayerInfo},
    view::widgets::{KeyBinding, KeyBindingDisplay, SessionDisplay},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlayingAction {
    ToggleTurbo,
    Pause,
    Quit,
}

impl PlayingAction {
    fn from_key_event(event: &KeyEvent) -> Option<Self> {
        match event.code {
            KeyCode::Char('t') => Some(Self::ToggleTurbo),
            KeyCode::Char('p') => Some(Self::Pause),
            KeyCode::Char('q') | KeyCode::Esc => Some(Self::Quit),
            _ => None,
        }
    }

    fn bindings(turbo: bool) -> &'static [KeyBinding<'static>] {
        if turbo {
            &[
                (&["t"], "Toggle Turbo (current:ON) "),
                (&["p"], "Pause"),
                (&["q", "Esc"], "Quit"),
            ]
        } else {
            &[
                (&["t"], "Toggle Turbo (current:OFF)"),
                (&["p"], "Pause"),
                (&["q", "Esc"], "Quit"),
            ]
        }
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
pub struct AutoPlayScreen {
    session: GameSession,
    turbo: bool,
    should_exit: bool,
    tx_request: mpsc::Sender<Request>,
    rx_session: mpsc::Receiver<GameSession>,
    rx_history: mpsc::Receiver<SessionHistory>,
}

impl AutoPlayScreen {
    pub fn new(
        tick_rate: f64,
        model: &AiModel,
        history_size: usize,
        turbo: bool,
    ) -> anyhow::Result<Self> {
        let rec_session = RecordingSession::new(
            tick_rate,
            PlayerInfo::Auto {
                model: model.clone(),
            },
            history_size,
        );
        let session = (*rec_session).clone();
        let auto_play = AutoPlay::new(rec_session, model)?;
        let (tx_request, mut rx_request) = mpsc::channel();
        let (mut tx_session, rx_session) = mpsc::channel();
        let (mut tx_history, rx_history) = mpsc::channel();
        thread::spawn(move || {
            ai_thread(auto_play, &mut tx_session, &mut tx_history, &mut rx_request);
        });
        Ok(Self {
            session,
            turbo,
            should_exit: false,
            tx_request,
            rx_session,
            rx_history,
        })
    }

    pub fn is_playing(&self) -> bool {
        !self.should_exit && self.session.session_state().is_playing()
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        let session_display = SessionDisplay::new(&self.session, false).turbo(self.turbo);
        let bindings = match self.session.session_state() {
            SessionState::Playing => PlayingAction::bindings(self.turbo),
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
                            PlayingAction::ToggleTurbo => self.turbo = !self.turbo,
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
        let req = {
            if self.session.session_state().is_paused() {
                Request::TogglePause
            } else if self.turbo {
                Request::TurboRun
            } else {
                Request::Run
            }
        };
        self.tx_request.send(req).unwrap();
        self.session = self.rx_session.recv().unwrap();
    }

    pub fn into_history(self) -> SessionHistory {
        self.tx_request
            .send(Request::SessionHistoryAndExit)
            .unwrap();
        self.rx_history.recv().unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
enum Request {
    TogglePause,
    Run,
    TurboRun,
    SessionHistoryAndExit,
}

fn ai_thread(
    auto_play: AutoPlay,
    tx_session: &mut mpsc::Sender<GameSession>,
    tx_history: &mut mpsc::Sender<SessionHistory>,
    rx_request: &mut mpsc::Receiver<Request>,
) {
    let mut auto_play = auto_play;
    let Ok(mut req) = rx_request.recv() else {
        return;
    };

    loop {
        let is_turbo = match req {
            Request::TogglePause => {
                auto_play.session.toggle_pause();
                false
            }
            Request::Run => {
                auto_play.increment_frame();
                false
            }
            Request::TurboRun => {
                auto_play.increment_frame();
                true
            }
            Request::SessionHistoryAndExit => {
                tx_history.send(auto_play.session.into_history()).unwrap();
                return;
            }
        };
        tx_session.send(auto_play.session.clone()).unwrap();

        req = if is_turbo {
            loop {
                match rx_request.try_recv() {
                    Ok(r) => break r,
                    Err(TryRecvError::Disconnected) => return,
                    Err(TryRecvError::Empty) => auto_play.increment_frame(),
                }
            }
        } else {
            match rx_request.recv() {
                Ok(r) => r,
                Err(RecvError) => return,
            }
        }
    }
}

#[derive(Debug)]
struct AutoPlay {
    session: RecordingSession,
    turn_evaluator: TurnEvaluator<'static>,
    best_turn: Option<(TurnPlan, PlacementAnalysis)>,
}

impl AutoPlay {
    fn new(session: RecordingSession, model: &AiModel) -> anyhow::Result<Self> {
        let (features, weights) = model.to_feature_weights()?;
        let placement_evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
        let turn_evaluator = TurnEvaluator::new(Box::new(placement_evaluator));
        Ok(Self {
            session,
            turn_evaluator,
            best_turn: None,
        })
    }

    fn increment_frame(&mut self) {
        if !self.session.session_state().is_playing() {
            return;
        }

        // Check if a piece was completed during this frame.
        // increment_frame() may trigger auto_drop_and_complete(), which spawns a new piece.
        // If that happens, we need to discard the old plan and select a new one.
        let turn = self.session.stats().turn();
        self.session.increment_frame();
        let piece_completed = turn != self.session.stats().turn();

        // Reselect plan if:
        // - A piece was completed (new piece spawned)
        // - No plan exists (previous operation failed or completed)
        if piece_completed || self.best_turn.is_none() {
            let hold_available = !self.session.hold_used();
            self.best_turn = self
                .turn_evaluator
                .select_best_turn(self.session.field(), hold_available);
        }

        if self.operate_game().is_break() {
            self.best_turn = None;
        }
    }

    /// Executes one step of the current plan.
    ///
    /// Operations are executed in order: hold → rotation → horizontal movement → drop.
    /// Each operation that fails will cause the plan to be discarded and reselected.
    ///
    /// # Returns
    ///
    /// - `ControlFlow::Continue(())` - Plan step executed successfully, continue in next frame
    /// - `ControlFlow::Break(())` - Plan is complete or failed, needs reselection
    pub fn operate_game(&mut self) -> ControlFlow<()> {
        fn ret<E>(res: Result<(), E>) -> ControlFlow<()> {
            match res {
                Ok(()) => ControlFlow::Continue(()),
                Err(_err) => ControlFlow::Break(()),
            }
        }

        let Some((target, _)) = self.best_turn else {
            return ControlFlow::Break(());
        };

        // Step 1: Use hold if the plan requires it and hold is available
        //
        // Note: target.use_hold() and self.session.hold_used() can be inconsistent
        // in the following scenario:
        //
        //   1. A plan with use_hold=true is selected and hold is executed
        //   2. hold_used() becomes true
        //   3. Subsequent operations (rotation/movement) fail
        //   4. The plan is discarded (best_turn = None)
        //   5. A new plan is selected with hold_available=false (since hold_used=true)
        //   6. The new plan has use_hold=false, but hold_used is still true
        //
        // This is valid - the evaluator provides the best plan for the current state.
        // So we only attempt to use hold if the plan requires it and hold is available.
        if target.use_hold() && !self.session.hold_used() {
            return ret(self.session.try_hold());
        }

        // Step 2: Rotate to target orientation
        let falling_piece = self.session.field().falling_piece();
        assert_eq!(target.placement().kind(), falling_piece.kind());
        if falling_piece.rotation() != target.placement().rotation() {
            return ret(self.session.try_rotate_right());
        }

        // Step 3: Move horizontally to target position
        if falling_piece.position().x() < target.placement().position().x() {
            return ret(self.session.try_move_right());
        }
        if falling_piece.position().x() > target.placement().position().x() {
            return ret(self.session.try_move_left());
        }
        assert_eq!(
            falling_piece.position().x(),
            target.placement().position().x()
        );

        // Step 4: Drop and complete placement
        self.session.hard_drop_and_complete();
        ControlFlow::Break(())
    }
}
