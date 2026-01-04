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

## Phase 3: KM-Based Normalization Infrastructure 🔄

**Status:** In Progress

**Objectives:**

1. Implement Kaplan-Meier survival analysis for feature normalization
2. Generate normalization parameters from gameplay data
3. Design integration approach for `BoardFeatureSource` trait

**Completed:**

- [x] Implement Kaplan-Meier estimator for survival curve analysis
- [x] Design two-stage normalization architecture (raw → KM median → 0-1)
- [x] Implement data structures for normalization parameters
- [x] Implement normalization parameter generation from gameplay data
- [x] Create tools to generate `data/normalization_params.json`

**In Progress:**

- [ ] Design `BoardFeatureSource` trait integration
  - How to load KM normalization parameters
  - How to implement KM-based `transform()` method
  - Maintain compatibility with existing analysis tools

**Deliverables:**

- Normalization parameter generation pipeline
- Design document for trait integration

---

## Phase 4: Survival Features with KM Normalization 📋

**Status:** Not Started

**Objectives:**

1. Apply KM-based normalization to survival features (holes, height)
2. Train and validate evaluator using KM-normalized survival features
3. Demonstrate that KM normalization improves survival prediction

**Tasks:**

- [ ] Clean up feature set
  - Remove duplicate `*_risk` features (use `*_penalty` equivalents)

- [ ] Integrate KM normalization into `BoardFeatureSource` trait
  - Implement trait extension designed in Phase 3
  - Load normalization params from generated JSON

- [ ] Implement KM-based survival features
  - `holes_penalty`, `hole_depth_penalty` (holes directly cause game over)
  - `max_height_penalty`, `center_columns_penalty`, `total_height_penalty` (height determines available space)
  - Use KM transform to capture non-linear survival relationships

- [ ] Validate survival feature effectiveness
  - Analyze KM curves for each feature
  - Measure correlation with survival time
  - Compare KM transform vs. linear transform

- [ ] Train and benchmark evaluator
  - Train using survival features with KM normalization
  - Compare performance vs. current evaluator (linear normalization)
  - Analyze learned weights and interpretability

**Deliverables:**

- Clean feature set without duplicates
- KM-based survival features integrated with trait system
- Trained survival-focused evaluator
- Validation analysis showing improvement over linear normalization

---

## Success Metrics

### Phase 3: Infrastructure

- Normalization parameters successfully generated from gameplay data
- Infrastructure ready for `BoardFeatureSource` trait integration
- KM curves show clear non-linear relationships for survival features

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
- Phase 3: Infrastructure for KM normalization (in progress)
- Phase 4: Survival features with KM normalization (project goal)
- Future Work: Structure features, score optimization, advanced techniques (out of scope)

---

## References

- [README](./README.md) - Project overview and current status
- [Design](./design.md) - Target architecture for KM-based normalization
- [Evaluator System](../../architecture/evaluator/README.md) - Overall evaluator system documentation
