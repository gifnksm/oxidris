use std::{
    io, thread,
    time::{Duration, Instant},
};

use crossterm::event::KeyCode;

use crate::{
    ai::{self, Move},
    ga::GenoSeq,
    game::{GameState, GameUi},
    input::Input,
    renderer::Renderer,
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
                ("Left/Right", "Move left/right"),
                ("Down", "Soft drop"),
                ("Up", "Hard drop"),
                ("z", "Rotate left"),
                ("x", "Rotate right"),
                ("Space", "Hold"),
                ("p", "Pause"),
                ("q", "Quit"),
            ],
            PlayMode::Auto => &[("q", "Quit")],
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
    let mut game = GameUi::new(FPS);
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
                match game.state() {
                    GameState::Playing => match action {
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
                    GameState::Paused => match action {
                        NormalModeAction::Pause => game.toggle_pause(),
                        NormalModeAction::Quit => quit = true,
                        _ => {}
                    },
                    GameState::GameOver => unreachable!(),
                }
            }
        }

        // Game progression
        if !quit && game.state().is_playing() {
            game.increment_frame();
        }

        renderer.draw(&game)?;

        if game.state().is_game_over() || quit {
            break;
        }

        let elapsed = now.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }

    renderer.cleanup()?;
    input.cleanup()?;

    Ok(())
}

pub(crate) fn auto() -> io::Result<()> {
    let mut game = GameUi::new(FPS);
    let mut renderer = Renderer::new(PlayMode::Auto).unwrap();
    renderer.draw(&game).unwrap();
    let mut input = Input::new()?;
    let mut target_move = None;

    let frame_duration = Duration::from_secs(1) / u32::try_from(FPS).unwrap();
    loop {
        let now = Instant::now();
        let mut quit = false;

        // Handle input
        while let Some(key) = input.try_read()? {
            if key == KeyCode::Char('q') {
                quit = true;
            }
        }

        // Game progression
        if !quit && game.state().is_playing() {
            game.increment_frame();
        }

        renderer.draw(&game)?;

        if game.state().is_game_over() || quit {
            break;
        }

        if target_move.is_none()
            && let Some((mv, _next_game)) = ai::eval(game.core(), GenoSeq([100, 1, 10, 100]))
        {
            target_move = Some(mv);
        }

        if let Some(tmv) = &target_move
            && operate_game(&mut game, tmv)
        {
            target_move = None;
        }

        let elapsed = now.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }
    renderer.cleanup()?;
    input.cleanup()?;
    Ok(())
}

fn operate_game(game: &mut GameUi, target: &Move) -> bool {
    assert!(target.is_hold_used || !game.core().is_hold_used());
    if target.is_hold_used && !game.core().is_hold_used() {
        return game.try_hold().is_err();
    }

    let falling_mino = game.core().falling_mino();
    assert_eq!(target.mino.kind(), falling_mino.kind());
    if falling_mino.rotation() != target.mino.rotation() {
        return game.try_rotate_right().is_err();
    }

    if falling_mino.position().x() < target.mino.position().x() {
        return game.try_move_right().is_err();
    } else if falling_mino.position().x() > target.mino.position().x() {
        return game.try_move_left().is_err();
    }
    assert_eq!(falling_mino.position().x(), target.mino.position().x());
    game.hard_drop_and_complete();

    true
}
