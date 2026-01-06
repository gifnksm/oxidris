# Agent Instructions

This file serves as the entry point and quick reference for AI assistants, providing project context, documentation structure, key guidelines, and code locations.

- **Document type**: Reference
- **Purpose**: Central guide for AI assistants working on Oxidris
- **Audience**: AI assistants (primary), human contributors (reference)
- **When to read**: At the start of any conversation about the project
- **Prerequisites**: None (this is the entry point)
- **Related documents**: [CONTRIBUTING.md](CONTRIBUTING.md) (for human contributors), [docs/README.md](docs/README.md) (documentation hub)

## Project Context

> **Important:** This is a **hobby project** focused on learning and experimentation. Practical utility and academic rigor are not primary goals. Future improvements are documented as independent project proposals to be pursued based on interest.

**Development Status:** This project is in active development and not yet published externally. The codebase is internal-only, which means:

- Breaking changes are acceptable and don't require special consideration
- Experimentation and refactoring are encouraged
- Focus is on learning and improvement, not backward compatibility

This is Oxidris, a playable Tetris game with AI players that learn through statistical analysis and genetic algorithms. You can play the game yourself or watch trained AI models play automatically.

## How to Use This Guide

This file (AGENTS.md) serves as the **entry point** for AI assistants working on Oxidris:

- **Project context** - Development status, goals, current focus
- **Navigation hub** - Documentation structure and where to find things
- **Quick reference** - Summary of key guidelines
- **Code index** - Important file locations

**For detailed guidance**:

- **How to organize docs** → [Documentation Guidelines](docs/ai/documentation-guidelines.md)
- **How to conduct reviews** → [Review Process](docs/ai/review-process.md)
- **When to ask first** → [When to Ask](docs/ai/when-to-ask.md)

## Documentation Structure

```text
docs/
├── README.md                          # Documentation hub
├── ai/                                # Guidelines for AI assistants
│   ├── documentation-guidelines.md    # [RULES] How to organize documentation
│   ├── review-process.md              # [PROCESS] How to conduct reviews
│   └── when-to-ask.md                 # [CHECKLIST] When to ask before changes
├── architecture/                      # System design documentation
│   ├── README.md                      # Architecture overview
│   ├── evaluator/                     # Evaluator system
│   │   └── README.md                  # Evaluator overview and architecture
│   ├── training/                      # Training system
│   │   └── README.md                  # GA, fitness functions, training process
│   └── engine/                        # Game engine
│       └── README.md                  # Engine details and limitations (simplified SRS)
├── projects/                          # Active project documentation
│   └── km-feature-transform/          # KM-based survival features (active)
│       ├── README.md                  # Project overview
│       ├── design.md                  # Design and architecture
│       └── roadmap.md                 # Phase-by-phase plan
└── future-projects.md                 # Improvement proposals (all systems)
```

## When to Read Documentation

Always read when:

- Starting a new conversation about the project
- User asks about design decisions, architecture, or roadmap
- Making changes that affect multiple systems
- Uncertain about project structure or conventions
- Implementing features related to documented systems (Evaluator, Training, Engine)
- Proposing new improvements or changes

Start here:

1. **`docs/README.md`** - Documentation hub and navigation
2. **`docs/architecture/README.md`** - Architecture overview
3. **System-specific docs** - Read based on what you're working on:
   - Evaluator: `docs/architecture/evaluator/`
   - Training: `docs/architecture/training/`
   - Engine: `docs/architecture/engine/`
4. **`docs/projects/km-feature-transform/`** - Current active project
5. **`docs/future-projects.md`** - When discussing new improvements

You don't need to read when:

- Making trivial fixes (typos, formatting)
- User asks unrelated questions
- Changes are limited to well-understood, isolated code

## Key Design Principles

See [Design Principles in README](README.md#design-principles) for the project principles that guide all development:

1. **Data-driven**: Use statistics, not intuition
2. **Interpretable**: Keep transformations meaningful
3. **Well-documented**: Update docs with code changes

## Current Focus

**Active Project:** KM-Based Survival Feature Normalization (Phase 4)

- ✅ Phase 1-2: Data generation and KM survival analysis (completed)
- ✅ Phase 3: Infrastructure and trait integration (completed 2026-01-06)
- 📋 Phase 4: Implementation and validation (next)

**Scope:** Survival features (holes, height) only. Other improvements are separate future projects.

See `docs/projects/km-feature-transform/` for details on the current active project.

## Guidelines for AI Assistants

### Communication

- **Language matching**: Always respond in the same language the user used
  - If user writes in Japanese, respond in Japanese
  - If user writes in English, respond in English
  - Match the language for all responses, including technical discussions

### Documentation

- **Distribution**: Rustdoc vs Markdown (source of truth for implementation)
  - **Rustdoc**: Current implementation, design decisions, API usage, trade-offs (single crate/module scope)
  - **Markdown (docs/)**: System-wide architecture (across crates), project context, navigation, future work
  - **Rule**: Implementation details and "why" go in rustdoc; system architecture and navigation go in Markdown
  - **No duplication**: Markdown should link to rustdoc, not duplicate implementation details
- **Organization**: See [Documentation Guidelines](docs/ai/documentation-guidelines.md)
  - Follow the documented structure strictly
  - Don't mix concerns (evaluator/training/engine)
  - Avoid duplication between Markdown and rustdoc
- **Maintenance**: Keep docs synchronized with code changes in the same commit

### Review Process

- **Review process**: See [Review Process](docs/ai/review-process.md)
  - Start with overview, then step-by-step details
  - Show progress indicators (3/5 items)
  - Support interruption and resumption
  - Group large changes into phases
  - Fix minor issues silently, ask about medium issues, stop for major issues

### When to Ask

- **Before making changes**: See [When to Ask](docs/ai/when-to-ask.md)
  - Documentation structure changes
  - Code architecture changes
  - Adding dependencies
  - Changing active project scope
  - **When in doubt, ask first**

### Terminal Tool Usage

- **Git commands**: Always use `--no-pager` flag for git commands that show output
  - Use `git --no-pager diff` instead of `git diff`
  - Use `git --no-pager log` instead of `git log`
  - Use `git --no-pager show` instead of `git show`
  - Without `--no-pager`, the pager (like `less`) interferes with terminal output capture

## Code Locations

### Evaluator system

- **Features**: `crates/oxidris-evaluator/src/board_feature/mod.rs`
- **Board analysis**: `crates/oxidris-evaluator/src/board_analysis.rs`
- **Placement evaluator**: `crates/oxidris-evaluator/src/placement_evaluator.rs`
- **Session evaluators**: `crates/oxidris-evaluator/src/session_evaluator.rs`
- **Turn evaluator**: `crates/oxidris-evaluator/src/turn_evaluator.rs`

### Training system

- **Genetic algorithm**: `crates/oxidris-training/src/genetic.rs`
- **Weight operations**: `crates/oxidris-training/src/weights.rs`
- **Training script**: `crates/oxidris-cli/src/train_ai.rs`
- **Data generation**: `crates/oxidris-cli/src/generate_boards.rs`

### KM-based normalization

- **KM estimator**: `crates/oxidris-stats/src/survival.rs`
- **Data structures**: `crates/oxidris-cli/src/data.rs`
- **Normalization generation**: `crates/oxidris-cli/src/analyze_censoring.rs`

### Models and Data

- **Trained models**: `models/ai/aggro.json`, `models/ai/defensive.json`
- **Training data**: `data/boards.json` (generated, not in repo)
- **Normalization params**: `data/normalization_params.json` (generated, not in repo)

## Quick Reference

### Current Project Status

- **Active:** KM-Based Survival Feature Normalization (Phase 4)
- **Phase 3:** Design complete (2026-01-06) - `MappedNormalized<S>` type, feature naming, coexistence strategy
- **Phase 4:** Implementation - type, FeatureBuilder integration, training tools
- **Focus:** Survival features (holes, height) only
- See [KM Project Docs](docs/projects/km-feature-transform/) for details

### Feature Categories

See [Evaluator Documentation](docs/architecture/evaluator/README.md) for details on:

- Survival Features (directly affect game termination)
- Structure Features (affect placement flexibility)
- Score Features (directly contribute to score)

### Technical Details

- **Evaluator:** [Evaluator System](docs/architecture/evaluator/README.md)
- **Training:** [Training System](docs/architecture/training/README.md)
- **Engine:** [Implementation Notes](docs/architecture/engine/README.md)
- **Future Work:** [Future Projects](docs/future-projects.md)

### Contributing

- **For humans:** See [CONTRIBUTING.md](CONTRIBUTING.md)
- **For AI assistants:** Follow guidelines in [docs/ai/](docs/ai/)
