# Evaluator System: Current Status and Improvement Plans

## Current Implementation

### Architecture

```
Raw Board State
    ↓
Feature Extraction (BoardAnalysis)
    ↓
Percentile-Based Normalization (0-1 scale)
    ↓
Weighted Combination (learned by GA)
    ↓
Final Score
```

### Components

#### 1. Feature Extraction

**Location:** `crates/oxidris-ai/src/board_feature/mod.rs`

15 features measuring different aspects of board state:

**Survival Features:**
- `holes_penalty`: Number of holes
- `hole_depth_penalty`: Depth of holes
- `max_height_penalty`: Maximum column height
- `total_height_penalty`: Sum of column heights
- `center_columns_penalty`: Center column heights
- `well_depth_penalty`: Depth of wells

**Structure Features:**
- `surface_bumpiness_penalty`: Surface unevenness
- `surface_roughness_penalty`: Surface variation
- `row_transitions_penalty`: Horizontal transitions
- `column_transitions_penalty`: Vertical transitions

**Risk Features (duplicates):**
- `top_out_risk`: Top-out danger
- `center_top_out_risk`: Center column danger
- `deep_well_risk`: Deep well danger

**Score Features:**
- `line_clear_bonus`: Lines cleared
- `i_well_reward`: I-piece well setup

#### 2. Normalization

**Current Method:** Percentile-based linear normalization (P05-P95)

**How it works:**
1. Percentile values (P05, P95, etc.) are automatically generated from gameplay data
2. Generated values are stored in `board_feature/stats.rs` (via `make regenerate-board-feature-stats`)
3. Each feature uses P05 and P95 as normalization range endpoints

```rust
// Auto-generated from data
const NORMALIZATION_MIN: f32 = Self::TRANSFORMED_P05;  // e.g., 0.0
const NORMALIZATION_MAX: f32 = Self::TRANSFORMED_P95;  // e.g., 8.0

// Linear transformation (default for most features)
fn transform(raw: u32) -> f32 {
    raw as f32
}

// Normalize to [0, 1] using P05-P95 range
fn normalize(transformed: f32) -> f32 {
    let span = NORMALIZATION_MAX - NORMALIZATION_MIN;
    ((transformed - NORMALIZATION_MIN) / span).clamp(0.0, 1.0)
}
```

**Characteristics:**
- ✅ Simple and fast
- ✅ Data-driven (percentiles computed from actual gameplay)
- ✅ Robust to outliers (P05-P95 range)
- ❌ Linear transformation doesn't capture non-linear survival relationships for most features
- ❌ Some features have heuristic custom transforms (LineClearBonus, IWellReward) creating inconsistency

#### 3. Weight Learning

**Location:** `crates/oxidris-ai/src/genetic.rs`

Genetic algorithm optimizes feature weights:

**Fitness Functions:**
- `AggroSessionEvaluator`: survival bonus + weighted line clears - height penalty
- `DefensiveSessionEvaluator`: survival bonus + basic efficiency - height penalty

**GA Parameters:**
```rust
POPULATION_COUNT: 30
MAX_GENERATIONS: 200
ELITE_COUNT: 2
TOURNAMENT_SIZE: 2
MUTATION_RATE: 0.3
BLX_ALPHA: 0.2

// Phase-dependent parameters
EXPLORATION (gen 0-30):
  max_weight: 0.5
  mutation_sigma: 0.05

TRANSITION (gen 30-80):
  max_weight: 0.8
  mutation_sigma: 0.02

CONVERGENCE (gen 80+):
  max_weight: 1.0
  mutation_sigma: 0.01
```

**Characteristics:**
- ✅ Automated weight optimization
- ✅ Phase-based parameter adaptation
- ❌ Hyperparameters manually tuned
- ❌ No systematic hyperparameter search

#### 4. Trained Models

**Location:** `models/ai/`

- `aggro.json`: 15 trained weights (fitness: 2.61)
- `defensive.json`: 15 trained weights

Both use same features but optimized for different objectives.

## Current Issues

### Issue 1: Linear Feature Transformation

**Problem:**
- Most features use linear transformation: `transform(raw) = raw as f32`
- Doesn't capture non-linear relationship between feature values and survival time
- Example: The survival impact of `holes: 0→1` is very different from `holes: 10→11`, but linear transform treats them equally

**Impact:**
- Suboptimal feature representation (linear when relationship is non-linear)
- GA must compensate for poor transformation through weight adjustments
- Harder to learn optimal strategy

**Evidence:**
- Duplicate features exist (`*_penalty` vs `*_risk`) with different normalization ranges
- These duplicates attempt to capture non-linearity through different scaling, but this is an ad-hoc solution
- Suggests systematic need for non-linear transformations

### Issue 2: Feature Redundancy

**Problem:**
- Multiple features measure similar things:
  - `top_out_risk` ≈ `max_height_penalty`
  - `center_top_out_risk` ≈ `center_columns_penalty`
  - `deep_well_risk` ≈ `well_depth_penalty`
- 15 features, some highly correlated

**Impact:**
- Wastes computational resources
- Makes weight interpretation harder
- May confuse GA learning

**Status:** Not analyzed systematically

### Issue 3: GA Hyperparameter Tuning

**Problem:**
- Current GA parameters chosen manually:
  - Population size: 30
  - Elite count: 2
  - Tournament size: 2
  - Mutation rates: phase-dependent
  - BLX alpha: 0.2
- No systematic search for optimal values

**Impact:**
- May be suboptimal convergence speed
- May miss better solutions
- Unclear if current parameters are good

**Status:** No tuning performed

### Issue 4: Score Features Not Optimized

**Problem:**
- `line_clear_bonus` and `i_well_reward` exist but not optimally integrated
- Fitness functions mix survival and score ad-hoc
- No principled multi-objective optimization

**Impact:**
- Can't express different play styles clearly
- Trade-offs between survival and score not well-understood

## Improvement Plans

### Project 1: KM-Based Feature Transform (Phase 3 - In Progress)

**Goal:** Replace linear normalization with survival-time-based transformation

**Approach:**
1. Use Kaplan-Meier survival analysis on gameplay data
2. Transform: `feature_value → KM median (survival time in turns)`
3. Normalize: `KM_median → 0-1` using KM medians of P05/P95 feature values as range endpoints

**Key consideration:**
- Games surviving beyond `MAX_TURNS` are right-censored
- Kaplan-Meier estimator properly handles this censoring to produce unbiased survival estimates
- This is only relevant for survival-time-based approaches (not an issue for current heuristic method)

**Benefits:**
- Non-linear, data-driven transformations
- Proper handling of right-censoring in survival analysis
- Interpretable intermediate values (expected survival time)
- Eliminates need for duplicate features

**Status:** WIP - dedicated documentation in `km-feature-transform/`

**Timeline:** Phase 3 (current)

### Project 2: GA Hyperparameter Tuning (Planned)

**Goal:** Find optimal GA parameters through systematic search

**Approach:**
1. Define search space:
   - Population size: [20, 50, 100]
   - Elite count: [1, 2, 5, 10]
   - Tournament size: [2, 3, 5]
   - Mutation sigma: [0.01, 0.05, 0.1]
   - BLX alpha: [0.1, 0.2, 0.5]
2. Use grid search or Bayesian optimization
3. Evaluate on multiple random seeds
4. Select parameters maximizing final fitness

**Benefits:**
- Better convergence speed
- Higher quality solutions
- Evidence-based parameter choices

**Status:** Not started

**Challenges:**
- Computationally expensive (many GA runs)
- Need to avoid overfitting to specific random seeds
- May need meta-optimization framework

### Project 3: Feature Selection (Planned)

**Goal:** Remove redundant features, identify most important ones

**Approach:**
1. Correlation analysis:
   - Compute pairwise feature correlations
   - Identify highly correlated pairs (|r| > 0.8)
2. Importance analysis:
   - Train multiple models
   - Analyze weight distributions
   - Identify features with consistently near-zero weights
3. Ablation study:
   - Remove features one by one
   - Measure impact on performance
   - Keep features that contribute significantly

**Benefits:**
- Faster evaluation (fewer features)
- Easier weight interpretation
- Better generalization (less overfitting)

**Status:** Not started

**Priority:** After KM-based transform (may change feature relationships)

### Project 4: Multi-Objective Optimization (Phase 4)

**Goal:** Principled optimization of survival + score

**Approach:**
1. Extend `SessionData` to capture score metrics
2. Design reward function: `w_survival * survival + w_score * score`
3. Allow user-configurable weights for different play styles
4. Optional: Pareto front exploration for trade-off analysis

**Benefits:**
- Clear expression of different objectives
- User can choose play style (defensive/balanced/aggressive)
- Better understanding of survival-score trade-offs

**Status:** Not started (requires KM-based transform first)

**Timeline:** Phase 4

## Code Locations

### Core Implementation
- Feature extraction: `crates/oxidris-ai/src/board_feature/mod.rs`
- Board analysis: `crates/oxidris-engine/src/board_analysis.rs`
- Placement analysis: `crates/oxidris-ai/src/placement_analysis.rs`
- Genetic algorithm: `crates/oxidris-ai/src/genetic.rs`
- Session evaluators: `crates/oxidris-ai/src/session_evaluator.rs`

### Training
- Training script: `crates/oxidris-cli/src/train_ai.rs`
- Models: `models/ai/aggro.json`, `models/ai/defensive.json`

### Data Structures
- SessionData: `crates/oxidris-cli/src/data.rs`
- Weights: `crates/oxidris-ai/src/weights.rs`

## Next Steps

1. **Complete KM-based feature transform** (Phase 3)
   - Remove duplicate `*_risk` features
   - Implement `KMBasedEvaluator`
   - Train and benchmark vs current approach

2. **Feature selection analysis**
   - Identify redundant features
   - Measure importance
   - Consolidate feature set

3. **GA hyperparameter tuning**
   - Design search space
   - Run systematic experiments
   - Document optimal parameters

4. **Multi-objective optimization** (Phase 4)
   - Extend data collection for score
   - Implement reward functions
   - Train models with different objectives