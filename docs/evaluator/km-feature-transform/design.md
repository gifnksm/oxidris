# Feature Normalization

> **Note:** This document is part of the [Evaluator Design](./evaluator_design.md) documentation.

## Overview

This document describes the survival-based normalization parameter generation system for board features. Normalization is a core component of the evaluator design, providing non-linear, data-driven transformations for feature values.

## Background

Traditional normalization approaches (min-max, z-score) fail to account for **right-censoring** in board survival data:

- Games that survive longer than `MAX_TURNS` are censored (we don't know their true survival time)
- Censoring predominantly affects "good" board states (long survival)
- Naive statistics (mean, percentiles) are biased when censoring is ignored

**Solution:** Use Kaplan-Meier survival analysis to estimate survival curves, then generate normalization parameters based on robust percentile ranges.

## Normalization Method: P05-P95 Robust KM

The implemented method is **P05-P95 robust normalization** with **2-stage transform and normalize**:

### Algorithm

1. **Calculate KM median** for each unique feature value
2. **Find P05 and P95 feature values** based on board count distribution
3. **Generate transform mapping**: raw value → KM median (survival time in turns)
4. **Store normalization range**:
   - `km_max = KM median of P05 feature value` (good state)
   - `km_min = KM median of P95 feature value` (bad state)

### Two-Stage Evaluation

When evaluating a board:

**Stage 1: Transform** (raw value → KM median)
```rust
let raw = feature.extract_raw(analysis);  // e.g., holes=3
let km_median = transform_mapping[raw];    // → 177.5 turns
```

**Stage 2: Normalize** (KM median → 0-1)
```rust
let normalized = (km_median - km_min) / (km_max - km_min);  // → 0.54
let normalized = normalized.clamp(0.0, 1.0);
```

### Why This Method?

#### ✅ Proportional to Survival Time

```
Feature: holes_penalty
  0 holes:  KM=322.8 → norm=1.00
  1 hole:   KM=276.1 → norm=0.85
  5 holes:  KM=120.0 → norm=0.36

Ratio preserved:
  km(1) / km(0) = 0.855
  norm(1) / norm(0) = 0.85
```

The normalized value is proportional to actual survival time.

#### ✅ Robust to Outliers

```
P05 = 0 holes  (bottom 5% of boards)
P95 = 33 holes (top 95% of boards)

Rare extreme values (e.g., 115 holes) don't affect the scale.
They're simply clamped to 0.0.
```

#### ✅ Frequency-Aware

P05/P95 are calculated based on board count, so common feature values determine the scale, not rare outliers.

#### ✅ Comparable Across Features

All features are normalized to 0-1 range, but the `km_range` statistic allows comparing their actual impact:

```
holes_penalty:     km_range = 315.7 turns
max_height_penalty: km_range = 327.2 turns
bumpiness_penalty:  km_range = 51.2 turns

→ holes and max_height have ~6x more impact than bumpiness
```

### Example

```
Feature: holes_penalty
Data:
  value=0:  KM=322.8, boards=16,186
  value=1:  KM=276.1, boards=11,213
  ...
  value=33: KM=7.1,   boards=500
  value=50: KM=0.5,   boards=10
  value=115:KM=0.0,   boards=1

Step 1: Find P05/P95 by board count
  Total boards = 100,000
  P05 (5,000th board)  → value=0
  P95 (95,000th board) → value=33

Step 2: Build transform mapping (raw → KM median)
  transform_mapping[0]   = 322.8 turns
  transform_mapping[1]   = 276.1 turns
  transform_mapping[3]   = 177.5 turns
  transform_mapping[33]  = 7.1 turns
  transform_mapping[50]  = 0.5 turns
  transform_mapping[115] = 0.0 turns

Step 3: Store normalization range
  km_max = 322.8  (P05's KM median)
  km_min = 7.1    (P95's KM median)

Step 4: At evaluation time (2-stage)
  holes=0:  transform → 322.8, normalize → (322.8-7.1)/315.7 = 1.00
  holes=3:  transform → 177.5, normalize → (177.5-7.1)/315.7 = 0.54
  holes=33: transform → 7.1,   normalize → (7.1-7.1)/315.7   = 0.00
  holes=50: transform → 0.5,   normalize → (0.5-7.1)/315.7   = -0.02 → 0.00 (clamp)
```

## Usage

### Generate Normalization Parameters

```bash
# Default features (holes_penalty, max_height_penalty, hole_depth_penalty)
make generate-normalization

# Or manually:
cargo run --release -- analyze-censoring data/boards.json \
    --kaplan-meier \
    --normalization-output data/normalization_params.json
```

### Select Specific Features

```bash
cargo run --release -- analyze-censoring data/boards.json \
    --kaplan-meier \
    --normalization-output data/norm_custom.json \
    --features holes_penalty,max_height_penalty,surface_bumpiness_penalty
```

### Available Features

Any feature in `ALL_BOARD_FEATURES` can be used. Common ones include:

- `holes_penalty`
- `hole_depth_penalty`
- `max_height_penalty`
- `surface_bumpiness_penalty`
- `surface_roughness_penalty`
- `well_depth_penalty`
- `row_transitions_penalty`
- `column_transitions_penalty`

## Output Format

```json
{
  "max_turns": 500,
  "normalization_method": "robust_km",
  "features": {
    "holes_penalty": {
      "transform_mapping": {
        "0": 322.8,
        "1": 276.1,
        "2": 235.0,
        "3": 177.5,
        ...
        "33": 7.1,
        "50": 0.5
      },
      "normalization": {
        "km_min": 7.1,
        "km_max": 322.8
      },
      "stats": {
        "p05_feature_value": 0,
        "p95_feature_value": 33,
        "p05_km_median": 322.8,
        "p95_km_median": 7.1,
        "total_unique_values": 112
      }
    }
  }
}
```

### Field Descriptions

#### Top Level

- **`max_turns`**: The MAX_TURNS value used during data generation. Used for validation.
- **`normalization_method`**: Always `"robust_km"` for this implementation.
- **`features`**: Map of feature_id → normalization data.

#### Per-Feature

- **`transform_mapping`**: Transform lookup: feature_value → KM_median (survival time in turns).
- **`normalization`**: Normalization range for stage 2.
  - `km_min`: Minimum KM median (P95's value, worst case)
  - `km_max`: Maximum KM median (P05's value, best case)
- **`stats`**: Metadata about the normalization.

#### Stats Object

- **`p05_feature_value`**: Feature value at 5th percentile (by board count). Represents "good" states.
- **`p95_feature_value`**: Feature value at 95th percentile (by board count). Represents "bad" states.
- **`p05_km_median`**: KM median survival time for P05 value. Used as `km_max` in normalization.
- **`p95_km_median`**: KM median survival time for P95 value. Used as `km_min` in normalization.
- **`total_unique_values`**: Number of unique feature values in the dataset.

### Computing KM Range

The KM range (actual survival time difference) can be computed as:

```rust
let km_range = stats.p05_km_median - stats.p95_km_median;
```

This is useful for initializing feature weights to equalize their impact.

## Integration with Evaluators

For complete evaluator implementation details, see [Evaluator Design](./evaluator_design.md).

### Loading Normalization Parameters

```rust
use std::collections::BTreeMap;
use serde::Deserialize;

#[derive(Deserialize)]
struct NormalizationParams {
    max_turns: usize,
    normalization_method: String,
    features: BTreeMap<String, FeatureNormalization>,
}

#[derive(Deserialize)]
struct FeatureNormalization {
    transform_mapping: BTreeMap<u32, f64>,
    normalization: NormalizationRange,
    stats: NormalizationStats,
}

#[derive(Deserialize)]
struct NormalizationRange {
    km_min: f64,
    km_max: f64,
}

#[derive(Deserialize)]
struct NormalizationStats {
    p05_feature_value: u32,
    p95_feature_value: u32,
    p05_km_median: f64,
    p95_km_median: f64,
    total_unique_values: usize,
}

impl NormalizationRange {
    fn normalize(&self, km_median: f64) -> f64 {
        if self.km_max == self.km_min {
            0.5
        } else {
            ((km_median - self.km_min) / (self.km_max - self.km_min)).clamp(0.0, 1.0)
        }
    }
}

impl FeatureNormalization {
    /// Two-stage: transform then normalize
    fn transform_and_normalize(&self, raw_value: u32) -> f64 {
        // Stage 1: Transform (raw → KM median)
        let km_median = self
            .transform_mapping
            .get(&raw_value)
            .copied()
            .unwrap_or(self.normalization.km_min); // Default to worst case
        
        // Stage 2: Normalize (KM median → 0-1)
        self.normalization.normalize(km_median)
    }
    
    fn km_range(&self) -> f64 {
        self.stats.p05_km_median - self.stats.p95_km_median
    }
}

// Usage:
let params: NormalizationParams = serde_json::from_reader(file)?;
let holes = board.count_holes();
let normalized = params.features["holes_penalty"]
    .transform_and_normalize(holes);
```

### Initializing Feature Weights

To equalize the impact of all features, initialize weights inversely proportional to KM range:

```rust
for (feature_id, feature_data) in &params.features {
    let km_range = feature_data.stats.km_range();
    let initial_weight = 1.0 / km_range;
    weights.insert(feature_id, initial_weight);
}

// Result:
//   w_holes = 1/315.7 = 0.00317
//   w_height = 1/327.2 = 0.00306
//   w_bumpiness = 1/51.2 = 0.0195
//
// Now, a 0.1 change in any normalized feature corresponds to
// roughly the same survival time change (~30 turns).
```

### Interpreting Learned Weights

After training, multiply weights by KM range to get normalized importance:

```rust
let learned_weight = 0.015;  // Trained weight for holes_penalty
let km_range = 315.7;
let importance = learned_weight * km_range;
// = 0.015 * 315.7 = 4.74

// Interpretation: holes_penalty is 4.74x as important as the baseline.
```

## Design Rationale

### Why Not Each Feature Independently (0-1)?

**Problem:** Different features have different survival time ranges.

```
holes:     range = 315 turns → 0.1 norm change = 31.5 turns
bumpiness: range = 51 turns  → 0.1 norm change = 5.1 turns

With equal weights (w=1.0), holes has 6x more impact!
```

**Solution:** The transform_mapping preserves KM median values (in turns), allowing proper weight initialization based on `km_range`.

### Why Not Global Normalization (All Features Same Scale)?

**Problem:** Small-range features lose precision.

```
Global scale = 0 to 500 turns (MAX_TURNS)
  holes:     100-500 → 0.2-1.0 (good precision)
  bumpiness: 10-60   → 0.02-0.12 (poor precision)
```

**Solution:** Normalize per-feature, but provide `km_range` to adjust weights.

### Why P05-P95 Instead of Min-Max?

**Problem:** Outliers distort the scale.

```
Min-Max:
  min = 0.0 turns (115 holes, 1 board)
  max = 322.8 turns (0 holes, 16,186 boards)
  
  → 1 rare board determines the scale for 100,000 boards
```

**Solution:** P05-P95 uses representative values from the bulk of the data.

### Why Store MAX_TURNS?

Normalization parameters are only valid for data generated with the same MAX_TURNS:
- Survival times depend on the censoring point
- Different MAX_TURNS → different KM estimates
- Storing MAX_TURNS enables validation at load time

## Key Design Decisions

### Why Two Stages (Transform + Normalize)?

**Separation of concerns:**
- **Transform**: Converts raw values to meaningful units (survival time in turns)
  - Makes the intermediate value interpretable
  - KM median = "expected remaining survival time"
- **Normalize**: Scales to 0-1 for evaluation function
  - Simple linear scaling
  - Easy to understand and debug

**Benefits:**
- ✅ Intermediate values (KM medians) have clear meaning
- ✅ Can analyze transform independently of normalization
- ✅ Easy to try different normalization strategies later
- ✅ Aligns with the original `transform()` and `normalize()` separation in BoardFeatureSource

### Original Design Intent

The original `BoardFeatureSource` had:
```rust
fn transform(raw: u32) -> f32;   // Was: linear (just cast)
fn normalize(transformed: f32) -> f32;  // Was: P05-P95 linear scaling
```

The KM-based approach replaces the linear transform with a **non-linear, survival-based transform**:
```rust
transform(raw) = km_median(raw)  // Non-linear, data-driven
normalize(km_median) = (km_median - km_min) / (km_max - km_min)
```

This eliminates duplicate features (e.g., `*_penalty` vs `*_risk`) because different normalization ranges are no longer needed—the KM transform captures the non-linear relationship.

## Validation Checklist

Before using normalization parameters:

1. ✅ Check `max_turns` matches current generation settings
2. ✅ Verify all required features are present in the JSON
3. ✅ Handle missing feature values in `transform_mapping`:
   - Default to `km_min` (worst case, normalized to 0.0)
   - Or interpolate from nearby values
4. ✅ Check `normalization_method == "robust_km"`

## Future Enhancements

### Short-term
- [ ] Add interpolation for unseen feature values
- [ ] Export validation statistics (coverage, outlier counts)
- [ ] Visualization: plot KM curves with P05/P95 markers

### Mid-term
- [ ] Support alternative P-values (P10-P90, P01-P99)
- [ ] Multi-feature joint normalization (PCA, consider correlations)
- [ ] Automatic hyperparameter tuning for P-values

### Long-term
- [ ] Non-linear transformations (log, power)
- [ ] Conditional normalization (context-dependent)
- [ ] Online/adaptive normalization

## See Also

- [Evaluator Design](./evaluator_design.md) - Overall evaluator architecture
- [Kaplan-Meier Survival Analysis](https://en.wikipedia.org/wiki/Kaplan%E2%80%93Meier_estimator)
- Feature definitions: `crates/oxidris-ai/src/board_feature/mod.rs`
- Implementation: `crates/oxidris-cli/src/analyze_censoring.rs`
- Data structures: `crates/oxidris-cli/src/data.rs`
