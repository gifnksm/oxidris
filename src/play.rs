use std::{
    process,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::event::KeyCode;

use crate::{ai, ga::GenoSeq, game::Game, input::Input, renderer::Renderer};

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

pub(crate) fn normal() -> ! {
    let game = Arc::new(Mutex::new(Game::new()));
    let mut renderer = Renderer::new(PlayMode::Normal).unwrap();
    renderer.draw(&game.lock().unwrap()).unwrap();
    let renderer = Arc::new(Mutex::new(renderer));

    let _ = thread::spawn({
        let game = Arc::clone(&game);
        let renderer = Arc::clone(&renderer);
        move || {
            loop {
                let level = game.lock().unwrap().level() as u64;
                let sleep_msec = 100 + u64::saturating_sub(900, level * 100);
                thread::sleep(Duration::from_millis(sleep_msec));

                let mut game = game.lock().unwrap();

                // Skip game progression while paused
                if game.state().is_paused() {
                    continue;
                }

                let gameover = game.auto_drop_and_complete().is_gameover();
                let mut renderer = renderer.lock().unwrap();
                renderer.draw(&game).unwrap();

                if gameover {
                    let _ = renderer.cleanup();
                    process::exit(0);
                }
            }
        }
    });

    let mut input = Input::new().unwrap();

    loop {
        let Ok(Some(action)) = input.read().map(NormalModeAction::from_key) else {
            continue;
        };

        let mut game = game.lock().unwrap();

        // During pause, only allow pause toggle and quit
        if game.state().is_paused()
            && !matches!(action, NormalModeAction::Pause | NormalModeAction::Quit)
        {
            continue;
        }

        let updated = match action {
            NormalModeAction::MoveLeft => game.try_move_left().is_ok(),
            NormalModeAction::MoveRight => game.try_move_right().is_ok(),
            NormalModeAction::RotateLeft => game.try_rotate_left().is_ok(),
            NormalModeAction::RotateRight => game.try_rotate_right().is_ok(),
            NormalModeAction::SoftDrop => game.try_soft_drop().is_ok(),
            NormalModeAction::HardDrop => {
                game.hard_drop_and_complete();
                true
            }
            NormalModeAction::Hold => game.try_hold().is_ok(),
            NormalModeAction::Pause => {
                game.toggle_pause();
                true
            }
            NormalModeAction::Quit => {
                renderer.lock().unwrap().cleanup().unwrap();
                crossterm::terminal::disable_raw_mode().unwrap();
                process::exit(0);
            }
        };
        if updated {
            let mut renderer = renderer.lock().unwrap();
            renderer.draw(&game).unwrap();

            // Exit after drawing game over state
            if game.state().is_gameover() {
                let _ = renderer.cleanup();
                let _ = input.cleanup();
                process::exit(0);
            }
        }
    }
}

pub(crate) fn auto() -> ! {
    let game = Game::new();
    let mut renderer = Renderer::new(PlayMode::Auto).unwrap();
    renderer.draw(&game).unwrap();
    let renderer = Arc::new(Mutex::new(renderer));

    let _ = thread::spawn({
        let renderer = Arc::clone(&renderer);
        move || {
            let mut game = game;
            renderer.lock().unwrap().draw(&game).unwrap();
            loop {
                let gameover;
                (game, gameover) = ai::eval(&game, GenoSeq([100, 1, 10, 100]));
                renderer.lock().unwrap().draw(&game).unwrap();

                if gameover {
                    renderer.lock().unwrap().cleanup().unwrap();
                    process::exit(0);
                }
            }
        }
    });

    let mut input = Input::new().unwrap();
    loop {
        if let Ok(KeyCode::Char('q')) = input.read() {
            let _ = renderer.lock().unwrap().cleanup();
            let _ = input.cleanup();
            process::exit(0);
        }
    }
}
