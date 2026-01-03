# Oxidris

A data-driven Tetris AI system that uses survival analysis and statistical methods to create intelligent game-playing agents.

## Overview

This project implements a Tetris game engine and AI evaluation system with a focus on **survival-based, data-driven approaches**. The core innovation is using **Kaplan-Meier survival analysis** to create non-linear feature transformations that properly handle right-censored data (games that survive beyond the observation window).

### Key Features

- **Game Engine**: Fast, accurate Tetris implementation with standard SRS rotation
- **AI Evaluators**: Multiple evaluation strategies including legacy heuristics and KM-based learned evaluators
- **Survival Analysis**: Statistical tools for analyzing game data with proper censoring handling
- **Training Infrastructure**: Genetic algorithm framework for weight optimization
- **Data Analysis Tools**: CLI tools for data generation, analysis, and visualization

## Project Structure

```
oxidris/
├── crates/
│   ├── oxidris-engine/    # Core Tetris game engine
│   ├── oxidris-ai/        # AI evaluation and placement selection
│   ├── oxidris-stats/     # Statistical analysis (Kaplan-Meier, etc.)
│   └── oxidris-cli/       # Command-line tools and data structures
├── docs/                  # Design documentation and guides
├── data/                  # Generated datasets and parameters
├── models/                # Trained model weights (future)
└── Makefile              # Common build and analysis tasks
```

## Quick Start

### Prerequisites

- Rust 1.80+ (edition 2024)
- Cargo

### Build

```bash
cargo build --release
```

### Generate Training Data

```bash
# Generate board states from diverse evaluators
cargo run --release -- generate-boards --output data/boards.json

# Analyze with Kaplan-Meier survival analysis
cargo run --release -- analyze-censoring data/boards.json --kaplan-meier

# Generate normalization parameters for features
make generate-normalization
```

### Play the Game

```bash
# Manual play (coming soon)
cargo run --release -- play

# AI auto-play (coming soon)
cargo run --release -- auto-play --evaluator km-based
```

## Documentation

Detailed documentation is available in the `docs/` directory:

- **[docs/README.md](docs/README.md)** - Documentation index and navigation
- **[docs/evaluator_design.md](docs/evaluator_design.md)** - Evaluator architecture and design philosophy
- **[docs/feature_normalization.md](docs/feature_normalization.md)** - Feature normalization algorithm details
- **[AGENTS.md](AGENTS.md)** - Instructions for AI assistants working on this project

## Key Concepts

### Survival-Based Evaluation

Instead of using intuition-based heuristics, this project:

1. **Collects data** from actual gameplay with diverse strategies
2. **Analyzes survival** using Kaplan-Meier estimation to handle censored data
3. **Transforms features** using data-driven, non-linear mappings (raw value → survival time)
4. **Learns weights** via genetic algorithms to optimize survival

### Two-Stage Normalization

Features undergo a two-stage transformation:

1. **Transform**: Raw value → KM median (survival time in turns)
2. **Normalize**: KM median → 0-1 range (for consistent weighting)

This keeps intermediate values interpretable (they represent expected survival time) while enabling effective weight learning.

### Example

```
holes=0  → KM median = 322.8 turns → normalized = 1.0 (excellent)
holes=3  → KM median = 177.5 turns → normalized = 0.54 (okay)
holes=10 → KM median = 50.0 turns  → normalized = 0.14 (bad)
```

## Development Status

### Completed

- ✅ Core game engine with SRS rotation
- ✅ Multiple legacy evaluators (heuristic-based)
- ✅ Data generation with diverse play styles
- ✅ Kaplan-Meier survival analysis implementation
- ✅ P05-P95 robust feature normalization design
- ✅ CLI tools for analysis and parameter generation

### In Progress

- 🔄 KM-based evaluator implementation
- 🔄 Feature consolidation (removing duplicates)
- 🔄 Integration with genetic algorithm training

### Planned

- 📋 Score-based features and multi-objective optimization
- 📋 Structure feature validation and alternative metrics
- 📋 Context-aware evaluation (game phase, piece sequence)
- 📋 Multiple play styles (speed vs. score vs. survival)

## Contributing

This is an active research/development project. Key principles:

1. **Data-driven**: Use statistics, not intuition
2. **Interpretable**: Transformations should have clear meanings
3. **Extensible**: Easy to add features and objectives
4. **Well-documented**: Keep docs synchronized with code

When making changes to the evaluator system, please update the relevant documentation in `docs/` (see `AGENTS.md` for guidelines).

## License

[License information to be added]

## Contact

[Contact information to be added]