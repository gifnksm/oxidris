use std::collections::VecDeque;

use arrayvec::ArrayVec;
use rand::{
    Rng, SeedableRng as _,
    distr::StandardUniform,
    prelude::{Distribution, StdRng},
    seq::SliceRandom,
};

use super::{
    block::BlockKind,
    board::{Board, INIT_PIECE_X, INIT_PIECE_Y, MAX_X, MAX_Y, MIN_X, MIN_Y},
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
            position: PiecePosition::INITIAL,
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

    pub(crate) fn shape(&self) -> &PieceShape {
        self.kind.shape(self.rotation)
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

    pub(crate) fn super_rotated_left(self, board: &Board) -> Option<Self> {
        let mut piece = self.rotated_left();
        if board.is_colliding(&piece) {
            piece = super_rotation(board, &piece)?;
        }
        Some(piece)
    }

    pub(crate) fn super_rotated_right(self, board: &Board) -> Option<Self> {
        let mut piece = self.rotated_right();
        if board.is_colliding(&piece) {
            piece = super_rotation(board, &piece)?;
        }
        Some(piece)
    }

    pub(crate) fn super_rotations(self, board: &Board) -> ArrayVec<Self, 4> {
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

    pub(crate) fn simulate_drop_position(&self, board: &Board) -> Self {
        let mut dropped = *self;
        while let Some(piece) = dropped.down().filter(|m| !board.is_colliding(m)) {
            dropped = piece;
        }
        dropped
    }
}

fn super_rotation(board: &Board, piece: &Piece) -> Option<Piece> {
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
    pub(crate) const INITIAL: Self = Self::new(INIT_PIECE_X, INIT_PIECE_Y);

    pub(crate) const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub(crate) fn x(self) -> usize {
        self.x
    }

    pub(crate) fn y(self) -> usize {
        self.y
    }

    pub(crate) const fn left(&self) -> Option<Self> {
        if self.x == MIN_X {
            None
        } else {
            Some(Self::new(self.x - 1, self.y))
        }
    }

    pub(crate) const fn right(&self) -> Option<Self> {
        if self.x >= MAX_X {
            None
        } else {
            Some(Self::new(self.x + 1, self.y))
        }
    }

    pub(crate) const fn up(&self) -> Option<Self> {
        if self.y == MIN_Y {
            None
        } else {
            Some(Self::new(self.x, self.y - 1))
        }
    }

    pub(crate) const fn down(&self) -> Option<Self> {
        if self.y >= MAX_Y {
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
    pub(crate) const LEN: usize = 7;

    /// Returns the piece shape for the given rotation.
    ///
    /// # Arguments
    ///
    /// * `rotation` - The rotation state.
    ///
    /// # Returns
    ///
    /// Reference to the piece shape.
    pub(crate) const fn shape(&self, rotation: PieceRotation) -> &PieceShape {
        &PIECES[*self as usize][rotation.as_usize()]
    }

    /// Returns the 2-row shape for NEXT/HOLD display.
    ///
    /// # Returns
    ///
    /// Reference to a 2-row shape for preview display.
    pub(crate) fn display_shape(&self) -> &[[BlockKind; 4]] {
        &PIECES[*self as usize][0][..2]
    }
}

/// Piece shape (4x4 block array).
pub(crate) type PieceShape = [[BlockKind; 4]; 4];

const fn gen_rotates(size: usize, shape: &PieceShape) -> [PieceShape; 4] {
    let mut rotates = [*shape; 4];
    let mut i = 1;
    while i < 4 {
        let mut new_shape = [[BlockKind::Empty; 4]; 4];
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

const PIECES: [[PieceShape; 4]; PieceKind::LEN] = {
    use BlockKind::Empty as E;
    const I: BlockKind = BlockKind::Piece(PieceKind::I);
    const O: BlockKind = BlockKind::Piece(PieceKind::O);
    const S: BlockKind = BlockKind::Piece(PieceKind::S);
    const Z: BlockKind = BlockKind::Piece(PieceKind::Z);
    const J: BlockKind = BlockKind::Piece(PieceKind::J);
    const L: BlockKind = BlockKind::Piece(PieceKind::L);
    const T: BlockKind = BlockKind::Piece(PieceKind::T);
    const EEEE: [BlockKind; 4] = [E; 4];
    [
        // I-piece
        gen_rotates(4, &[EEEE, [I, I, I, I], EEEE, EEEE]),
        // O-piece
        gen_rotates(2, &[[O, O, E, E], [O, O, E, E], EEEE, EEEE]),
        // S-piece
        gen_rotates(3, &[[E, S, S, E], [S, S, E, E], EEEE, EEEE]),
        // Z-piece
        gen_rotates(3, &[[Z, Z, E, E], [E, Z, Z, E], EEEE, EEEE]),
        // J-piece
        gen_rotates(3, &[[J, E, E, E], [J, J, J, E], EEEE, EEEE]),
        // L-piece
        gen_rotates(3, &[[E, E, L, E], [L, L, L, E], EEEE, EEEE]),
        // T-piece
        gen_rotates(3, &[[E, T, E, E], [T, T, T, E], EEEE, EEEE]),
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
