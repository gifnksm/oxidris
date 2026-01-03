# Agent Instructions

## Project Context

This is Oxidris, a Tetris AI project that uses statistical analysis for board evaluation. 

**Current State:**
- Feature-based evaluation with automated P05-P95 percentile normalization from gameplay data
- Genetic algorithm for weight optimization
- Improving to survival-time-based transformations using Kaplan-Meier analysis (Phase 3)

**Future:** Score-based optimization and multi-objective strategies (Phase 4+)

## Documentation Structure

```
docs/
├── README.md                          # Documentation hub
├── evaluator/                         # Evaluator system
│   ├── README.md                      # Overview
│   ├── current-status.md              # Current implementation and issues
│   └── km-feature-transform/          # KM-based feature transformation
│       ├── README.md                  # Overview and status
│       ├── design.md                  # Algorithm details
│       └── roadmap.md                 # Phase 1-6 plan
└── engine/                            # Engine implementation
    └── implementation-notes.md        # Engine details and limitations
```

## When to Read Documentation

### Always read when:
- Discussing evaluator design or feature normalization
- Planning new features or modifications
- Answering questions about design decisions
- Implementing evaluation-related code
- Discussing roadmap or next steps
- Making changes to the evaluator system
- Discussing game engine mechanics or rotation system
- Answering questions about SRS implementation or limitations

### Start here:
1. **`docs/README.md`** - Understand overall structure and navigation
2. **`docs/evaluator/README.md`** - Evaluator system overview
3. **`docs/evaluator/current-status.md`** - Current implementation and issues
4. **`docs/evaluator/km-feature-transform/`** - KM feature transformation project
5. **`docs/engine/implementation-notes.md`** - Engine limitations (simplified SRS, etc.)

### You don't need to read when:
- Fixing unrelated bugs
- Making trivial code changes
- Discussing topics unrelated to evaluator or engine mechanics

## Key Design Principles

1. **Data-driven**: Use statistical analysis of actual gameplay data
2. **Trade-offs via learning**: Let genetic algorithms discover optimal weights, don't hard-code rules
3. **Interpretable**: Transformations should have clear meaning
4. **Two-stage transform/normalize** (Phase 3 goal): Feature value → survival time → 0-1 range

## Current Status (Phase 3)

- ✅ Feature-based evaluators with automated P05-P95 calculation
- ✅ Data generation and KM survival analysis tools
- 🔄 Survival-based transformation (feature value → survival time)
- 🔄 KM-based evaluator implementation
- 📋 Integration with GA training and benchmarking

See `docs/evaluator/current-status.md` for detailed status.

## Documentation Maintenance

**When making changes, you MUST update documentation in the same commit.**

### Update `docs/evaluator/current-status.md` when:
- Changing current implementation details
- Discovering new issues or limitations
- Updating improvement plans
- Adding/removing features

### Update `docs/evaluator/km-feature-transform/design.md` when:
- Modifying KM-based normalization algorithm
- Changing data structures (`NormalizationParams`, `NormalizationRange`, etc.)
- Adding usage examples or integration patterns

### Update `docs/evaluator/km-feature-transform/README.md` when:
- Changing implementation status (completed/in-progress/not-started)
- Updating overview or concept explanation
- Discovering limitations or issues

### Update `docs/evaluator/km-feature-transform/roadmap.md` when:
- Completing phases or tasks
- Making design decisions that affect future phases
- Adding new phases or changing priorities
- Updating timeline estimates

### Update `docs/README.md` when:
- Adding new documentation files or sections
- Completing major milestones (phases)
- Changing project status overview

### Keep documentation synchronized:
- Update docs in the same commit as code changes
- Document design decisions immediately after making them
- Add new issues to "Known Limitations" when discovered
- Move resolved questions from "Open Questions" to "Design Decisions"

## Code Locations

### Core Implementation
- **Data structures**: `crates/oxidris-cli/src/data.rs`
- **Normalization generation**: `crates/oxidris-cli/src/analyze_censoring.rs`
- **Features**: `crates/oxidris-ai/src/board_feature/mod.rs`
- **KM estimator**: `crates/oxidris-stats/src/survival.rs`
- **Board analysis**: `crates/oxidris-engine/src/board_analysis.rs`

### Evaluators
- **Legacy evaluators**: `crates/oxidris-ai/src/evaluator/`
- **KM-based evaluator**: `crates/oxidris-ai/src/evaluator/km_based.rs` (planned)

## Quick Reference

### Feature Categories
1. **Survival Features**: Directly affect game termination (holes, height)
2. **Structure Features**: Affect placement flexibility (bumpiness, transitions)
3. **Score Features**: Directly contribute to score (line clears) - Phase 4

### Normalization Approach
- **P05-P95 robust scaling**: Use 5th and 95th percentiles by board count
- **Two-stage pipeline**: Transform (raw → KM median) then normalize (KM median → 0-1)
- **Interpretable**: KM median = expected survival time in turns

### Current Phase (Phase 3)
Focus on completing KM-based evaluator implementation:
- Remove duplicate `*_risk` features
- Implement `KMBasedEvaluator` struct
- Integrate with genetic algorithm
- Test and benchmark

See roadmap for Phases 4-6 (score features, structure validation, advanced techniques).