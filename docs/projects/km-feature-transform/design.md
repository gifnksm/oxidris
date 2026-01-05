# KM-Based Feature Normalization Design

This document describes the target architecture for KM-based feature normalization applied to survival features.

- **Document type**: Explanation
- **Purpose**: Detailed technical design for KM-based normalization integration into BoardFeature trait
- **Audience**: AI assistants, human contributors implementing feature normalization
- **When to read**: When implementing KM-based features or understanding the normalization architecture
- **Prerequisites**: [README.md](./README.md) for project overview; [Evaluator Documentation](../../architecture/evaluator/README.md) for trait context
- **Related documents**: [roadmap.md](./roadmap.md) (implementation status)

> [!IMPORTANT]
> **Status:** Design document - describes target architecture, not current implementation.
> See [roadmap.md](./roadmap.md) for implementation status.

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

```rust
// KM-based transform would be implemented in LinearNormalized or custom feature
impl BoardFeature for LinearNormalized<HolesPenalty> {
    fn transform(&self, raw: u32) -> f32 {
        // Look up KM median from loaded normalization params
        self.km_params.get(raw).unwrap_or(self.km_min)  // → 177.5 turns
    }
}
```

**Stage 2: Normalize** (KM median → 0-1)

```rust
impl BoardFeature for LinearNormalized<HolesPenalty> {
    fn normalize(&self, transformed: f32) -> f32 {
        let span = self.normalize_max - self.normalize_min;
        ((transformed - self.normalize_min) / span).clamp(0.0, 1.0)
    }
}
```

Where `normalize_min` = P95's KM median, `normalize_max` = P05's KM median.

**Note:** The current implementation uses `LinearNormalized<S>` wrapper struct that holds normalization parameters as instance fields. KM-based features would extend this pattern with KM lookup tables.

### Why This Method?

#### ✅ Proportional to Survival Time

```text
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
holes_penalty:     km_range = 315.7 turns
max_height_penalty: km_range = 327.2 turns
bumpiness_penalty:  km_range = 51.2 turns

→ holes and max_height have ~6x more impact than bumpiness
```

### Example

```text
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

## Target Features: Survival Features Only

This design applies to features that **directly affect game termination**:

- `holes_penalty` - holes prevent piece placement, causing game over
- `hole_depth_penalty` - deeper holes are harder to clear
- `max_height_penalty` - height determines remaining vertical space
- `center_columns_penalty` - center height affects placement options
- `total_height_penalty` - overall board pressure

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
    --kaplan-meier \
    --normalization-output data/normalization_params.json
```

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

## Integration with BoardFeature Trait

### Target Architecture

KM normalization will be integrated into the `BoardFeature` trait architecture using instance-based methods:

```rust
// Target design (Phase 3-4)
// KM-based feature using custom transformation
pub struct KMNormalized<S> {
    id: Cow<'static, str>,
    name: Cow<'static, str>,
    signal: FeatureSignal,
    km_transform: BTreeMap<u32, f32>,  // raw → KM median lookup
    normalize_min: f32,  // P95's KM median
    normalize_max: f32,  // P05's KM median
    source: S,
}

impl<S: BoardFeatureSource> BoardFeature for KMNormalized<S> {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn clone_boxed(&self) -> BoxedBoardFeature {
        Box::new(self.clone())
    }
    
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        self.source.extract_raw(analysis)
    }
    
    // Transform: raw → KM median (from lookup table)
    fn transform(&self, raw: u32) -> f32 {
        self.km_transform.get(&raw)
            .copied()
            .unwrap_or(self.normalize_min)
    }
    
    // Normalize: KM median → 0-1 (linear scaling)
    fn normalize(&self, transformed: f32) -> f32 {
        let span = self.normalize_max - self.normalize_min;
        let norm = ((transformed - self.normalize_min) / span).clamp(0.0, 1.0);
        match self.signal {
            FeatureSignal::Positive => norm,
            FeatureSignal::Negative => 1.0 - norm,
        }
    }
}
```

### Loading KM Transform Data

```rust
// Load normalization parameters from JSON at build/startup time
fn load_km_normalization(path: &Path) -> Result<NormalizationParams> {
    let params: NormalizationParams = serde_json::from_reader(File::open(path)?)?;
    Ok(params)
}

// Create KM-based feature instances with loaded data
fn create_km_features(params: &NormalizationParams) -> Vec<BoxedBoardFeature> {
    let mut features = Vec::new();
    
    // For each feature in the params
    for (feature_id, feature_data) in &params.features {
        let km_transform: BTreeMap<u32, f32> = feature_data.transform_mapping
            .iter()
            .map(|(k, v)| (*k, *v as f32))
            .collect();
        
        let feature = KMNormalized {
            id: Cow::Owned(feature_id.clone()),
            name: Cow::Owned(format!("{} (KM)", feature_id)),
            signal: FeatureSignal::Negative,  // Most survival features are negative
            km_transform,
            normalize_min: feature_data.km_min as f32,
            normalize_max: feature_data.km_max as f32,
            source: get_source_for_feature(feature_id),
        };
        
        features.push(Box::new(feature) as BoxedBoardFeature);
    }
    
    features
}
```

### Example: KM-Based HolesPenalty

```rust
// Feature source remains simple - just extracts raw values
#[derive(Debug, Clone)]
pub struct HolesPenalty;

impl BoardFeatureSource for HolesPenalty {
    fn extract_raw(&self, analysis: &PlacementAnalysis) -> u32 {
        analysis.board_analysis().num_holes().into()
    }
}

// KM-based feature constant (loaded from normalization_params.json)
pub const KM_HOLES_PENALTY: KMNormalized<HolesPenalty> = KMNormalized {
    id: Cow::Borrowed("km_holes_penalty"),
    name: Cow::Borrowed("Holes Penalty (KM)"),
    signal: FeatureSignal::Negative,
    km_transform: load_km_transform("holes_penalty"),  // Loaded at build/startup
    normalize_min: 7.1,    // P95's KM median
    normalize_max: 322.8,  // P05's KM median
    source: HolesPenalty,
};

// Usage
let feature_value = KM_HOLES_PENALTY.compute_feature_value(&analysis);
// holes=0:  raw=0 → transform → 322.8 → normalize → 1.00
// holes=3:  raw=3 → transform → 177.5 → normalize → 0.54
// holes=33: raw=33 → transform → 7.1 → normalize → 0.00
```

### Compatibility with Existing Tools

Because KM normalization is integrated into the `BoardFeature` trait, all existing tools work without modification:

- `analyze-board-features` TUI - visualizes KM-normalized features
- `generate-board-feature-stats` - can generate KM-based percentile stats

The two-stage design maintains the same trait interface:

- `extract_raw()` - unchanged
- `transform()` - now KM-based instead of linear
- `normalize()` - unchanged (still linear 0-1 scaling)

### Weight Initialization Strategy

Initialize weights based on KM range to equalize feature impact:

```rust
// km_range = NORMALIZATION_MAX - NORMALIZATION_MIN
let km_range_holes = 322.8 - 7.1;      // = 315.7 turns
let km_range_bumpiness = 60.5 - 9.3;   // = 51.2 turns

let initial_weight_holes = 1.0 / km_range_holes;      // = 0.00317
let initial_weight_bumpiness = 1.0 / km_range_bumpiness;  // = 0.0195

// Result: A 0.1 change in any normalized feature represents
// roughly the same survival time impact (~30 turns)
```

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

- `max_height_penalty` (P05-P95 linear) vs `top_out_risk` (P75-P95 threshold)
- `center_columns_penalty` (P05-P95) vs `center_top_out_risk` (P75-P95)
- `well_depth_penalty` (P05-P95) vs `deep_well_risk` (P75-P95)

With KM normalization, the transform captures non-linearity:

- `holes=0→1`: large KM drop (322.8→276.1, -14%)
- `holes=10→11`: small KM drop (already near game over)

Different normalization ranges are no longer needed—KM transform handles it.

## Implementation Roadmap

See [roadmap.md](./roadmap.md) for detailed implementation plan:

- **Phase 1-2**: Data generation and KM survival analysis (✅ completed)
- **Phase 3**: Infrastructure (KM estimator, data structures, trait design) (🔄 in progress)
- **Phase 4**: Survival features with KM normalization (📋 project goal)

## Open Design Questions

### Table Loading Strategy

- How to load KM transform tables into feature constants?
  - Build-time codegen? Runtime loading? Lazy initialization?
- Performance considerations for BTreeMap lookup during evaluation
  - Consider using arrays for dense mappings (e.g., holes 0-50)

### Handling Missing Values

- How to handle missing raw values in transform table?
  - Default to `NORMALIZATION_MIN` (worst case)?
  - Interpolate from nearby values?
  - Return error and require complete coverage?

### Feature Elimination

- Can we eliminate all duplicate `*_risk` features with KM transform?
- Do any survival features have non-monotonic relationships that need special handling?

## See Also

- [Roadmap](./roadmap.md) - Implementation status and phases
- [README](./README.md) - Overview and current status
- [Evaluator System](../../architecture/evaluator/README.md) - Overall evaluator system documentation
- [Kaplan-Meier Survival Analysis](https://en.wikipedia.org/wiki/Kaplan%E2%80%93Meier_estimator)

### Code References

- Feature definitions: `crates/oxidris-evaluator/src/board_feature/mod.rs`
- KM estimator: `crates/oxidris-stats/src/survival.rs`
- Normalization generation: `crates/oxidris-cli/src/analyze_censoring.rs`
- Data structures: `crates/oxidris-cli/src/data.rs`
