# KM-Based Feature Transform Project

This project applies survival-time-based transformations using Kaplan-Meier analysis to survival features (holes, height).

- **Document type**: Explanation
- **Purpose**: Project overview, design rationale, and implementation status for KM-based feature normalization
- **Audience**: AI assistants, human contributors working on evaluator features
- **When to read**: When working on feature normalization, survival analysis, or understanding the KM-based approach
- **Prerequisites**: [Evaluator Documentation](../../architecture/evaluator/README.md) for feature system context
- **Related documents**: [design.md](./design.md) (detailed architecture), [roadmap.md](./roadmap.md) (implementation phases)

## Overview

This project applies **survival-time-based transformations** using Kaplan-Meier (KM) survival analysis to **survival features** (holes, height). The goal is to replace linear normalization with non-linear, data-driven transformations that capture the true relationship between feature values and survival time.

**Scope**: This project focuses on survival features only. Structure features (bumpiness, transitions) and score optimization are out of scope and may be addressed in future projects.

## The Core Idea

### Current Approach (Percentile-Based Linear Normalization)

Feature values are linearly transformed then normalized to 0-1 using auto-generated P05-P95 percentiles:

```text
holes=0  → 0.0  → normalized = 1.0 (best, at P05)
holes=5  → 5.0  → normalized = 0.375
holes=8  → 8.0  → normalized = 0.0 (worst, at P95)
holes=20 → 20.0 → normalized = 0.0 (clamped)
```

> [!NOTE]
> The P05 and P95 values (0 and 8 in this example) are automatically computed from gameplay data, not manually defined.

**Problem**: Linear transformation (`raw as f32`) doesn't reflect actual impact on survival.

- Treats `holes=0→1` the same as `holes=7→8` in the transformation step
- The relationship between holes/height and survival time is non-linear, but transformation is linear

### New Approach (KM-Based Transform)

Transform feature values through **survival time** before normalizing:

```text
holes=0  → survival=322.8 turns → normalized = 1.0 (best)
holes=5  → survival=120.0 turns → normalized = 0.36
holes=10 → survival=50.0 turns  → normalized = 0.14
holes=20 → survival=8.2 turns   → normalized = 0.0 (worst)
```

**Benefit**: Non-linear transformation captures actual relationship between feature values and game outcomes.

## Why Kaplan-Meier Analysis?

### The Right-Censoring Problem

Games that survive beyond `MAX_TURNS` are "censored" - we don't know their true survival time:

```text
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
   - Current: `linear_max_height_penalty` and `linear_top_out_risk` both needed (different normalization ranges)
   - With KM: Single feature with non-linear transform captures both behaviors

## Project Scope

### In Scope: Survival Features

Features that directly affect game termination:

- `num_holes` - holes prevent piece placement
- `sum_of_hole_depth` - deeper holes are harder to clear
- `max_height` - height determines remaining space
- `center_column_max_height` - center height affects placement options
- `total_height` - overall board pressure

These features have clear, direct impact on survival time, making KM-based normalization appropriate.

### Out of Scope

**Structure Features** (bumpiness, transitions, well depth):

- Indirect impact on survival (via placement flexibility)
- May require different normalization approach (e.g., placement-flexibility-based)
- Should be evaluated in a separate project after survival features succeed

**Score Features** (line clears, tetris setup):

- Different optimization objective (maximize score, not survival)
- May require score-based normalization or multi-objective optimization
- Future work after survival optimization is validated

## Current Status

**Last Updated:** 2026-01-06

### ✅ Completed (Phase 1-3)

#### Phase 1-2: Data & Analysis

- Data generation with diverse evaluators
- KM survival analysis implementation
- Normalization parameter generation
- Data structures for KM normalization
- Two-stage design (transform → normalize)
- CLI tools and documentation

#### Phase 3: Infrastructure (2026-01-06) - Design Complete

- `MappedNormalized<S>` type design
  - Separate type from `LinearNormalized<S>` with `BTreeMap<u32, f32>` mapping
  - Clipping logic for out-of-range values (clip to min/max key)
- `FeatureProcessing` integration design
  - `MappedNormalized` variant for serialization
- Feature naming convention
  - Linear: `*_linear_penalty`, `*_linear_risk`
  - Mapped (KM): `*_km_penalty`
  - Feature set coexistence strategy
- Model selection mechanism
  - Model name determines feature set (`aggro_linear` vs `aggro_km`)
- Follows `FeatureBuilder` pattern (dynamic runtime construction)

### 📋 Not Started (Phase 4)

- Implement `MappedNormalized<S>` type
  - Type with `BTreeMap<u32, f32>` mapping field
  - `transform()` with clipping logic
  - `BoardFeature` trait implementation
- Add `MappedNormalized` variant to `FeatureProcessing` enum
- Extend `FeatureBuilder` for mapped features
  - Construct both linear and mapped feature sets
  - Feature set selection logic
- Implement KM-based survival features
  - `num_holes_km_penalty`, `sum_of_hole_depth_km_penalty`
  - `max_height_km_penalty`, `center_column_max_height_km_penalty`, `total_height_km_penalty`
- Update training tools for model name-based feature set selection
- Update `analyze-board-features` to display both feature sets
- Train and benchmark KM-based evaluator
- Validate improvements over linear normalization

## Recent Changes (2026-01-06)

The evaluator system underwent major refactoring:

- **Static → Dynamic**: Removed static feature constants, moved to runtime construction via `FeatureBuilder`
- **Instance-based traits**: `BoardFeature` now uses instance methods for `transform()` and `normalize()`
- **Data-driven normalization**: All normalization parameters computed from session data at runtime

The KM feature design follows this new architecture pattern.

## Usage

### Generate Normalization Parameters

```bash
# Collect gameplay data
cargo run --release -- generate-boards --output data/boards.json

# Generate KM-based normalization parameters
cargo run --release -- analyze-censoring data/boards.json \
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
    "linear_holes_penalty": {
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

### Train AI Models (Planned)

```bash
# Train with linear features (existing approach)
cargo run --release -- train-ai \
    --model-name aggro_linear \
    --sessions data/boards.json

# Train with KM-based features (new approach)
cargo run --release -- train-ai \
    --model-name aggro_km \
    --sessions data/boards.json \
    --normalization-params data/normalization_params.json
```

### Analyze Features (Planned)

```bash
# View both linear and KM features side-by-side
cargo run --release -- analyze-board-features \
    --sessions data/boards.json \
    --normalization-params data/normalization_params.json \
    --show-both-feature-sets
```

**Feature Set Coexistence:**

- Linear features (`*_linear_penalty`, `*_linear_risk`) remain available for backward compatibility
- KM features (`*_km_penalty`) are added as a new feature set
- Model name determines which feature set is used during training
- Analysis tools can display both for comparison

## Next Steps

See [roadmap.md](./roadmap.md) for detailed implementation plan.

### Phase 4: Implement Survival Features

1. Implement `MappedNormalized<S>` type with clipping logic
2. Add `MappedNormalized` variant to `FeatureProcessing` enum
3. Extend `FeatureBuilder` to construct mapped features
4. Implement KM-based survival features (`*_km_penalty`)
5. Update training tools for model name-based feature set selection
6. Update `analyze-board-features` to display both linear and KM features
7. Train KM-based evaluator and compare with linear baseline
8. Validate KM transform improvement over linear transform

### Success Criteria

- Survival-based evaluator achieves ≥ current heuristic evaluator survival time
- Survival features show strong correlation with survival (|r| > 0.5)
- Learned weights are interpretable (correlate with feature km_range)
- KM transform demonstrates clear improvement over linear transform

## Documentation

- **[design.md](./design.md)** - Target architecture for KM-based normalization
- **[roadmap.md](./roadmap.md)** - Implementation phases and status

## Code Locations

- **KM Estimator**: `crates/oxidris-stats/src/survival.rs`
- **Data Structures**: `crates/oxidris-cli/src/data.rs`
- **Normalization Generation**: `crates/oxidris-cli/src/analyze_censoring.rs`
- **Feature Definitions**: `crates/oxidris-evaluator/src/board_feature/mod.rs`
