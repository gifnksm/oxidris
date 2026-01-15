use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{BufWriter, Write as _},
    ops::Deref,
    path::Path,
};

use anyhow::Context;
use chrono::Utc;
use oxidris_engine::{GameSession, GameStats, HoldError, PieceCollisionError, PieceSeed};
use rand::Rng as _;

use crate::schema::record::{PlayerInfo, RecordedSession, TurnRecord};

/// A wrapper around [`GameSession`] that automatically records piece placements.
///
/// This type provides the same gameplay API as `GameSession`, but tracks each
/// piece placement for later replay or analysis. Use [`into_history`](Self::into_history)
/// to extract the recorded history after the game ends.
///
/// # Recording Mechanism
///
/// Before each operation that may complete a piece placement (`hard_drop_and_complete`,
/// `increment_frame`), the current board state and falling piece are captured.
/// If the turn number changes after the operation, the snapshot is recorded.
#[derive(Debug)]
pub struct RecordingSession {
    session: GameSession,
    history: SessionHistory,
}

/// Provides read-only access to the underlying `GameSession`.
///
/// # Why `DerefMut` is NOT implemented
///
/// `DerefMut` must NOT be implemented for this type. Mutable operations on
/// `GameSession` (like `hard_drop_and_complete`, `increment_frame`) must go
/// through `RecordingSession`'s methods to ensure piece placements are recorded.
/// Implementing `DerefMut` would allow bypassing the recording logic.
impl Deref for RecordingSession {
    type Target = GameSession;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}

impl RecordingSession {
    /// Creates a new recording session with a random seed.
    ///
    /// # Arguments
    ///
    /// * `fps` - Frames per second for game timing
    /// * `player` - Player information (manual or AI)
    /// * `history_size` - Maximum number of turns to keep in the ring buffer
    pub fn new(fps: u64, player: PlayerInfo, history_size: usize) -> Self {
        let seed = rand::rng().random();
        let session = GameSession::with_seed(fps, seed);
        let history = SessionHistory::new(seed, player, history_size);
        Self { session, history }
    }

    /// Consumes the session and returns the recorded history.
    ///
    /// This method captures the final game statistics before returning.
    /// The returned [`SessionHistory`] can be saved to a file using [`SessionHistory::save`].
    pub fn into_history(mut self) -> SessionHistory {
        self.history.set_stats(self.session.stats().clone());
        self.history
    }

    fn capture_snapshot(&self) -> TurnRecord {
        TurnRecord {
            turn: self.session.stats().turn(),
            before_placement: self.session.field().board().clone(),
            placement: self.session.falling_piece(),
            hold_used: self.session.hold_used(),
        }
    }

    fn record_if_completed(&mut self, snapshot: TurnRecord) {
        if self.session.stats().turn() != snapshot.turn {
            self.history.record(snapshot);
        }
    }

    pub fn toggle_pause(&mut self) {
        self.session.toggle_pause();
    }

    pub fn increment_frame(&mut self) {
        let snapshot = self.capture_snapshot();
        self.session.increment_frame();
        self.record_if_completed(snapshot);
    }

    pub fn try_move_left(&mut self) -> Result<(), PieceCollisionError> {
        self.session.try_move_left()
    }

    pub fn try_move_right(&mut self) -> Result<(), PieceCollisionError> {
        self.session.try_move_right()
    }

    pub fn try_soft_drop(&mut self) -> Result<(), PieceCollisionError> {
        self.session.try_soft_drop()
    }

    pub fn try_rotate_left(&mut self) -> Result<(), PieceCollisionError> {
        self.session.try_rotate_left()
    }

    pub fn try_rotate_right(&mut self) -> Result<(), PieceCollisionError> {
        self.session.try_rotate_right()
    }

    pub fn try_hold(&mut self) -> Result<(), HoldError> {
        self.session.try_hold()
    }

    pub fn hard_drop_and_complete(&mut self) {
        let snapshot = self.capture_snapshot();
        self.session.hard_drop_and_complete();
        self.record_if_completed(snapshot);
    }
}

/// Recorded history of a game session.
///
/// Contains all the information needed to replay a game session:
/// - The random seed for deterministic piece generation
/// - Player information (manual or AI with model data)
/// - Final game statistics
/// - A ring buffer of recent turn records
///
/// This type is created by [`RecordingSession::into_history`] and can be
/// saved to a file using [`save`](Self::save).
#[derive(Debug)]
pub struct SessionHistory {
    seed: PieceSeed,
    player: PlayerInfo,
    final_stats: Option<GameStats>,
    buffer: RingBuffer<TurnRecord>,
}

impl SessionHistory {
    fn new(seed: PieceSeed, player: PlayerInfo, capacity: usize) -> Self {
        Self {
            seed,
            player,
            final_stats: None,
            buffer: RingBuffer::with_capacity(capacity),
        }
    }

    fn record(&mut self, snapshot: TurnRecord) {
        self.buffer.push(snapshot);
    }

    fn set_stats(&mut self, stats: GameStats) {
        self.final_stats = Some(stats);
    }

    /// Saves the recorded session to a JSON file.
    ///
    /// The filename is automatically generated based on the player type and
    /// current timestamp: `{player}_{YYYYMMDD_HHMMSS}.json`
    ///
    /// # Arguments
    ///
    /// * `record_dir` - Directory to save the recording (created if it doesn't exist)
    ///
    /// # Panics
    ///
    /// Panics if called before [`RecordingSession::into_history`] sets the final stats.
    /// This should not happen in normal usage since `SessionHistory` is only
    /// accessible through `into_history`.
    pub fn save(&self, record_dir: &Path) -> anyhow::Result<()> {
        let final_stats = self
            .final_stats
            .clone()
            .expect("final_stats should be set before save");

        fs::create_dir_all(record_dir)
            .with_context(|| format!("Failed to create directory {}", record_dir.display()))?;

        let timestamp = Utc::now();
        let prefix = match &self.player {
            PlayerInfo::Manual => "manual".to_owned(),
            PlayerInfo::Auto { model } => {
                format!("ai_{}", model.name)
            }
        };
        let filename = format!("{prefix}_{}.json", timestamp.format("%Y%m%d_%H%M%S"));
        let filepath = record_dir.join(filename);

        let data = RecordedSession {
            recorded_at: timestamp,
            seed: self.seed,
            player: self.player.clone(),
            final_stats,
            boards: self.buffer.to_vec(),
        };

        let file = File::create(&filepath)
            .with_context(|| format!("Failed to create file: {}", filepath.display()))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &data)
            .with_context(|| format!("Failed to write JSON to {}", filepath.display()))?;
        writer
            .flush()
            .with_context(|| format!("Failed to flush output to {}", filepath.display()))?;

        Ok(())
    }
}

/// A fixed-capacity ring buffer that overwrites oldest entries when full.
///
/// Used to limit memory usage when recording long game sessions.
/// When the buffer reaches capacity, new entries replace the oldest ones.
#[derive(Debug)]
struct RingBuffer<T> {
    capacity: usize,
    buf: VecDeque<T>,
}

impl<T> RingBuffer<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            buf: VecDeque::with_capacity(capacity),
        }
    }

    fn push(&mut self, item: T) {
        if self.capacity == 0 {
            return;
        }
        if self.buf.len() >= self.capacity {
            self.buf.pop_front();
        }
        self.buf.push_back(item);
    }

    fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.buf.clone().into()
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.buf.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic_push_and_to_vec() {
        let mut buf: RingBuffer<i32> = RingBuffer::with_capacity(5);

        buf.push(1);
        buf.push(2);
        buf.push(3);

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_ring_buffer_overwrites_oldest_when_full() {
        let mut buf: RingBuffer<i32> = RingBuffer::with_capacity(3);

        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.to_vec(), vec![1, 2, 3]);

        // Push beyond capacity - oldest (1) should be removed
        buf.push(4);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.to_vec(), vec![2, 3, 4]);

        // Push more - oldest (2) should be removed
        buf.push(5);
        assert_eq!(buf.to_vec(), vec![3, 4, 5]);
    }

    #[test]
    fn test_ring_buffer_capacity_one() {
        let mut buf: RingBuffer<&str> = RingBuffer::with_capacity(1);

        buf.push("first");
        assert_eq!(buf.to_vec(), vec!["first"]);

        buf.push("second");
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.to_vec(), vec!["second"]);
    }

    #[test]
    fn test_ring_buffer_capacity_zero() {
        let mut buf: RingBuffer<i32> = RingBuffer::with_capacity(0);

        // With capacity 0, nothing should be stored
        buf.push(1);
        buf.push(2);

        assert_eq!(buf.len(), 0);
        assert_eq!(buf.to_vec(), Vec::<i32>::new());
    }

    #[test]
    fn test_ring_buffer_empty() {
        let buf: RingBuffer<i32> = RingBuffer::with_capacity(10);

        assert_eq!(buf.len(), 0);
        assert_eq!(buf.to_vec(), Vec::<i32>::new());
    }

    #[test]
    fn test_ring_buffer_exactly_at_capacity() {
        let mut buf: RingBuffer<i32> = RingBuffer::with_capacity(3);

        buf.push(1);
        buf.push(2);
        buf.push(3);

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_ring_buffer_large_overflow() {
        let mut buf: RingBuffer<i32> = RingBuffer::with_capacity(3);

        // Push 10 items into a buffer of capacity 3
        for i in 1..=10 {
            buf.push(i);
        }

        // Should only have the last 3 items
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.to_vec(), vec![8, 9, 10]);
    }
}
