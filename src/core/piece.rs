use std::collections::VecDeque;

use arrayvec::ArrayVec;
use rand::{
    Rng, SeedableRng as _,
    distr::StandardUniform,
    prelude::{Distribution, StdRng},
    seq::SliceRandom,
};

use super::{
    bit_board::{BitBoard, PIECE_SPAWN_X, PIECE_SPAWN_Y},
    render_board::RenderCell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Piece {
    position: PiecePosition,
    rotation: PieceRotation,
    kind: PieceKind,
}

impl Piece {
    pub(crate) fn new(kind: PieceKind) -> Self {
        Self {
            position: PiecePosition::SPAWN_POSITION,
            rotation: PieceRotation::default(),
            kind,
        }
    }

    pub(crate) fn position(&self) -> PiecePosition {
        self.position
    }

    pub(crate) fn rotation(&self) -> PieceRotation {
        self.rotation
    }

    pub(crate) fn kind(&self) -> PieceKind {
        self.kind
    }

    pub(crate) fn mask(&self) -> PieceMask {
        self.kind.mask(self.rotation)
    }

    pub(crate) fn occupied_positions(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.kind
            .occupied_positions(self.rotation)
            .map(move |(dx, dy)| (self.position.x() + dx, self.position.y() + dy))
    }

    pub(crate) fn left(&self) -> Option<Self> {
        let new_pos = self.position.left()?;
        Some(Self {
            position: new_pos,
            rotation: self.rotation,
            kind: self.kind,
        })
    }

    pub(crate) fn right(&self) -> Option<Self> {
        let new_pos = self.position.right()?;
        Some(Self {
            position: new_pos,
            rotation: self.rotation,
            kind: self.kind,
        })
    }

    pub(crate) fn up(&self) -> Option<Self> {
        let new_pos = self.position.up()?;
        Some(Self {
            position: new_pos,
            rotation: self.rotation,
            kind: self.kind,
        })
    }

    pub(crate) fn down(&self) -> Option<Self> {
        let new_pos = self.position.down()?;
        Some(Self {
            position: new_pos,
            rotation: self.rotation,
            kind: self.kind,
        })
    }

    pub(crate) fn rotated_right(&self) -> Self {
        Self {
            position: self.position,
            rotation: self.rotation.rotated_right(),
            kind: self.kind,
        }
    }

    pub(crate) fn rotated_left(&self) -> Self {
        Self {
            position: self.position,
            rotation: self.rotation.rotated_left(),
            kind: self.kind,
        }
    }

    pub(crate) fn super_rotated_left(self, board: &BitBoard) -> Option<Self> {
        let mut piece = self.rotated_left();
        if board.is_colliding(&piece) {
            piece = super_rotation(board, &piece)?;
        }
        Some(piece)
    }

    pub(crate) fn super_rotated_right(self, board: &BitBoard) -> Option<Self> {
        let mut piece = self.rotated_right();
        if board.is_colliding(&piece) {
            piece = super_rotation(board, &piece)?;
        }
        Some(piece)
    }

    pub(crate) fn super_rotations(self, board: &BitBoard) -> ArrayVec<Self, 4> {
        let mut rotations = ArrayVec::new();
        rotations.push(self);
        if self.kind == PieceKind::O {
            return rotations;
        }
        let mut prev = self;
        for _ in 0..3 {
            let Some(piece) = prev.super_rotated_right(board) else {
                break;
            };
            rotations.push(piece);
            prev = piece;
        }
        rotations
    }

    pub(crate) fn simulate_drop_position(&self, board: &BitBoard) -> Self {
        let mut dropped = *self;
        while let Some(piece) = dropped.down().filter(|m| !board.is_colliding(m)) {
            dropped = piece;
        }
        dropped
    }
}

fn super_rotation(board: &BitBoard, piece: &Piece) -> Option<Piece> {
    let pieces = [piece.up(), piece.right(), piece.down(), piece.left()];
    for piece in pieces.iter().flatten() {
        if !board.is_colliding(piece) {
            return Some(*piece);
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PiecePosition {
    x: usize,
    y: usize,
}

impl PiecePosition {
    const SPAWN_POSITION: Self = Self::new(PIECE_SPAWN_X, PIECE_SPAWN_Y);

    const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub(crate) fn x(self) -> usize {
        self.x
    }

    pub(crate) fn y(self) -> usize {
        self.y
    }

    const fn left(&self) -> Option<Self> {
        if self.x == 0 {
            None
        } else {
            Some(Self::new(self.x - 1, self.y))
        }
    }

    const fn right(&self) -> Option<Self> {
        if self.x >= BitBoard::TOTAL_WIDTH - 1 {
            None
        } else {
            Some(Self::new(self.x + 1, self.y))
        }
    }

    const fn up(&self) -> Option<Self> {
        if self.y == 0 {
            None
        } else {
            Some(Self::new(self.x, self.y - 1))
        }
    }

    const fn down(&self) -> Option<Self> {
        if self.y >= BitBoard::TOTAL_HEIGHT - 1 {
            None
        } else {
            Some(Self::new(self.x, self.y + 1))
        }
    }
}

/// Represents the rotation state of a piece.
///
/// 0: 0 degrees, 1: 90° right, 2: 180°, 3: 90° left.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PieceRotation(u8);

impl PieceRotation {
    pub(crate) fn rotated_right(self) -> Self {
        PieceRotation((self.0 + 1) % 4)
    }

    pub(crate) fn rotated_left(self) -> Self {
        PieceRotation((self.0 + 3) % 4)
    }

    const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// Enum representing the type of piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum PieceKind {
    /// I-piece.
    I = 0,
    /// O-piece.
    O = 1,
    /// S-piece.
    S = 2,
    /// Z-piece.
    Z = 3,
    /// J-piece.
    J = 4,
    /// L-piece.
    L = 5,
    /// T-piece.
    T = 6,
}

impl Distribution<PieceKind> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PieceKind {
        match rng.random_range(0..=6) {
            0 => PieceKind::I,
            1 => PieceKind::O,
            2 => PieceKind::S,
            3 => PieceKind::Z,
            4 => PieceKind::J,
            5 => PieceKind::L,
            _ => PieceKind::T,
        }
    }
}

impl PieceKind {
    /// Number of piece types (7).
    const LEN: usize = 7;

    pub(crate) fn mask(self, rotation: PieceRotation) -> PieceMask {
        PIECE_MASKS[self as usize][rotation.as_usize()]
    }

    /// Returns an iterator of occupied positions for the piece in the given rotation.
    pub(crate) fn occupied_positions(
        &self,
        rotation: PieceRotation,
    ) -> impl Iterator<Item = (usize, usize)> + '_ {
        PIECE_SHAPES[*self as usize][rotation.as_usize()]
            .iter()
            .enumerate()
            .flat_map(move |(dy, row)| {
                row.iter().enumerate().filter_map(move |(dx, &cell)| {
                    if cell.is_empty() {
                        None
                    } else {
                        Some((dx, dy))
                    }
                })
            })
    }
}

pub(crate) type PieceMask = [u16; 4];

const fn mask_rotations(size: usize, mask: PieceMask) -> [PieceMask; 4] {
    let mut rotates = [mask; 4];
    let mut i = 1;
    while i < 4 {
        let mut new_mask = [0; 4];
        let mut y = 0;
        while y < size {
            let mut x = 0;
            while x < size {
                if (rotates[i - 1][size - 1 - x] & (1 << y)) != 0 {
                    new_mask[y] |= 1 << x;
                }
                x += 1;
            }
            y += 1;
        }
        rotates[i] = new_mask;
        i += 1;
    }
    rotates
}

const PIECE_MASKS: [[PieceMask; 4]; PieceKind::LEN] = {
    const fn m(bits: [bool; 4]) -> u16 {
        let mut mask = 0;
        let mut i = 0;
        while i < 4 {
            if bits[i] {
                mask |= 1 << i;
            }
            i += 1;
        }
        mask
    }

    const C: bool = true;
    const E: bool = false;
    const EEEE: u16 = m([E; 4]);

    [
        // I-piece
        mask_rotations(4, [EEEE, m([C, C, C, C]), EEEE, EEEE]),
        // O-piece
        mask_rotations(2, [m([C, C, E, E]), m([C, C, E, E]), EEEE, EEEE]),
        // S-piece
        mask_rotations(3, [m([E, C, C, E]), m([C, C, E, E]), EEEE, EEEE]),
        // Z-piece
        mask_rotations(3, [m([C, C, E, E]), m([E, C, C, E]), EEEE, EEEE]),
        // J-piece
        mask_rotations(3, [m([C, E, E, E]), m([C, C, C, E]), EEEE, EEEE]),
        // L-piece
        mask_rotations(3, [m([E, E, C, E]), m([C, C, C, E]), EEEE, EEEE]),
        // T-piece
        mask_rotations(3, [m([E, C, E, E]), m([C, C, C, E]), EEEE, EEEE]),
    ]
};

/// Piece shape (4x4 cell array).
type PieceShape = [[RenderCell; 4]; 4];

const fn shape_rotations(size: usize, shape: &PieceShape) -> [PieceShape; 4] {
    let mut rotates = [*shape; 4];
    let mut i = 1;
    while i < 4 {
        let mut new_shape = [[RenderCell::Empty; 4]; 4];
        let mut y = 0;
        while y < size {
            let mut x = 0;
            while x < size {
                new_shape[y][x] = rotates[i - 1][size - 1 - x][y];
                x += 1;
            }
            y += 1;
        }
        rotates[i] = new_shape;
        i += 1;
    }
    rotates
}

const PIECE_SHAPES: [[PieceShape; 4]; PieceKind::LEN] = {
    use RenderCell::Empty as E;
    const I: RenderCell = RenderCell::Piece(PieceKind::I);
    const O: RenderCell = RenderCell::Piece(PieceKind::O);
    const S: RenderCell = RenderCell::Piece(PieceKind::S);
    const Z: RenderCell = RenderCell::Piece(PieceKind::Z);
    const J: RenderCell = RenderCell::Piece(PieceKind::J);
    const L: RenderCell = RenderCell::Piece(PieceKind::L);
    const T: RenderCell = RenderCell::Piece(PieceKind::T);
    const EEEE: [RenderCell; 4] = [E; 4];
    [
        // I-piece
        shape_rotations(4, &[EEEE, [I, I, I, I], EEEE, EEEE]),
        // O-piece
        shape_rotations(2, &[[O, O, E, E], [O, O, E, E], EEEE, EEEE]),
        // S-piece
        shape_rotations(3, &[[E, S, S, E], [S, S, E, E], EEEE, EEEE]),
        // Z-piece
        shape_rotations(3, &[[Z, Z, E, E], [E, Z, Z, E], EEEE, EEEE]),
        // J-piece
        shape_rotations(3, &[[J, E, E, E], [J, J, J, E], EEEE, EEEE]),
        // L-piece
        shape_rotations(3, &[[E, E, L, E], [L, L, L, E], EEEE, EEEE]),
        // T-piece
        shape_rotations(3, &[[E, T, E, E], [T, T, T, E], EEEE, EEEE]),
    ]
};

/// Manages the order and random generation of pieces.
///
/// Supplies pieces using the 7-bag system.
#[derive(Debug, Clone)]
pub(crate) struct PieceGenerator {
    rng: StdRng,
    bag: VecDeque<PieceKind>,
}

impl PieceGenerator {
    /// Creates a new [`PieceGenerator`].
    ///
    /// The random seed is initialized from the OS's random data source.
    pub(crate) fn new() -> Self {
        let rng = StdRng::from_os_rng();
        let bag = VecDeque::with_capacity(PieceKind::LEN * 2);
        let mut this = Self { rng, bag };
        this.fill_bag();
        this
    }

    /// Fills the bag with a shuffled set of 7 pieces when needed.
    ///
    /// After filling, the bag will always contain at least 8 elements
    /// (so that even after one `pop_next`, there are still 7 left).
    fn fill_bag(&mut self) {
        while self.bag.len() <= PieceKind::LEN {
            let mut new_bag = [
                PieceKind::I,
                PieceKind::O,
                PieceKind::S,
                PieceKind::Z,
                PieceKind::J,
                PieceKind::L,
                PieceKind::T,
            ];
            new_bag.shuffle(&mut self.rng);
            self.bag.extend(new_bag);
        }
    }

    /// Pops the next piece from the bag.
    ///
    /// # Panics
    ///
    /// Panics if the bag is empty (should never happen).
    pub(crate) fn pop_next(&mut self) -> PieceKind {
        self.fill_bag();
        self.bag
            .pop_front()
            .expect("Piece bag should never be empty")
    }

    /// Returns an iterator of upcoming pieces in the bag.
    ///
    /// The iterator always contains at least 8 elements.
    pub(crate) fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.bag.iter().copied()
    }
}
