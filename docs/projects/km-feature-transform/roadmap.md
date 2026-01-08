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

## Phase 4: Survival Features with KM Normalization 📋

**Status:** Not Started

**Objectives:**

1. Apply KM-based normalization to survival features (holes, height)
2. Train and validate evaluator using KM-normalized survival features
3. Demonstrate that KM normalization improves survival prediction

**Tasks:**

- [ ] Implement `TableTransform<S>` type
  - Implement type with `BTreeMap<u32, f32>` mapping field
  - Implement `transform()` with clipping logic for out-of-range values
  - Implement `BoardFeature` trait

- [ ] Add `TableTransform` variant to `FeatureProcessing` enum
  - Add variant with mapping, signal, and normalization range fields
  - Update serialization/deserialization
  - Update `apply()` method for feature reconstruction

- [ ] Extend `FeatureBuilder` for mapped features
  - Add method to construct `TableTransform<S>` from KM normalization parameters
  - Support building both linear and mapped feature sets
  - Implement feature set selection logic

- [ ] Implement KM-based survival features
  - `num_holes_table_km`, `sum_of_hole_depth_table_km` (holes directly cause game over)
  - `max_height_table_km`, `center_column_max_height_table_km`, `total_height_table_km` (height determines available space)
  - Use KM transform to capture non-linear survival relationships

- [ ] Update training tools for feature set selection
  - Support model name-based feature set selection (`aggro_linear` vs `aggro_km`)
  - Keep linear features for backward compatibility
  - Add KM features as new feature set

- [ ] Update `analyze-board-features` for dual feature sets
  - Display both linear and KM features
  - Enable side-by-side comparison
  - Show transformation differences

- [ ] Validate survival feature effectiveness
  - Analyze KM curves for each feature
  - Measure correlation with survival time
  - Compare KM transform vs. linear transform

- [ ] Train and benchmark evaluator
  - Train using KM-based survival features
  - Compare performance vs. linear normalization
  - Analyze learned weights and interpretability

- [ ] Consider feature set cleanup (optional, after validation)
  - Evaluate removing duplicate `*_risk` features if KM approach is validated
  - Keep both linear and KM sets for comparison initially

**Deliverables:**

- `TableTransform<S>` type implementation
- KM-based survival features integrated with trait system
- Trained KM-based evaluator
- Both linear and KM feature sets available for comparison
- Validation analysis showing improvement over linear normalization

---

## Success Metrics

### Phase 3: Infrastructure ✅

- ✅ Normalization parameters successfully generated from gameplay data
- ✅ Infrastructure ready for `BoardFeature` trait integration
- ✅ KM curves show clear non-linear relationships for survival features
- ✅ Design approach finalized (mapping-based transform)

### Phase 4: Survival Features (Project Goal)

- Survival-based evaluator achieves ≥ current heuristic evaluator survival time
- Survival features show strong correlation with survival (|r| > 0.5)
- KM transform shows clear improvement over linear transform
- Learned weights are interpretable (correlate with feature km_range)
- Training converges in reasonable time (<1000 generations)

---

## Dependencies

```text
Phase 1 → Phase 2 → Phase 3 → Phase 4 (Complete)
                               ↓
                          Future Work (separate projects)
```

- Phase 1-2: Data generation and KM survival analysis (completed)
- Phase 3: Infrastructure for KM normalization (completed 2026-01-06)
- Phase 4: Survival features with KM normalization (project goal)
- Future Work: Structure features, score optimization, advanced techniques (out of scope)

---

## References

- [README](./README.md) - Project overview and current status
- [Design](./design.md) - Target architecture for KM-based normalization
- [Evaluator System](../../architecture/evaluator/README.md) - Overall evaluator system documentation
