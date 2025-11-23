use crate::terminal::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub(crate) enum BlockKind {
    #[default]
    Empty,
    Wall,
    Ghost,
    I,
    O,
    S,
    Z,
    J,
    L,
    T,
}

/// Block display information combining color and symbol
#[derive(Debug, Clone, Copy)]
pub(crate) struct BlockDisplay {
    bg: Color,
    symbol: &'static str,
}

impl BlockDisplay {
    const fn new(bg: Color, symbol: &'static str) -> Self {
        Self { bg, symbol }
    }

    pub(crate) const fn fg(&self) -> Color {
        Color::WHITE
    }

    pub(crate) const fn bg(&self) -> Color {
        self.bg
    }

    pub(crate) const fn symbol(&self) -> &'static str {
        self.symbol
    }
}

impl BlockKind {
    /// Get the display information for this block
    pub(crate) const fn display(&self) -> BlockDisplay {
        match self {
            BlockKind::Empty => BlockDisplay::new(Color::BLACK, "  "),
            BlockKind::Wall => BlockDisplay::new(Color::GRAY, "__"),
            BlockKind::Ghost => BlockDisplay::new(Color::BLACK, "[]"),
            BlockKind::I => BlockDisplay::new(Color::CYAN, "__"),
            BlockKind::O => BlockDisplay::new(Color::YELLOW, "__"),
            BlockKind::S => BlockDisplay::new(Color::GREEN, "__"),
            BlockKind::Z => BlockDisplay::new(Color::RED, "__"),
            BlockKind::J => BlockDisplay::new(Color::BLUE, "__"),
            BlockKind::L => BlockDisplay::new(Color::ORANGE, "__"),
            BlockKind::T => BlockDisplay::new(Color::MAGENTA, "__"),
        }
        //BLOCK_DISPLAYS[*self as usize]
    }

    pub(crate) fn is_empty(&self) -> bool {
        *self == BlockKind::Empty
    }
}
