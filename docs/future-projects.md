# Future Improvement Projects

This document lists potential improvement projects beyond the current [KM-Based Survival Features](./projects/km-feature-transform/) work. These are **independent projects** that can be pursued based on interest, learning goals, and available time.

- **Document type**: Reference
- **Purpose**: Catalog of potential future improvements across all systems (evaluator, training, engine)
- **Audience**: Developers, AI assistants, contributors looking for project ideas
- **When to read**: When planning next steps after current work or looking for improvement opportunities
- **Prerequisites**: [Architecture Overview](./architecture/README.md) for system understanding
- **Related documents**: [KM Feature Transform](./projects/km-feature-transform/) (current active project), [Architecture Documentation](./architecture/README.md)

> [!NOTE]
> These are ideas, not commitments. Implementation order will be decided as needed.

---

## Engine Improvements

### Full SRS Rotation System

**Problem:** Current rotation system uses simplified 4-direction kicks instead of official Super Rotation System (SRS):

- No official kick tables (5 test positions per rotation)
- No piece-specific kick patterns
- No T-spin detection or scoring
- Some standard placements are impossible

This means learned strategies don't transfer to real Tetris, and advanced techniques can't be learned.

**Improvement (breaking change):** Implement full SRS according to Tetris guidelines, including kick tables, rotation state tracking, and spin detection. All existing trained models, training data, and benchmarks become invalid. Only pursue if committed to long-term continuation.

**Dependencies:** None (engine-level change).

**Effort:** Large (detailed implementation + retrain everything)

---

## Placement Search Improvements

**Problem:** Current placement search is very simple - just "rotate → move left/right → hard drop". This misses potentially useful placements:

- Soft-drop positions (landing at intermediate heights)
- Multi-step movement sequences (rotate → move → rotate again)
- Tuck moves (rotation after landing)

The limited search space may prevent the AI from reaching beneficial board states.

**Improvement:** Expand the search to include more placement options. Start with soft-drop positions, then consider more complex movement sequences if needed. Need to balance search completeness with computational cost.

**Dependencies:** None, though may interact with evaluation speed (more placements = more evaluations).

**Effort:** Medium (search algorithm redesign)

---

## UI Improvements

### Interactive Replay Viewer

**Problem:** Currently there's no way to review and analyze past game sessions. Understanding how and why games ended requires re-running sessions and watching in real-time. This makes it difficult to:

- Debug unexpected AI behavior in specific game situations
- Learn from AI mistakes or successes
- Compare different AI models on the same game sequence
- Share interesting game sessions with others

**Improvement:** Add a replay viewer subcommand that can load saved game sessions and provide interactive playback:

- Load game session from file (serialize/deserialize game state history)
- Playback controls: play/pause, speed control, frame-by-frame stepping
- Visual display of current board state, statistics, and upcoming pieces
- Optional: feature value overlays, placement decision explanations
- Save game sessions during play/auto-play for later review

This would reuse existing ratatui widgets (BoardDisplay, SessionDisplay, etc.) but add playback control UI.

**Dependencies:** None (uses existing widget infrastructure)

**Effort:** Medium (session serialization + playback UI + controls)

---

## Evaluator / AI Improvements

### Feature Redundancy Analysis and Cleanup

**Problem:** Multiple features measure similar things, wasting computation and complicating interpretation:

- `linear_top_out_risk` ≈ `linear_max_height_penalty`
- `linear_center_top_out_risk` ≈ `linear_center_columns_penalty`
- `linear_deep_well_risk` ≈ `linear_well_depth_penalty`

These duplicates exist because linear normalization couldn't capture non-linearity, so duplicate features with different scaling were added as a workaround. With KM-based transformation, they may no longer be needed.

**Improvement:** Analyze feature correlations, remove redundant features, and validate that the cleaner feature set performs as well or better.

**Dependencies:** Should be done after KM-based survival features (transformation changes feature behavior).

**Effort:** Small-Medium (mostly analysis)

---

### Feature Interaction Modeling (Non-Linear Models)

**Problem:** Linear weighted combination cannot capture feature interactions:

- `holes=5 + height=15` is much worse than linear model suggests (can't clear holes)
- `bumpiness=high + height=low` is manageable
- `bumpiness=high + height=high` is dangerous

Even with KM-based transformation of individual features, the model still combines them linearly.

**Improvement:** Use models that capture interactions:

- **Polynomial features:** Add `holes * height`, `bumpiness * height`, etc.
- **Neural network:** Small network learns interactions automatically
- **Gradient boosted trees:** Decision trees naturally handle interactions

Trade-off between interpretability and performance.

**Dependencies:** Should wait until KM-based features are stable (establishes baseline).

**Effort:** Medium (polynomial features) to Large (neural network/trees)

---

### Multi-Step Lookahead Turn Evaluator

**Problem:** Current turn evaluator uses greedy 1-step lookahead - it only considers the immediate next placement without planning ahead. This limits strategic planning:

- Cannot optimize multi-turn sequences (e.g., back-to-back Tetrises)
- Purely reactive rather than strategic
- May make locally optimal choices that harm future options
- Cannot balance "build I-well now" vs "clear lines now" trade-offs

While 1-step lookahead is fast enough for training (which requires thousands of games), it's suboptimal for actual play.

**Improvement:** Implement multi-step lookahead turn evaluator for use during inference/autoplay:

- **Training mode:** Keep 1-step greedy evaluator (fast, ~100-200 placements/turn)
- **Play mode:** Use 2-4 step lookahead (slower but strategic)
- Algorithms to consider: Beam search (limit branches), MCTS, or alpha-beta pruning
- Handle 7-bag piece sequence probabilistically

The evaluator would use existing learned weights but search deeper to find better move sequences within the current evaluation framework.

Trade-off: Computational cost increases exponentially (2-step: ~10k-40k nodes, 3-step: ~1M+ nodes), so beam search or pruning is essential.

**Note:** This improves planning with the current evaluation function. Advanced techniques like T-spins would require additional work: T-spin detection features, Full SRS implementation, and training methods that can learn multi-step setups.

**Dependencies:** None (extends existing turn evaluator, doesn't change training). May benefit from better placement search first.

**Effort:** Medium-Large (search algorithm complexity + 7-bag integration)

---

### Structure Feature Normalization

**Problem:** Structure features (bumpiness, transitions, well depth) have **indirect** impact on survival - they affect placement flexibility rather than directly causing game over. It's unclear if survival-time-based KM normalization is appropriate for them, or if they need a different approach (e.g., placement-flexibility-based normalization).

**Improvement:** Analyze correlation between structure features and survival time. If correlation is weak, investigate alternative normalization methods tailored to structure quality rather than survival time.

**Dependencies:** Should be done after KM-based survival features (need baseline for comparison).

**Effort:** Medium (analysis-heavy, implementation depends on findings)

---

### Score Optimization and Multi-Objective Training

**Problem:** Current models focus primarily on survival. Score features (`line_clear_bonus`, `i_well_reward`) exist but aren't well-integrated. The trade-off between survival and score isn't systematically explored. Real Tetris play requires balancing both objectives.

Each model optimizes a single fitness function that manually combines multiple objectives (survival, score) into one scalar value. This prevents systematic exploration of trade-offs:

- Currently only 2 fixed models: Aggro (balanced) and Defensive (survival-focused)
- Creating intermediate play styles requires hand-designing new fitness functions
- No way to answer "what's the best survival/score balance?" objectively
- Can't generate a spectrum of models from defensive to aggressive

**Improvement:** Develop multi-objective optimization framework that treats survival and score as separate objectives:

- Generate Pareto front of optimal survival/score trade-offs
- Produce a spectrum of models with different play styles automatically
- Allow users to select models based on preferred trade-off point
- Remove need to manually design fitness function weights
- Use score-based normalization for score features (similar to KM approach but for expected score)

**Dependencies:** Beneficial to complete KM-based survival features first (stable survival baseline).

**Effort:** Medium-Large

---

## Training Improvements

### Fitness Function Design

**Problem:** Current fitness functions use manually-chosen formulas and coefficients without formal justification:

**Aggro Evaluator:**

- Formula: `(efficiency + (1 - max_height_penalty) + (1 - peak_max_height_penalty)) / 3`
- Efficiency: weighted line clears (weights `[0,1,3,5,8]` for 0-4 lines) normalized by theoretical maximum
- Height penalties: quadratic penalties for average and peak max height above cutoff (4.0)
- Survival time: indirectly penalized (early termination assumes worst-case height for remaining turns)

**Defensive Evaluator:**

- Formula: `((1 - max_height_penalty) + (1 - peak_max_height_penalty)) / 2`
- No efficiency component (pure height minimization)
- Height penalties: same quadratic approach but with cutoff 0.0 (penalizes all height)
- Survival time: indirectly penalized (same mechanism as Aggro)

**Issues:**

- Coefficients (line clear weights `[0,1,3,5,8]`, height cutoffs, penalty scaling) chosen by intuition
- Unclear if these formulas effectively capture desired play styles (aggressive vs defensive)
- No theoretical or empirical validation of formula design
- Different evaluators use inconsistent design principles

The fitness function defines "what is good play", directly affecting what the AI learns. Better fitness design could significantly improve play quality.

**Improvement:** Systematically design fitness functions based on desired play style:

- Analyze current fitness behavior (what does it actually reward/penalize in practice?)
- Design principled formulas based on game theory or domain knowledge
- Use data-driven approach (fit formula to human play or desired outcomes)

**Dependencies:** None (can be done independently, though may relate to "Score Optimization and Multi-Objective Training").

**Effort:** Medium (requires experimentation and analysis)

---

### GA Hyperparameter Tuning

**Problem:** Current GA parameters (population size: 30, tournament size: 2, mutation rates, etc.) are manually chosen without justification. There's no evidence these are optimal for convergence speed or solution quality.

**Improvement:** Systematically search the parameter space (grid search, random search, or Bayesian optimization) to find better settings. Measure impact on final fitness, convergence speed, and consistency across runs.

**Dependencies:** None - can be done anytime. May be beneficial before major changes to establish a good baseline.

**Effort:** Large (requires many GA training runs)

---

### Advanced Training Techniques

**Problem:** Current training uses simple genetic algorithm on static data from weak AIs. More sophisticated techniques might improve learning.

**Potential directions:**

- **Context-dependent evaluation:** Different strategies for early/mid/late game
- **Curriculum learning:** Train progressively on increasingly difficult boards
- **Self-play:** Generate training data from best models, iterate
- **Ensemble methods:** Combine multiple models for robustness

Each direction addresses different aspects of training quality and model robustness.

**Dependencies:** Best done after baseline is stable.

**Effort:** Variable (each direction is its own project)

---

## Analysis Tools

### Board-Level Feature Analysis and Visualization

**Problem:** Current analysis tools focus on aggregate statistics across all boards. There's no easy way to:

- Inspect feature values for individual board states
- Understand why AI chose a specific placement
- Debug evaluation behavior on specific scenarios
- Compare feature values across similar board states

The existing `analyze-board-features` TUI provides basic board state browsing, but lacks detailed placement-level analysis and decision explanation.

**Improvement:** Enhanced analysis tools for board-level inspection:

- Interactive board state browser with feature breakdowns
- Placement comparison view (show all candidates and their scores)
- Feature contribution visualization (which features influenced the decision most)
- Historical playback with feature evolution over time
- Export capabilities for detailed analysis in external tools

**Use Cases:**

- Debugging unexpected AI behavior
- Understanding model decision-making
- Feature engineering validation
- Creating test cases for specific scenarios
- Comparing different models on the same board states

**Dependencies:** None (analysis/visualization only)

**Effort:** Medium (UI/UX design + implementation)

---

## Choosing What to Work On

Consider these factors when deciding which project to pursue:

1. **Interest:** What sounds fun to work on right now?
2. **Learning:** What new technique do you want to learn?
3. **Impact:** Can improvement be clearly measured?
4. **Effort:** How much time is available?
5. **Dependencies:** Are prerequisites complete?
6. **Risk:** What happens if it doesn't work out?

**Examples:**

- Want quick wins? → Feature Redundancy or GA Tuning
- Want to learn ML techniques? → Feature Interactions or Advanced Training
- Want better gameplay? → Placement Search or Score Optimization
- Want standards compliance? → Full SRS (but high cost)

---

## See Also

- [KM Feature Transform](./projects/km-feature-transform/) - Current project documentation
- [AGENTS.md](../AGENTS.md) - Development guidelines
