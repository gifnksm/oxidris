# Evaluator System

This document provides an overview of the three-level evaluator architecture and serves as a navigation hub to detailed implementation documentation.

- **Document type**: Explanation
- **Purpose**: Explain the evaluator system architecture and guide readers to detailed implementation documentation
- **Audience**: AI assistants, developers working on evaluation or training
- **When to read**: When understanding the overall evaluator system or looking for implementation details
- **Prerequisites**: [Architecture Overview](../README.md)
- **Related documents**: [Training System](../training/README.md), [Future Projects](../../future-projects.md), [KM Feature Transform Project](../../projects/km-feature-transform/README.md)

> [!NOTE]
> **Recent Changes (2026-01-06):** The evaluator system migrated from static feature constants
> to dynamic runtime construction via `FeatureBuilder`. Features are now constructed with
> normalization parameters computed from session data at runtime.

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
2. **Feature Transformation** - Transform raw values into meaningful representations (currently linear, KM-based in development)
3. **Feature Normalization** - Scale to [0, 1] using data-driven percentiles computed from gameplay sessions
4. **Weighted Evaluation** - Compute weighted sum: `score = Σ(wᵢ × featureᵢ)`

**Feature Construction:** Features are built dynamically at runtime via `FeatureBuilder` (see `oxidris-analysis::feature_builder`), which computes normalization parameters from session statistics.

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

Both models use the same feature set but different weights, resulting in different play styles. Models store feature IDs and weights; normalization parameters are computed at runtime from training data.

## Implementation Documentation

For detailed implementation documentation, design decisions, API usage, and current limitations, see the rustdoc in the following files:

**Evaluator System:**

- **`crates/oxidris-evaluator/src/lib.rs`** - Crate overview, three-level architecture, design principles
- **`crates/oxidris-evaluator/src/board_feature/mod.rs`** - Feature trait architecture, processing pipeline
- **`crates/oxidris-evaluator/src/board_feature/source.rs`** - Feature source definitions and measurements
- **`crates/oxidris-evaluator/src/placement_evaluator.rs`** - Weighted sum evaluation, linear model advantages/limitations
- **`crates/oxidris-evaluator/src/turn_evaluator.rs`** - Turn selection, greedy lookahead strategy
- **`crates/oxidris-evaluator/src/session_evaluator.rs`** - Fitness functions (Aggro/Defensive), design rationale and limitations

**Feature Construction (Analysis System):**

- **`crates/oxidris-analysis/src/feature_builder.rs`** - Runtime feature construction, terminology, design decisions
- **`crates/oxidris-analysis/src/normalization.rs`** - Normalization parameter computation
- **`crates/oxidris-analysis/src/statistics.rs`** - Feature statistics from session data
- **`crates/oxidris-analysis/src/session.rs`** - Session data structures
- **`crates/oxidris-analysis/src/sample.rs`** - Feature sample extraction

Run `cargo doc --open --package oxidris-evaluator` or `cargo doc --open --package oxidris-analysis` to view rendered documentation.

## Further Reading

- **Active project**: [KM Feature Transform](../../projects/km-feature-transform/README.md) - Survival-time-based feature normalization
- **Future improvements**: See [Future Projects](../../future-projects.md) for potential enhancements
