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

const COLOR_TABLE: [&str; 10] = [
    "\x1b[48;2;000;000;000m  ", // Empty
    "\x1b[48;2;127;127;127m__", // Wall
    "\x1b[48;2;000;000;000m[]", // Ghost
    "\x1b[48;2;000;255;255m__", // I
    "\x1b[48;2;255;255;000m__", // O
    "\x1b[48;2;000;255;000m__", // S
    "\x1b[48;2;255;000;000m__", // Z
    "\x1b[48;2;000;000;255m__", // J
    "\x1b[48;2;255;127;000m__", // L
    "\x1b[48;2;255;000;255m__", // T
];

impl BlockKind {
    pub(crate) fn color(&self) -> &'static str {
        COLOR_TABLE[*self as usize]
    }

    pub(crate) fn is_empty(&self) -> bool {
        *self == BlockKind::Empty
    }
}
