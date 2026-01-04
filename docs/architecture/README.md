# Architecture Documentation

This directory contains design documentation for the three core systems of Oxidris: the Evaluator (board state evaluation), Training (weight optimization), and Engine (game mechanics).

- **Document type**: Explanation
- **Purpose**: Explain the architecture of Oxidris's three core systems and guide readers to detailed documentation
- **Audience**: AI assistants, developers
- **When to read**: When you need to understand system architecture or check interactions between systems
- **Prerequisites**: Basic project understanding (see AGENTS.md)
- **Related documents**: [AGENTS.md](../../AGENTS.md) (overall project context), [Evaluator](./evaluator/README.md), [Training](./training/README.md), [Engine](./engine/README.md)

## Overview

Oxidris is built around three core systems:

1. **Evaluator** - Board state evaluation using statistical features
2. **Training** - Weight optimization using genetic algorithms
3. **Engine** - Tetris game mechanics and rules

Each system is documented in its own subdirectory.

## System Documentation

### [Evaluator](./evaluator/README.md)

The evaluator system assigns scores to board states to guide AI placement decisions.

- Feature extraction (holes, height, bumpiness, etc.)
- Feature normalization (P05-P95 percentile scaling)
- Weighted evaluation (linear combination)

### [Training](./training/README.md)

The training system optimizes feature weights using genetic algorithms.

- Genetic algorithm (GA) parameters and operators
- Fitness functions (AggroSessionEvaluator, DefensiveSessionEvaluator)
- Training data generation
- Model export

### [Engine](./engine/README.md)

The game engine implements Tetris mechanics for AI training.

- Simplified SRS rotation system (4-direction kicks)
- Standard 7-bag piece generation
- Hold system
- **Note:** Uses simplified rotation, not full SRS

## Data Flow

### Training Flow

```text
Weak AI Gameplay
    ↓
Training Data (board states)
    ↓
Genetic Algorithm
    ↓
Feature Weights (models/ai/*.json)
```

### Evaluation Flow

```text
Game State (Engine)
    ↓
Feature Extraction (Evaluator)
    ↓
Normalization (Evaluator)
    ↓
Weighted Sum (Evaluator)
    ↓
Placement Score
```

## Key Design Decisions

### Data-Driven Approach

- Feature normalization uses percentiles from actual gameplay data
- No hand-tuned parameters in evaluation

### Separation of Concerns

- **Engine**: Game mechanics only, no AI logic
- **Evaluator**: Board evaluation only, no training logic
- **Training**: Weight optimization only, no game mechanics

### Linear Evaluation Model

- Simple weighted sum of normalized features
- Easy to interpret and debug
- Weights learned by genetic algorithm
- **Limitation:** Cannot capture feature interactions (non-linear relationships)

See [Future Projects](../future-projects.md) for potential improvements that address current limitations.

## External References

- [Kaplan-Meier Estimator](https://en.wikipedia.org/wiki/Kaplan%E2%80%93Meier_estimator)
- [Genetic Algorithms](https://en.wikipedia.org/wiki/Genetic_algorithm)
- [Tetris SRS](https://tetris.wiki/Super_Rotation_System) (note: we use simplified version)
