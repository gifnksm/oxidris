use std::{
    io,
    ops::ControlFlow,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use crossterm::event::KeyCode;
use oxidris_ai::{
    placement_evaluator::FeatureBasedPlacementEvaluator,
    turn_evaluator::{TurnEvaluator, TurnPlan},
};
use oxidris_engine::{GameSession, SessionState};

use crate::{
    data,
    ui::{input::Input, renderer::Renderer},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlayMode {
    Manual,
    Auto,
}

impl PlayMode {
    pub(crate) fn controls(self) -> &'static [(&'static str, &'static str)] {
        match self {
            PlayMode::Manual => &[
                ("Left", "Move left"),
                ("Right", "Move right"),
                ("Down", "Soft drop"),
                ("Up", "Hard drop"),
                ("z", "Rotate left"),
                ("x", "Rotate right"),
                ("Space", "Hold"),
                ("p", "Pause"),
                ("q", "Quit"),
            ],
            PlayMode::Auto => &[("p", "Pause"), ("q", "Quit")],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManualPlayAction {
    MoveLeft,
    MoveRight,
    RotateLeft,
    RotateRight,
    SoftDrop,
    HardDrop,
    Hold,
    Pause,
    Quit,
}

impl ManualPlayAction {
    fn from_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Left => Some(ManualPlayAction::MoveLeft),
            KeyCode::Right => Some(ManualPlayAction::MoveRight),
            KeyCode::Down => Some(ManualPlayAction::SoftDrop),
            KeyCode::Up => Some(ManualPlayAction::HardDrop),
            KeyCode::Char('z') => Some(ManualPlayAction::RotateLeft),
            KeyCode::Char('x') => Some(ManualPlayAction::RotateRight),
            KeyCode::Char(' ') => Some(ManualPlayAction::Hold),
            KeyCode::Char('p') => Some(ManualPlayAction::Pause),
            KeyCode::Char('q') => Some(ManualPlayAction::Quit),
            _ => None,
        }
    }
}

const FPS: u64 = 60;

fn run_game_loop<F>(play_mode: PlayMode, mut handler: F) -> anyhow::Result<()>
where
    F: FnMut(&mut Input, &mut GameSession) -> io::Result<ControlFlow<()>>,
{
    let mut session = GameSession::new(FPS);
    let mut renderer = Renderer::new(play_mode)?;
    let mut input = Input::new()?;

    renderer.draw(&session)?;

    let frame_duration = Duration::from_secs(1) / u32::try_from(FPS).unwrap();

    loop {
        let now = Instant::now();
        let result = handler(&mut input, &mut session)?;
        renderer.draw(&session)?;
        if result.is_break() {
            break;
        }

        let elapsed = now.elapsed();
        if let Some(rest) = frame_duration.checked_sub(elapsed) {
            thread::sleep(rest);
        }
    }

    renderer.cleanup()?;
    input.cleanup()?;

    Ok(())
}

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct ManualPlayArg {}

pub(crate) fn manual(arg: &ManualPlayArg) -> anyhow::Result<()> {
    let ManualPlayArg {} = arg;
    run_game_loop(PlayMode::Manual, |input, game| {
        while let Some(key) = input.try_read()? {
            if let Some(action) = ManualPlayAction::from_key(key) {
                match game.session_state() {
                    SessionState::Playing => match action {
                        ManualPlayAction::MoveLeft => _ = game.try_move_left(),
                        ManualPlayAction::MoveRight => _ = game.try_move_right(),
                        ManualPlayAction::RotateLeft => _ = game.try_rotate_left(),
                        ManualPlayAction::RotateRight => _ = game.try_rotate_right(),
                        ManualPlayAction::SoftDrop => _ = game.try_soft_drop(),
                        ManualPlayAction::HardDrop => game.hard_drop_and_complete(),
                        ManualPlayAction::Hold => _ = game.try_hold(),
                        ManualPlayAction::Pause => game.toggle_pause(),
                        ManualPlayAction::Quit => return Ok(ControlFlow::Break(())),
                    },
                    SessionState::Paused => match action {
                        ManualPlayAction::Pause => game.toggle_pause(),
                        ManualPlayAction::Quit => return Ok(ControlFlow::Break(())),
                        _ => {}
                    },
                    SessionState::GameOver => unreachable!(),
                }
            }
        }

        match game.session_state() {
            SessionState::Paused => Ok(ControlFlow::Continue(())),
            SessionState::GameOver => Ok(ControlFlow::Break(())),
            SessionState::Playing => {
                game.increment_frame();
                Ok(ControlFlow::Continue(()))
            }
        }
    })
}

#[derive(Default, Debug, Clone, clap::Args)]
pub(crate) struct AutoPlayArg {
    /// Path to the model file (JSON format)
    model_path: PathBuf,
}

pub(crate) fn auto(arg: &AutoPlayArg) -> anyhow::Result<()> {
    let AutoPlayArg { model_path } = arg;

    let model = data::load_model(model_path)?;

    let (features, weights) = model.to_feature_weights()?;
    let placement_evaluator = FeatureBasedPlacementEvaluator::new(features, weights);
    let turn_evaluator = TurnEvaluator::new(Box::new(placement_evaluator));
    let mut best_turn = None;
    run_game_loop(PlayMode::Auto, |input, game| {
        while let Some(key) = input.try_read()? {
            match key {
                KeyCode::Char('q') => return Ok(ControlFlow::Break(())),
                KeyCode::Char('p') => game.toggle_pause(),
                _ => {}
            }
        }

        match game.session_state() {
            SessionState::Paused => return Ok(ControlFlow::Continue(())),
            SessionState::GameOver => return Ok(ControlFlow::Break(())),
            SessionState::Playing => game.increment_frame(),
        }

        if best_turn.is_none() {
            best_turn = turn_evaluator.select_best_turn(game.field());
        }

        if let Some((target, _)) = best_turn
            && operate_game(game, target)
        {
            best_turn = None;
        }

        Ok(ControlFlow::Continue(()))
    })
}

fn operate_game(game: &mut GameSession, target: TurnPlan) -> bool {
    assert!(target.use_hold() || !game.hold_used());
    if target.use_hold() && !game.hold_used() {
        return game.try_hold().is_err();
    }

    let falling_piece = game.field().falling_piece();
    assert_eq!(target.placement().kind(), falling_piece.kind());
    if falling_piece.rotation() != target.placement().rotation() {
        return game.try_rotate_right().is_err();
    }

    if falling_piece.position().x() < target.placement().position().x() {
        return game.try_move_right().is_err();
    } else if falling_piece.position().x() > target.placement().position().x() {
        return game.try_move_left().is_err();
    }
    assert_eq!(
        falling_piece.position().x(),
        target.placement().position().x()
    );
    game.hard_drop_and_complete();

    true
}
