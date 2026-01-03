# Oxidris Documentation

Welcome to the Oxidris documentation. This is a Tetris AI project that uses statistical analysis for intelligent game-playing.

## Documentation

### Evaluator System
- **[Evaluator Overview](./evaluator/README.md)** - Evaluation system introduction
- **[Current Status](./evaluator/current-status.md)** - Implementation details, issues, improvement plans
- **[KM Feature Transform](./evaluator/km-feature-transform/README.md)** - KM-based feature transformation project
- **[Roadmap](./evaluator/km-feature-transform/roadmap.md)** - Phase 1-6 development plan

### Engine Implementation
- **[Implementation Notes](./engine/implementation-notes.md)** - Engine details, simplifications, and limitations

### Developer Resources
- **[AGENTS.md](../AGENTS.md)** - Development guidelines for AI assistants
- **[Project README](../README.md)** - Quick start and project overview

## Quick Start

```bash
# Generate and analyze data
make generate-board-data
make analyze-censoring-km
make generate-normalization
```

See [Makefile](../Makefile) or run `make help` for common commands.

## Code Locations

- **`oxidris-engine`** - Game engine (`crates/oxidris-engine/`)
- **`oxidris-ai`** - AI evaluation and features (`crates/oxidris-ai/`)
- **`oxidris-stats`** - Kaplan-Meier estimator (`crates/oxidris-stats/`)
- **`oxidris-cli`** - Data structures and tools (`crates/oxidris-cli/`)

## External References

- [Kaplan-Meier Estimator](https://en.wikipedia.org/wiki/Kaplan%E2%80%93Meier_estimator)
- [Survival Analysis](https://en.wikipedia.org/wiki/Survival_analysis)
- [Tetris SRS](https://tetris.wiki/Super_Rotation_System)
