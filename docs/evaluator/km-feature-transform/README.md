# KM-Based Feature Transform Project

## Overview

This project replaces the current linear feature normalization with **survival-time-based transformations** using Kaplan-Meier (KM) survival analysis.

## The Core Idea

### Current Approach (Percentile-Based Linear Normalization)

Feature values are linearly transformed then normalized to 0-1 using auto-generated P05-P95 percentiles:

```
holes=0  → 0.0  → normalized = 1.0 (best, at P05)
holes=5  → 5.0  → normalized = 0.375
holes=8  → 8.0  → normalized = 0.0 (worst, at P95)
holes=20 → 20.0 → normalized = 0.0 (clamped)
```

**Note**: The P05 and P95 values (0 and 8 in this example) are automatically computed from gameplay data, not manually defined.

**Problem**: Linear transformation (`raw as f32`) doesn't reflect actual impact on survival.
- Treats `holes=0→1` the same as `holes=7→8` in the transformation step
- The relationship between feature values and survival time is non-linear, but transformation is linear

### New Approach (KM-Based Transform)

Transform feature values through **survival time** before normalizing:

```
holes=0  → survival=322.8 turns → normalized = 1.0 (best)
holes=5  → survival=120.0 turns → normalized = 0.36
holes=10 → survival=50.0 turns  → normalized = 0.14
holes=20 → survival=8.2 turns   → normalized = 0.0 (worst)
```

**Benefit**: Non-linear transformation captures actual relationship between feature values and game outcomes.

## Why Kaplan-Meier Analysis?

### The Right-Censoring Problem

Games that survive beyond `MAX_TURNS` are "censored" - we don't know their true survival time:

```
Game A: holes=0, survived 500 turns → CENSORED at 500
Game B: holes=10, died at 50 turns  → OBSERVED at 50
```

**Issue**: Naive statistics (mean, median) underestimate survival for good states.
- Good boards are more likely to be censored
- Bias can be up to 56% for some features

**Solution**: Kaplan-Meier estimator properly handles censored data to produce unbiased survival estimates.

## Two-Stage Pipeline

### Stage 1: Transform (Feature Value → Survival Time)

For each unique feature value, compute KM median survival time from gameplay data:

```rust
// From data: holes=3 appears in 5000 boards
// - 3200 boards were censored (survived to MAX_TURNS)
// - 1800 boards died (observed death times)
// KM analysis → median survival = 177.5 turns

transform_mapping[3] = 177.5
```

### Stage 2: Normalize (Survival Time → 0-1)

Use P05-P95 percentiles of feature values to determine survival time range:

```rust
// P05 feature value (best 5%): holes=0 → KM median = 322.8
// P95 feature value (worst 5%): holes=33 → KM median = 50.0

km_min = 50.0   // worst survival
km_max = 322.8  // best survival

normalized = (survival_time - km_min) / (km_max - km_min)
normalized = normalized.clamp(0.0, 1.0)
```

### At Runtime

```rust
let raw = feature.extract_raw(analysis);           // holes=3
let survival = transform_mapping[raw];              // 177.5 turns
let normalized = (survival - km_min) / (km_max - km_min);  // 0.54
```

## Benefits

1. **Non-linear**: Captures actual impact on outcomes
   - Large changes in bad region: `holes=10→11` has big impact
   - Small changes in good region: `holes=0→1` has small impact

2. **Data-driven**: Based on actual gameplay data, not intuition
   - No manual tuning of ranges
   - Automatically adapts to feature characteristics

3. **Interpretable**: Intermediate values have clear meaning
   - `survival_time = 177.5` means "expected to survive 177.5 more turns"
   - Easy to understand and debug

4. **Robust**: P05-P95 scaling handles outliers
   - Rare extreme values (e.g., 115 holes) don't distort scale
   - Focuses on common gameplay scenarios

5. **Eliminates duplicates**: Non-linear transform removes need for `*_risk` features
   - Current: `max_height_penalty` and `top_out_risk` both needed
   - With KM: Single feature with non-linear transform captures both

## Current Status

### ✅ Completed

- Data generation with diverse evaluators
- KM survival analysis implementation (`crates/oxidris-stats/src/survival.rs`)
- Normalization parameter generation (`crates/oxidris-cli/src/analyze_censoring.rs`)
- Data structures (`NormalizationParams`, `FeatureNormalization`, `NormalizationRange`)
- Two-stage design (`transform_mapping` + `normalization`)
- CLI tool with `--normalization-output` flag
- Documentation

### 🔄 In Progress

- Remove duplicate `*_risk` features
- Implement `KMBasedEvaluator` using normalization parameters
- Integrate with genetic algorithm training

### 📋 Not Started

- Training with KM-based evaluator
- Benchmarking vs percentile-based approach
- Weight interpretation analysis

## Usage

### Generate Normalization Parameters

```bash
# Collect gameplay data
cargo run --release -- generate-boards --output data/boards.json

# Generate KM-based normalization parameters
cargo run --release -- analyze-censoring data/boards.json \
    --kaplan-meier \
    --normalization-output data/normalization_params.json
```

### Output Format

```json
{
  "generator": {
    "max_turns": 500,
    "generated_at": "2024-01-04T12:00:00Z"
  },
  "features": {
    "holes_penalty": {
      "transform_mapping": {
        "0": 322.8,
        "1": 304.5,
        "3": 177.5,
        "10": 50.0
      },
      "normalization": {
        "km_min": 50.0,
        "km_max": 322.8
      },
      "stats": {
        "p05_feature_value": 0,
        "p95_feature_value": 33,
        "total_unique_values": 116
      }
    }
  }
}
```

### Load and Use (Planned)

```rust
let params = NormalizationParams::load("data/normalization_params.json")?;
let evaluator = KMBasedEvaluator::new(params);
let score = evaluator.evaluate(&board, piece);
```

## Next Steps

1. **Remove duplicate features** from `ALL_BOARD_FEATURES`
   - Keep: `max_height_penalty`, `center_columns_penalty`, `well_depth_penalty`
   - Remove: `top_out_risk`, `center_top_out_risk`, `deep_well_risk`

2. **Implement KMBasedEvaluator**
   - Load normalization parameters
   - Apply two-stage transform+normalize
   - Initialize weights: `weight = 1.0 / km_range`

3. **Train and benchmark**
   - Train with genetic algorithm
   - Compare fitness vs percentile-based
   - Analyze learned weights

4. **Validate improvements**
   - Measure survival time improvement
   - Check weight interpretability
   - Verify non-linear effects captured

## Documentation

- **[design.md](./design.md)** - Detailed algorithm specification and data structures
- **[roadmap.md](./roadmap.md)** - Phase 1-6 development plan with tasks and timeline

## Code Locations

- **KM Estimator**: `crates/oxidris-stats/src/survival.rs`
- **Data Structures**: `crates/oxidris-cli/src/data.rs`
- **Normalization Generation**: `crates/oxidris-cli/src/analyze_censoring.rs`
- **Feature Definitions**: `crates/oxidris-ai/src/board_feature/mod.rs`
