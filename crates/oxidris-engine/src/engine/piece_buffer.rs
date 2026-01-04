use std::collections::VecDeque;

use rand::{SeedableRng as _, prelude::StdRng, seq::SliceRandom};

use crate::PieceKind;

/// Manages piece generation and hold system using the 7-bag randomization algorithm.
///
/// # 7-Bag System
///
/// The 7-bag system ensures fair piece distribution by:
///
/// 1. Creating a "bag" containing all 7 piece types (I, O, S, Z, J, L, T)
/// 2. Shuffling the bag randomly
/// 3. Drawing pieces in order from the bag
/// 4. Refilling with a new shuffled bag when 7 or fewer pieces remain
///
/// This prevents long droughts of any piece type while maintaining randomness.
///
/// # Hold System
///
/// - Can hold one piece at a time
/// - First hold stores the current piece and draws from the queue
/// - Subsequent holds swap the current piece with the held piece
///
/// # Example
///
/// ```
/// use oxidris_engine::engine::PieceBuffer;
///
/// let mut buffer = PieceBuffer::new();
///
/// // Draw pieces
/// let first = buffer.pop_next();
/// let second = buffer.pop_next();
///
/// // Preview upcoming pieces
/// let upcoming: Vec<_> = buffer.next_pieces().take(5).collect();
/// ```
#[derive(Debug, Clone)]
pub struct PieceBuffer {
    rng: StdRng,
    bag: VecDeque<PieceKind>,
    held: Option<PieceKind>,
}

impl Default for PieceBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceBuffer {
    /// Creates a new piece buffer with OS-seeded randomization.
    ///
    /// The bag is immediately filled with the first shuffled set of 7 pieces.
    #[must_use]
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
    /// Refills when the bag has 7 or fewer pieces remaining. After filling,
    /// the bag will contain at least 8 elements (ensuring 7 remain after the next pop).
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

    /// Draws the next piece from the bag.
    ///
    /// Automatically refills the bag when needed to maintain the 7-bag system.
    ///
    /// # Panics
    ///
    /// Panics if the bag is empty (should never happen with proper refill logic).
    pub fn pop_next(&mut self) -> PieceKind {
        self.fill_bag();
        self.bag
            .pop_front()
            .expect("Piece bag should never be empty")
    }

    /// Returns an iterator over the upcoming pieces in the queue.
    ///
    /// Useful for previewing what pieces are coming next. The iterator
    /// always contains at least 8 elements due to the refill strategy.
    pub fn next_pieces(&self) -> impl Iterator<Item = PieceKind> + '_ {
        self.bag.iter().copied()
    }

    /// Returns what piece would be received if hold is used now.
    ///
    /// - If a piece is held: returns the held piece
    /// - If no piece is held: returns the next piece from the queue
    #[must_use]
    pub fn peek_hold_result(&self) -> PieceKind {
        self.held.unwrap_or_else(|| self.bag[0])
    }

    /// Executes a hold operation: swaps current piece with held piece or queue.
    ///
    /// # Arguments
    ///
    /// * `current` - The piece to store in hold
    ///
    /// # Returns
    ///
    /// - If a piece is held: returns the held piece (swap)
    /// - If no piece is held: returns the next piece from queue
    pub fn hold(&mut self, current: PieceKind) -> PieceKind {
        self.held
            .replace(current)
            .unwrap_or_else(|| self.pop_next())
    }

    /// Returns the currently held piece, if any.
    ///
    /// Returns `None` if no piece has been held yet in the current session.
    #[must_use]
    pub fn held_piece(&self) -> Option<PieceKind> {
        self.held
    }
}
