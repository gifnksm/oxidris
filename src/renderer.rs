use std::{io, sync::Once};

use crate::{block::BlockKind, game::Game, play::PlayMode, terminal::Terminal};

// UI layout coordinates
const FIELD_ROW: usize = 1;
const FIELD_COL: usize = 15;

const HOLD_ROW: usize = 1;
const HOLD_COL: usize = 1;

const SCORE_ROW: usize = 10;
const SCORE_COL: usize = 1;

const NEXT_ROW: usize = 1;
const NEXT_COL: usize = 45;

const CONTROLS_ROW: usize = 1;
const CONTROLS_COL: usize = 65;

/// Draw the game field
fn draw_field(terminal: &mut Terminal, game: &Game, mode: PlayMode) -> io::Result<()> {
    terminal.reset_styles()?;

    // Prepare field with ghost piece and falling mino
    let field = game.field();
    let (falling_mino_pos, falling_mino) = game.falling_mino();
    let mut field_buf = field.clone();

    // Show ghost piece only in normal mode
    if mode == PlayMode::Normal {
        let drop_pos = game.simulate_drop_position();
        field_buf.fill_mino_as(&drop_pos, falling_mino, BlockKind::Ghost);
    }
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
        .set_bold()?
        .set_underline()?
        .write("HOLD")?
        .reset_styles()?;

    if let Some(hold) = game.held_mino() {
        for (row_offset, mino_row) in hold.iter().enumerate() {
            terminal.move_to(HOLD_ROW + row_offset + 2, HOLD_COL)?;
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
        .set_bold()?
        .set_underline()?
        .write("NEXT")?
        .reset_styles()?;

    for (mino_idx, next) in game.next_minos().iter().take(7).enumerate() {
        // Show only rows 1 and 2 (indices 1 and 2) for compact display
        for (display_row, mino_row) in next[1..=2].iter().enumerate() {
            let row_position = NEXT_ROW + 2 + mino_idx * 3 + display_row; // 3 rows per mino (2 display + 1 gap)
            terminal.move_to(row_position, NEXT_COL)?;
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
        .set_bold()?
        .set_underline()?
        .write("SCORE")?
        .reset_styles()?
        .move_to(SCORE_ROW + 2, SCORE_COL)?
        .write(format!("{:>8}", game.score()))?;
    Ok(())
}

/// Draw the controls panel
fn draw_controls_panel(terminal: &mut Terminal, _game: &Game, mode: PlayMode) -> io::Result<()> {
    terminal
        .reset_styles()?
        .move_to(CONTROLS_ROW, CONTROLS_COL)?
        .set_bold()?
        .set_underline()?
        .write("CONTROLS")?
        .reset_styles()?;

    for (line_offset, (key, description)) in mode.controls().iter().enumerate() {
        terminal
            .move_to(CONTROLS_ROW + line_offset + 2, CONTROLS_COL)?
            .write(format!("{:<12} : {}", key, description))?;
    }
    Ok(())
}

/// Draw pause overlay on the game field
fn draw_pause_overlay(terminal: &mut Terminal) -> io::Result<()> {
    use crate::terminal::Color;

    let msg_row = FIELD_ROW + 9;
    let msg_col = FIELD_COL;
    let field_width = 24;

    terminal
        .reset_styles()?
        .move_to(msg_row, msg_col)?
        .set_bg(Color::YELLOW)?
        .set_fg(Color::BLACK)?
        .write(format_args!("{:width$}", "", width = field_width))?
        .move_to(msg_row + 1, msg_col)?
        .set_bold()?
        .write(format_args!("{:^width$}", "PAUSED", width = field_width))?
        .move_to(msg_row + 2, msg_col)?
        .write(format_args!("{:width$}", "", width = field_width))?
        .reset_styles()?;

    Ok(())
}

pub(crate) fn draw(game: &Game, term: &mut Terminal, mode: PlayMode) -> io::Result<()> {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = term.clear_screen();
        let _ = term.hide_cursor();
    });

    // Draw each UI element
    draw_field(term, game, mode)?;
    draw_hold_panel(term, game)?;
    draw_next_panel(term, game)?;
    draw_score_panel(term, game)?;
    draw_controls_panel(term, game, mode)?;

    if game.is_paused() {
        draw_pause_overlay(term)?;
    }

    term.flush()?;
    Ok(())
}

pub(crate) fn gameover(game: &Game, term: &mut Terminal, mode: PlayMode) -> io::Result<()> {
    use crate::terminal::Color;

    draw(game, term, mode)?;
    let msg_row = FIELD_ROW + 9;
    let msg_col = FIELD_COL;
    let field_width = 24;

    term.reset_styles()?
        .move_to(msg_row, msg_col)?
        .set_bg(Color::RED)?
        .set_fg(Color::WHITE)?
        .write(format_args!("{:width$}", "", width = field_width))?
        .move_to(msg_row + 1, msg_col)?
        .set_bold()?
        .write(format_args!(
            "{:^width$}",
            "GAME OVER!!",
            width = field_width
        ))?
        .move_to(msg_row + 2, msg_col)?
        .write(format_args!("{:width$}", "", width = field_width))?
        .reset_styles()?;

    term.flush()?;
    Ok(())
}

pub(crate) fn cleanup(term: &mut Terminal) -> io::Result<()> {
    term.reset_styles()?
        .move_to(25, 1)? // Move cursor to bottom of screen to prevent overwriting
        .show_cursor()?;
    term.flush()?;
    Ok(())
}
