use std::collections::VecDeque;

use rand::{
    Rng, SeedableRng as _,
    distr::StandardUniform,
    prelude::{Distribution, StdRng},
    seq::SliceRandom,
};

use crate::block::BlockKind;

/// Represents the rotation state of a tetromino.
///
/// 0: 0 degrees, 1: 90° right, 2: 180°, 3: 90° left.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MinoRotate(u8);

impl MinoRotate {
    pub(crate) fn rotate_right(self) -> Self {
        MinoRotate((self.0 + 1) % 4)
    }

    pub(crate) fn rotate_left(self) -> Self {
        MinoRotate((self.0 + 3) % 4)
    }

    const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

/// Enum representing the type of tetromino.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum MinoKind {
    /// I tetromino.
    I = 0,
    /// O tetromino.
    O = 1,
    /// S tetromino.
    S = 2,
    /// Z tetromino.
    Z = 3,
    /// J tetromino.
    J = 4,
    /// L tetromino.
    L = 5,
    /// T tetromino.
    T = 6,
}

impl Distribution<MinoKind> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MinoKind {
        match rng.random_range(0..=6) {
            0 => MinoKind::I,
            1 => MinoKind::O,
            2 => MinoKind::S,
            3 => MinoKind::Z,
            4 => MinoKind::J,
            5 => MinoKind::L,
            _ => MinoKind::T,
        }
    }
}

impl MinoKind {
    /// Number of tetromino types (7).
    pub(crate) const LEN: usize = 7;

    /// Returns the tetromino shape for the given rotation.
    ///
    /// # Arguments
    ///
    /// * `rotate` - The rotation state.
    ///
    /// # Returns
    ///
    /// Reference to the tetromino shape.
    pub(crate) const fn shape(&self, rotate: MinoRotate) -> &MinoShape {
        &MINOS[*self as usize][rotate.as_usize()]
    }

    /// Returns the 2-row shape for NEXT/HOLD display.
    ///
    /// # Returns
    ///
    /// Reference to a 2-row shape for preview display.
    pub(crate) fn display_shape(&self) -> &[[BlockKind; 4]] {
        &MINOS[*self as usize][0][..2]
    }
}

/// Tetromino shape (4x4 block array).
pub(crate) type MinoShape = [[BlockKind; 4]; 4];

const fn gen_rotates(size: usize, shape: &MinoShape) -> [MinoShape; 4] {
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

const MINOS: [[MinoShape; 4]; MinoKind::LEN] = {
    use crate::block::BlockKind::Empty as E;
    const I: BlockKind = BlockKind::Mino(MinoKind::I);
    const O: BlockKind = BlockKind::Mino(MinoKind::O);
    const S: BlockKind = BlockKind::Mino(MinoKind::S);
    const Z: BlockKind = BlockKind::Mino(MinoKind::Z);
    const J: BlockKind = BlockKind::Mino(MinoKind::J);
    const L: BlockKind = BlockKind::Mino(MinoKind::L);
    const T: BlockKind = BlockKind::Mino(MinoKind::T);
    const EEEE: [BlockKind; 4] = [E; 4];
    [
        // I-Mino
        gen_rotates(4, &[EEEE, [I, I, I, I], EEEE, EEEE]),
        // O-Mino
        gen_rotates(2, &[[O, O, E, E], [O, O, E, E], EEEE, EEEE]),
        // S-Mino
        gen_rotates(3, &[[E, S, S, E], [S, S, E, E], EEEE, EEEE]),
        // Z-Mino
        gen_rotates(3, &[[Z, Z, E, E], [E, Z, Z, E], EEEE, EEEE]),
        // J-Mino
        gen_rotates(3, &[[J, E, E, E], [J, J, J, E], EEEE, EEEE]),
        // L-Mino
        gen_rotates(3, &[[E, E, L, E], [L, L, L, E], EEEE, EEEE]),
        // T-Mino
        gen_rotates(3, &[[E, T, E, E], [T, T, T, E], EEEE, EEEE]),
    ]
};

/// Manages the order and random generation of tetrominoes.
///
/// Supplies tetrominoes using the 7-bag system.
#[derive(Debug, Clone)]
pub(crate) struct MinoGenerator {
    rng: StdRng,
    bag: VecDeque<MinoKind>,
}

impl MinoGenerator {
    /// Creates a new [`MinoGenerator`].
    ///
    /// The random seed is initialized from the OS's random data source.
    pub(crate) fn new() -> Self {
        let rng = StdRng::from_os_rng();
        let bag = VecDeque::with_capacity(MinoKind::LEN * 2);
        let mut this = Self { rng, bag };
        this.fill_bag();
        this
    }

    /// Fills the bag with a shuffled set of 7 tetrominoes when needed.
    ///
    /// After filling, the bag will always contain at least 8 elements
    /// (so that even after one `pop_next`, there are still 7 left).
    fn fill_bag(&mut self) {
        while self.bag.len() <= MinoKind::LEN {
            let mut new_bag = [
                MinoKind::I,
                MinoKind::O,
                MinoKind::S,
                MinoKind::Z,
                MinoKind::J,
                MinoKind::L,
                MinoKind::T,
            ];
            new_bag.shuffle(&mut self.rng);
            self.bag.extend(new_bag);
        }
    }

    /// Pops the next tetromino from the bag.
    ///
    /// # Panics
    ///
    /// Panics if the bag is empty (should never happen).
    pub(crate) fn pop_next(&mut self) -> MinoKind {
        self.fill_bag();
        self.bag
            .pop_front()
            .expect("Mino bag should never be empty")
    }

    /// Returns an iterator of upcoming tetrominoes in the bag.
    ///
    /// The iterator always contains at least 8 elements.
    pub(crate) fn next_minos(&self) -> impl Iterator<Item = MinoKind> + '_ {
        self.bag.iter().copied()
    }
}
