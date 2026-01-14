use oxidris_engine::{Block, PieceKind};
use ratatui::{
    buffer::Cell,
    prelude::{Buffer, Rect},
    style::Style,
    widgets::{Paragraph, Widget},
};

use crate::ui::widgets::style;

#[derive(Debug)]
pub struct BlockDisplay {
    style: Style,
    symbol: &'static str,
}

impl BlockDisplay {
    pub const fn new(style: Style, symbol: &'static str) -> Self {
        Self { style, symbol }
    }

    pub fn width() -> u16 {
        2
    }

    pub fn height() -> u16 {
        1
    }

    pub fn from_block(block: Block, show_dots: bool) -> Self {
        match block {
            Block::Empty => {
                if show_dots {
                    Self::new(style::EMPTY_DOT, ".")
                } else {
                    Self::new(style::EMPTY, "")
                }
            }
            Block::Wall => Self::new(style::WALL, ""),
            Block::Ghost => Self::new(style::GHOST, "[]"),
            Block::Piece(piece_kind) => {
                let style = match piece_kind {
                    PieceKind::I => style::I_BLOCK,
                    PieceKind::O => style::O_BLOCK,
                    PieceKind::S => style::S_BLOCK,
                    PieceKind::Z => style::Z_BLOCK,
                    PieceKind::J => style::J_BLOCK,
                    PieceKind::L => style::L_BLOCK,
                    PieceKind::T => style::T_BLOCK,
                };
                Self::new(style, "")
            }
        }
    }

    pub fn draw(&self, cell: &mut Cell) {
        cell.set_style(self.style);
        cell.set_symbol(self.symbol);
    }
}

impl Widget for BlockDisplay {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &BlockDisplay {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Use a Paragraph to fill the whole area, not just the cells with the symbol
        Paragraph::new(self.symbol)
            .style(self.style)
            .centered()
            .render(area, buf);
    }
}
