use std::{
    process,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use getch_rs::{Getch, Key};

use crate::{ai, ga::GenoSeq, game::Game, renderer, terminal::Terminal};

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
    Quit,
}

impl NormalModeAction {
    fn from_key(key: Key) -> Option<Self> {
        match key {
            Key::Left => Some(NormalModeAction::MoveLeft),
            Key::Right => Some(NormalModeAction::MoveRight),
            Key::Down => Some(NormalModeAction::SoftDrop),
            Key::Up => Some(NormalModeAction::HardDrop),
            Key::Char('z') => Some(NormalModeAction::RotateLeft),
            Key::Char('x') => Some(NormalModeAction::RotateRight),
            Key::Char(' ') => Some(NormalModeAction::Hold),
            Key::Char('q') => Some(NormalModeAction::Quit),
            _ => None,
        }
    }
}

pub(crate) fn normal() -> ! {
    let game = Arc::new(Mutex::new(Game::new()));
    let term = Arc::new(Mutex::new(Terminal::stdout()));
    renderer::draw(
        &game.lock().unwrap(),
        &mut term.lock().unwrap(),
        PlayMode::Normal,
    )
    .unwrap();

    {
        let game = Arc::clone(&game);
        let term = Arc::clone(&term);
        let _ = thread::spawn(move || {
            loop {
                let level = game.lock().unwrap().level() as u64;
                let sleep_msec = 100 + u64::saturating_sub(900, level * 100);
                thread::sleep(Duration::from_millis(sleep_msec));

                let mut game = game.lock().unwrap();
                let mut term = term.lock().unwrap();
                if game.auto_drop_and_complete().is_gameover() {
                    renderer::gameover(&game, &mut term, PlayMode::Normal).unwrap();
                    let _ = renderer::cleanup(&mut term);
                    process::exit(0);
                }
                renderer::draw(&game, &mut term, PlayMode::Normal).unwrap();
            }
        });
    }

    let g = Getch::new();
    loop {
        let Ok(Some(action)) = g.getch().map(NormalModeAction::from_key) else {
            continue;
        };

        let mut game = game.lock().unwrap();
        let updated = match action {
            NormalModeAction::MoveLeft => game.try_move_left().is_ok(),
            NormalModeAction::MoveRight => game.try_move_right().is_ok(),
            NormalModeAction::RotateLeft => game.try_rotate_left().is_ok(),
            NormalModeAction::RotateRight => game.try_rotate_right().is_ok(),
            NormalModeAction::SoftDrop => game.try_soft_drop().is_ok(),
            NormalModeAction::HardDrop => {
                if game.hard_drop_and_complete().is_gameover() {
                    let mut term = term.lock().unwrap();
                    renderer::gameover(&game, &mut term, PlayMode::Normal).unwrap();
                    let _ = renderer::cleanup(&mut term);
                    process::exit(0);
                }
                true
            }
            NormalModeAction::Hold => game.try_hold().is_ok(),
            NormalModeAction::Quit => {
                let mut term = term.lock().unwrap();
                renderer::cleanup(&mut term).unwrap();
                process::exit(0);
            }
        };
        if updated {
            let mut term = term.lock().unwrap();
            renderer::draw(&game, &mut term, PlayMode::Normal).unwrap();
        }
    }
}

pub(crate) fn auto() -> ! {
    let _ = thread::spawn(|| {
        let mut game = Game::new();
        let mut term = Terminal::stdout();
        renderer::draw(&game, &mut term, PlayMode::Auto).unwrap();
        loop {
            let gameover;
            (game, gameover) = ai::eval(&game, &GenoSeq([100, 1, 10, 100]));
            if gameover {
                let _ = renderer::gameover(&game, &mut term, PlayMode::Auto);
                let _ = renderer::cleanup(&mut term);
                process::exit(0);
            }
            renderer::draw(&game, &mut term, PlayMode::Auto).unwrap();
        }
    });

    let g = Getch::new();
    let mut term = Terminal::stdout();
    loop {
        if let Ok(Key::Char('q')) = g.getch() {
            renderer::cleanup(&mut term).unwrap();
            process::exit(0);
        }
    }
}
