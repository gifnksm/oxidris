# Engine Implementation Notes

This document describes implementation details, simplifications, and limitations of the Oxidris game engine.

## Overview

Oxidris implements a Tetris-like game engine for AI training and evaluation purposes. While it follows standard Tetris mechanics in many respects, several areas use simplified implementations that differ from official Tetris guidelines.

## Rotation System (SRS)

### Current Implementation: Simplified

The rotation system is **not** a full Super Rotation System (SRS) implementation. Instead, it uses a simplified wall kick algorithm.

**Location:** `crates/oxidris-engine/src/core/piece.rs` (`super_rotation` function)

```rust
fn super_rotation(board: &BitBoard, piece: Piece) -> Option<Piece> {
    let pieces = [piece.up(), piece.down(), piece.left(), piece.right()];
    for piece in pieces.iter().flatten() {
        if !board.is_colliding(*piece) {
            return Some(*piece);
        }
    }
    None
}
```

**What it does:**
- Attempts basic rotation first
- If collision detected, tries 4 simple offsets: up, down, left, right (in that order)
- Returns first valid position found, or None if all fail

**What's missing:**
- ❌ No official SRS kick tables (5 test positions per rotation state)
- ❌ No piece-specific kick patterns (I-piece vs. other pieces)
- ❌ No rotation state-aware offsets
- ❌ T-spin detection and scoring
- ❌ Other spin detection (I-spin, etc.)

### Why This Matters

**For AI training:**
- ✅ Simplified rotation is consistent and deterministic
- ✅ Reduces search space for placement evaluation
- ✅ Still allows most reasonable placements
- ⚠️ Some advanced techniques (T-spin setups, specific kicks) are not possible
- ⚠️ AI strategies learned here may not transfer to standard Tetris

**For evaluation:**
- The simplified rotation affects what board states are reachable
- Feature analysis and survival statistics are based on this simplified system
- Comparisons to standard Tetris play should account for this difference

### Future Considerations

Implementing full SRS would require:
1. Adding kick offset tables per piece type and rotation state
2. Implementing proper test sequence (5 positions)
3. Adding spin detection logic
4. Potentially adjusting scoring system

This is **not currently planned** because:
- Current system is sufficient for AI training purposes
- Full SRS adds complexity without clear benefit for statistical analysis
- Existing data and trained models are based on the simplified system

## Piece Generation

### 7-Bag System: Standard Implementation

**Location:** `crates/oxidris-engine/src/engine/piece_buffer.rs`

The piece randomization uses the standard **7-bag system**:
- All 7 piece types (I, O, S, Z, J, L, T) are shuffled into a bag
- Pieces are drawn in order from the bag
- When 7 or fewer pieces remain, a new shuffled bag of 7 is added
- This ensures relatively even distribution and prevents long droughts

✅ This matches modern Tetris guidelines and is implemented correctly.

## Hold System

**Location:** `crates/oxidris-engine/src/engine/game_field.rs`

Standard hold mechanics are implemented:
- ✅ Can hold current piece and swap with held piece
- ✅ If no piece is held, draws from next queue
- ✅ Hold is validated for collision before allowing swap

## Scoring and Spins

### Simplified Scoring

**Current implementation:**
- Line clears are counted but scoring is minimal
- No combo tracking
- No back-to-back (B2B) bonus
- No T-spin scoring

**Used primarily for:**
- Session evaluation (line clear counts)
- Fitness functions in genetic algorithm
- Not for competitive scoring comparison

### No Spin Detection

- ❌ No T-spin detection
- ❌ No other spin detection (I-spin, etc.)
- ❌ No mini vs. full spin distinction

**Rationale:** Spin detection requires full SRS implementation. Since rotation is simplified, spin mechanics are not applicable.

## Game Termination

Games end when a piece collides at spawn position (top-out):
- ✅ Standard Tetris game-over condition
- No artificial turn limits in the engine itself

**Note:** Data generation for training may use turn limits (e.g., MAX_TURNS=500), but this is a training constraint, not an engine limitation.

## Board Dimensions

**Standard 10×20 board:**
- Width: 10 columns
- Visible height: 20 rows
- Additional buffer rows above for piece spawning

✅ This matches standard Tetris dimensions.

## Movement Mechanics

### Supported Operations

- ✅ Left/right movement
- ✅ Rotation (left and right)
- ✅ Hard drop (instant drop to bottom)
- ✅ Soft drop (accelerated downward movement)
- ✅ Hold piece

### Input System

The engine is designed for programmatic control (AI placement selection) rather than real-time human input:
- No gravity timing or frame-perfect inputs
- Placements are evaluated and executed atomically
- Focus is on state-to-state transitions, not real-time gameplay

## Performance Optimizations

### BitBoard Representation

**Location:** `crates/oxidris-engine/src/core/bit_board.rs`

The board uses bitboard representation for efficient collision detection:
- Each row is a 16-bit integer
- Fast bitwise operations for collision checks
- Optimized for AI search (evaluating many possible placements)

✅ This is an implementation detail that doesn't affect game mechanics.

## Implications for AI Development

### What This Means for Training

1. **Simplified rotation is consistent:** AI doesn't need to learn complex kick patterns
2. **Reduced state space:** Fewer valid placements to evaluate
3. **No advanced techniques:** T-spins, specific kicks not part of strategy space
4. **Focus on fundamentals:** Height management, hole avoidance, structure building

### What This Means for Evaluation

1. **Feature analysis:** All board features and survival statistics are based on simplified rotation
2. **Normalization parameters:** Generated from gameplay using this engine
3. **Model transferability:** Trained models are specific to this implementation
4. **Benchmarking:** Comparisons should be made within this system, not against standard Tetris records

## Summary

| Feature | Implementation | Standard Tetris |
|---------|---------------|-----------------|
| Rotation system | Simplified 4-direction kicks | Full SRS with kick tables |
| Piece generation | 7-bag system | ✅ Matches |
| Hold system | Standard hold | ✅ Matches |
| Board size | 10×20 | ✅ Matches |
| T-spin detection | Not implemented | Full detection |
| Spin scoring | Not implemented | T-spin/mini-T scoring |
| Combo tracking | Not implemented | Combo bonuses |
| Movement | Programmatic placement | Real-time input |

## See Also

- [Tetris SRS Documentation](https://tetris.wiki/Super_Rotation_System) - Official SRS specification
- [Evaluator Documentation](../evaluator/README.md) - How features are extracted and evaluated
- Engine source: `crates/oxidris-engine/src/`

## Documentation Maintenance

When modifying the engine:
- ✅ Update this document if game mechanics change
- ✅ Note any changes that affect feature extraction or board analysis
- ✅ Document new limitations or simplifications
- ✅ Update "Implications for AI Development" if search space changes