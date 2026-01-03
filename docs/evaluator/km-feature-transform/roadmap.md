# KM-Based Evaluator Development Roadmap

## Overview

This roadmap outlines the phased development of the KM-Based Evaluator, from initial data generation through advanced techniques. Each phase builds on the previous ones, with clear objectives and deliverables.

---

## Phase 1: Data Generation & Initial Analysis ✅

**Status:** Completed

**Objectives:**
- Generate diverse gameplay data with multiple evaluators
- Identify and quantify the right-censoring problem
- Establish data collection infrastructure

**Completed Tasks:**
- [x] Implement board data generation pipeline
- [x] Add multiple evaluators (random, height_only, heuristic, noisy_heuristic)
- [x] Generate diverse dataset (~100k boards, ~9k sessions)
- [x] Identify right-censoring problem (~78% → 8.7% after improvements)
- [x] Implement basic censoring analysis tools

**Key Findings:**
- Right-censoring is substantial in the initial dataset (78%)
- Censoring predominantly affects "good" board states
- Naive statistics (mean, median) are biased by censoring
- Need for survival analysis methods (Kaplan-Meier)

**Deliverables:**
- `crates/oxidris-cli/src/data.rs`: SessionData structure
- `generate-boards` CLI command
- Initial dataset: `data/boards.json`

---

## Phase 2: Kaplan-Meier Survival Analysis ✅

**Status:** Completed

**Objectives:**
- Implement Kaplan-Meier estimator for unbiased survival estimates
- Quantify bias in naive statistics
- Establish survival curve visualization

**Completed Tasks:**
- [x] Implement Kaplan-Meier estimator for survival curves
- [x] Add `analyze-censoring` CLI tool with KM analysis
- [x] Identify bias in naive statistics (up to 56% underestimation)
- [x] Implement board-count percentile selection for representative values
- [x] Export KM curves to CSV for visualization
- [x] Persist generator metadata (MAX_TURNS) in dataset

**Key Findings:**
- KM medians differ substantially from naive medians (up to 56% for some features)
- Censoring bias is feature-dependent (worse for "good" feature values)
- P05-P95 range by board count provides robust normalization bounds
- KM curves show clear non-linear relationships (e.g., holes vs. survival)

**Deliverables:**
- `crates/oxidris-stats/src/survival.rs`: KM estimator implementation
- `analyze-censoring` CLI command with `--kaplan-meier` flag
- KM curve CSV exports for visualization
- Documentation of censoring bias

---

## Phase 3: KM-Based Feature Normalization 🔄

**Status:** In Progress

**Objectives:**
- Design and implement two-stage normalization pipeline
- Generate normalization parameters from survival data
- Remove duplicate features
- Implement KMBasedEvaluator

**Completed Tasks:**
- [x] Design P05-P95 robust KM normalization algorithm
- [x] Implement two-stage transform and normalize pipeline
- [x] Separate transform mapping (raw → KM median) from normalization range
- [x] Add helper method `transform_and_normalize()`
- [x] Generate normalization parameters with `--normalization-output` flag
- [x] Use BTreeMap for deterministic JSON ordering
- [x] Document design rationale and usage patterns

**In Progress:**
- [ ] Remove duplicate features (`*_risk` variants) from `ALL_BOARD_FEATURES`
  - `top_out_risk` → use `max_height_penalty`
  - `center_top_out_risk` → use `center_columns_penalty`
  - `deep_well_risk` → use `well_depth_penalty`
  - Location: `crates/oxidris-ai/src/board_feature/mod.rs`

- [ ] Implement `KMBasedEvaluator` in oxidris-ai crate
  - Create `crates/oxidris-ai/src/evaluator/km_based.rs`
  - Load normalization JSON
  - Implement `evaluate()` with two-stage pipeline
  - Initialize weights with `1 / km_range`
  - Integrate with AI infrastructure

**Not Started:**
- [ ] Unit tests for `transform_and_normalize()`
- [ ] Integration tests with genetic algorithm
- [ ] Performance benchmarks vs. legacy evaluators
- [ ] Validate learned weights are interpretable

**Deliverables:**
- `NormalizationParams` data structure with two-stage design
- `generate-normalization` Make target
- Normalization parameters: `data/normalization_params.json`
- `KMBasedEvaluator` implementation
- Documentation updates

---

## Phase 4: Score Features 📋

**Status:** Not Started

**Objectives:**
- Extend data collection to capture score metrics
- Implement score-based normalization
- Design reward function for survival/score trade-offs
- Enable multi-objective optimization

**Planned Tasks:**
- [ ] Extend `SessionData` to capture score fields
  - `final_score`: Total game score
  - `lines_cleared`: Breakdown by clear type (single, double, triple, tetris)
  - `score_per_turn`: Score efficiency metric
  - Location: `crates/oxidris-cli/src/data.rs`

- [ ] Generate new dataset with score metrics
  - Re-run data generation with updated SessionData
  - Ensure sufficient diversity in score outcomes

- [ ] Implement score-based normalization
  - Transform: `feature_value → expected_final_score`
  - Requires Kaplan-Meier-like analysis for score outcomes
  - Alternative: use reward function directly

- [ ] Design reward function
  - `reward = w_survival * (turns/MAX_TURNS) + w_score * (score/MAX_SCORE)`
  - Allow user-configurable weights for play styles
  - Consider: survival-focused, balanced, score-focused

- [ ] Multi-objective optimization
  - Option 1: Weighted reward function (simple)
  - Option 2: Multi-objective GA with Pareto fronts
  - Option 3: Separate models, combine at decision time

**Open Questions:**
- Should score features use separate normalization or unified reward?
- How to balance survival vs. score in fitness function?
- Should we train multiple evaluators for different play styles?

**Deliverables:**
- Extended `SessionData` with score fields
- Score-based normalization algorithm
- Reward function implementation
- Multi-objective training framework

---

## Phase 5: Structure Features Analysis 📋

**Status:** Not Started

**Objectives:**
- Validate that structure features correlate with survival
- Explore alternative metrics if correlation is weak
- Refine feature set based on analysis

**Planned Tasks:**
- [ ] Analyze correlation of structure features with survival
  - Compute Pearson/Spearman correlation: feature value ↔ survival time
  - Visualize KM curves for structure features
  - Identify features with weak/non-monotonic relationships

- [ ] Decide on normalization approach per feature
  - High correlation → use survival-based KM (current approach)
  - Low correlation → explore alternatives (see below)

- [ ] Alternative metric: Placement flexibility
  - Define: "number of valid placements for next N pieces"
  - Collect data: simulate N-step lookahead from each board state
  - Normalize: `feature_value → expected_placement_count`
  - Rationale: directly measures board "quality" for future moves

- [ ] Alternative metric: Hole-generation risk
  - Define: "expected turns until holes appear"
  - Collect data: simulate from each state until first hole
  - Normalize: `feature_value → expected_turns_until_hole`
  - Rationale: captures structure quality impact

- [ ] Refine feature set
  - Remove features with weak predictive power
  - Add new features if gaps identified
  - Re-generate normalization parameters

**Structure Features to Validate:**
- `surface_bumpiness_penalty`
- `row_transitions_penalty`
- `column_transitions_penalty`
- `well_depth_penalty` (known trade-off feature)
- `total_height_penalty`

**Open Questions:**
- What correlation threshold distinguishes "strong" vs. "weak"?
- Is placement flexibility computationally feasible for training?
- Should trade-off features (well depth) use special handling?

**Deliverables:**
- Correlation analysis results
- Decision document: which features use which normalization
- Alternative metric implementations (if needed)
- Refined feature set

---

## Phase 6: Advanced Techniques 📋

**Status:** Not Started (Future)

**Objectives:**
- Explore advanced evaluation techniques beyond linear combination
- Enable context-aware evaluation
- Support multiple play styles

**Potential Directions:**

### 6.1: Context-Dependent Evaluation
- **Early game**: Emphasize speed, score, and setup
- **Mid game**: Balance survival and score
- **Late game**: Prioritize survival, defensive play
- **Implementation**: Different weight sets per game phase, or phase as additional feature

### 6.2: Multiple Play Styles
- **Defensive**: Maximize survival, minimize risk
- **Balanced**: Trade-off survival and score
- **Aggressive**: Maximize score, accept higher risk
- **Implementation**: Train separate evaluators or parameterize single evaluator

### 6.3: Non-Linear Feature Interactions
- **Polynomial features**: Cross-terms like `holes × height`
- **Neural network**: Replace linear combination with MLP
- **Decision trees**: Capture complex interactions
- **Rationale**: Some features interact (e.g., holes are worse when height is high)

### 6.4: Piece Sequence Awareness
- **Context**: Consider upcoming pieces (e.g., I-piece coming → value wells)
- **Implementation**: Extend features to include piece sequence info
- **Challenge**: Significantly increases state space

### 6.5: Deep Learning Integration
- **CNN/Transformer**: Learn features directly from board representation
- **Hybrid**: Use KM-based features + learned features
- **Challenge**: Requires large datasets, less interpretable

**Open Questions:**
- Which directions provide best ROI?
- How to maintain interpretability with advanced techniques?
- What dataset size is needed for deep learning?

---

## Success Metrics

### Phase 3 (Current)
- KMBasedEvaluator achieves ≥ legacy heuristic evaluator survival time
- Learned weights are interpretable (high weight = high km_range features)
- Genetic algorithm converges within reasonable iterations (<1000 generations)

### Phase 4
- Multi-objective evaluator achieves Pareto-optimal survival/score trade-offs
- Score-focused evaluator achieves ≥2x score of survival-focused evaluator
- Reward function weights allow intuitive play style control

### Phase 5
- Structure features show clear correlation with survival (|r| > 0.3)
- Alternative metrics (if implemented) improve prediction accuracy by ≥10%
- Refined feature set reduces redundancy without losing performance

### Phase 6
- Context-aware evaluation improves survival by ≥20% vs. static weights
- Non-linear models capture interactions not visible in linear model
- Deep learning achieves state-of-the-art performance (if pursued)

---

## Timeline Estimates

- **Phase 3 (Current)**: 1-2 weeks
  - Feature consolidation: 1-2 days
  - Evaluator implementation: 3-5 days
  - Testing and validation: 3-5 days

- **Phase 4**: 2-3 weeks
  - Data collection extension: 3-5 days
  - Score normalization: 5-7 days
  - Multi-objective optimization: 5-7 days

- **Phase 5**: 1-2 weeks
  - Correlation analysis: 2-3 days
  - Alternative metrics (if needed): 5-10 days

- **Phase 6**: Open-ended (research phase)
  - Each sub-direction: 1-4 weeks

---

## Dependencies

```
Phase 1 → Phase 2 → Phase 3 → Phase 4
                              ↘ Phase 5 → Phase 6
```

- Phase 4 and 5 can proceed in parallel after Phase 3
- Phase 6 requires insights from Phases 4 and 5

---

## References

- [Evaluator Design](../design.md) - Overall architecture
- [Feature Normalization](../feature-normalization.md) - Algorithm details
- [KM-Based Evaluator README](./README.md) - Current status and usage