use crate::mino::MinoKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub(crate) enum BlockKind {
    #[default]
    Empty,
    Wall,
    Ghost,
    Mino(MinoKind),
}

impl BlockKind {
    pub(crate) fn is_empty(&self) -> bool {
        *self == BlockKind::Empty
    }
}
