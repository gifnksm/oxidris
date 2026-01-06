# KM-Based Feature Normalization Design

This document describes the target architecture for KM-based feature normalization applied to survival features.

- **Document type**: Explanation
- **Purpose**: High-level design concepts for KM-based normalization integration
- **Audience**: AI assistants, human contributors implementing feature normalization
- **When to read**: When implementing KM-based features or understanding the normalization architecture
- **Prerequisites**: [README.md](./README.md) for project overview; [Evaluator Documentation](../../architecture/evaluator/README.md) for trait context
- **Related documents**: [roadmap.md](./roadmap.md) (implementation status)

> [!IMPORTANT]
> **Status:** Design document - describes target architecture, not current implementation.
> See [roadmap.md](./roadmap.md) for implementation status.
>
> **Recent Changes (2026-01-06):** The evaluator system migrated from static feature constants
> to dynamic runtime construction via `FeatureBuilder`. This design reflects that architecture.

## Overview

This document describes the target design for KM-based feature normalization applied to **survival features** (holes, height). The design integrates KM-based transforms into the `BoardFeature` trait architecture, enabling non-linear, survival-time-based transformations while maintaining compatibility with existing analysis tools.

**Scope**: This design focuses on survival features only. Structure features and score optimization are out of scope for this project.

## Background

Traditional normalization approaches (min-max, z-score) fail to account for **right-censoring** in board survival data:

- Games that survive longer than `MAX_TURNS` are censored (we don't know their true survival time)
- Censoring predominantly affects "good" board states (long survival)
- Naive statistics (mean, percentiles) are biased when censoring is ignored

**Solution:** Use Kaplan-Meier survival analysis to estimate survival curves, then integrate KM-based transforms into the `BoardFeature` trait architecture.

## Design Goals

1. **Non-linear transformation**: Capture true relationship between survival feature values and survival time
2. **Trait integration**: Work within existing `BoardFeature` trait architecture
3. **Tool compatibility**: Maintain compatibility with `analyze-board-features` and other tools
4. **Interpretability**: Intermediate values (KM medians) have clear meaning as survival time
5. **Data-driven**: Transformations learned from actual gameplay data

## Normalization Method: P05-P95 Robust KM

The implemented method is **P05-P95 robust normalization** with **2-stage transform and normalize**:

### Algorithm

1. **Calculate KM median** for each unique feature value
2. **Find P05 and P95 feature values** based on board count distribution
3. **Generate transform mapping**: raw value → KM median (survival time in turns)
4. **Store normalization range**:
   - `km_max = KM median of P05 feature value` (good state)
   - `km_min = KM median of P95 feature value` (bad state)

### Two-Stage Evaluation via BoardFeature Trait

The `BoardFeature` trait provides instance methods for `transform()` and `normalize()`:

**Stage 1: Transform** (raw value → KM median)

- Look up raw value in mapping table to get KM median survival time
- Apply clipping for out-of-range values (see `MappedNormalized<S>` design below)

**Stage 2: Normalize** (KM median → 0-1)

- Linear scaling using P05/P95 KM medians as bounds
- Same normalization logic as `LinearNormalized<S>`, but applied to KM median values

This two-stage approach is implemented in the `MappedNormalized<S>` type (see Integration section below).

### Why This Method?

#### ✅ Proportional to Survival Time

```text
FeatureSource: num_holes
  0 holes:  KM=322.8 → norm=1.00
  1 hole:   KM=276.1 → norm=0.85
  5 holes:  KM=120.0 → norm=0.36

Ratio preserved:
  km(1) / km(0) = 0.855
  norm(1) / norm(0) = 0.85
```

The normalized value is proportional to actual survival time.

#### ✅ Robust to Outliers

```text
P05 = 0 holes  (bottom 5% of boards)
P95 = 33 holes (top 95% of boards)

Rare extreme values (e.g., 115 holes) don't affect the scale.
They're simply clamped to 0.0.
```

#### ✅ Frequency-Aware

P05/P95 are calculated based on board count, so common feature values determine the scale, not rare outliers.

#### ✅ Comparable Across Features

All features are normalized to 0-1 range, but the `km_range` statistic allows comparing their actual impact:

```text
num_holes:     km_range = 315.7 turns
max_height: km_range = 327.2 turns
surface_bumpiness:  km_range = 51.2 turns

→ num_holes and max_height have ~6x more impact than bumpiness
```

### Example

```text
Feature: num_holes
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
  ...
  transform_mapping[33]  = 7.1 turns
  (values beyond P95=33 are not in mapping; clipping applies)

Step 3: Store normalization range
  km_max = 322.8  (P05's KM median)
  km_min = 7.1    (P95's KM median)

Step 4: At evaluation time (2-stage)
  holes=0:  transform → 322.8, normalize → (322.8-7.1)/315.7 = 1.00
  holes=3:  transform → 177.5, normalize → (177.5-7.1)/315.7 = 0.54
  holes=33: transform → 7.1,   normalize → (7.1-7.1)/315.7   = 0.00
  holes=50: transform → 7.1 (clipped to max key 33), normalize → 0.00
```

## Target Features: Survival Features Only

This design applies to features that **directly affect game termination**:

- `num_holes` - holes prevent piece placement, causing game over
- `sum_of_hole_depth` - deeper holes are harder to clear
- `max_height` - height determines remaining vertical space
- `center_column_max_height` - center height affects placement options
- `total_height` - overall board pressure

**Why these features?**

- Direct, clear impact on survival time
- Strong expected correlation with survival (|r| > 0.5)
- Non-linear relationship suitable for KM-based normalization

**Out of Scope:**

- **Structure features** (bumpiness, transitions, well depth): Indirect impact through placement flexibility. May require different normalization approach.
- **Score features** (line clears, tetris setup): Different optimization objective. Requires separate strategy.

## Usage

### Generate Normalization Parameters

```bash
# Generate KM normalization parameters
make generate-normalization

# Or manually:
cargo run --release -- analyze-censoring data/boards.json \
    --normalization-output data/normalization_params.json
```

## Output Format

```json
{
  "max_turns": 500,
  "normalization_method": "robust_km",
  "features": {
    "num_holes": {
      "transform_mapping": {
        "0": 322.8,
        "1": 276.1,
        "2": 235.0,
        "3": 177.5,
        ...
        "33": 7.1
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

## Integration with BoardFeature Trait

### MappedNormalized<S> Type Design

KM normalization is implemented as a separate type from `LinearNormalized<S>`:

**Type Name:** `MappedNormalized<S>`

- Represents mapping-based transformation (lookup table)
- Generic over feature source `S: BoardFeatureSource`
- Parallel to `LinearNormalized<S>` but with different transform logic

**Key Concept:** Instead of `transform(raw) = raw as f32`, use a lookup table: `transform(raw) = mapping[raw]`

**Data Structure:**

```rust
pub struct MappedNormalized<S> {
    id: Cow<'static, str>,
    name: Cow<'static, str>,
    signal: FeatureSignal,
    mapping: BTreeMap<u32, f32>,  // raw → transformed value (e.g., KM median)
    normalize_min: f32,            // worst case (P95's transformed value)
    normalize_max: f32,            // best case (P05's transformed value)
    source: S,
}
```

**Transform Logic with Clipping:**

For raw values outside the mapping range `[a, b]`:

- `raw < a` → use `mapping[a]` (clip to minimum observed value)
- `raw > b` → use `mapping[b]` (clip to maximum observed value)
- `a ≤ raw ≤ b` → use `mapping[raw]` or interpolate if missing

This clipping ensures graceful handling of extreme values not seen during training.

**Example:**

```rust
// Mapping range: [0, 33] holes
// raw=0  → mapping[0] = 322.8 turns
// raw=3  → mapping[3] = 177.5 turns
// raw=33 → mapping[33] = 7.1 turns
// raw=50 → mapping[33] = 7.1 turns (clipped to max key)
```

### Feature Construction via FeatureBuilder

KM-based features follow the same construction pattern as current features:

1. **Data Collection:**

   ```text
   Sessions → RawBoardSample → RawFeatureStatistics
   ```

2. **KM Analysis:**

   ```text
   RawFeatureStatistics → KM Survival Analysis → KM Transform Mapping
   ```

3. **Feature Construction:**

   ```text
   FeatureBuilder::new(km_normalization_params)
       .build_all_features()
   ```

The `FeatureBuilder` would construct features with KM mapping data, similar to how it currently constructs features with percentile-based normalization parameters.

### FeatureProcessing Integration

Add a new variant to the `FeatureProcessing` enum:

```rust
pub enum FeatureProcessing {
    LinearNormalized {
        signal: FeatureSignal,
        normalize_min: f32,
        normalize_max: f32,
    },
    MappedNormalized {
        signal: FeatureSignal,
        mapping: BTreeMap<u32, f32>,
        normalize_min: f32,
        normalize_max: f32,
    },
    LineClearBonus,
    IWellReward,
}
```

This allows model serialization to preserve the feature type and reconstruction at runtime.

### Feature Naming Convention

To support coexistence of linear and mapped features:

**Linear Features (existing):**

- `num_holes_linear_penalty`
- `max_height_linear_penalty`
- `max_height_linear_risk`

**Mapped Features (new - KM-based):**

- `num_holes_km_penalty`
- `max_height_km_penalty`
- `sum_of_hole_depth_km_penalty`

**Benefits:**

- Clear distinction between transformation types
- Both feature sets can coexist in codebase
- AI models specify which set to use via model name (e.g., `aggro_linear` vs `aggro_km`)
- `analyze-board-features` can display both for comparison

**Feature Set Selection:**

- Training: `--model-name aggro_linear` → uses linear features
- Training: `--model-name aggro_km` → uses mapped (KM) features
- Analysis: Both feature sets available for side-by-side comparison

### Compatibility

Because KM normalization integrates into the existing `BoardFeature` trait:

- **`analyze-board-features`** - works without modification (visualizes KM-normalized features)
- **Training tools** - use the same `FeatureBuilder` pattern
- **Model serialization** - features identified by ID, mapping data stored separately

The two-stage design maintains the same trait interface:

- `extract_raw()` - unchanged (extracts raw measurement)
- `transform()` - KM lookup instead of linear cast
- `normalize()` - unchanged (linear 0-1 scaling)

## Design Rationale

### Why Not Each Feature Independently (0-1)?

**Problem:** Different features have different survival time ranges.

```text
holes:     range = 315 turns → 0.1 norm change = 31.5 turns
bumpiness: range = 51 turns  → 0.1 norm change = 5.1 turns

With equal weights (w=1.0), holes has 6x more impact!
```

**Solution:** The transform_mapping preserves KM median values (in turns), allowing proper weight initialization based on `km_range`.

### Why Not Global Normalization (All Features Same Scale)?

**Problem:** Small-range features lose precision.

```text
Global scale = 0 to 500 turns (MAX_TURNS)
  holes:     100-500 → 0.2-1.0 (good precision)
  bumpiness: 10-60   → 0.02-0.12 (poor precision)
```

**Solution:** Normalize per-feature, but provide `km_range` to adjust weights.

### Why P05-P95 Instead of Min-Max?

**Problem:** Outliers distort the scale.

```text
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

### Why Integrate with BoardFeature Trait?

**Maintains existing architecture:**

- No separate normalization system to maintain
- All tools work automatically with KM-normalized features
- Single source of truth for feature definitions

**Leverages existing two-stage design:**

- `transform()` - was linear cast, now KM lookup
- `normalize()` - was P05-P95 linear, still P05-P95 linear (but of KM medians)

**Benefits:**

- ✅ No changes to evaluator code (still uses `BoardFeature` trait)
- ✅ No changes to analysis tools (`analyze-board-features`, etc.)
- ✅ Easy to compare linear vs. KM normalization (just swap feature impl)
- ✅ Intermediate values (KM medians) remain interpretable

### Why Two Stages (Transform + Normalize)?

**Separation of concerns:**

- **Transform**: Converts raw values to meaningful units (survival time in turns)
  - Makes the intermediate value interpretable
  - KM median = "expected remaining survival time"
- **Normalize**: Scales to 0-1 for evaluation function
  - Simple linear scaling
  - Easy to understand and debug

**Alignment with BoardFeature Trait:**
The trait architecture already has this separation:

```rust
fn transform(raw: u32) -> f32;           // Was: linear cast
fn normalize(transformed: f32) -> f32;   // Was: P05-P95 linear scaling
```

KM-based approach keeps the same interface:

```rust
fn transform(raw: u32) -> f32;           // Now: KM lookup
fn normalize(transformed: f32) -> f32;   // Still: P05-P95 linear scaling (of KM medians)
```

### Why This Eliminates Duplicate Features

Current implementation has duplicate features with different normalization ranges:

- `linear_max_height_penalty` (P05-P95 linear) vs `linear_top_out_risk` (P75-P95 threshold)
- `linear_center_columns_penalty` (P05-P95) vs `linear_center_top_out_risk` (P75-P95)
- `linear_well_depth_penalty` (P05-P95) vs `linear_deep_well_risk` (P75-P95)

With KM normalization, the transform captures non-linearity:

- `holes=0→1`: large KM drop (322.8→276.1, -14%)
- `holes=10→11`: small KM drop (already near game over)

Different normalization ranges are no longer needed—KM transform handles it.

## Implementation Roadmap

See [roadmap.md](./roadmap.md) for detailed implementation plan:

- **Phase 1-2**: Data generation and KM survival analysis (✅ completed)
- **Phase 3**: Infrastructure (KM estimator, data structures, trait design) (🔄 in progress)
- **Phase 4**: Survival features with KM normalization (📋 project goal)

## Implementation Notes

### Feature Construction Pattern

Following the current `FeatureBuilder` pattern (established 2026-01-06):

- **Data-driven**: Normalization parameters computed from session data at runtime
- **No static constants**: Features constructed dynamically via `FeatureBuilder`
- **Mapping storage**: Lookup table stored as instance field (`BTreeMap<u32, f32>`)
- **Clipping behavior**: Out-of-range values clip to nearest boundary (min/max key)

### Design Decisions Summary

1. **Type Structure:** ✅ Decided
   - New type: `MappedNormalized<S>` (separate from `LinearNormalized<S>`)
   - Rationale: Different transform logic, clearer separation of concerns

2. **Lookup Table:** ✅ Decided
   - Use `BTreeMap<u32, f32>` for flexibility
   - Supports sparse mappings and efficient range queries

3. **Clipping Behavior:** ✅ Decided
   - `raw < min_key` → use `mapping[min_key]`
   - `raw > max_key` → use `mapping[max_key]`
   - Gracefully handles extreme values

4. **FeatureProcessing Integration:** ✅ Decided
   - Add `MappedNormalized` variant with mapping field
   - Enables model serialization and feature reconstruction

5. **Feature Naming:** ✅ Decided
   - Linear: `*_linear_penalty`, `*_linear_risk`
   - Mapped (KM): `*_km_penalty`
   - Both coexist; selection via model name

6. **Feature Set Strategy:** ✅ Decided
   - Keep linear features (backward compatibility)
   - Add KM features as new set
   - Model name determines which set to use
   - Both visible in analysis tools for comparison

### Remaining Implementation Tasks

Phase 3 design is now complete. Phase 4 implementation tasks:

1. Implement `MappedNormalized<S>` type with clipping logic
2. Add `MappedNormalized` variant to `FeatureProcessing` enum
3. Extend `FeatureBuilder` to construct mapped features
4. Implement KM-based survival features (`*_km_penalty`)
5. Update training tools to support model name-based feature set selection
6. Update `analyze-board-features` to display both feature sets

## See Also

- [Roadmap](./roadmap.md) - Implementation status and phases
- [README](./README.md) - Overview and current status
- [Evaluator System](../../architecture/evaluator/README.md) - Overall evaluator system documentation
- [Kaplan-Meier Survival Analysis](https://en.wikipedia.org/wiki/Kaplan%E2%80%93Meier_estimator)

### Code References

- **Feature trait architecture**: `crates/oxidris-evaluator/src/board_feature/mod.rs`
- **Feature sources**: `crates/oxidris-evaluator/src/board_feature/source.rs`
- **Feature builder**: `crates/oxidris-cli/src/analysis/feature_builder.rs`
- **Normalization params**: `crates/oxidris-cli/src/analysis/normalization.rs`
- **KM estimator**: `crates/oxidris-stats/src/survival.rs`
- **KM normalization generation**: `crates/oxidris-cli/src/command/analyze_censoring/mod.rs`
- **Data structures**: `crates/oxidris-cli/src/data.rs`
