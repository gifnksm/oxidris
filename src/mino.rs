use rand::{Rng, distr::StandardUniform, prelude::Distribution, seq::SliceRandom};

use crate::block::BlockKind;

pub(crate) type MinoShape = [[BlockKind; 4]; 4];

const MINO_KIND_MAX: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum MinoKind {
    I,
    O,
    S,
    Z,
    J,
    L,
    T,
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
    use crate::block::BlockKind::{Empty as E, *};
    [
        // Iミノ
        [[E, E, E, E], [E, E, E, E], [I, I, I, I], [E, E, E, E]],
        // Oミノ
        [[E, E, E, E], [E, O, O, E], [E, O, O, E], [E, E, E, E]],
        // Sミノ
        [[E, E, E, E], [E, S, S, E], [S, S, E, E], [E, E, E, E]],
        // Zミノ
        [[E, E, E, E], [Z, Z, E, E], [E, Z, Z, E], [E, E, E, E]],
        // Jミノ
        [[E, E, E, E], [J, E, E, E], [J, J, J, E], [E, E, E, E]],
        // Lミノ
        [[E, E, E, E], [E, E, L, E], [L, L, L, E], [E, E, E, E]],
        // Tミノ
        [[E, E, E, E], [E, T, E, E], [T, T, T, E], [E, E, E, E]],
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
