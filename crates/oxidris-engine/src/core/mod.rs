pub use self::{bit_board::*, block_board::*, piece::*};

pub(crate) mod bit_board;
pub(crate) mod block_board;
pub(crate) mod piece;

const PLAYABLE_WIDTH: usize = 10;
const PLAYABLE_HEIGHT: usize = 20;

const TOTAL_WIDTH: usize = PLAYABLE_WIDTH + (SENTINEL_MARGIN_LEFT + SENTINEL_MARGIN_RIGHT);
const TOTAL_HEIGHT: usize = PLAYABLE_HEIGHT + (SENTINEL_MARGIN_TOP + SENTINEL_MARGIN_BOTTOM);

const SENTINEL_MARGIN_TOP: usize = 2;
const SENTINEL_MARGIN_BOTTOM: usize = 2;
const SENTINEL_MARGIN_LEFT: usize = 2;
const SENTINEL_MARGIN_RIGHT: usize = 2;
