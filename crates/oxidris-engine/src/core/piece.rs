use arrayvec::ArrayVec;
use rand::{Rng, distr::StandardUniform, prelude::Distribution};
use serde::{Deserialize, Serialize};

use super::{
    bit_board::{BitBoard, PIECE_SPAWN_X, PIECE_SPAWN_Y},
    block_board::Block,
};

/// A Tetris piece (tetromino) with position, rotation, and type.
///
/// This represents a piece at a specific location and orientation on the board.
/// Pieces are immutable - movement and rotation operations return new `Piece` instances.
///
/// # Coordinate System
///
/// - Position is relative to the top-left of the board
/// - Rotation is tracked as 0° (spawn), 90° right, 180°, or 270° right
/// - Each piece type has a 4×4 bounding box that rotates
///
/// # Example
///
/// ```
/// use oxidris_engine::{Piece, PieceKind};
///
/// let piece = Piece::new(PieceKind::T);
/// let moved = piece.right().unwrap();
/// let rotated = moved.rotated_right();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    position: PiecePosition,
    rotation: PieceRotation,
    kind: PieceKind,
}

impl Serialize for Piece {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Format: "kind#rotation@x,y" (e.g., "S#1@4,18")
        let s = format!(
            "{}#{}@{},{}",
            self.kind.as_char(),
            self.rotation.0,
            self.position.x,
            self.position.y
        );
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Piece {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Parse format: "kind#rotation@x,y" (e.g., "S#1@4,18")
        // First split by '#' to get kind and rest
        let mut parts = s.splitn(2, '#');
        let kind_str = parts.next().ok_or_else(|| {
            serde::de::Error::custom(format!("expected format 'kind#rotation@x,y', got '{s}'"))
        })?;
        let rest = parts.next().ok_or_else(|| {
            serde::de::Error::custom(format!(
                "missing '#' in format 'kind#rotation@x,y', got '{s}'"
            ))
        })?;

        // Parse kind
        let kind_char = kind_str
            .chars()
            .next()
            .ok_or_else(|| serde::de::Error::custom("missing piece kind"))?;
        if kind_str.len() != 1 {
            return Err(serde::de::Error::custom(format!(
                "piece kind must be single character, got '{kind_str}'"
            )));
        }
        let kind = PieceKind::from_char(kind_char)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid piece kind: {kind_char}")))?;

        // Split by '@' to get rotation and position
        let mut parts = rest.splitn(2, '@');
        let rotation_str = parts.next().ok_or_else(|| {
            serde::de::Error::custom(format!("missing rotation after '#', got '{s}'"))
        })?;
        let position_str = parts.next().ok_or_else(|| {
            serde::de::Error::custom(format!(
                "missing '@' in format 'kind#rotation@x,y', got '{s}'"
            ))
        })?;

        // Parse rotation
        let rotation_num = rotation_str.parse::<u8>().map_err(|e| {
            serde::de::Error::custom(format!("invalid rotation: {rotation_str} ({e})"))
        })?;
        if rotation_num > 3 {
            return Err(serde::de::Error::custom(format!(
                "rotation must be 0-3, got {rotation_num}"
            )));
        }
        let rotation = PieceRotation(rotation_num);

        // Split position by ',' to get x and y
        let mut parts = position_str.splitn(2, ',');
        let x_str = parts.next().ok_or_else(|| {
            serde::de::Error::custom(format!("missing x coordinate after '@', got '{s}'"))
        })?;
        let y_str = parts.next().ok_or_else(|| {
            serde::de::Error::custom(format!(
                "missing ',' in format 'kind#rotation@x,y', got '{s}'"
            ))
        })?;

        // Parse x and y
        let x = x_str
            .parse::<u8>()
            .map_err(|e| serde::de::Error::custom(format!("invalid x position: {x_str} ({e})")))?;

        let y = y_str
            .parse::<u8>()
            .map_err(|e| serde::de::Error::custom(format!("invalid y position: {y_str} ({e})")))?;

        let position = PiecePosition::new(x, y);

        Ok(Piece {
            position,
            rotation,
            kind,
        })
    }
}

impl Piece {
    #[must_use]
    pub fn new(kind: PieceKind) -> Self {
        Self {
            position: PiecePosition::SPAWN_POSITION,
            rotation: PieceRotation::default(),
            kind,
        }
    }

    #[must_use]
    pub fn position(&self) -> PiecePosition {
        self.position
    }

    #[must_use]
    pub fn rotation(&self) -> PieceRotation {
        self.rotation
    }

    #[must_use]
    pub fn kind(&self) -> PieceKind {
        self.kind
    }

    #[must_use]
    pub fn mask(&self) -> PieceMask {
        self.kind.mask(self.rotation)
    }

    pub fn occupied_positions(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.kind
            .occupied_positions(self.rotation)
            .map(move |(dx, dy)| (self.position.x() + dx, self.position.y() + dy))
    }

    #[must_use]
    pub fn left(&self) -> Option<Self> {
        let new_pos = self.position.left()?;
        Some(Self {
            position: new_pos,
            rotation: self.rotation,
            kind: self.kind,
        })
    }

    #[must_use]
    pub fn right(&self) -> Option<Self> {
        let new_pos = self.position.right()?;
        Some(Self {
            position: new_pos,
            rotation: self.rotation,
            kind: self.kind,
        })
    }

    #[must_use]
    pub fn up(&self) -> Option<Self> {
        let new_pos = self.position.up()?;
        Some(Self {
            position: new_pos,
            rotation: self.rotation,
            kind: self.kind,
        })
    }

    #[must_use]
    pub fn down(&self) -> Option<Self> {
        let new_pos = self.position.down()?;
        Some(Self {
            position: new_pos,
            rotation: self.rotation,
            kind: self.kind,
        })
    }

    #[must_use]
    pub fn rotated_right(&self) -> Self {
        Self {
            position: self.position,
            rotation: self.rotation.rotated_right(),
            kind: self.kind,
        }
    }

    #[must_use]
    pub fn rotated_left(&self) -> Self {
        Self {
            position: self.position,
            rotation: self.rotation.rotated_left(),
            kind: self.kind,
        }
    }

    #[must_use]
    pub fn super_rotated_left(self, board: &BitBoard) -> Option<Self> {
        let mut piece = self.rotated_left();
        if board.is_colliding(piece) {
            piece = super_rotation(board, piece)?;
        }
        Some(piece)
    }

    #[must_use]
    pub fn super_rotated_right(self, board: &BitBoard) -> Option<Self> {
        let mut piece = self.rotated_right();
        if board.is_colliding(piece) {
            piece = super_rotation(board, piece)?;
        }
        Some(piece)
    }

    #[must_use]
    pub fn super_rotations(&self, board: &BitBoard) -> ArrayVec<Self, 4> {
        let mut rotations = ArrayVec::new();
        rotations.push(*self);
        if self.kind == PieceKind::O {
            return rotations;
        }
        let mut prev = *self;
        for _ in 0..3 {
            let Some(piece) = prev.super_rotated_right(board) else {
                break;
            };
            rotations.push(piece);
            prev = piece;
        }
        rotations
    }

    #[must_use]
    pub fn simulate_drop_position(&self, board: &BitBoard) -> Self {
        let mut dropped = *self;
        while let Some(piece) = dropped.down().filter(|m| !board.is_colliding(*m)) {
            dropped = piece;
        }
        dropped
    }
}

/// Attempts simplified wall kick after a failed rotation.
///
/// This is **not** a full Super Rotation System (SRS) implementation. Instead, it tries
/// 4 simple offsets in order: up, right, down, left. The first valid position is returned.
///
/// # Differences from Standard SRS
///
/// - No official kick tables (5 test positions per rotation state)
/// - No piece-specific patterns (I-piece vs. other pieces)
/// - No rotation state-aware offsets
///
/// See [Engine Implementation Notes](../../../docs/architecture/engine/README.md) for details.
///
/// # Arguments
///
/// * `board` - Current board state for collision detection
/// * `piece` - The rotated piece that collided
///
/// # Returns
///
/// The first valid kick position, or `None` if all kicks fail.
fn super_rotation(board: &BitBoard, piece: Piece) -> Option<Piece> {
    let pieces = [piece.up(), piece.right(), piece.down(), piece.left()];
    for piece in pieces.iter().flatten() {
        if !board.is_colliding(*piece) {
            return Some(*piece);
        }
    }
    None
}

/// Position of a piece on the board.
///
/// Coordinates are stored as `u8` for compactness and represent the anchor point
/// of the piece within its 4×4 bounding box.
///
/// # Coordinate System
///
/// - (0, 0) is at the top-left of the playable area
/// - X increases rightward (columns)
/// - Y increases downward (rows)
/// - Includes sentinel margins for boundary checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct PiecePosition {
    x: u8,
    y: u8,
}

impl PiecePosition {
    #[expect(clippy::cast_possible_truncation)]
    pub const SPAWN_POSITION: Self = Self::new(PIECE_SPAWN_X as u8, PIECE_SPAWN_Y as u8);

    #[must_use]
    pub const fn new(x: u8, y: u8) -> Self {
        assert!((x as usize) < BitBoard::TOTAL_WIDTH);
        assert!((y as usize) < BitBoard::TOTAL_HEIGHT);
        Self { x, y }
    }

    #[must_use]
    pub fn x(self) -> usize {
        usize::from(self.x)
    }

    #[must_use]
    pub fn y(self) -> usize {
        usize::from(self.y)
    }

    #[must_use]
    pub const fn left(&self) -> Option<Self> {
        if self.x == 0 {
            None
        } else {
            Some(Self::new(self.x - 1, self.y))
        }
    }

    #[must_use]
    pub const fn right(&self) -> Option<Self> {
        if self.x as usize >= BitBoard::TOTAL_WIDTH - 1 {
            None
        } else {
            Some(Self::new(self.x + 1, self.y))
        }
    }

    #[must_use]
    pub const fn up(&self) -> Option<Self> {
        if self.y == 0 {
            None
        } else {
            Some(Self::new(self.x, self.y - 1))
        }
    }

    #[must_use]
    pub const fn down(&self) -> Option<Self> {
        if self.y as usize >= BitBoard::TOTAL_HEIGHT - 1 {
            None
        } else {
            Some(Self::new(self.x, self.y + 1))
        }
    }
}

/// Rotation state of a piece.
///
/// Represents one of four rotation states:
///
/// - `0`: 0° (spawn orientation)
/// - `1`: 90° clockwise
/// - `2`: 180°
/// - `3`: 270° clockwise (90° counterclockwise)
///
/// Rotation operations wrap around modulo 4.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PieceRotation(u8);

impl PieceRotation {
    #[must_use]
    pub fn rotated_right(self) -> Self {
        PieceRotation((self.0 + 1) % 4)
    }

    #[must_use]
    pub fn rotated_left(self) -> Self {
        PieceRotation((self.0 + 3) % 4)
    }

    const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// Enum representing the type of piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[repr(u8)]
pub enum PieceKind {
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
    pub const LEN: usize = 7;

    pub(crate) fn mask(self, rotation: PieceRotation) -> PieceMask {
        PIECE_MASKS[self as usize][rotation.as_usize()]
    }

    /// Returns an iterator of occupied positions for the piece in the given rotation.
    pub fn occupied_positions(
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

    /// Returns the single character representation of this piece kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_engine::PieceKind;
    ///
    /// assert_eq!(PieceKind::I.as_char(), 'I');
    /// assert_eq!(PieceKind::T.as_char(), 'T');
    /// ```
    #[must_use]
    pub const fn as_char(self) -> char {
        match self {
            PieceKind::I => 'I',
            PieceKind::O => 'O',
            PieceKind::S => 'S',
            PieceKind::Z => 'Z',
            PieceKind::J => 'J',
            PieceKind::L => 'L',
            PieceKind::T => 'T',
        }
    }

    /// Parses a piece kind from a single character.
    ///
    /// # Examples
    ///
    /// ```
    /// use oxidris_engine::PieceKind;
    ///
    /// assert_eq!(PieceKind::from_char('I'), Some(PieceKind::I));
    /// assert_eq!(PieceKind::from_char('T'), Some(PieceKind::T));
    /// assert_eq!(PieceKind::from_char('X'), None);
    /// ```
    #[must_use]
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'I' => Some(PieceKind::I),
            'O' => Some(PieceKind::O),
            'S' => Some(PieceKind::S),
            'Z' => Some(PieceKind::Z),
            'J' => Some(PieceKind::J),
            'L' => Some(PieceKind::L),
            'T' => Some(PieceKind::T),
            _ => None,
        }
    }
}

/// Bitboard representation of a piece within its 4×4 bounding box.
///
/// Each element is a 16-bit integer representing 4 rows of 4 bits each.
/// Used for efficient collision detection with the board's bitboard representation.
pub(crate) type PieceMask = [u16; 4];

/// Generates all 4 rotation states of a piece mask by rotating 90° clockwise.
///
/// # Arguments
///
/// * `size` - Effective size of the piece (3 for most pieces, 4 for I, 2 for O)
/// * `mask` - Initial piece mask at 0° rotation
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

/// Piece shape represented as a 4×4 cell array.
///
/// Used for rendering and analysis. Each cell is either `Block::Empty` or
/// `Block::Piece(kind)` indicating the piece type.
type PieceShape = [[Block; 4]; 4];

/// Generates all 4 rotation states of a piece shape by rotating 90° clockwise.
///
/// # Arguments
///
/// * `size` - Effective size of the piece (3 for most pieces, 4 for I, 2 for O)
/// * `shape` - Initial piece shape at 0° rotation
const fn shape_rotations(size: usize, shape: &PieceShape) -> [PieceShape; 4] {
    let mut rotates = [*shape; 4];
    let mut i = 1;
    while i < 4 {
        let mut new_shape = [[Block::Empty; 4]; 4];
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
    use Block::Empty as E;
    const I: Block = Block::Piece(PieceKind::I);
    const O: Block = Block::Piece(PieceKind::O);
    const S: Block = Block::Piece(PieceKind::S);
    const Z: Block = Block::Piece(PieceKind::Z);
    const J: Block = Block::Piece(PieceKind::J);
    const L: Block = Block::Piece(PieceKind::L);
    const T: Block = Block::Piece(PieceKind::T);
    const EEEE: [Block; 4] = [E; 4];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_serialization() {
        // Test basic serialization format: "kind#rotation@x,y"
        let piece = Piece {
            position: PiecePosition::new(4, 18),
            rotation: PieceRotation(1),
            kind: PieceKind::S,
        };

        let serialized = serde_json::to_string(&piece).unwrap();
        println!("Piece serialized format: {serialized}");

        assert_eq!(serialized, "\"S#1@4,18\"");

        let deserialized: Piece = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, piece);
    }

    #[test]
    fn test_piece_serialization_all_kinds() {
        let kinds = [
            PieceKind::I,
            PieceKind::O,
            PieceKind::S,
            PieceKind::Z,
            PieceKind::J,
            PieceKind::L,
            PieceKind::T,
        ];

        for kind in kinds {
            let piece = Piece {
                position: PiecePosition::new(5, 10),
                rotation: PieceRotation(2),
                kind,
            };

            let serialized = serde_json::to_string(&piece).unwrap();
            let deserialized: Piece = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, piece);
        }
    }

    #[test]
    fn test_piece_serialization_all_rotations() {
        for rotation_num in 0..4 {
            let piece = Piece {
                position: PiecePosition::new(3, 7),
                rotation: PieceRotation(rotation_num),
                kind: PieceKind::T,
            };

            let serialized = serde_json::to_string(&piece).unwrap();
            let expected = format!("\"T#{rotation_num}@3,7\"");
            assert_eq!(serialized, expected);

            let deserialized: Piece = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, piece);
        }
    }

    #[test]
    fn test_piece_deserialization_error_cases() {
        // Invalid format
        assert!(serde_json::from_str::<Piece>("\"S1@4,18\"").is_err());
        assert!(serde_json::from_str::<Piece>("\"S#1#4,18\"").is_err());
        assert!(serde_json::from_str::<Piece>("\"S#1@4\"").is_err());

        // Invalid piece kind
        assert!(serde_json::from_str::<Piece>("\"X#1@4,18\"").is_err());

        // Invalid rotation (must be 0-3)
        assert!(serde_json::from_str::<Piece>("\"S#4@4,18\"").is_err());
        assert!(serde_json::from_str::<Piece>("\"S#-1@4,18\"").is_err());

        // Invalid coordinates
        assert!(serde_json::from_str::<Piece>("\"S#1@abc,18\"").is_err());
        assert!(serde_json::from_str::<Piece>("\"S#1@4,xyz\"").is_err());
    }

    #[test]
    fn test_piece_kind_char_conversion() {
        assert_eq!(PieceKind::I.as_char(), 'I');
        assert_eq!(PieceKind::O.as_char(), 'O');
        assert_eq!(PieceKind::S.as_char(), 'S');
        assert_eq!(PieceKind::Z.as_char(), 'Z');
        assert_eq!(PieceKind::J.as_char(), 'J');
        assert_eq!(PieceKind::L.as_char(), 'L');
        assert_eq!(PieceKind::T.as_char(), 'T');

        assert_eq!(PieceKind::from_char('I'), Some(PieceKind::I));
        assert_eq!(PieceKind::from_char('O'), Some(PieceKind::O));
        assert_eq!(PieceKind::from_char('S'), Some(PieceKind::S));
        assert_eq!(PieceKind::from_char('Z'), Some(PieceKind::Z));
        assert_eq!(PieceKind::from_char('J'), Some(PieceKind::J));
        assert_eq!(PieceKind::from_char('L'), Some(PieceKind::L));
        assert_eq!(PieceKind::from_char('T'), Some(PieceKind::T));

        assert_eq!(PieceKind::from_char('X'), None);
        assert_eq!(PieceKind::from_char('x'), None);
    }
}
