# Session Recording and Replay - Implementation Roadmap

This document provides a detailed, step-by-step implementation plan for the session recording and replay functionality.

- **Document type**: Implementation Plan
- **Purpose**: Phase-by-phase guide for implementing recording and replay features
- **Audience**: Developers, AI assistants implementing the features
- **When to read**: Before starting implementation, or when checking progress
- **Prerequisites**: [Project Overview](README.md), [Design Documentation](design.md)
- **Related documents**: [Architecture Overview](../../architecture/README.md)

## Overview

The implementation is divided into 7 steps, each building on the previous one. Each step is designed to be completable independently and produces a testable increment of functionality.

## Step 1: Data Structures and Memory Management

**Goal**: Establish the foundation for recording functionality.

**Status**: Complete

**Tasks:**

- [x] Create `RecordedSession`, `TurnRecord`, `PlayerInfo` types in `schema/record.rs`
- [x] Implement `SessionHistory` with ring buffer for memory-efficient history storage
- [x] Implement `RecordingSession` wrapper for automatic piece placement recording
- [x] Add `PieceSeed` serialize/deserialize support
- [x] Write unit tests for ring buffer

**Validation:**

- [x] Unit tests pass
- [x] Ring buffer overwrites oldest entries when full
- [x] JSON serialization works correctly

---

## Step 2: Manual Play Recording

**Goal**: Implement basic recording functionality in manual play mode.

**Status**: Complete

**Dependencies**: Step 1 complete

**Tasks:**

- [x] Add `--save-recording`, `--recording-dir`, `--history-size` options to `play` command
- [x] Integrate `RecordingSession` into `ManualPlayScreen`
- [x] Capture board states after each piece placement
- [x] Save recording to file on game end with filename `manual_YYYYMMDD_HHMMSS.json`
- [x] Fix keyboard control display (lowercase for non-Shift keys)
- ~~Add recording indicator in UI~~ (Not needed: recording is always active in memory)

**Validation:**

- [x] `play --save-recording` creates recording file on game end
- [x] File contains valid JSON and can be loaded
- [x] `--history-size` option works correctly

---

## Step 3: Auto-Play Recording

**Goal**: Extend recording functionality to auto-play mode.

**Status**: Not Started

**Dependencies**: Step 2 complete

**Tasks:**

- Add `--save-recording`, `--recording-dir`, `--history-size` options to `auto-play` command
- Integrate `RecordingSession` into `AutoPlayScreen`
- Include full `AiModel` data in `PlayerInfo::Auto` metadata
- Save recording with filename `ai_{model_name}_YYYYMMDD_HHMMSS.json`
- Ensure `--turbo` mode works with recording

**Validation:**

- `auto-play MODEL --save-recording` creates recording with AI model data
- Turbo mode recording works without performance issues

---

## Step 4: Replay Command (Basic Playback)

**Goal**: Create a standalone replay viewer with basic playback controls.

**Status**: Not Started

**Dependencies**: Step 3 complete (need recording files to test)

**Tasks:**

- Create `replay` subcommand with file loading
- Implement replay screen displaying board state and metadata
- Add playback controls: j/k or ↓/↑ (1 turn), h/l or ←/→ (10 turns), g/Home (first), G/End (last), Space (play/pause), q/Esc (quit)
- Auto-advance turns when playing (~60 FPS)
- Display board only (no hold/next/score as they're not saved per-turn)

**Validation:**

- Can load and replay recordings
- All playback controls work correctly
- Handles invalid files gracefully

---

## Step 5: In-Game Playback (History Browsing)

**Goal**: Allow players to rewind and review history during pause or game over.

**Status**: Not Started

**Dependencies**: Step 4 complete (reuse playback UI components)

**Tasks:**

- Add history mode toggle (H key) to pause and game over screens
- Implement history navigation: ←/→ (step), Home/End or ^/$ (first/last), Esc (exit)
- Display "HISTORY MODE" indicator and turn offset
- Preserve game state when exiting history mode

**Validation:**

- History mode works in pause and game over screens
- Navigation works correctly
- Exiting returns to correct state

---

## Step 6: Advanced Features (Feature Visualization)

**Goal**: Add optional feature overlays and placement analysis during replay.

**Status**: Not Started

**Dependencies**: Step 4 complete (basic replay working)

**Tasks:**

- Add feature overlay toggle (F key) showing feature values calculated on-demand
- Add placement candidate view (P key) showing all possible placements with scores
- Add playback speed control (+/- keys)

**Validation:**

- Feature values and placement candidates are calculated correctly
- Speed control works smoothly

---

## Step 7: Documentation and Polish

**Goal**: Finalize documentation and polish user experience.

**Status**: Not Started

**Dependencies**: Steps 1-6 complete

**Tasks:**

- Update README.md with recording/replay documentation and usage examples
- Update future-projects.md to mark "Interactive Replay Viewer" as complete
- Write integration tests if feasible (non-UI components)
- Polish error messages and UI

**Validation:**

- Documentation is complete and accurate
- All tests pass
- Features work as expected in real usage

---

## Progress Tracking

### Current Status

**Overall Progress**: ~30% (Steps 1-2 complete)

**Completed Steps**: Step 1, Step 2

**Current Step**: Step 3 (Not Started)

**Next Milestone**: Complete Step 3 (Auto-Play Recording)

### Step Status

- [x] **Step 1**: Data Structures and Memory Management
- [x] **Step 2**: Manual Play Recording
- [ ] **Step 3**: Auto-Play Recording
- [ ] **Step 4**: Replay Command (Basic Playback)
- [ ] **Step 5**: In-Game Playback (History Browsing)
- [ ] **Step 6**: Advanced Features (Feature Visualization)
- [ ] **Step 7**: Documentation and Polish

---

## Dependencies and Prerequisites

### External Dependencies

- No new crate dependencies required
- All functionality uses existing dependencies (serde, ratatui, oxidris-* crates)

### Internal Dependencies

- `oxidris-analysis` for `SessionData` type
- `oxidris-engine` for `GameStats` type
- `oxidris-evaluator` for feature calculation (Step 6 only)
- Existing TUI infrastructure (ratatui widgets)

### Order Constraints

- Steps 1-3 must be completed sequentially (each builds on previous)
- Step 4 can start after Step 3 (needs recording files to test)
- Step 5 can start after Step 4 (reuses playback components)
- Step 6 can start after Step 4 (independent feature additions)
- Step 7 should be last (finalizes everything)

---

## Testing Strategy

### Per-Step Testing

Each step includes its own validation criteria. Complete these before moving to the next step:

- **Step 1**: Unit tests for all new types
- **Step 2**: Manual testing of recording in play mode
- **Step 3**: Manual testing of recording in auto-play mode
- **Step 4**: Manual testing of replay functionality
- **Step 5**: Manual testing of history browsing
- **Step 6**: Manual testing of advanced features
- **Step 7**: Integration tests and final validation

### Test Scenarios

**Recording Tests:**

- Record a short game (manual and auto-play)
- Record a long game exceeding history size
- Record with turbo mode enabled
- Verify file format is correct

**Replay Tests:**

- Replay all saved recordings
- Test all playback controls
- Test with corrupted/missing files
- Test with old file format (after future changes)

**History Tests:**

- Browse history during pause
- Browse history after game over
- Test boundary conditions (first/last turn)
- Test exiting and re-entering history mode

---

## Risk Management

### Known Risks

1. **Performance**: Recording in turbo mode might add overhead
   - Mitigation: Use efficient ring buffer, avoid unnecessary copies
   - Validation: Benchmark turbo mode with/without recording

2. **Memory**: Very large history buffers could consume too much memory
   - Mitigation: Document reasonable limits, add validation
   - Validation: Test with extreme values (e.g., 1,000,000 turns)

3. **UI Complexity**: Adding many features could clutter the interface
   - Mitigation: Use toggles, keep defaults simple
   - Validation: User testing for usability

4. **File Format Changes**: Future changes might break old recordings
   - Mitigation: Use serde's forward compatibility features
   - Validation: Test loading old files with new code

---

## Future Enhancements (Beyond This Project)

After completing all 7 steps, consider these additional improvements:

- **Auto-save**: Periodic saving during long games (crash recovery)
- **Bookmarks**: Mark interesting moments for quick navigation
- **Session comparison**: Compare two recordings side-by-side
- **Export formats**: Export to video, animated GIF, or analysis CSV
- **Cloud sharing**: Upload recordings to share with others
- **Replay analysis**: Automated analysis of recordings (e.g., identify mistakes)

These should be tracked as separate future projects if pursued.

---

## See Also

- [Project Overview](README.md) - Goals and scope
- [Design Documentation](design.md) - Detailed technical design
- [Architecture Overview](../../architecture/README.md) - System design
- [AGENTS.md](../../../AGENTS.md) - Development guidelines
