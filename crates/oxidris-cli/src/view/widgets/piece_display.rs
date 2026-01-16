use oxidris_engine::{Block, PieceKind, PieceRotation};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block as BlockWidget, BlockExt as _, Widget},
};

use crate::view::widgets::BlockDisplay;

#[derive(Debug)]
pub struct PieceDisplay<'a> {
    piece: Option<PieceKind>,
    block: Option<BlockWidget<'a>>,
}

impl<'a> PieceDisplay<'a> {
    pub fn new() -> Self {
        Self {
            piece: None,
            block: None,
        }
    }

    pub fn piece(self, piece: PieceKind) -> Self {
        Self {
            piece: Some(piece),
            ..self
        }
    }

    pub fn block(self, block: BlockWidget<'a>) -> Self {
        Self {
            block: Some(block),
            ..self
        }
    }

    pub fn width(&self) -> u16 {
        4 * BlockDisplay::width() + super::block_horizontal_margin(self.block.as_ref())
    }

    pub fn height(&self) -> u16 {
        2 * BlockDisplay::height() + super::block_vertical_margin(self.block.as_ref())
    }
}

impl Widget for PieceDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &PieceDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.as_ref().render(area, buf);
        let area = self.block.inner_if_some(area);

        let mut piece_size = (0, 0);
        if let Some(piece) = self.piece {
            let (w, h) = piece.size(PieceRotation::default());
            piece_size.0 = u16::from(w);
            piece_size.1 = u16::from(h);
        }
        let piece_area = area.centered(
            Constraint::Length(piece_size.0 * BlockDisplay::width()),
            Constraint::Length(piece_size.1 * BlockDisplay::height()),
        );

        let col_constraints = (0..piece_size.0).map(|_| Constraint::Length(BlockDisplay::width()));
        let row_constraints = (0..piece_size.1).map(|_| Constraint::Length(BlockDisplay::height()));
        let horizontal = Layout::horizontal(col_constraints).flex(Flex::Center);
        let vertical = Layout::vertical(row_constraints);
        let grid_rows = piece_area
            .layout_vec(&vertical)
            .into_iter()
            .map(|row| row.layout_vec(&horizontal));

        let empty_block = BlockDisplay::from_block(Block::Empty, false);

        if let Some(piece) = self.piece {
            let occupied_block = BlockDisplay::from_block(Block::Piece(piece), false);
            for (y, grid_row) in grid_rows.enumerate() {
                for (x, grid_cell) in grid_row.into_iter().enumerate() {
                    if piece.is_occupied(PieceRotation::default(), (x, y)) {
                        Widget::render(&occupied_block, grid_cell, buf);
                    } else {
                        Widget::render(&empty_block, grid_cell, buf);
                    }
                }
            }
        } else {
            for cell in grid_rows.flatten() {
                Widget::render(&empty_block, cell, buf);
            }
        }
    }
}
