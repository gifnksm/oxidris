# Session Recording and Replay - Design Documentation

This document describes the detailed design decisions, data structures, and architecture for the session recording and replay functionality.

- **Document type**: Design Documentation
- **Purpose**: Detailed technical design for recording and replay implementation
- **Audience**: Developers, AI assistants implementing the features
- **When to read**: Before implementing any recording/replay functionality
- **Prerequisites**: [Project Overview](README.md), [Architecture Overview](../../architecture/README.md)
- **Related documents**: [Roadmap](roadmap.md)

## Design Principles

1. **Memory Efficiency**: Use ring buffer to limit memory usage regardless of game length
2. **Compatibility**: Reuse existing `SessionData` type for consistency with analysis tools
3. **Simplicity**: Record only essential data; compute derived values on-demand
4. **Non-Intrusive**: Recording should not affect gameplay performance

## Data Structures

### Module Organization

```text
oxidris-cli/src/record/
├── mod.rs                  # Module exports
├── recorded_session.rs     # RecordedSession, RecordMetadata
└── session_history.rs      # SessionHistory (ring buffer)
```

**Design Rationale:**

- `RecordedSession` is CLI-specific (recording/replay functionality)
- `SessionData` is analysis-specific (training data structure)
- Both can coexist: CLI layer reuses `SessionData` for compatibility
- Avoids circular dependency (analysis doesn't depend on CLI)

### Data Type Hierarchy

```text
oxidris-analysis/src/session.rs
├─ SessionCollection    # Training data (multiple sessions)
├─ SessionData          # Shared: session information
└─ BoardAndPlacement    # Shared: board state + piece

oxidris-cli/src/record/
├─ RecordedSession      # Recording (single session + metadata)
├─ RecordMetadata       # CLI-specific metadata
└─ SessionHistory       # Ring buffer for memory management
```

### RecordedSession

**Location**: `oxidris-cli/src/record/recorded_session.rs`

```rust
use oxidris_analysis::session::SessionData;
use oxidris_cli::schema::AiModel;
use oxidris_engine::{GameStats, PieceSeed};
use serde::{Deserialize, Serialize};

/// Recorded play session with metadata for replay functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedSession {
    /// Metadata about the recording
    pub metadata: RecordMetadata,
    /// The actual session data (reuses analysis layer's type)
    pub session_data: SessionData,
}

/// Metadata for a recorded play session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordMetadata {
    /// Timestamp when recording was created (ISO 8601 format)
    pub recorded_at: String,
    /// Random seed used for deterministic piece generation
    pub seed: PieceSeed,
    /// Player information (manual or AI with model data)
    pub player: PlayerInfo,
    /// Final game statistics at the time of recording
    pub final_stats: GameStats,
}

/// Information about the player type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerInfo {
    /// Manual play by human
    Manual,
    /// AI play with full model data
    Auto { model: AiModel },
}
```

**Field Descriptions:**

- `recorded_at`: ISO 8601 timestamp (e.g., `"2026-01-06T15:30:45Z"`) for sorting and identification
- `seed`: `PieceSeed` for deterministic piece generation. Enables exact replay of piece sequences.
- `player`: Enum distinguishing manual vs AI play, with full `AiModel` data for AI (preserves model even if file is modified/deleted)
- `final_stats`: Score, lines cleared, etc. at recording time
- `session_data`: Reuses existing `SessionData` type from analysis layer

**Why Separate from SessionCollection:**

- `SessionCollection` is for training data (multiple sessions, batch analysis)
- `RecordedSession` is for gameplay recording (single session, playback)
- Different use cases warrant different top-level structures

### SessionHistory (Ring Buffer)

**Location**: `oxidris-cli/src/record/session_history.rs`

Ring buffer for maintaining recent game history in memory. Stores up to a configured capacity of board states in a circular buffer. When full, oldest entries are overwritten by new ones.

**Design Rationale:**

- **Ring buffer** prevents unbounded memory growth
- **Fixed capacity** determined by `--history-size` option
- Default capacity: 10,000 entries (~640 KB memory usage)

## File Format

File format is determined by serde JSON serialization of the `RecordedSession` structure.

**Format Choice:**

- **JSON**: Human-readable, debuggable, standard tooling support
- **Compatibility**: `session_data` matches existing `SessionData` format from analysis layer
- **Forward compatibility**: Can add new metadata fields without breaking old readers

### File Naming Convention

**Pattern**: `{prefix}_{YYYYMMDD_HHMMSS}.json`

Where prefix is:

- Manual play: `manual`
- AI play: `ai_{model_name}`

**Examples:**

- Manual play: `manual_20260106_153045.json`
- Auto-play (aggro): `ai_aggro_20260106_153045.json`
- Auto-play (defensive): `ai_defensive_20260106_153045.json`

**Rationale:**

- **Sortable by timestamp** in file listings
- **Self-documenting** (can identify player type from filename)
- **Collision-resistant** (second-level precision)

## Command-Line Interface

### Recording Options

**Manual Play:**

```bash
oxidris play [OPTIONS]
  --record                    Enable recording
  --record-dir <DIR>          Recording output directory (default: data/recordings/)
  --history-size <N>          Number of recent turns to keep (default: 10000)
```

**Auto-Play:**

```bash
oxidris auto-play <MODEL> [OPTIONS]
  --record                    Enable recording
  --record-dir <DIR>          Recording output directory (default: data/recordings/)
  --history-size <N>          Number of recent turns to keep (default: 10000)
  --turbo                     Run in turbo mode (works with --record)
```

### Replay Command

```bash
oxidris replay <FILE>
```

**Playback Controls:**

- `Space`: Toggle play/pause
- `j` / `k` or `↓` / `↑`: Step backward/forward (1 turn)
- `h` / `l` or `←` / `→`: Jump backward/forward (10 turns)
- `g` or `Home`: Jump to first turn
- `G` or `End`: Jump to last turn (Shift+g)
- `q` / `Esc`: Quit replay viewer

## UI Design

### Replay Viewer Screen

```text
┌────────────────────────────────────────────────────────────┐
│  Replay: ai_aggro_20260106_153045.json                     │
│  Turn: 234 / 450                                           │
├────────────────────────────────────────────────────────────┤
│                                                            │
│         ┌──────────────────────┐                          │
│         │                      │                          │
│         │                      │                          │
│         │   [Board Display]    │                          │
│         │                      │                          │
│         │                      │                          │
│         └──────────────────────┘                          │
│                                                            │
├────────────────────────────────────────────────────────────┤
│  Space (Play/Pause) | j/k or ↓/↑ (1 turn) | h/l or ←/→ (10 turns)
│  g/Home (First) | G/End (Last) | q/Esc (Quit)             │
└────────────────────────────────────────────────────────────┘

Note: Lowercase = no Shift needed; Uppercase (e.g., `G`, `H`) = Shift required.
```

### In-Game History Mode

**Entry Points:**

- Pause screen: Press `H` to enter history mode (Shift+h)
- Game Over screen: Press `H` to enter history mode (Shift+h)

**UI Indicator:**

```text
┌────────────────────────────────────────────────────────────┐
│  [HISTORY MODE] Turn: 234 / 450 (-216 from current)       │
├────────────────────────────────────────────────────────────┤
│         ┌──────────────────────┐                          │
│         │   [Board Display]    │                          │
│         └──────────────────────┘                          │
├────────────────────────────────────────────────────────────┤
│  j/k or ↓/↑ (1 turn) | h/l or ←/→ (10 turns)              │
│  g/Home (First) | G/End (Last) | Space (Play) | q/Esc (Exit)
└────────────────────────────────────────────────────────────┘

Note: `H` (entering history) and `G` (last turn) require Shift. `h`/`l` (10-turn jumps within history) do not.
```

**History Mode Controls:**

- `j` / `k` or `↓` / `↑`: Step backward/forward (1 turn)
- `h` / `l` or `←` / `→`: Jump backward/forward (10 turns)
- `g` or `Home`: Jump to first turn
- `G` or `End`: Jump to last turn (Shift+g)
- `Space`: Toggle auto-playback
- `q` / `Esc`: Exit history mode, return to current state

## Memory Management Strategy

### Recording Behavior

**During Gameplay:**

1. Each turn, capture `BoardAndPlacement` (before piece is placed)
2. Push to `SessionHistory` ring buffer
3. If buffer is full, oldest entry is overwritten
4. Memory usage remains constant at ~640 KB (for 10,000 turns)

**On Game Termination:**

1. Collect all boards from ring buffer
2. Create `RecordedSession` with metadata (including full `AiModel` for AI play)
3. Serialize to JSON
4. Write to file in `--record-dir`
5. Clear history buffer

### Edge Cases

**Very Long Games:**

- If game exceeds `--history-size` turns, only the most recent turns are saved
- Example: With `--history-size 10000`, a 50,000 turn game saves only turns 40,001-50,000
- This is acceptable: focus is on game-over analysis, not full playback

**Game Crashes:**

- Recording is lost (not written to disk until game ends)
- Future enhancement: Periodic auto-save every N minutes

**Disk Write Failures:**

- Log error to console
- Display warning in UI
- Game continues normally
- History remains in memory (user can retry save manually in future version)

## Integration Points

### With Game Engine

**Hook Point**: After each piece placement in `GameSession::complete_placement()`

```rust
// Pseudo-code
impl GameSession {
    pub fn complete_placement(&mut self) {
        let board = self.field.board().clone();
        let piece = self.active_piece.clone();
        
        // Existing logic...
        self.place_piece();
        self.clear_lines();
        
        // Recording hook
        if let Some(history) = &mut self.history {
            history.push(BoardAndPlacement {
                turn: self.turn,
                before_placement: board,
                placement: piece,
            });
        }
    }
}
```

### With Evaluator (Replay Feature Values)

**During Replay** (not during recording):

```rust
// Pseudo-code: Calculate features on-demand during replay
let analysis = PlacementAnalysis::new(&board, &piece);
let features = evaluator.evaluate_all_features(&analysis);

// Display in UI
println!("Holes: {}", features.holes);
println!("Height: {}", features.max_height);
```

**Rationale**: Computing features during gameplay would add overhead. Better to compute on-demand during replay when performance is less critical.

## Testing Strategy

### Unit Tests

**`SessionHistory` tests:**

- ✅ Push and retrieve boards
- ✅ Ring buffer wrapping (overwrite oldest when full)
- ✅ Empty buffer handling
- ✅ Capacity edge cases

**`RecordedSession` tests:**

- ✅ Metadata generation (timestamp format, filename generation)
- ✅ PlayerInfo enum handling (manual vs auto)

**File I/O tests:**

- ✅ Save recording to disk
- ✅ Load recording from disk
- ✅ Handle missing files gracefully
- ✅ Handle corrupted JSON gracefully

### Integration Tests

**Manual tests** (TUI is difficult to automate):

- Record a manual play session
- Record an auto-play session
- Replay a saved recording
- Use in-game history browsing
- Verify file contents match expected format

**Automated tests** (if feasible):

- Programmatically create a game session
- Record it without UI
- Verify saved file structure
- Load and parse the file

## Performance Considerations

### Recording Overhead

**Per-turn cost:**

- Clone `BitBoard` (~8 bytes)
- Clone `Piece` (~16 bytes)
- Push to ring buffer (O(1))
- **Total**: Negligible (<1μs per turn)

**Memory overhead:**

- Ring buffer: ~640 KB (for 10,000 turns)
- **Total**: Acceptable for modern systems

### Replay Performance

**Loading:**

- Deserialize JSON file (depends on file size)
- For 10,000 turns: ~10-50ms (acceptable)

**Playback:**

- No per-frame computation (just display stored boards)
- Feature calculation only if user requests overlay

## Future Enhancements

### Feature Overlays (Step 6)

Display feature values during replay:

```text
┌────────────────────────────────────────────────────────────┐
│  [FEATURES] Turn: 234 / 450                                │
├────────────────────────────────────────────────────────────┤
│  Holes: 3              Bumpiness: 5                        │
│  Max Height: 12        Well Depth: 4                       │
│  Transitions: 8        Top-out Risk: 0.23                  │
└────────────────────────────────────────────────────────────┘
```

**Implementation:**

- Press `F` during replay to toggle feature overlay
- Compute features on-demand using `PlacementAnalysis`
- Display in separate panel or overlay on board

### Placement Comparison (Step 6)

Show alternative placements the AI could have made:

```text
┌────────────────────────────────────────────────────────────┐
│  [PLACEMENTS] Turn: 234                                    │
├────────────────────────────────────────────────────────────┤
│  ✓ Selected: I piece at (3, 0) - Score: 0.85              │
│    Alt 1: I piece at (0, 0) - Score: 0.72                 │
│    Alt 2: I piece at (6, 0) - Score: 0.68                 │
└────────────────────────────────────────────────────────────┘
```

**Implementation:**

- Press `P` during replay to toggle placement view
- Re-run placement search for the current turn
- Display all candidates with their scores

### Speed Control

Allow adjusting playback speed:

- `+`: Increase speed (2x, 4x, 8x)
- `-`: Decrease speed (0.5x, 0.25x)
- Display current speed in UI

## Open Questions

None at this time. All design decisions have been finalized.

## See Also

- [Project Overview](README.md) - Goals and scope
- [Roadmap](roadmap.md) - Implementation plan
- [Architecture Overview](../../architecture/README.md) - System design
- [Engine Documentation](../../architecture/engine/README.md) - Game mechanics
