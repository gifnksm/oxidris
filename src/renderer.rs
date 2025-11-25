use std::{io, sync::Once};

use crate::{block::BlockKind, game::Game, terminal::Terminal};

// UI layout coordinates
const FIELD_ROW: usize = 1;
const FIELD_COL: usize = 15;

const HOLD_ROW: usize = 1;
const HOLD_COL: usize = 1;

const SCORE_ROW: usize = 10;
const SCORE_COL: usize = 1;

const NEXT_ROW: usize = 1;
const NEXT_COL: usize = 45;

const CONTROLS_ROW: usize = 10;
const CONTROLS_COL: usize = 45;

/// Draw the game field
fn draw_field(terminal: &mut Terminal, game: &Game) -> io::Result<()> {
    terminal.reset_styles()?;

    // Prepare field with ghost piece and falling mino
    let field = game.field();
    let (falling_mino_pos, falling_mino) = game.falling_mino();
    let mut field_buf = field.clone();

    let drop_pos = game.simulate_drop_position();
    field_buf.fill_mino_as(&drop_pos, falling_mino, BlockKind::Ghost);
    field_buf.fill_mino(&falling_mino_pos, falling_mino);

    for (row_offset, field_row) in field_buf.render_rows().enumerate() {
        terminal.move_to(FIELD_ROW + row_offset, FIELD_COL)?;
        for block in field_row {
            let display = block.display();
            terminal
                .set_fg(display.fg())?
                .set_bg(display.bg())?
                .write(display.symbol())?;
        }
    }
    Ok(())
}

/// Draw the HOLD panel
fn draw_hold_panel(terminal: &mut Terminal, game: &Game) -> io::Result<()> {
    terminal
        .reset_styles()?
        .move_to(HOLD_ROW, HOLD_COL)?
        .write("HOLD")?;

    if let Some(hold) = game.held_mino() {
        for (row_offset, mino_row) in hold.iter().enumerate() {
            terminal.move_to(HOLD_ROW + row_offset + 1, HOLD_COL)?;
            for &block in mino_row {
                let display = block.display();
                terminal
                    .set_fg(display.fg())?
                    .set_bg(display.bg())?
                    .write(display.symbol())?;
            }
        }
    }
    Ok(())
}

/// Draw the NEXT panel
fn draw_next_panel(terminal: &mut Terminal, game: &Game) -> io::Result<()> {
    terminal
        .reset_styles()?
        .move_to(NEXT_ROW, NEXT_COL)?
        .write("NEXT")?;

    for (mino_idx, next) in game.next_minos().iter().take(3).enumerate() {
        for (row_offset, mino_row) in next.iter().enumerate() {
            terminal.move_to(NEXT_ROW + mino_idx * 4 + row_offset + 1, NEXT_COL)?;
            for block in mino_row {
                let display = block.display();
                terminal
                    .set_fg(display.fg())?
                    .set_bg(display.bg())?
                    .write(display.symbol())?;
            }
        }
    }
    Ok(())
}

/// Draw the score panel
fn draw_score_panel(terminal: &mut Terminal, game: &Game) -> io::Result<()> {
    terminal
        .reset_styles()?
        .move_to(SCORE_ROW, SCORE_COL)?
        .write("SCORE")?
        .move_to(SCORE_ROW + 1, SCORE_COL)?
        .write(format!("{:>8}", game.score()))?;
    Ok(())
}

/// Draw the controls panel
fn draw_controls_panel(terminal: &mut Terminal, _game: &Game) -> io::Result<()> {
    const CONTROLS: &[(&str, &str)] = &[
        ("Left/Right", "Move left/right"),
        ("Down", "Soft drop"),
        ("Up", "Hard drop"),
        ("z", "Rotate left"),
        ("x", "Rotate right"),
        ("Space", "Hold"),
        ("q", "Quit"),
    ];

    terminal
        .reset_styles()?
        .move_to(CONTROLS_ROW, CONTROLS_COL)?
        .write("CONTROLS")?;

    for (line_offset, (key, description)) in CONTROLS.iter().enumerate() {
        terminal
            .move_to(CONTROLS_ROW + line_offset + 1, CONTROLS_COL)?
            .write(format!("{:<12} : {}", key, description))?;
    }
    Ok(())
}

pub(crate) fn draw(game: &Game, term: &mut Terminal) -> io::Result<()> {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = term.clear_screen();
        let _ = term.hide_cursor();
    });

    // Draw each UI element
    draw_field(term, game)?;
    draw_hold_panel(term, game)?;
    draw_next_panel(term, game)?;
    draw_score_panel(term, game)?;
    draw_controls_panel(term, game)?;

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
