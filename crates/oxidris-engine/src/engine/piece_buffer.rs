use std::{collections::VecDeque, fmt::Write as _};

use rand::{
    Rng, SeedableRng as _,
    distr::{Distribution, StandardUniform},
    seq::SliceRandom,
};
use rand_pcg::Pcg32;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
    rng: Pcg32,
    bag: VecDeque<PieceKind>,
    held: Option<PieceKind>,
}

impl Default for PieceBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Seed for deterministic piece generation.
///
/// This is a 128-bit (16-byte) seed used to initialize the random number
/// generator for piece generation. Using the same seed will produce the same
/// sequence of pieces, enabling:
///
/// - Reproducible gameplay for debugging
/// - Session recording and replay
/// - Deterministic testing
///
/// # Example
///
/// ```
/// use oxidris_engine::{GameSession, PieceSeed};
/// use rand::Rng as _;
///
/// // Generate a random seed
/// let seed: PieceSeed = rand::rng().random();
///
/// // Create two sessions with the same seed
/// let session1 = GameSession::with_seed(60, seed);
/// let session2 = GameSession::with_seed(60, seed);
///
/// // Both sessions will have the same piece sequence
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PieceSeed([u8; 16]);

impl Serialize for PieceSeed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let num = u128::from_be_bytes(self.0);
        let mut hex_str = String::with_capacity(2 * self.0.len());
        write!(&mut hex_str, "{num:032x}").unwrap();
        serializer.serialize_str(&hex_str)
    }
}

impl<'de> Deserialize<'de> for PieceSeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        if hex_str.len() != 32 {
            return Err(serde::de::Error::custom(format!(
                "invalid hex: expected 32 characters, got {}",
                hex_str.len()
            )));
        }
        let num = u128::from_str_radix(&hex_str, 16)
            .map_err(|e| serde::de::Error::custom(format!("invalid hex: {hex_str} ({e})")))?;
        Ok(Self(num.to_be_bytes()))
    }
}

/// Allows generating random `PieceSeed` values using the standard random distribution.
///
/// This implementation enables idiomatic seed generation with `rng.random()`.
impl Distribution<PieceSeed> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PieceSeed {
        let mut seed = [0; 16];
        rng.fill(&mut seed);
        PieceSeed(seed)
    }
}

impl PieceBuffer {
    /// Creates a new piece buffer with a random seed.
    ///
    /// The bag is immediately filled with the first shuffled set of 7 pieces.
    /// For deterministic piece generation, use [`Self::with_seed`] instead.
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(rand::rng().random())
    }

    /// Like [`Self::new`], but with a specific seed for deterministic piece generation.
    #[must_use]
    pub fn with_seed(seed: PieceSeed) -> Self {
        let rng = Pcg32::from_seed(seed.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    mod piece_seed_serialization {
        use super::*;

        /// Helper to create a `PieceSeed` from a byte array
        fn seed_from_bytes(bytes: [u8; 16]) -> PieceSeed {
            PieceSeed(bytes)
        }

        #[test]
        fn test_roundtrip_random_seed() {
            // Generate a random seed and verify roundtrip
            let seed: PieceSeed = rand::rng().random();
            let serialized = serde_json::to_string(&seed).unwrap();
            let deserialized: PieceSeed = serde_json::from_str(&serialized).unwrap();
            assert_eq!(seed.0, deserialized.0);
        }

        #[test]
        fn test_format_is_32_char_hex_string() {
            let seed: PieceSeed = rand::rng().random();
            let serialized = serde_json::to_string(&seed).unwrap();

            // Remove quotes from JSON string
            let hex_str = serialized.trim_matches('"');

            // Should be exactly 32 hex characters (128 bits / 4 bits per char)
            assert_eq!(hex_str.len(), 32);

            // All characters should be valid hex
            assert!(hex_str.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn test_known_value_all_zeros() {
            let seed = seed_from_bytes([0u8; 16]);
            let serialized = serde_json::to_string(&seed).unwrap();

            assert_eq!(serialized, "\"00000000000000000000000000000000\"");

            let deserialized: PieceSeed = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized.0, [0u8; 16]);
        }

        #[test]
        fn test_known_value_all_ones() {
            let seed = seed_from_bytes([0xFFu8; 16]);
            let serialized = serde_json::to_string(&seed).unwrap();

            assert_eq!(serialized, "\"ffffffffffffffffffffffffffffffff\"");

            let deserialized: PieceSeed = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized.0, [0xFFu8; 16]);
        }

        #[test]
        fn test_known_value_sequential_bytes() {
            // Test big-endian ordering: first byte should appear first in hex string
            let seed = seed_from_bytes([
                0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
                0x32, 0x10,
            ]);
            let serialized = serde_json::to_string(&seed).unwrap();

            // Big-endian: bytes appear in order as hex pairs
            assert_eq!(serialized, "\"0123456789abcdeffedcba9876543210\"");

            let deserialized: PieceSeed = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized.0, seed.0);
        }

        #[test]
        fn test_deserialize_uppercase_hex() {
            // Should accept uppercase hex characters
            let json = "\"0123456789ABCDEFFEDCBA9876543210\"";
            let deserialized: PieceSeed = serde_json::from_str(json).unwrap();

            assert_eq!(
                deserialized.0,
                [
                    0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76,
                    0x54, 0x32, 0x10
                ]
            );
        }

        #[test]
        fn test_error_invalid_hex_characters() {
            let json = "\"ghijklmnopqrstuvwxyzghijklmnopqr\""; // 32 chars but not hex
            let result: Result<PieceSeed, _> = serde_json::from_str(json);

            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("invalid hex"));
        }

        #[test]
        fn test_error_too_short() {
            let json = "\"0123456789abcdef0123456789abcde\""; // 31 chars
            let result: Result<PieceSeed, _> = serde_json::from_str(json);

            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("invalid hex"));
        }

        #[test]
        fn test_error_too_long() {
            let json = "\"0123456789abcdef0123456789abcdef0\""; // 33 chars
            let result: Result<PieceSeed, _> = serde_json::from_str(json);

            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("invalid hex"));
        }

        #[test]
        fn test_error_empty_string() {
            let json = "\"\"";
            let result: Result<PieceSeed, _> = serde_json::from_str(json);

            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("invalid hex"));
        }

        #[test]
        fn test_deterministic_piece_generation() {
            // Same seed should produce same piece sequence
            let seed = seed_from_bytes([
                0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
                0x77, 0x88,
            ]);

            let mut buffer1 = PieceBuffer::with_seed(seed);
            let mut buffer2 = PieceBuffer::with_seed(seed);

            // First 20 pieces should be identical
            for _ in 0..20 {
                assert_eq!(buffer1.pop_next(), buffer2.pop_next());
            }
        }

        #[test]
        fn test_serialize_deserialize_preserves_piece_generation() {
            // Serialize and deserialize a seed, then verify piece sequence is preserved
            let original_seed: PieceSeed = rand::rng().random();
            let serialized = serde_json::to_string(&original_seed).unwrap();
            let deserialized_seed: PieceSeed = serde_json::from_str(&serialized).unwrap();

            let mut buffer1 = PieceBuffer::with_seed(original_seed);
            let mut buffer2 = PieceBuffer::with_seed(deserialized_seed);

            // First 20 pieces should be identical
            for _ in 0..20 {
                assert_eq!(buffer1.pop_next(), buffer2.pop_next());
            }
        }
    }
}
