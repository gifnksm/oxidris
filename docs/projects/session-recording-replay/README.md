# Recording and In-Game Replay

This project implements gameplay session recording and replay functionality for debugging, analysis, and sharing.

- **Document type**: Project Documentation
- **Purpose**: Record gameplay sessions and replay them with full playback controls
- **Audience**: Developers, AI assistants
- **When to read**: When working on recording, replay, or playback features
- **Prerequisites**: [Architecture Overview](../../architecture/README.md), [Engine Documentation](../../architecture/engine/README.md)
- **Related documents**: [Design](design.md), [Roadmap](roadmap.md)

## Overview

This project adds the ability to record gameplay sessions (both manual and AI play) and replay them with full playback controls. This enables:

- **Debugging AI behavior** - Understand why the AI made specific decisions
- **Performance analysis** - Review game-over situations to identify weaknesses
- **Learning** - Study successful strategies by reviewing high-score games
- **Sharing** - Share interesting gameplay sessions with others

## Project Status

**Status**: Implementation Phase

**Current Phase**: Step 4 complete (Replay command with full playback controls)

**Next Steps**: Step 5 (In-Game Replay)

## Goals

### Primary Goals

1. **Record gameplay sessions** during manual and auto-play
2. **Save recordings** to disk with metadata (timestamp, seed, model name, final stats)
3. **Recording replay** - View saved recordings with a dedicated replay command
4. **In-game replay** - Rewind and review history during pause/game-over

### Secondary Goals (Future)

1. **Feature visualization** - Overlay feature values during replay
2. **Placement analysis** - Show alternative placements and their scores
3. **Speed control** - Adjust playback speed during replay

## Scope

### In Scope

- Recording board states and piece placements during gameplay
- Ring buffer for memory-efficient history storage
- Saving recordings to JSON files with metadata
- Loading and viewing saved recordings (recording replay)
- Playback controls (play/pause, step forward/backward)
- In-game replay (rewind during pause/game-over)

### Out of Scope

- Real-time feature calculation during gameplay (calculated on-demand during replay)
- Video recording or screen capture
- Network-based sharing or cloud storage
- Automated replay analysis or comparison tools

## Key Design Decisions

See [Design Documentation](design.md) for detailed rationale.

### Data Format

- **Use existing `SessionData` type** from `oxidris-analysis` for consistency
- **Add CLI-specific `RecordedSession` type** with metadata (timestamp, seed, player type, final stats)
- **JSON serialization** for human-readable files

### Memory Management

- **Ring buffer** stores only the most recent N turns (default: 10,000)
- **Memory-only until game end** - write to disk only when game terminates
- **Configurable replay buffer** via `--max-replay-turns N` option

### Recording Control

- **No in-game toggle** - recording is enabled at command start
- **Automatic save** on game termination
- **Rewind available** any time in memory (during pause/game-over)

### File Management

- **Directory-based output**: `--record-dir DIR` (default: `data/recordings/`)
- **Auto-generated filenames**: `{prefix}_{YYYYMMDD_HHMMSS}.json`
  - Manual play: `manual_20260106_153045.json`
  - Auto-play: `ai_aggro_20260106_153045.json`

## Implementation Plan

See [Roadmap](roadmap.md) for detailed step-by-step implementation plan.

### Phase Overview

1. **Data structures and memory management** (Ring buffer, `RecordedSession` type)
2. **Manual play recording** (Basic recording in manual mode)
3. **Auto-play recording** (Recording in auto-play mode with model metadata)
4. **Recording replay command** (Standalone replay viewer with playback controls)
5. **In-game replay** (Rewind history during pause/game-over)
6. **Advanced features** (Feature visualization, placement analysis)
7. **Documentation and testing** (Final polish and integration tests)

## Usage Examples

### Recording a Manual Play Session

```bash
# Record with default settings (10,000 turn replay buffer)
oxidris play --save-recording

# Record with custom replay buffer size
oxidris play --save-recording --max-replay-turns 5000

# Record to custom directory
oxidris play --save-recording --record-dir ./my_recordings/
```

### Recording an Auto-Play Session

```bash
# Record AI gameplay
oxidris auto-play models/ai/aggro.json --save-recording

# Record in turbo mode
oxidris auto-play models/ai/aggro.json --save-recording --turbo
```

### Recording Replay (Viewing Saved Recordings)

```bash
# Replay a recording
oxidris replay data/recordings/ai_aggro_20260106_153045.json

# Playback controls:
#   Space: Play/Pause
#   j/k or ↓/↑: Step backward/forward (1 turn)
#   h/l or ←/→: Jump backward/forward (10 turns)
#   g/Home: First turn | G/End: Last turn (Shift+g)
#   q/Esc: Quit
```

### In-Game Replay (During Gameplay)

```text
During gameplay:
  p: Pause game
  (While paused) R: Enter in-game replay mode
  (In replay mode) j/k or ↓/↑: Step backward/forward (1 turn)
  (In replay mode) h/l or ←/→: Jump backward/forward (10 turns)
  (In replay mode) g/Home: First | G/End: Last (Shift+g)
  (In replay mode) Space: Play/Pause
  (In replay mode) q/Esc: Return to current state
```

## Success Criteria

- ✅ Can record manual and auto-play sessions (Steps 1-3 complete)
- ✅ Recordings save correctly with metadata (Steps 1-3 complete)
- ✅ Can view saved recordings with full playback controls (Step 4 complete)
- ⬜ Can use in-game replay during pause/game-over (Step 5)
- ✅ Memory usage remains bounded (ring buffer works correctly)
- ✅ Handles invalid files gracefully (serde error messages)

## Future Enhancements

After core functionality is complete, consider:

- **Feature overlays** - Display feature values during replay
- **Placement comparison** - Show alternative moves and their evaluations
- **Speed control** - Adjust playback speed (0.5x, 2x, etc.)
- **Bookmarks** - Mark interesting moments for quick navigation
- **Session comparison** - Compare two recordings side-by-side

## See Also

- [Design Documentation](design.md) - Detailed design decisions and data structures
- [Roadmap](roadmap.md) - Step-by-step implementation plan
- [Engine Documentation](../../architecture/engine/README.md) - Game mechanics reference
- [Analysis Documentation](../../architecture/evaluator/README.md) - Feature system reference
