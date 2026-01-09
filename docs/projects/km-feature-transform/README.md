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

**Last Updated:** 2026-01-08

### ✅ Completed (Phase 1-4)

#### Phase 1-2: Data & Analysis

- Data generation with diverse evaluators
- KM survival analysis implementation
- Normalization parameter generation
- Data structures for KM normalization
- Two-stage design (transform → normalize)
- CLI tools and documentation

#### Phase 3: Infrastructure (2026-01-06)

- `TableTransform<S>` type design
  - Separate type from `RawTransform<S>` with table-based mapping
  - Clipping logic for out-of-range values
- `FeatureProcessing` integration design
  - `TableTransform` variant for serialization
- Feature naming convention
  - Linear: `*_raw_penalty`, `*_raw_risk`
  - Mapped (KM): `*_table_km`
  - Feature set coexistence strategy
- Model selection mechanism
  - Model name determines feature set (`aggro_linear` vs `aggro_km`)
- Follows `FeatureBuilder` pattern (dynamic runtime construction)

#### Phase 4: Implementation (2026-01-09) - ✅ Implementation Complete

- ✅ Implemented `TableTransform<S>` type
  - Type with `Vec<f32>` lookup table (P05-P95 range)
  - `transform()` with clamping logic for out-of-range values
  - `BoardFeature` trait implementation with proper normalization
  - Edge case handling (zero division prevention)
- ✅ Added `TableTransform` variant to `FeatureProcessing` enum
  - Proper serialization/deserialization support
- ✅ Extended `FeatureBuilder` for table-based features
  - `build_all_features()` constructs both raw and table features
  - `build_raw_features()` constructs only raw features
  - Feature set selection via `FeatureSet` enum
- ✅ Implemented KM-based survival features
  - `num_holes_table_km`, `sum_of_hole_depth_table_km`
  - `max_height_table_km`, `center_column_max_height_table_km`, `total_height_table_km`
- ✅ Updated CLI tools for feature set selection
  - `train-ai` uses `FeatureSet::Raw` (raw features only)
  - `analyze-board-features` uses `FeatureSet::All` (both raw and table)
- ✅ Implemented survival statistics pipeline
  - `SurvivalStatsMap` for grouping by feature value
  - KM median calculation with linear interpolation for missing values
  - P05-P95 percentile-based table range selection

- ✅ Extended training infrastructure for KM/Raw comparison
  - Added `FeatureSet::Km` to `util.rs` for KM-only feature sets
  - Extended `AiType` enum to support 4 model types:
    - `AggroKm`, `DefensiveKm` (KM-normalized survival features)
    - `AggroRaw`, `DefensiveRaw` (raw-normalized survival features)
  - Added `build_km_features()` to `FeatureBuilder`
    - Survival features use TableTransform (KM)
    - Structure features use RawTransform
    - Score features unchanged
  - Updated Makefile targets for 4 model variants:
    - `train-ai-all`, `train-ai-aggro-km`, `train-ai-defensive-km`
    - `train-ai-aggro-raw`, `train-ai-defensive-raw`
    - `auto-play-aggro-km`, `auto-play-defensive-km`
    - `auto-play-aggro-raw`, `auto-play-defensive-raw`
  - Generated 4 baseline models for comparison:
    - `models/ai/aggro-km.json` (fitness=2.56, trained 2026-01-09)
    - `models/ai/defensive-km.json` (trained 2026-01-09)
    - `models/ai/aggro-raw.json` (fitness=2.51, trained 2026-01-09)
    - `models/ai/defensive-raw.json` (trained 2026-01-09)

### 📋 Phase 5: Validation (CURRENT PHASE - Not Started)

**Status**: Implementation complete (Phase 4), validation NOT started

**Baseline Models Available:**

- `aggro-km.json` (fitness=2.56, trained 2026-01-09)
- `defensive-km.json` (trained 2026-01-09)
- `aggro-raw.json` (fitness=2.51, trained 2026-01-09)
- `defensive-raw.json` (trained 2026-01-09)

**Validation Tasks:**

1. **Model Performance Comparison**
   - Run multiple games (≥100) for each of 4 models
   - Measure survival time (mean, median, P25/P75)
   - Compare KM-based vs. Raw-based models
   - Expected: KM-based ≥ Raw-based survival

2. **Feature Correlation Analysis**
   - Measure correlation between features and survival time
   - Verify non-linear transformation captures relationships
   - Expected: |r| > 0.5 for survival features

3. **Weight Interpretability Analysis**
   - Compare learned weights between KM and Raw models
   - Verify weights correlate with survival impact
   - Check if KM features have more stable weights

4. **Training Convergence Analysis**
   - Compare training convergence speed (KM vs. Raw)
   - Analyze fitness progression over generations
   - Check for overfitting or instability

5. **Documentation and Decision**
   - Document validation results and findings
   - Decide on feature set for production use
   - Create recommendations for future work

**Success Criteria:**

- KM-based evaluator achieves ≥ raw-based evaluator survival time
- Survival features show strong correlation with survival (|r| > 0.5)
- Learned weights are interpretable (correlate with feature survival ranges)
- KM transform demonstrates measurable improvement over raw transform

## Recent Changes

### 2026-01-09: Phase 4 Complete - Implementation Done, Validation Pending

**Implementation Complete:**

- Extended training infrastructure to support both KM and Raw feature sets
- Added `FeatureSet::Km` variant for KM-only features
- Extended `AiType` to 4 variants (`AggroKm`, `DefensiveKm`, `AggroRaw`, `DefensiveRaw`)
- Added `build_km_features()` method to `FeatureBuilder`
- Updated Makefile with 8 new targets (train/auto-play × 4 models)
- Generated 4 baseline models for comparison

**Baseline Model Status:**

- ✅ `aggro-km.json`: fitness=2.56 (uses TableTransform for survival features)
- ✅ `defensive-km.json`: trained (uses TableTransform for survival features)
- ✅ `aggro-raw.json`: fitness=2.51 (uses RawTransform for survival features)
- ✅ `defensive-raw.json`: trained (uses RawTransform for survival features)

**Next:** Phase 5 validation (see "Phase 5: Validation" section above)

### 2026-01-08: Phase 4 Implementation Progress

Completed implementation of `TableTransform<S>` and KM-based survival features:

- **Implementation Details**:
  - `TableTransform<S>` uses `Vec<f32>` for lookup table (covering P05-P95 range)
  - Clamping logic handles out-of-range values (clamp to table boundaries)
  - Linear interpolation fills missing KM median values in survival statistics
  - Zero-division prevention in normalization (returns 0.5 when range is zero)

- **Feature Set Management**:
  - `FeatureSet` enum controls which features to build (`All` or `Raw`)
  - Training uses raw features only (`FeatureSet::Raw`)
  - Analysis uses both feature sets (`FeatureSet::All`)

- **CLI Integration**:
  - `build_feature_from_session()` performs complete pipeline:
    - Raw value extraction
    - Raw statistics computation
    - Survival statistics computation (KM analysis)
    - Normalization parameter generation
    - Feature construction

### 2026-01-08: Feature Naming Convention

Renamed types and features to better reflect their transformation approach:

- **Types**:
  - `LinearNormalized<S>` → `RawTransform<S>` (emphasizes minimal transformation of raw values)
  - `MappedNormalized<S>` → `TableTransform<S>` (describes lookup table-based transformation)

- **Enum variants**:
  - `FeatureProcessing::LinearNormalized` → `FeatureProcessing::RawTransform`

- **Feature IDs**:
  - `*_linear_penalty` → `*_raw_penalty` (e.g., `num_holes_raw_penalty`)
  - `*_linear_risk` → `*_raw_risk` (e.g., `max_height_raw_risk`)
  - `*_km_penalty` → `*_table_km` (implemented, e.g., `num_holes_table_km`)

**Rationale**: "Raw" better describes the minimal `raw as f32` transformation, while "table" explicitly indicates lookup-based transformation. This naming makes the distinction between transformation methods clearer.

### 2026-01-06: Evaluator Architecture Refactoring

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

The normalization parameters are embedded in the feature builder at runtime:

```rust
// Automatically computed from session data
let norm_params = BoardFeatureNormalizationParamCollection::from_stats(
    &sources,
    &raw_stats,
    &survival_stats,
);

// Used to construct features
let builder = FeatureBuilder::new(norm_params);
let features = builder.build_all_features()?;
```

Internal structure of normalization parameters:

```rust
pub struct BoardFeatureNormalizationParam {
    pub value_percentiles: ValuePercentiles,  // P05, P95, etc.
    pub survival_table: SurvivalTable,
}

pub struct SurvivalTable {
    pub feature_min_value: u32,               // P05 feature value
    pub median_survival_turns: Vec<f32>,      // KM medians for [P05, P95]
    pub normalize_min: f32,                    // Min survival time
    pub normalize_max: f32,                    // Max survival time
}
```

### Train AI Models

```bash
# Train with KM features (uses TableTransform for survival features)
make train-ai-aggro-km
make train-ai-defensive-km

# Train with raw features (uses RawTransform for survival features)
make train-ai-aggro-raw
make train-ai-defensive-raw

# Train all 4 models
make train-ai-all
```

Training uses different feature sets:

- KM models (`*-km`): Use `FeatureSet::Km` (TableTransform for survival, RawTransform for structure)
- Raw models (`*-raw`): Use `FeatureSet::Raw` (RawTransform for all features)

### Auto-Play with Trained Models

```bash
# Play with KM-based models
make auto-play-aggro-km
make auto-play-defensive-km

# Play with raw-based models
make auto-play-aggro-raw
make auto-play-defensive-raw
```

### Analyze Features

```bash
# View both raw and table-based features side-by-side
cargo run --release -- analyze-board-features data/boards.json
```

The analysis tool automatically:

1. Loads session data
2. Computes raw statistics and survival statistics
3. Builds both raw and table-based features (`FeatureSet::All`)
4. Launches interactive TUI for exploration

**Feature Set Coexistence:**

- Raw features (`*_raw_penalty`, `*_raw_risk`) used for training
- Table features (`*_table_km`) available for analysis and comparison
- Both feature sets computed from same session data
- Analysis tools display both for side-by-side comparison

## Next Steps

See [roadmap.md](./roadmap.md) for detailed implementation plan.

### Phase 5: Validation (Current Phase)

**See "Phase 5: Validation" section above for detailed validation tasks and success criteria.**

Key validation tasks:

1. Model performance comparison (KM vs. Raw)
2. Feature correlation analysis
3. Weight interpretability analysis
4. Training convergence analysis
5. Documentation and decision

## Documentation

- **[design.md](./design.md)** - Target architecture for KM-based normalization
- **[roadmap.md](./roadmap.md)** - Implementation phases and status

## Code Locations

### Analysis System

- **Session data**: `crates/oxidris-analysis/src/session.rs`
- **Feature samples**: `crates/oxidris-analysis/src/sample.rs`
- **Statistics**: `crates/oxidris-analysis/src/statistics.rs`
- **Normalization**: `crates/oxidris-analysis/src/normalization.rs`
- **Feature builder**: `crates/oxidris-analysis/src/feature_builder.rs`
- **Survival analysis**: `crates/oxidris-analysis/src/survival.rs`

### Supporting Systems

- **KM Estimator**: `crates/oxidris-stats/src/survival.rs`
- **Normalization Generation Tool**: `crates/oxidris-cli/src/command/analyze_censoring/mod.rs`
- **Feature Definitions**: `crates/oxidris-evaluator/src/board_feature/mod.rs`
