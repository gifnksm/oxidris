use std::{io, sync::Once};

use crate::{
    block::BlockKind,
    game::{self, Game},
    terminal::Terminal,
};

const FIELD_WIDTH: usize = 12 + 2;
const FIELD_HEIGHT: usize = 22 + 1;

pub(crate) fn draw(game: &Game, term: &mut Terminal) -> io::Result<()> {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = term.clear_screen();
        let _ = term.hide_cursor();
    });

    let field = game.field();
    let pos = game.pos();
    let mino = game.mino();
    let hold = game.hold();
    let next = game.next();
    let score = game.score();

    let mut field_buf = *field;

    let ghost_pos = game::ghost_pos(field, pos, mino);
    for (y, row) in mino.iter().enumerate() {
        for (x, block) in row.iter().enumerate() {
            if !block.is_empty() {
                field_buf[y + ghost_pos.y][x + ghost_pos.x] = BlockKind::Ghost;
            }
        }
    }

    for (y, row) in mino.iter().enumerate() {
        for (x, block) in row.iter().enumerate() {
            if !block.is_empty() {
                field_buf[y + pos.y][x + pos.x] = *block;
            }
        }
    }

    term.reset_styles()?;
    term.move_to(1, 26)?.write("HOLD")?;
    if let Some(hold) = hold {
        for (y, row) in hold.iter().enumerate() {
            term.move_to(y + 2, 26)?;
            for &block in row {
                let display = block.display();
                term.set_fg(display.fg())?
                    .set_bg(display.bg())?
                    .write(display.symbol())?;
            }
        }
    }

    term.reset_styles()?;
    term.move_to(7, 26)?.write("NEXT")?;
    for (i, next) in next.iter().take(3).enumerate() {
        for (y, row) in next.iter().enumerate() {
            term.move_to(i * 4 + y + 8, 26)?;
            for block in row {
                let display = block.display();
                term.set_fg(display.fg())?
                    .set_bg(display.bg())?
                    .write(display.symbol())?;
            }
        }
    }

    term.reset_styles()?;
    term.move_to(21, 26)?.write("SCORE")?;
    term.move_to(22, 26)?.write(format!("{score:>8}"))?;

    // Display controls section together
    term.move_to(1, 40)?.write("CONTROLS")?;
    term.move_to(2, 40)?.write("Left/Right : Move left/right")?;
    term.move_to(3, 40)?.write("Down       : Soft drop")?;
    term.move_to(4, 40)?.write("Up         : Hard drop")?;
    term.move_to(5, 40)?.write("z          : Rotate left")?;
    term.move_to(6, 40)?.write("x          : Rotate right")?;
    term.move_to(7, 40)?.write("Space      : Hold")?;
    term.move_to(8, 40)?.write("q          : Quit")?;

    term.move_home()?;
    for row in &field_buf[0..FIELD_HEIGHT - 1] {
        for &block in &row[1..FIELD_WIDTH - 1] {
            let display = block.display();
            term.set_fg(display.fg())?
                .set_bg(display.bg())?
                .write(display.symbol())?;
        }
        term.newline()?;
    }

    term.flush()?;
    Ok(())
}

pub(crate) fn gameover(game: &Game, term: &mut Terminal) -> io::Result<()> {
    draw(game, term)?;
    term.write("GAME OVER")?;
    term.newline()?;
    term.flush()?;
    Ok(())
}

pub(crate) fn cleanup(term: &mut Terminal) -> io::Result<()> {
    term.show_cursor()?;
    term.flush()?;
    Ok(())
}
