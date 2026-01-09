# KM-Based Feature Transform Roadmap

This roadmap outlines the development phases for implementing KM-based normalization for survival features.

- **Document type**: Reference
- **Purpose**: Phase-by-phase implementation plan and current status for KM-based feature normalization
- **Audience**: AI assistants, human contributors tracking project progress
- **When to read**: When checking project status, planning next steps, or understanding implementation sequence
- **Prerequisites**: [README.md](./README.md) for project overview and goals
- **Related documents**: [design.md](./design.md) (detailed architecture)

## Overview

This roadmap outlines the development of KM-based normalization for **survival features** (holes, height). The project focuses on replacing linear normalization with survival-time-based transformations that capture the non-linear relationship between feature values and survival time.

**Scope**: This project is limited to survival features. Structure features and score optimization are out of scope and will be addressed in future projects if needed.

---

## Phase 1: Data Generation & Initial Analysis ✅

**Status:** Completed

**Objectives:**

1. Generate diverse gameplay data from multiple evaluators
2. Identify the right-censoring problem in survival analysis
3. Establish data collection infrastructure

**Completed:**

- [x] Implement data generation pipeline
  - Multiple evaluator types (random, height-only, heuristic, noisy)
  - Capture board states and survival times
  - Generate diverse dataset (~100k boards, ~9k sessions)

- [x] Identify and quantify censoring problem
  - Measure censoring rate (~78% initially, improved to 8.7%)
  - Identify that censoring affects "good" board states
  - Recognize bias in naive statistics (mean, median)

- [x] Implement basic analysis tools
  - Tools to analyze censoring patterns
  - Identify need for survival analysis methods

**Key Findings:**

- Right-censoring is substantial when games don't reach MAX_TURNS
- Censoring biases naive statistics toward pessimistic estimates
- Need Kaplan-Meier estimator to handle censored data properly

**Deliverables:**

- Data generation pipeline
- Initial dataset with diverse gameplay
- Censoring analysis tools

---

## Phase 2: Kaplan-Meier Survival Analysis ✅

**Status:** Completed

**Objectives:**

1. Implement Kaplan-Meier estimator to handle censored data
2. Quantify bias in naive statistics vs. KM estimates
3. Establish P05-P95 percentile selection for robust normalization

**Completed:**

- [x] Implement Kaplan-Meier estimator
  - Compute unbiased survival curves from censored data
  - Calculate median survival times per feature value

- [x] Quantify censoring bias
  - Compare KM medians vs. naive medians (up to 56% difference)
  - Show bias is feature-dependent and value-dependent
  - Demonstrate censoring affects "good" feature values more

- [x] Implement robust percentile selection
  - Use board-count percentiles (P05-P95) for normalization bounds
  - More robust than value-based percentiles

- [x] Build analysis and visualization tools
  - CLI tools for KM analysis
  - CSV export for KM curve visualization

**Key Findings:**

- Naive statistics significantly underestimate survival for good board states
- KM curves reveal non-linear relationships (e.g., holes vs. survival)
- P05-P95 by board count provides robust normalization range

**Deliverables:**

- Kaplan-Meier estimator implementation
- Analysis tools for censored survival data
- Understanding of censoring bias patterns

---

## Phase 3: KM-Based Normalization Infrastructure ✅

**Status:** Completed (2026-01-06)

**Objectives:**

1. Implement Kaplan-Meier survival analysis for feature normalization
2. Generate normalization parameters from gameplay data
3. Design integration approach for `BoardFeature` trait

**Completed:**

- [x] Implement Kaplan-Meier estimator for survival curve analysis
- [x] Design two-stage normalization architecture (raw → KM median → 0-1)
- [x] Implement data structures for normalization parameters
- [x] Implement normalization parameter generation from gameplay data
- [x] Create tools to generate `data/normalization_params.json`
- [x] Design `TableTransform<S>` type
  - Separate type from `RawTransform<S>` with mapping-based transform
  - Uses `BTreeMap<u32, f32>` for raw → transformed value lookup
  - Clipping behavior for out-of-range values (clip to min/max key)
  - Follows `FeatureBuilder` construction pattern (established 2026-01-06)
- [x] Design `FeatureProcessing` integration
  - Add `TableTransform` variant to enum for serialization
- [x] Define feature naming convention
  - Linear features: `*_raw_penalty`, `*_raw_risk` (renamed from `*_linear_penalty`, `*_linear_risk`)
  - Mapped (KM) features: `*_table_km` (renamed from planned `*_km_penalty`)
  - Type names: `RawTransform<S>` (renamed from `LinearNormalized<S>`), `TableTransform<S>` (renamed from `MappedNormalized<S>`)
  - Feature set selection via model name (`aggro_linear` vs `aggro_km`)

**Key Decisions:**

- **Type Structure**: `TableTransform<S>` as separate type (not extension of `RawTransform<S>`)
- **Lookup Table**: `BTreeMap<u32, f32>` for flexibility with sparse mappings
- **Clipping**: Out-of-range values clip to nearest boundary (min/max key in mapping)
- **Feature Coexistence**: Keep linear features, add KM features as new set
- **Model Selection**: Model name determines feature set (`aggro_linear` vs `aggro_km`)
- **Naming Convention**: `raw` for minimal transformation, `table` for lookup-based transformation, `km` for KM survival analysis method

**Deliverables:**

- Normalization parameter generation pipeline
- Complete design document with type structure, clipping logic, and naming conventions
- Infrastructure ready for Phase 4 implementation

---

## Phase 4: Survival Features with KM Normalization ✅

**Status:** Implementation Complete (2026-01-09), Validation Pending (Phase 5)

**Objectives:**

1. ✅ Apply KM-based normalization to survival features (holes, height)
2. ✅ Implement training infrastructure for KM/Raw comparison
3. 📋 Train and validate evaluator using KM-normalized survival features (→ Phase 5)
4. 📋 Demonstrate that KM normalization improves survival prediction (→ Phase 5)

**Completed:**

- [x] Implement `TableTransform<S>` type
  - Implemented with `Vec<f32>` lookup table (P05-P95 range)
  - `transform()` with clamping logic for out-of-range values
  - Full `BoardFeature` trait implementation
  - Edge case handling (zero division in normalization)

- [x] Add `TableTransform` variant to `FeatureProcessing` enum
  - Added variant with serializable parameters
  - Proper serialization/deserialization support
  - Integration with feature reconstruction

- [x] Extend `FeatureBuilder` for table-based features
  - `build_table_km_for()` constructs `TableTransform<S>` from normalization parameters
  - `build_all_features()` builds both raw and table features
  - `build_raw_features()` builds only raw features
  - `FeatureSet` enum for feature selection

- [x] Implement KM-based survival features
  - `num_holes_table_km`, `sum_of_hole_depth_table_km`
  - `max_height_table_km`, `center_column_max_height_table_km`, `total_height_table_km`
  - All features use KM median survival time as transformation

- [x] Update CLI tools for feature set selection
  - `train-ai` uses `FeatureSet::Raw` (backward compatible)
  - `analyze-board-features` uses `FeatureSet::All` (displays both)
  - `build_feature_from_session()` utility performs complete pipeline

- [x] Implement survival statistics pipeline
  - `SurvivalStatsMap::collect_by_feature_value()` groups by feature value
  - KM median calculation with proper censoring handling
  - Linear interpolation for missing KM median values
  - P05-P95 percentile-based table range selection
  - `SurvivalTable::from_survival_stats()` generates lookup tables

- [x] Update `analyze-board-features` for dual feature sets
  - Automatically builds both raw and table features
  - Interactive TUI displays all features
  - Side-by-side comparison enabled

- [x] Extend training infrastructure for KM/Raw comparison
  - Add `FeatureSet::Km` for KM-only feature sets
  - Extend `AiType` to support 4 model types (AggroKm, DefensiveKm, AggroRaw, DefensiveRaw)
  - Add `build_km_features()` to `FeatureBuilder`
  - Update Makefile targets for 4 model variants
  - Generate 4 baseline models for validation

- [x] Generate baseline models for comparison
  - `aggro-km.json` (fitness=2.56, trained 2026-01-09)
  - `defensive-km.json` (trained 2026-01-09)
  - `aggro-raw.json` (fitness=2.51, trained 2026-01-09)
  - `defensive-raw.json` (trained 2026-01-09)

**Implementation Details:**

- **Table Structure**: `Vec<f32>` covering P05-P95 feature value range
  - Index = `raw_value - feature_min_value`
  - Out-of-range values clamped to table boundaries
  - Linear interpolation fills gaps in KM median data

- **Normalization**: Two-stage pipeline
  - Stage 1: `transform()` - raw value → survival time (via table lookup)
  - Stage 2: `normalize()` - survival time → [0, 1] (using survival range)

- **Feature Set Management**:
  - Training uses `FeatureSet::Raw` (established raw features)
  - Analysis uses `FeatureSet::All` (both raw and table for comparison)

**Deliverables:**

- ✅ `TableTransform<S>` type fully implemented
- ✅ KM-based survival features integrated with trait system
- ✅ Both raw and table feature sets available
- ✅ CLI tools updated for feature set selection
- ✅ 4 baseline models generated (aggro-km, defensive-km, aggro-raw, defensive-raw)
- ✅ Training infrastructure supports KM/Raw comparison
- 📋 Training validation (→ Phase 5)

---

## Phase 5: Validation and Training 📋

**Status:** Ready to Start (Implementation Complete 2026-01-09)

**Baseline Models Available:**

- `models/ai/aggro-km.json` (fitness=2.56, trained 2026-01-09)
- `models/ai/defensive-km.json` (trained 2026-01-09)
- `models/ai/aggro-raw.json` (fitness=2.51, trained 2026-01-09)
- `models/ai/defensive-raw.json` (trained 2026-01-09)

**Objectives:**

1. Validate that KM transform captures non-linear relationships
2. Compare KM-based evaluator performance vs. raw baseline
3. Measure feature correlation with survival time
4. Analyze learned weights for interpretability
5. Document results and make recommendation

**Tasks:**

- [ ] **Benchmark Model Performance**
  - Run auto-play for each model (≥100 games per model)
  - Record survival times for each game
  - Calculate statistics (mean, median, P25, P75, std dev)
  - Compare KM-based vs. Raw-based for both aggro and defensive

- [ ] **Feature Correlation Analysis**
  - Extract feature values and survival times from validation games
  - Calculate Pearson correlation for each feature
  - Compare correlation strength: KM features vs. Raw features
  - Verify |r| > 0.5 for survival features
  - Identify features with strongest survival correlation

- [ ] **Weight Interpretability Analysis**
  - Compare learned weights between models:
    - aggro-km vs. aggro-raw
    - defensive-km vs. defensive-raw
  - Check if weights correlate with feature survival ranges
  - Analyze weight stability (variation between runs)
  - Verify higher weights for features with larger survival impact

- [ ] **Training Convergence Analysis**
  - Compare fitness progression over generations
  - Analyze convergence speed (generations to stable fitness)
  - Check for overfitting patterns
  - Compare training stability (KM vs. Raw)

- [ ] **Document Results and Make Decision**
  - Create validation report with all metrics
  - Summarize findings (what worked, what didn't)
  - Make recommendation: adopt KM features, keep Raw, or iterate
  - If KM features succeed, plan Phase 6 (refinement)
  - If KM features fail, document lessons and pivot

**Validation Criteria:**

- KM-based models achieve ≥ Raw-based survival time (statistical significance)
- Survival features show |r| > 0.5 correlation
- KM transform demonstrates measurable improvement over Raw
- Learned weights are interpretable and stable

**Deliverables:**

- Trained KM-based evaluator model
- Performance comparison report (KM vs. raw)
- Feature correlation analysis
- Weight interpretability analysis
- Decision on feature set adoption

---

## Success Metrics

### Phase 3: Infrastructure ✅

- ✅ Normalization parameters successfully generated from gameplay data
- ✅ Infrastructure ready for `BoardFeature` trait integration
- ✅ KM curves show clear non-linear relationships for survival features
- ✅ Design approach finalized (table-based transform)

### Phase 4: Implementation ✅

- ✅ `TableTransform<S>` type successfully implemented
- ✅ Both raw and table feature sets available
- ✅ CLI tools support feature set selection
- ✅ Survival statistics pipeline working correctly
- ✅ Interactive analysis tool displays all features
- ✅ 4 baseline models generated for validation
- ✅ Training infrastructure supports KM/Raw comparison

### Phase 5: Validation (Project Goal)

- KM-based evaluator achieves ≥ raw-based evaluator survival time
- Survival features show strong correlation with survival (|r| > 0.5)
- KM transform demonstrates measurable improvement over raw transform
- Learned weights are interpretable (correlate with survival ranges)
- Training converges in reasonable time (<1000 generations)

---

## Dependencies

```text
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 (Ready to Start)
                                          ↓
                                     Future Work (separate projects)
```

- Phase 1-2: Data generation and KM survival analysis (completed)
- Phase 3: Infrastructure for KM normalization (completed 2026-01-06)
- Phase 4: Implementation of table-based features (completed 2026-01-09)
- Phase 5: Validation and training (ready to start, project goal)
- Future Work: Structure features, score optimization, advanced techniques (out of scope)

---

## References

- [README](./README.md) - Project overview and current status
- [Design](./design.md) - Target architecture for KM-based normalization
- [Evaluator System](../../architecture/evaluator/README.md) - Overall evaluator system documentation
