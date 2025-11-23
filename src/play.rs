use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use getch_rs::{Getch, Key};

use crate::{
    ga::GenoSeq,
    game::{self, Game},
    ai,
};

pub(crate) fn normal() -> ! {
    let game = Arc::new(Mutex::new(Game::new()));
    game::draw(&game.lock().unwrap());

    {
        let game = Arc::clone(&game);
        let _ = thread::spawn(move || loop {
            let level = game.lock().unwrap().level() as u64;
            let sleep_msec = 100 + u64::saturating_sub(900, level * 100);
            thread::sleep(Duration::from_millis(sleep_msec));

            let mut game = game.lock().unwrap();
            if game::try_drop(&mut game).is_err() && game::landing(&mut game).is_err() {
                game::gameover(&game);
            }
            game::draw(&game);
        });
    }

    let g = Getch::new();
    loop {
        match g.getch() {
            Ok(Key::Left) => {
                let mut game = game.lock().unwrap();
                if game::try_move_left(&mut game).is_ok() {
                    game::draw(&game);
                }
            }
            Ok(Key::Right) => {
                let mut game = game.lock().unwrap();
                if game::try_move_right(&mut game).is_ok() {
                    game::draw(&game);
                }
            }
            Ok(Key::Down) => {
                let mut game = game.lock().unwrap();
                if game::try_drop(&mut game).is_ok() {
                    game::draw(&game);
                }
            }
            Ok(Key::Up) => {
                let mut game = game.lock().unwrap();
                let _ = game::try_hard_drop(&mut game);
                if game::landing(&mut game).is_err() {
                    game::gameover(&game);
                }
                game::draw(&game);
            }
            Ok(Key::Char('z')) => {
                let mut game = game.lock().unwrap();
                if game::try_rotate_left(&mut game).is_ok() {
                    game::draw(&game);
                }
            }
            Ok(Key::Char('x')) => {
                let mut game = game.lock().unwrap();
                if game::try_rotate_right(&mut game).is_ok() {
                    game::draw(&game);
                }
            }
            Ok(Key::Char(' ')) => {
                let mut game = game.lock().unwrap();
                if game::try_hold(&mut game).is_ok() {
                    game::draw(&game);
                }
            }
            Ok(Key::Char('q')) => game::quit(),
            _ => {}
        }
    }
}

pub(crate) fn auto() -> ! {
    let _ = thread::spawn(|| {
        let mut game = Game::new();
        game::draw(&game);
        loop {
            let gameover;
            (game, gameover) = ai::eval(&game, &GenoSeq([100, 1, 10, 100]));
            if gameover {
                game::gameover(&game);
            }
            game::draw(&game);
        }
    });

    let g = Getch::new();
    loop {
        if let Ok(Key::Char('q')) = g.getch() {
            game::quit();
        }
    }
}
