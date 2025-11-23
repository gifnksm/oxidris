use std::{
    process,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use getch_rs::{Getch, Key};

use crate::{
    ai,
    ga::GenoSeq,
    game::{self, Game},
    renderer,
    terminal::Terminal,
};

pub(crate) fn normal() -> ! {
    let game = Arc::new(Mutex::new(Game::new()));
    let term = Arc::new(Mutex::new(Terminal::stdout()));
    renderer::draw(&game.lock().unwrap(), &mut term.lock().unwrap()).unwrap();

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
                if game::try_drop(&mut game).is_err() && game::landing(&mut game).is_err() {
                    renderer::gameover(&game, &mut term).unwrap();
                    let _ = renderer::cleanup(&mut term);
                    process::exit(0);
                }
                renderer::draw(&game, &mut term).unwrap();
            }
        });
    }

    let g = Getch::new();
    loop {
        match g.getch() {
            Ok(Key::Left) => {
                let mut game = game.lock().unwrap();
                let mut term = term.lock().unwrap();
                if game::try_move_left(&mut game).is_ok() {
                    renderer::draw(&game, &mut term).unwrap();
                }
            }
            Ok(Key::Right) => {
                let mut game = game.lock().unwrap();
                let mut term = term.lock().unwrap();
                if game::try_move_right(&mut game).is_ok() {
                    renderer::draw(&game, &mut term).unwrap();
                }
            }
            Ok(Key::Down) => {
                let mut game = game.lock().unwrap();
                let mut term = term.lock().unwrap();
                if game::try_drop(&mut game).is_ok() {
                    renderer::draw(&game, &mut term).unwrap();
                }
            }
            Ok(Key::Up) => {
                let mut game = game.lock().unwrap();
                let mut term = term.lock().unwrap();
                let _ = game::try_hard_drop(&mut game);
                if game::landing(&mut game).is_err() {
                    let _ = renderer::gameover(&game, &mut term);
                    let _ = renderer::cleanup(&mut term);
                    process::exit(0);
                }
                renderer::draw(&game, &mut term).unwrap();
            }
            Ok(Key::Char('z')) => {
                let mut game = game.lock().unwrap();
                let mut term = term.lock().unwrap();
                if game::try_rotate_left(&mut game).is_ok() {
                    renderer::draw(&game, &mut term).unwrap();
                }
            }
            Ok(Key::Char('x')) => {
                let mut game = game.lock().unwrap();
                let mut term = term.lock().unwrap();
                if game::try_rotate_right(&mut game).is_ok() {
                    renderer::draw(&game, &mut term).unwrap();
                }
            }
            Ok(Key::Char(' ')) => {
                let mut game = game.lock().unwrap();
                let mut term = term.lock().unwrap();
                if game::try_hold(&mut game).is_ok() {
                    renderer::draw(&game, &mut term).unwrap();
                }
            }
            Ok(Key::Char('q')) => {
                let mut term = term.lock().unwrap();
                renderer::cleanup(&mut term).unwrap();
                process::exit(0);
            }
            _ => {}
        }
    }
}

pub(crate) fn auto() -> ! {
    let _ = thread::spawn(|| {
        let mut game = Game::new();
        let mut term = Terminal::stdout();
        renderer::draw(&game, &mut term).unwrap();
        loop {
            let gameover;
            (game, gameover) = ai::eval(&game, &GenoSeq([100, 1, 10, 100]));
            if gameover {
                let _ = renderer::gameover(&game, &mut term);
                let _ = renderer::cleanup(&mut term);
                process::exit(0);
            }
            renderer::draw(&game, &mut term).unwrap();
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
