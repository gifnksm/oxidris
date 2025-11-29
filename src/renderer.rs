use std::io;

use crate::{
    block::BlockKind,
    field::Field,
    game::{Game, GameState},
    mino::{MinoKind, MinoShape},
    play::PlayMode,
    terminal::{Color, Terminal},
};

// UI layout coordinates
const TERMINAL_TOP: usize = 1;
const TERMINAL_LEFT: usize = 1;

const CHARS_PER_BLOCK: usize = 2;
const MINO_DISPLAY_WIDTH: usize = 4 * CHARS_PER_BLOCK;
const MINO_DISPLAY_HEIGHT: usize = 2;

const LEFT_PANE_BODY_WIDTH: usize = 16;

const HOLD_PANEL: Panel = Panel {
    top: TERMINAL_TOP,
    left: TERMINAL_LEFT,
    body_width: LEFT_PANE_BODY_WIDTH,
    body_height: MINO_DISPLAY_HEIGHT,
    title: "HOLD",
};

const STATS_PANEL: Panel = Panel {
    top: HOLD_PANEL.bottom() + 1,
    left: TERMINAL_LEFT,
    body_width: LEFT_PANE_BODY_WIDTH,
    body_height: 3,
    title: "STATS",
};

const FIELD_PANEL: Panel = Panel {
    top: TERMINAL_TOP,
    left: HOLD_PANEL.right() + 2,
    body_width: CHARS_PER_BLOCK * Field::BLOCKS_WIDTH,
    body_height: Field::BLOCKS_HEIGHT,
    title: "",
};

const NEXT_PANEL: Panel = Panel {
    top: TERMINAL_TOP,
    left: FIELD_PANEL.right() + 2,
    body_width: MINO_DISPLAY_WIDTH,
    body_height: 20,
    title: "NEXT",
};

const CONTROLS_PANEL: Panel = Panel {
    top: TERMINAL_TOP,
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

const fn mino_color(mino: MinoKind) -> Color {
    match mino {
        MinoKind::I => Color::CYAN,
        MinoKind::O => Color::YELLOW,
        MinoKind::S => Color::GREEN,
        MinoKind::Z => Color::RED,
        MinoKind::J => Color::BLUE,
        MinoKind::L => Color::ORANGE,
        MinoKind::T => Color::MAGENTA,
    }
}

struct BlockDisplay {
    bg: Color,
    symbol: &'static str,
}

impl BlockDisplay {
    const fn from_kind(kind: BlockKind, show_dots: bool) -> Self {
        let dot = if show_dots { " ." } else { "  " };
        match kind {
            BlockKind::Empty => Self::new(Color::BLACK, dot),
            BlockKind::Wall => Self::new(Color::GRAY, dot),
            BlockKind::Ghost => Self::new(Color::BLACK, "[]"),
            BlockKind::Mino(mino) => Self::new(mino_color(mino), "  "),
        }
    }

    const fn new(bg: Color, symbol: &'static str) -> Self {
        Self { bg, symbol }
    }

    const fn fg(&self) -> Color {
        Color::WHITE
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

impl Renderer {
    pub(crate) fn new(mode: PlayMode) -> io::Result<Self> {
        let mut term = Terminal::stdout();
        term.clear_screen()?.hide_cursor()?;
        Ok(Self { mode, term })
    }

    pub(crate) fn cleanup(&mut self) -> io::Result<()> {
        self.term
            .reset_styles()?
            .move_to(FIELD_PANEL.bottom() + 3, TERMINAL_LEFT)? // Move cursor to bottom of screen to prevent overwriting
            .show_cursor()?;
        self.term.flush()?;
        Ok(())
    }

    pub(crate) fn draw(&mut self, game: &Game) -> io::Result<()> {
        self.draw_field_panel(game)?;
        self.draw_hold_panel(game)?;
        self.draw_next_panel(game)?;
        self.draw_stats_panel(game)?;
        self.draw_controls_panel()?;
        self.term.flush()?;
        Ok(())
    }

    fn draw_field_panel(&mut self, game: &Game) -> io::Result<()> {
        self.term.reset_styles()?;

        // Prepare field with ghost piece and falling mino
        let field = game.field();
        let (falling_mino_pos, falling_mino) = game.falling_mino();
        let mut field_buf = field.clone();

        // Show ghost piece only in normal mode
        if self.mode == PlayMode::Normal {
            let drop_pos = game.simulate_drop_position();
            field_buf.fill_mino_as(&drop_pos, falling_mino, BlockKind::Ghost);
        }
        field_buf.fill_mino(&falling_mino_pos, falling_mino);

        FIELD_PANEL.draw_border(&mut self.term)?;
        for (row_offset, field_row) in field_buf.block_rows().enumerate() {
            self.term
                .move_to(FIELD_PANEL.body_top() + row_offset, FIELD_PANEL.body_left())?;
            for block in field_row {
                let display = BlockDisplay::from_kind(*block, true);
                self.term
                    .set_fg(display.fg())?
                    .set_bg(display.bg())?
                    .write(display.symbol())?;
            }
        }

        // Draw overlays based on game state
        match game.state() {
            GameState::Playing => {}
            GameState::Paused => self.draw_overlay(Color::YELLOW, Color::BLACK, "PAUSED")?,
            GameState::GameOver => self.draw_overlay(Color::RED, Color::WHITE, "GAME OVER!!")?,
        }

        Ok(())
    }

    fn draw_hold_panel(&mut self, game: &Game) -> io::Result<()> {
        HOLD_PANEL.draw_border(&mut self.term)?;
        if let Some(mino) = game.held_mino() {
            let mino_left =
                HOLD_PANEL.body_left() + (HOLD_PANEL.body_width - MINO_DISPLAY_WIDTH) / 2;
            self.draw_mino_at(mino.shape(), HOLD_PANEL.body_top(), mino_left)?;
        }
        Ok(())
    }

    fn draw_next_panel(&mut self, game: &Game) -> io::Result<()> {
        NEXT_PANEL.draw_border(&mut self.term)?;
        for (mino_idx, mino) in game.next_minos().iter().take(7).enumerate() {
            let mino_top = NEXT_PANEL.body_top() + mino_idx * 3;
            let mino_left =
                NEXT_PANEL.body_left() + (NEXT_PANEL.body_width - MINO_DISPLAY_WIDTH) / 2;
            self.draw_mino_at(mino.shape(), mino_top, mino_left)?;
        }
        Ok(())
    }

    fn draw_stats_panel(&mut self, game: &Game) -> io::Result<()> {
        STATS_PANEL.draw_border(&mut self.term)?;
        self.term
            .move_to(STATS_PANEL.body_top() + 1, STATS_PANEL.body_left())?
            .write(format_args!(
                "SCORE: {:>width$}",
                game.score(),
                width = STATS_PANEL.body_width - 7
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
                .write(format!("{:<12} : {}", key, description))?;
        }
        Ok(())
    }

    fn draw_overlay(&mut self, bg: Color, fg: Color, message: &str) -> io::Result<()> {
        let top = FIELD_PANEL.body_top() + (FIELD_PANEL.body_height / 2) - 1;
        let left = FIELD_PANEL.body_left();
        let width = FIELD_PANEL.body_width;

        self.term
            .reset_styles()?
            .move_to(top, left)?
            .set_bg(bg)?
            .set_fg(fg)?
            .write(format_args!("{:width$}", ""))?
            .move_to(top + 1, left)?
            .set_bold()?
            .write(format_args!("{:^width$}", message))?
            .move_to(top + 2, left)?
            .write(format_args!("{:width$}", ""))?
            .reset_styles()?;

        Ok(())
    }

    fn draw_mino_at(&mut self, mino: &MinoShape, top: usize, left: usize) -> io::Result<()> {
        for (row_offset, mino_row) in mino[1..=2].iter().enumerate() {
            self.term.move_to(top + row_offset, left)?;
            for block in mino_row {
                let display = BlockDisplay::from_kind(*block, false);
                self.term
                    .set_fg(display.fg())?
                    .set_bg(display.bg())?
                    .write(display.symbol())?;
            }
        }
        Ok(())
    }
}
