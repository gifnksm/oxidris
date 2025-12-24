use std::collections::VecDeque;

use rand::{SeedableRng as _, prelude::StdRng, seq::SliceRandom};

use crate::PieceKind;

/// Manages the order and random generation of pieces.
///
/// Supplies pieces using the 7-bag system.
#[derive(Debug, Clone)]
pub struct PieceBuffer {
    rng: StdRng,
    bag: VecDeque<PieceKind>,
    held: Option<PieceKind>,
}

impl PieceBuffer {
    /// Creates a new [`PieceGenerator`].
    ///
    /// The random seed is initialized from the OS's random data source.
    pub fn new() -> Self {
        let rng = StdRng::from_os_rng();
        let bag = VecDeque::with_capacity(PieceKind::LEN * 2);
        let mut this = Self {
            rng,
            bag,
            held: None,
        };
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
    pub fn pop_next(&mut self) -> PieceKind {
        self.fill_bag();
        self.bag
            .pop_front()
            .expect("Piece bag should never be empty")
    }

    /// Returns an iterator of upcoming pieces in the bag.
    ///
    /// The iterator always contains at least 8 elements.
    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.bag.iter().copied()
    }

    /// Swaps the currently held piece with the given piece.
    pub fn swap_hold(&mut self, current: PieceKind) -> Option<PieceKind> {
        self.held.replace(current)
    }

    /// Returns the currently held piece, if any.
    pub fn held_piece(&self) -> Option<PieceKind> {
        self.held
    }
}
