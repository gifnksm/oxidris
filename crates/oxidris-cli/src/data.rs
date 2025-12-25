use oxidris_engine::{BitBoard, Piece};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct BoardCollection {
    pub boards: Vec<BoardAndPlacement>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct BoardAndPlacement {
    pub board: BitBoard,
    pub placement: Piece,
}
