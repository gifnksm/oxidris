use std::{io, sync::Once};

use crate::{block::BlockKind, game::Game, terminal::Terminal};

pub(crate) fn draw(game: &Game, term: &mut Terminal) -> io::Result<()> {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = term.clear_screen();
        let _ = term.hide_cursor();
    });

    let field = game.field();
    let (falling_mino_pos, falling_mino) = game.falling_mino();
    let held_mino = game.held_mino();
    let next_minos = game.next_minos();
    let score = game.score();

    let mut field_buf = field.clone();

    let drop_pos = game.simulate_drop_position();
    field_buf.fill_mino_as(&drop_pos, falling_mino, BlockKind::Ghost);
    field_buf.fill_mino(&falling_mino_pos, falling_mino);

    term.reset_styles()?;
    term.move_to(1, 26)?.write("HOLD")?;
    if let Some(hold) = held_mino {
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
    for (i, next) in next_minos.iter().take(3).enumerate() {
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
    for row in field_buf.render_rows() {
        for block in row {
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
