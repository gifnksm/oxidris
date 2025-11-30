use std::{
    io, process,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::KeyCode;

use crate::{
    ai,
    ga::GenoSeq,
    game::{Game, GameState},
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
    let mut game = Game::new(FPS);
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
                        NormalModeAction::HardDrop => _ = game.hard_drop_and_complete(),
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
            let _ = game.increment_frame();
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

pub(crate) fn auto() -> ! {
    let game = Game::new(FPS);
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
