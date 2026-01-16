use std::iter;

use oxidris_engine::{BitBoard, Block, BlockBoard, Piece};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block as BlockWidget, BlockExt, Widget},
};

use crate::view::widgets::BlockDisplay;

#[derive(Debug)]
pub struct BoardDisplay<'a> {
    board: &'a BlockBoard,
    ghost: Option<Piece>,
    falling_piece: Option<Piece>,
    block: Option<BlockWidget<'a>>,
}

impl<'a> BoardDisplay<'a> {
    pub fn new(board: &'a BlockBoard) -> Self {
        Self {
            board,
            ghost: None,
            falling_piece: None,
            block: None,
        }
    }

    pub fn ghost(self, piece: Piece) -> Self {
        Self {
            ghost: Some(piece),
            ..self
        }
    }

    pub fn falling_piece(self, piece: Piece) -> Self {
        Self {
            falling_piece: Some(piece),
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
        10 * BlockDisplay::width() + super::block_horizontal_margin(self.block.as_ref())
    }

    pub fn height(&self) -> u16 {
        20 * BlockDisplay::height() + super::block_vertical_margin(self.block.as_ref())
    }
}

impl Widget for BoardDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &BoardDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.as_ref().render(area, buf);
        let area = self.block.inner_if_some(area);

        let mut board = self.board.clone();
        if let Some(ghost) = self.ghost {
            board.fill_piece_as(ghost, Block::Ghost);
        }
        if let Some(piece) = self.falling_piece {
            board.fill_piece(piece);
        }

        let col_constraints =
            (0..BitBoard::PLAYABLE_WIDTH).map(|_| Constraint::Length(BlockDisplay::width()));
        let row_constraints =
            (0..BitBoard::PLAYABLE_HEIGHT).map(|_| Constraint::Length(BlockDisplay::height()));
        let horizontal = Layout::horizontal(col_constraints).flex(Flex::Center);
        let vertical = Layout::vertical(row_constraints);

        let grid_cells = area
            .layout::<{ BitBoard::PLAYABLE_HEIGHT }>(&vertical)
            .into_iter()
            .map(|row| row.layout::<{ BitBoard::PLAYABLE_WIDTH }>(&horizontal));

        for (grid_row, row) in iter::zip(grid_cells, board.playable_rows()) {
            for (grid_cell, block) in iter::zip(grid_row, row) {
                let block_display = BlockDisplay::from_block(*block, true);
                block_display.render(grid_cell, buf);
            }
        }
    }
}
