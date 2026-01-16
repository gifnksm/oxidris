use std::iter;

use oxidris_engine::PieceKind;
use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::{Buffer, Rect},
    widgets::{Block as BlockWidget, BlockExt as _, Widget},
};

use crate::view::widgets::{BlockDisplay, PieceDisplay};

#[derive(Debug)]
pub struct PieceStackDisplay<'a> {
    pieces: Vec<PieceKind>,
    block: Option<BlockWidget<'a>>,
}

impl<'a> PieceStackDisplay<'a> {
    pub fn new<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = PieceKind>,
    {
        Self {
            pieces: pieces.into_iter().collect(),
            block: None,
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
        let num_pieces = u16::try_from(self.pieces.len()).unwrap();
        let padding = num_pieces.saturating_sub(1);
        2 * BlockDisplay::height() * num_pieces
            + padding
            + super::block_vertical_margin(self.block.as_ref())
    }
}

impl Widget for PieceStackDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &PieceStackDisplay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.block.as_ref().render(area, buf);
        let area = self.block.inner_if_some(area);
        let layout = Layout::vertical(
            (0..self.pieces.len()).map(|_| Constraint::Length(2 * BlockDisplay::height())),
        )
        .flex(Flex::SpaceBetween);
        let cells = area.layout_vec(&layout);

        for (cell, piece) in iter::zip(cells, &self.pieces) {
            PieceDisplay::new().piece(*piece).render(cell, buf);
        }
    }
}
