use rand::{Rng, distr::StandardUniform, prelude::Distribution, seq::SliceRandom};

use crate::block::BlockKind;

pub(crate) type MinoShape = [[BlockKind; 4]; 4];

const MINO_KIND_MAX: usize = 7;

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
    pub(crate) const fn shape(&self) -> &MinoShape {
        &MINOS[*self as usize]
    }
}

const MINOS: [MinoShape; MINO_KIND_MAX] = {
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
        [EEEE, EEEE, [I, I, I, I], EEEE],
        // O-Mino
        [EEEE, [E, O, O, E], [E, O, O, E], EEEE],
        // S-Mino
        [EEEE, [E, S, S, E], [S, S, E, E], EEEE],
        // Z-Mino
        [EEEE, [Z, Z, E, E], [E, Z, Z, E], EEEE],
        // J-Mino
        [EEEE, [J, E, E, E], [J, J, J, E], EEEE],
        // L-Mino
        [EEEE, [E, E, L, E], [L, L, L, E], EEEE],
        // T-Mino
        [EEEE, [E, T, E, E], [T, T, T, E], EEEE],
    ]
};

pub(crate) fn gen_mino_7() -> [MinoShape; MINO_KIND_MAX] {
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
    que.map(|mino| *mino.shape())
}
