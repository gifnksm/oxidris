use std::{
    io, thread,
    time::{Duration, Instant},
};

use crossterm::event::KeyCode;

use crate::{
    ai::evaluator::{Evaluator, Move},
    engine::session::{GameSession, SessionState},
    ui::{input::Input, renderer::Renderer},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlayMode {
    Normal,
    Auto,
}

impl PlayMode {
    pub(crate) fn controls(self) -> &'static [(&'static str, &'static str)] {
        match self {
            PlayMode::Normal => &[
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
enum NormalModeAction {
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

impl NormalModeAction {
    fn from_key(key: KeyCode) -> Option<Self> {
        match key {
            KeyCode::Left => Some(NormalModeAction::MoveLeft),
            KeyCode::Right => Some(NormalModeAction::MoveRight),
            KeyCode::Down => Some(NormalModeAction::SoftDrop),
            KeyCode::Up => Some(NormalModeAction::HardDrop),
            KeyCode::Char('z') => Some(NormalModeAction::RotateLeft),
            KeyCode::Char('x') => Some(NormalModeAction::RotateRight),
            KeyCode::Char(' ') => Some(NormalModeAction::Hold),
            KeyCode::Char('p') => Some(NormalModeAction::Pause),
            KeyCode::Char('q') => Some(NormalModeAction::Quit),
            _ => None,
        }
    }
}

const FPS: u64 = 60;

pub(crate) fn normal() -> io::Result<()> {
    let mut game = GameSession::new(FPS);
    let mut renderer = Renderer::new(PlayMode::Normal)?;
    renderer.draw(&game)?;
    let mut input = Input::new()?;

    let frame_duration = Duration::from_secs(1) / u32::try_from(FPS).unwrap();
    loop {
        let now = Instant::now();
        let mut quit = false;

        // Handle input
        while let Some(key) = input.try_read()? {
            if let Some(action) = NormalModeAction::from_key(key) {
                match game.session_state() {
                    SessionState::Playing => match action {
                        NormalModeAction::MoveLeft => _ = game.try_move_left(),
                        NormalModeAction::MoveRight => _ = game.try_move_right(),
                        NormalModeAction::RotateLeft => _ = game.try_rotate_left(),
                        NormalModeAction::RotateRight => _ = game.try_rotate_right(),
                        NormalModeAction::SoftDrop => _ = game.try_soft_drop(),
                        NormalModeAction::HardDrop => game.hard_drop_and_complete(),
                        NormalModeAction::Hold => _ = game.try_hold(),
                        NormalModeAction::Pause => game.toggle_pause(),
                        NormalModeAction::Quit => quit = true,
                    },
                    SessionState::Paused => match action {
                        NormalModeAction::Pause => game.toggle_pause(),
                        NormalModeAction::Quit => quit = true,
                        _ => {}
                    },
                    SessionState::GameOver => unreachable!(),
                }
            }
        }

        // Game progression
        if !quit && game.session_state().is_playing() {
            game.increment_frame();
        }

        renderer.draw(&game)?;

        if game.session_state().is_game_over() || quit {
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

pub(crate) fn auto() -> io::Result<()> {
    let mut game = GameSession::new(FPS);
    let mut renderer = Renderer::new(PlayMode::Auto)?;
    renderer.draw(&game)?;
    let mut input = Input::new()?;
    let mut target_move = None;

    let evaluator = Evaluator::default();

    let frame_duration = Duration::from_secs(1) / u32::try_from(FPS).unwrap();
    loop {
        let now = Instant::now();
        let mut quit = false;

        // Handle input
        while let Some(key) = input.try_read()? {
            match key {
                KeyCode::Char('q') => quit = true,
                KeyCode::Char('p') => game.toggle_pause(),
                _ => {}
            }
        }

        // Game progression
        if !quit && game.session_state().is_playing() {
            game.increment_frame();
        }

        renderer.draw(&game)?;

        if game.session_state().is_game_over() || quit {
            break;
        }

        // AI move selection and operation
        if game.session_state().is_playing() {
            if target_move.is_none()
                && let Some((mv, _next_game)) = evaluator.select_move(game.game_state())
            {
                target_move = Some(mv);
            }

            if let Some(tmv) = &target_move
                && operate_game(&mut game, tmv)
            {
                target_move = None;
            }
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

fn operate_game(game: &mut GameSession, target: &Move) -> bool {
    assert!(target.is_hold_used || !game.game_state().is_hold_used());
    if target.is_hold_used && !game.game_state().is_hold_used() {
        return game.try_hold().is_err();
    }

    let falling_piece = game.game_state().falling_piece();
    assert_eq!(target.piece.kind(), falling_piece.kind());
    if falling_piece.rotation() != target.piece.rotation() {
        return game.try_rotate_right().is_err();
    }

    if falling_piece.position().x() < target.piece.position().x() {
        return game.try_move_right().is_err();
    } else if falling_piece.position().x() > target.piece.position().x() {
        return game.try_move_left().is_err();
    }
    assert_eq!(falling_piece.position().x(), target.piece.position().x());
    game.hard_drop_and_complete();

    true
}
