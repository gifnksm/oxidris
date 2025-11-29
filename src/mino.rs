use rand::{Rng, distr::StandardUniform, prelude::Distribution, seq::SliceRandom};

use crate::block::BlockKind;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MinoRotate(u8);

impl MinoRotate {
    pub(crate) fn rotate_right(&self) -> Self {
        MinoRotate((self.0 + 1) % 4)
    }

    pub(crate) fn rotate_left(&self) -> Self {
        MinoRotate((self.0 + 3) % 4)
    }

    const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum MinoKind {
    I = 0,
    O = 1,
    S = 2,
    Z = 3,
    J = 4,
    L = 5,
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
    pub(crate) const LEN: usize = 7;

    pub(crate) const fn shape(&self, rotate: MinoRotate) -> &MinoShape {
        &MINOS[*self as usize][rotate.as_usize()]
    }

    pub(crate) fn display_shape(&self) -> &[[BlockKind; 4]] {
        &MINOS[*self as usize][0][..2]
    }
}

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

pub(crate) fn gen_mino_sequence() -> [MinoKind; MinoKind::LEN] {
    let mut rng = rand::rng();
    let mut que = [
        MinoKind::I,
        MinoKind::O,
        MinoKind::S,
        MinoKind::Z,
        MinoKind::J,
        MinoKind::L,
        MinoKind::T,
    ];
    que.shuffle(&mut rng);
    que
}
