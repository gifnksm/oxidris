# Evaluator System

## Overview

The evaluator system is the core of the Oxidris AI. It assigns scores to board states after piece placements, guiding the AI to choose placements that optimize game outcomes.

## Current Architecture

```
Board State + Piece Placement
    ↓
Feature Extraction (15 features)
    ↓
Percentile-Based Normalization (0-1 scale)
    ↓
Weighted Sum (learned weights)
    ↓
Placement Score
```

### Components

1. **Feature Extraction**: Measures board properties (holes, height, bumpiness, etc.)
2. **Normalization**: Scales feature values to 0-1 range using P05-P95 percentiles
3. **Weighting**: Linear combination using weights learned by genetic algorithm
4. **Fitness Function**: Defines optimization objective (survival, score, or both)

## Available Models

Trained models optimized for different objectives:

- **Aggro** (`models/ai/aggro.json`): Aggressive play with line clear efficiency
- **Defensive** (`models/ai/defensive.json`): Conservative play prioritizing survival

Both use the same 15 features but different fitness functions during training.

## Feature Categories

### Survival Features
Directly affect game termination: holes, height, well depth

### Structure Features
Affect placement quality: bumpiness, transitions, roughness

### Score Features
Directly contribute to score: line clears, I-well setup

## Training Process

1. **Data Generation**: Weak AIs (Random, HeightOnly, etc.) generate diverse board states
2. **Weight Optimization**: Genetic algorithm searches for optimal feature weights
3. **Fitness Evaluation**: Play sessions and score based on chosen fitness function
4. **Model Export**: Save best weights to `models/ai/`

## Current Status and Improvement Plans

See [Current Status](./current-status.md) for:
- Detailed implementation
- Known issues (linear normalization, feature redundancy, GA tuning)
- Improvement projects (KM feature transform, feature selection, GA tuning)

## Active Projects

### KM-Based Feature Transform (Phase 3 - In Progress)

Replacing percentile-based normalization with survival-time-based transformations using Kaplan-Meier analysis.

**Why**: Current linear normalization (`holes=0→1` treated same as `holes=10→11`) doesn't capture non-linear relationships with actual survival outcomes.

**Approach**: Transform feature values through survival time → normalize to 0-1

See [km-feature-transform/](./km-feature-transform/) for details.

## Documentation

- **[Current Status](./current-status.md)** - Implementation details, issues, improvement plans
- **[km-feature-transform/](./km-feature-transform/)** - KM-based feature transformation project

## Code Locations

- **Features**: `crates/oxidris-ai/src/board_feature/mod.rs`
- **Genetic Algorithm**: `crates/oxidris-ai/src/genetic.rs`
- **Session Evaluators**: `crates/oxidris-ai/src/session_evaluator.rs`
- **Training**: `crates/oxidris-cli/src/train_ai.rs`
- **Models**: `models/ai/`
