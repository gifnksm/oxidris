# Normalization Parameter Generation

## Overview

This document describes the survival-based normalization parameter generation system for board features in the Tetris AI project.

## Background

Traditional normalization approaches (min-max, z-score) fail to account for **right-censoring** in board survival data:

- Games that survive longer than `MAX_TURNS` are censored (we don't know their true survival time)
- Censoring predominantly affects "good" board states (long survival)
- Naive statistics (mean, percentiles) are biased when censoring is ignored

**Solution:** Use Kaplan-Meier survival analysis to estimate survival curves, then generate normalization parameters based on robust percentile ranges.

## Normalization Method: P05-P95 Robust KM

The implemented method is **P05-P95 robust normalization** based on Kaplan-Meier median survival times:

### Algorithm

1. **Calculate KM median** for each unique feature value
2. **Find P05 and P95 feature values** based on board count distribution
3. **Use their KM medians as normalization bounds**:
   - `km_max = KM median of P05 feature value` (good state)
   - `km_min = KM median of P95 feature value` (bad state)
4. **Normalize all feature values**:
   ```
   normalized = (km_median - km_min) / (km_max - km_min)
   normalized = clamp(normalized, 0.0, 1.0)
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

Step 2: Get their KM medians
  km_max = km(0)  = 322.8
  km_min = km(33) = 7.1
  km_range = 315.7

Step 3: Normalize all values
  norm(0)  = (322.8 - 7.1) / 315.7 = 1.00
  norm(1)  = (276.1 - 7.1) / 315.7 = 0.85
  norm(33) = (7.1 - 7.1) / 315.7   = 0.00
  norm(50) = (0.5 - 7.1) / 315.7   = -0.02 → 0.00 (clamped)
  norm(115)= (0.0 - 7.1) / 315.7   = -0.02 → 0.00 (clamped)
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
      "mapping": {
        "0": 1.0,
        "1": 0.85,
        "2": 0.72,
        ...
        "33": 0.0,
        "50": 0.0
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

- **`mapping`**: Direct lookup table: feature_value → normalized_value (0.0-1.0).
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

## Integration

### Loading in Evaluators

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
    mapping: BTreeMap<u32, f64>,
    stats: NormalizationStats,
}

#[derive(Deserialize)]
struct NormalizationStats {
    p05_feature_value: u32,
    p95_feature_value: u32,
    p05_km_median: f64,
    p95_km_median: f64,
    total_unique_values: usize,
}

impl NormalizationStats {
    fn km_range(&self) -> f64 {
        self.p05_km_median - self.p95_km_median
    }
}

// Usage:
let params: NormalizationParams = serde_json::from_reader(file)?;
let holes = board.count_holes();
let normalized = params.features["holes_penalty"]
    .mapping
    .get(&holes)
    .copied()
    .unwrap_or(0.0); // Unseen values default to worst (0.0)
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

**Solution:** Save `km_range` in stats, use it to initialize weights.

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

## Validation Checklist

Before using normalization parameters:

1. ✅ Check `max_turns` matches current generation settings
2. ✅ Verify all required features are present in the JSON
3. ✅ Handle missing feature values:
   - Option A: Default to 0.0 (worst case)
   - Option B: Interpolate from nearby values
   - Option C: Use P95 value as fallback
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

## References

- [Kaplan-Meier Survival Analysis](https://en.wikipedia.org/wiki/Kaplan%E2%80%93Meier_estimator)
- Feature definitions: `crates/oxidris-ai/src/board_feature/mod.rs`
- Implementation: `crates/oxidris-cli/src/analyze_censoring.rs`
- Data structures: `crates/oxidris-cli/src/data.rs`
