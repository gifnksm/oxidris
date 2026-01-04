# Engine Implementation Notes

This document describes implementation details, simplifications, and limitations of the Oxidris game engine.

- **Document type**: Reference
- **Purpose**: Detailed specification of engine mechanics and differences from standard Tetris
- **Audience**: AI assistants, human contributors, players interested in technical details
- **When to read**: When working on engine code, evaluator features, or understanding gameplay mechanics
- **Prerequisites**: Basic Tetris knowledge; [README.md](../../../README.md) for project overview
- **Related documents**: [Evaluator Documentation](../evaluator/README.md) (features depend on engine mechanics)

## Overview

Oxidris implements a Tetris-like game engine that supports both human play and AI training/evaluation. While it follows standard Tetris mechanics in many respects, several areas use simplified implementations that differ from official Tetris guidelines.

| Feature | Implementation | Standard Tetris |
| ------- | ------------- | --------------- |
| Rotation system | Simplified 4-direction kicks | Full SRS with kick tables |
| Piece generation | 7-bag system | ✅ Matches |
| Hold system | Standard hold | ✅ Matches |
| Board size | 10×20 | ✅ Matches |
| Scoring | Basic line clears only | Combo and B2B bonuses |
| Spin detection | Not implemented | T-spin/mini-T detection |
| Movement | Standard operations | ✅ Matches |

## Rotation System

**Location:** `crates/oxidris-engine/src/core/piece.rs` (`super_rotation` function)

The rotation system is **not** a full Super Rotation System (SRS) implementation. Instead, it uses a simplified wall kick algorithm:

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

**How it works:**

- Attempts basic rotation first
- If collision detected, tries 4 simple offsets: up, down, left, right (in that order)
- Returns first valid position found, or None if all fail

**Differences from standard SRS:**

- ❌ No official SRS kick tables (5 test positions per rotation state)
- ❌ No piece-specific kick patterns (I-piece vs. other pieces)
- ❌ No rotation state-aware offsets

## Piece Generation

**Location:** `crates/oxidris-engine/src/engine/piece_buffer.rs`

The piece randomization uses the standard **7-bag system**:

- All 7 piece types (I, O, S, Z, J, L, T) are shuffled into a bag
- Pieces are drawn in order from the bag
- When 7 or fewer pieces remain, a new shuffled bag of 7 is added
- This ensures relatively even distribution and prevents long droughts

✅ This matches modern Tetris guidelines.

## Hold System

**Location:** `crates/oxidris-engine/src/engine/game_field.rs`

Standard hold mechanics are implemented:

- ✅ Can hold current piece and swap with held piece
- ✅ If no piece is held, draws from next queue
- ✅ Hold is validated for collision before allowing swap
- ✅ Can only hold once per piece

## Scoring

Line clears are counted but scoring is simplified:

- ✅ Line clear counts tracked
- ❌ No combo tracking
- ❌ No back-to-back (B2B) bonus
- ❌ No T-spin detection or scoring
- ❌ No other spin detection (I-spin, etc.)
- ❌ No mini vs. full spin distinction

Spin detection depends on tracking rotation states and kick patterns, which are not available in the simplified rotation system.

## Board Dimensions

Standard 10×20 board:

- Width: 10 columns
- Visible height: 20 rows
- Additional buffer rows above for piece spawning

✅ This matches standard Tetris dimensions.

## Movement

**Location:** `crates/oxidris-engine/src/engine/game_field.rs`

Standard operations are supported:

- ✅ Left/right movement
- ✅ Rotation (left and right)
- ✅ Hard drop (instant drop to bottom)
- ✅ Soft drop (accelerated downward movement)
- ✅ Hold piece

The engine supports both programmatic control (AI agents) and human input:

- Human play mode uses standard move/rotate/drop actions
- AI mode evaluates placements and executes them atomically
- No gravity timing or frame-perfect inputs required

## Game Termination

Games end when a piece collides at spawn position (top-out):

- ✅ Standard Tetris game-over condition

## Implications

### For Human Players

1. **Simplified rotation:** Most placements work as expected, but some advanced kicks are missing
2. **No T-spins:** Scoring and strategies based on spins are not available
3. **Standard fundamentals:** Core Tetris gameplay (stacking, line clears, holds) works normally

### For AI Development

**Training:**

1. Simplified rotation is consistent - AI doesn't need to learn complex kick patterns
2. Reduced state space - fewer valid placements to evaluate
3. No advanced techniques - T-spins, specific kicks not part of strategy space
4. Focus on fundamentals - height management, hole avoidance, structure building

**Evaluation:**

1. Feature analysis - all board features and survival statistics are based on simplified rotation
2. Normalization parameters - generated from gameplay using this engine
3. Model transferability - trained models are specific to this implementation
4. Benchmarking - comparisons should be made within this system, not against standard Tetris records

## Implementation Details

### BitBoard Representation

**Location:** `crates/oxidris-engine/src/core/bit_board.rs`

The board uses bitboard representation for efficient collision detection:

- Each row is a 16-bit integer
- Fast bitwise operations for collision checks
- Optimized for AI search (evaluating many possible placements)

This is an implementation detail that doesn't affect game mechanics.

## See Also

- [Tetris SRS Documentation](https://tetris.wiki/Super_Rotation_System) - Official SRS specification
- [Evaluator Documentation](../evaluator/README.md) - How features are extracted and evaluated
- Engine source: `crates/oxidris-engine/src/`
