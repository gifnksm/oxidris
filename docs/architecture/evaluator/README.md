# Evaluator System

This document provides an overview of the three-level evaluator architecture and serves as a navigation hub to detailed implementation documentation.

- **Document type**: Explanation
- **Purpose**: Explain the evaluator system architecture and guide readers to detailed implementation documentation
- **Audience**: AI assistants, developers working on evaluation or training
- **When to read**: When understanding the overall evaluator system or looking for implementation details
- **Prerequisites**: [Architecture Overview](../README.md)
- **Related documents**: [Training System](../training/README.md), [Future Projects](../../future-projects.md)

## Overview

The evaluator system operates at three levels:

1. **Placement Evaluation** - Scores a single piece placement using features
2. **Turn Evaluation** - Selects the best placement for the current turn
3. **Session Evaluation** - Evaluates entire game sessions for training (fitness functions)

Each level builds on the previous one to enable both AI gameplay and training.

## Architecture

```text
┌─────────────────────────────────────────────────┐
│              Session Evaluation                 │
│   (Fitness function for GA training)            │
│                                                 │
│   oxidris-evaluator::session_evaluator          │
│   - AggroSessionEvaluator                       │
│   - DefensiveSessionEvaluator                   │
└──────────────────┬──────────────────────────────┘
                   │ Uses
                   ↓
┌─────────────────────────────────────────────────┐
│              Turn Evaluation                    │
│   (Select best placement for current turn)      │
│                                                 │
│   oxidris-evaluator::turn_evaluator             │
│   - Enumerates all valid placements             │
│   - Selects highest-scoring placement           │
└──────────────────┬──────────────────────────────┘
                   │ Uses
                   ↓
┌─────────────────────────────────────────────────┐
│            Placement Evaluation                 │
│   (Score a single piece placement)              │
│                                                 │
│   oxidris-evaluator::placement_evaluator        │
│   Extract → Transform → Normalize → Weighted Sum│
│   (features from board_feature module)          │
└─────────────────────────────────────────────────┘
```

## Three Levels

### Placement Evaluation

Assigns a score to a single piece placement using a four-step pipeline:

1. **Feature Extraction** - Measure board properties (holes, height, bumpiness, etc.)
2. **Feature Transformation** - Transform raw values into meaningful representations
3. **Feature Normalization** - Scale to [0, 1] using percentiles (P05-P95 for penalties, P75-P95 for risks, fixed ranges for bonuses/rewards)
4. **Weighted Evaluation** - Compute weighted sum: `score = Σ(wᵢ × featureᵢ)`

**Implementation:** `oxidris-evaluator::placement_evaluator`, `oxidris-evaluator::board_feature`

### Turn Evaluation

Selects the best placement for the current turn using greedy one-step lookahead:

1. Enumerate all valid placements (current piece + hold piece)
2. Score each placement using Placement Evaluator
3. Select placement with highest score

**Implementation:** `oxidris-evaluator::turn_evaluator`

### Session Evaluation

Evaluates entire game sessions using fitness functions for genetic algorithm training:

- **Aggro**: Balances survival time with line clearing efficiency
- **Defensive**: Prioritizes survival time above all else

Different fitness functions produce different play styles through different learned weights.

**Implementation:** `oxidris-evaluator::session_evaluator`

## Trained Models

Trained models are stored in `models/ai/`:

- **`aggro.json`** - Balances survival with line clear efficiency (trained with AggroSessionEvaluator)
- **`defensive.json`** - Prioritizes survival time (trained with DefensiveSessionEvaluator)

Both models use the same features but different weights, resulting in different play styles.

## Implementation Documentation

For detailed implementation documentation, design decisions, API usage, and current limitations, see the rustdoc in the following files:

- **`crates/oxidris-evaluator/src/lib.rs`** - Crate overview, three-level architecture, design principles
- **`crates/oxidris-evaluator/src/board_feature/mod.rs`** - Feature system, typology, transformation, normalization, redundancy analysis
- **`crates/oxidris-evaluator/src/placement_evaluator.rs`** - Weighted sum evaluation, linear model advantages/limitations
- **`crates/oxidris-evaluator/src/turn_evaluator.rs`** - Turn selection, greedy lookahead strategy
- **`crates/oxidris-evaluator/src/session_evaluator.rs`** - Fitness functions (Aggro/Defensive), design rationale and limitations

Run `cargo doc --open --package oxidris-evaluator` to view rendered documentation.

## Further Reading

- **Future improvements**: See [Future Projects](../../future-projects.md) for potential enhancements
