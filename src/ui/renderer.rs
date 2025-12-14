use std::io;

use crate::{
    core::{
        piece::{PieceKind, PieceRotation},
        render_board::{RenderBoard, RenderCell},
    },
    engine::session::{GameSession, SessionState},
    modes::PlayMode,
};

use super::terminal::{Color, Terminal};

// UI layout coordinates
const TERMINAL_TOP: usize = 0;
const TERMINAL_BOTTOM: usize = COMPONENT_BOTTOM + 2;
const TERMINAL_LEFT: usize = 0;
const TERMINAL_RIGHT: usize = COMPONENT_RIGHT + 1;

const COMPONENT_TOP: usize = TERMINAL_TOP + 1;
const COMPONENT_BOTTOM: usize = BOARD_PANEL.bottom();
const COMPONENT_LEFT: usize = TERMINAL_LEFT + 2;
const COMPONENT_RIGHT: usize = CONTROLS_PANEL.right();

const CHARS_PER_CELL: usize = 2;
const PIECE_DISPLAY_WIDTH: usize = 4 * CHARS_PER_CELL;
const PIECE_DISPLAY_HEIGHT: usize = 2;

const LEFT_PANE_BODY_WIDTH: usize = 16;

const HOLD_PANEL: Panel = Panel {
    top: COMPONENT_TOP,
    left: COMPONENT_LEFT,
    body_width: LEFT_PANE_BODY_WIDTH,
    body_height: PIECE_DISPLAY_HEIGHT,
    title: "HOLD",
};

const STATS_PANEL: Panel = Panel {
    top: HOLD_PANEL.bottom() + 1,
    left: COMPONENT_LEFT,
    body_width: LEFT_PANE_BODY_WIDTH,
    body_height: 13,
    title: "STATS",
};

const BOARD_PANEL: Panel = Panel {
    top: COMPONENT_TOP,
    left: HOLD_PANEL.right() + 2,
    body_width: CHARS_PER_CELL * RenderBoard::PLAYABLE_WIDTH,
    body_height: RenderBoard::PLAYABLE_HEIGHT,
    title: "",
};

const NEXT_PANEL: Panel = Panel {
    top: COMPONENT_TOP,
    left: BOARD_PANEL.right() + 2,
    body_width: PIECE_DISPLAY_WIDTH,
    body_height: 20,
    title: "NEXT",
};

const CONTROLS_PANEL: Panel = Panel {
    top: COMPONENT_TOP,
    left: NEXT_PANEL.right() + 2,
    body_width: 30,
    body_height: 12,
    title: "CONTROLS",
};

#[derive(Debug, Clone, Copy)]
struct Panel {
    top: usize,
    left: usize,
    body_width: usize,
    body_height: usize,
    title: &'static str,
}

impl Panel {
    const fn width(&self) -> usize {
        self.body_width + 4 // for borders and padding
    }

    const fn height(&self) -> usize {
        self.body_height + 2 // for borders and title
    }

    const fn bottom(&self) -> usize {
        self.top + self.height()
    }

    const fn right(&self) -> usize {
        self.left + self.width()
    }

    const fn body_top(&self) -> usize {
        self.top + 1 // for title and border
    }

    const fn body_left(&self) -> usize {
        self.left + 2 // for border and padding
    }

    fn draw_border(&self, term: &mut Terminal) -> io::Result<()> {
        // Draw top border with title
        term.reset_styles()?
            .set_bg(Color::BLACK)?
            .set_fg(Color::WHITE)?;

        term.move_to(self.top, self.left)?
            .write("┌─")?
            .write(format_args!(
                "{:─<width$}",
                self.title,
                width = self.width() - 4
            ))?
            .write("─┐")?;

        // Draw side borders
        for row in (self.top + 1)..(self.bottom() - 1) {
            term.move_to(row, self.left)?
                .write("│")?
                .move_to(row, self.right() - 1)?
                .write("│")?;
        }

        // Draw bottom border
        term.move_to(self.bottom() - 1, self.left)?
            .write("└")?
            .write(format_args!("{:─<width$}", "", width = self.width() - 2))?
            .write("┘")?;

        Ok(())
    }
}

const fn piece_color(piece: PieceKind) -> Color {
    match piece {
        PieceKind::I => Color::CYAN,
        PieceKind::O => Color::YELLOW,
        PieceKind::S => Color::GREEN,
        PieceKind::Z => Color::RED,
        PieceKind::J => Color::BLUE,
        PieceKind::L => Color::ORANGE,
        PieceKind::T => Color::MAGENTA,
    }
}

struct CellDisplay {
    fg: Color,
    bg: Color,
    symbol: &'static str,
}

impl CellDisplay {
    const fn from_kind(kind: RenderCell, show_dots: bool) -> Self {
        let dot = if show_dots { " ." } else { "  " };
        match kind {
            RenderCell::Empty => Self::new(Color::GRAY, Color::BLACK, dot),
            RenderCell::Wall => Self::new(Color::GRAY, Color::GRAY, dot),
            RenderCell::Ghost => Self::new(Color::WHITE, Color::BLACK, "[]"),
            RenderCell::Piece(piece) => Self::new(piece_color(piece), piece_color(piece), "  "),
        }
    }

    const fn new(fg: Color, bg: Color, symbol: &'static str) -> Self {
        Self { fg, bg, symbol }
    }

    const fn fg(&self) -> Color {
        self.fg
    }

    const fn bg(&self) -> Color {
        self.bg
    }

    const fn symbol(&self) -> &'static str {
        self.symbol
    }
}

pub(crate) struct Renderer {
    mode: PlayMode,
    term: Terminal,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

impl Renderer {
    pub(crate) fn new(mode: PlayMode) -> io::Result<Self> {
        let mut term = Terminal::stdout();
        term.clear_screen()?.hide_cursor()?;

        term.reset_styles()?
            .set_fg(Color::WHITE)?
            .set_bg(Color::BLACK)?;
        for y in TERMINAL_TOP..TERMINAL_BOTTOM {
            term.move_to(y, TERMINAL_LEFT)?;
            for _ in TERMINAL_LEFT..TERMINAL_RIGHT {
                term.write(" ")?;
            }
        }

        Ok(Self { mode, term })
    }

    pub(crate) fn cleanup(&mut self) -> io::Result<()> {
        self.term
            .reset_styles()?
            .move_to(TERMINAL_BOTTOM, TERMINAL_LEFT)? // Move cursor to bottom of screen to prevent overwriting
            .show_cursor()?;
        self.term.flush()?;
        Ok(())
    }

    pub(crate) fn draw(&mut self, game: &GameSession) -> io::Result<()> {
        self.draw_board_panel(game)?;
        self.draw_hold_panel(game)?;
        self.draw_next_panel(game)?;
        self.draw_stats_panel(game)?;
        self.draw_controls_panel()?;
        self.term.flush()?;
        Ok(())
    }

    fn draw_board_panel(&mut self, game: &GameSession) -> io::Result<()> {
        self.term.reset_styles()?;

        // Prepare board with ghost piece and falling piece
        let board = game.render_board();
        let falling_piece = game.falling_piece();
        let mut board_buf = board.clone();

        // Show ghost piece only in normal mode
        if self.mode == PlayMode::Normal {
            let dropped = game.simulate_drop_position();
            board_buf.fill_piece_as(&dropped, RenderCell::Ghost);
        }
        board_buf.fill_piece(falling_piece);

        BOARD_PANEL.draw_border(&mut self.term)?;
        for (row_offset, board_row) in board_buf.playable_rows().enumerate() {
            self.term
                .move_to(BOARD_PANEL.body_top() + row_offset, BOARD_PANEL.body_left())?;
            for cell in board_row {
                let display = CellDisplay::from_kind(*cell, true);
                self.term
                    .set_fg(display.fg())?
                    .set_bg(display.bg())?
                    .write(display.symbol())?;
            }
        }

        // Draw overlays based on game state
        match game.session_state() {
            SessionState::Playing => {}
            SessionState::Paused => self.draw_overlay(Color::YELLOW, Color::BLACK, "PAUSED")?,
            SessionState::GameOver => self.draw_overlay(Color::RED, Color::WHITE, "GAME OVER!!")?,
        }

        Ok(())
    }

    fn draw_hold_panel(&mut self, game: &GameSession) -> io::Result<()> {
        HOLD_PANEL.draw_border(&mut self.term)?;
        if let Some(piece) = game.held_piece() {
            let piece_left =
                HOLD_PANEL.body_left() + (HOLD_PANEL.body_width - PIECE_DISPLAY_WIDTH) / 2;
            self.draw_piece_at(piece, HOLD_PANEL.body_top(), piece_left)?;
        }
        Ok(())
    }

    fn draw_next_panel(&mut self, game: &GameSession) -> io::Result<()> {
        NEXT_PANEL.draw_border(&mut self.term)?;
        for (piece_idx, piece) in game.next_pieces().take(7).enumerate() {
            let piece_top = NEXT_PANEL.body_top() + piece_idx * 3;
            let piece_left =
                NEXT_PANEL.body_left() + (NEXT_PANEL.body_width - PIECE_DISPLAY_WIDTH) / 2;
            self.draw_piece_at(piece, piece_top, piece_left)?;
        }
        Ok(())
    }

    fn draw_stats_panel(&mut self, game: &GameSession) -> io::Result<()> {
        STATS_PANEL.draw_border(&mut self.term)?;
        self.term
            .move_to(STATS_PANEL.body_top(), STATS_PANEL.body_left())?
            .write(format_args!("SCORE:",))?
            .move_to(STATS_PANEL.body_top() + 1, STATS_PANEL.body_left())?
            .write(format_args!(
                "{:>width$}",
                game.score(),
                width = STATS_PANEL.body_width
            ))?
            .move_to(STATS_PANEL.body_top() + 2, STATS_PANEL.body_left())?
            .write(format_args!("TIME:",))?
            .move_to(STATS_PANEL.body_top() + 3, STATS_PANEL.body_left())?
            .write(format_args!(
                "{:>width$}",
                format!(
                    "{:0}:{:0>2}.{:0>2}",
                    game.duration().as_secs() / 60,
                    game.duration().as_secs() % 60,
                    game.duration().subsec_millis() / 10,
                ),
                width = STATS_PANEL.body_width
            ))?
            .move_to(STATS_PANEL.body_top() + 5, STATS_PANEL.body_left())?
            .write(format_args!(
                "LEVEL: {:>width$}",
                game.level(),
                width = STATS_PANEL.body_width - 7
            ))?
            .move_to(STATS_PANEL.body_top() + 6, STATS_PANEL.body_left())?
            .write(format_args!(
                "LINES: {:>width$}",
                game.total_cleared_lines(),
                width = STATS_PANEL.body_width - 7
            ))?
            .move_to(STATS_PANEL.body_top() + 8, STATS_PANEL.body_left())?
            .write(format_args!(
                "PIECES: {:>width$}",
                game.completed_pieces(),
                width = STATS_PANEL.body_width - 7
            ))?
            .move_to(STATS_PANEL.body_top() + 9, STATS_PANEL.body_left())?
            .write(format_args!(
                "SINGLES: {:>width$}",
                game.line_cleared_counter()[1],
                width = STATS_PANEL.body_width - 8
            ))?
            .move_to(STATS_PANEL.body_top() + 10, STATS_PANEL.body_left())?
            .write(format_args!(
                "DOUBLES: {:>width$}",
                game.line_cleared_counter()[2],
                width = STATS_PANEL.body_width - 8
            ))?
            .move_to(STATS_PANEL.body_top() + 11, STATS_PANEL.body_left())?
            .write(format_args!(
                "TRIPLES: {:>width$}",
                game.line_cleared_counter()[3],
                width = STATS_PANEL.body_width - 8
            ))?
            .move_to(STATS_PANEL.body_top() + 12, STATS_PANEL.body_left())?
            .write(format_args!(
                "TETRISES: {:>width$}",
                game.line_cleared_counter()[4],
                width = STATS_PANEL.body_width - 9
            ))?;
        Ok(())
    }

    fn draw_controls_panel(&mut self) -> io::Result<()> {
        CONTROLS_PANEL.draw_border(&mut self.term)?;
        for (line_offset, (key, description)) in self.mode.controls().iter().enumerate() {
            self.term
                .move_to(
                    CONTROLS_PANEL.body_top() + line_offset + 1,
                    CONTROLS_PANEL.body_left(),
                )?
                .write(format_args!("{key:<12} : {description}"))?;
        }
        Ok(())
    }

    fn draw_overlay(&mut self, bg: Color, fg: Color, message: &str) -> io::Result<()> {
        let top = BOARD_PANEL.body_top() + (BOARD_PANEL.body_height / 2) - 1;
        let left = BOARD_PANEL.body_left();
        let width = BOARD_PANEL.body_width;

        self.term
            .reset_styles()?
            .move_to(top, left)?
            .set_bg(bg)?
            .set_fg(fg)?
            .write(format_args!("{:width$}", ""))?
            .move_to(top + 1, left)?
            .set_bold()?
            .write(format_args!("{message:^width$}"))?
            .move_to(top + 2, left)?
            .write(format_args!("{:width$}", ""))?
            .reset_styles()?;

        Ok(())
    }

    fn draw_piece_at(&mut self, piece: PieceKind, top: usize, left: usize) -> io::Result<()> {
        for dy in 0..PIECE_DISPLAY_HEIGHT {
            let display = CellDisplay::from_kind(RenderCell::Empty, false);
            self.term.move_to(top + dy, left)?;
            for _ in 0..PIECE_DISPLAY_WIDTH / CHARS_PER_CELL {
                self.term
                    .set_fg(display.fg())?
                    .set_bg(display.bg())?
                    .write(display.symbol())?;
            }
        }
        for (dx, dy) in piece.occupied_positions(PieceRotation::default()) {
            assert!(dx < PIECE_DISPLAY_WIDTH / CHARS_PER_CELL);
            assert!(dy < PIECE_DISPLAY_HEIGHT);
            let display = CellDisplay::from_kind(RenderCell::Piece(piece), false);
            self.term
                .move_to(top + dy, left + dx * CHARS_PER_CELL)?
                .set_fg(display.fg())?
                .set_bg(display.bg())?
                .write(display.symbol())?;
        }
        Ok(())
    }
}
