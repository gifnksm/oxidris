# Oxidris

A Tetris AI system using statistical analysis for board evaluation.

## Overview

This project implements a Tetris game engine and AI evaluation system with genetic algorithm-based weight optimization. Currently improving feature normalization from percentile-based to survival-time-based transformations using Kaplan-Meier analysis.

### Key Components

- **Game Engine**: Tetris implementation with SRS rotation
- **Feature-Based Evaluation**: Board state evaluation with multiple features
- **Genetic Algorithm**: Weight optimization framework
- **Statistical Tools**: Kaplan-Meier survival analysis for data-driven feature transformations

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
├── models/                # Trained model weights (aggro, defensive)
└── Makefile              # Common build and analysis tasks
```

## Quick Start

```bash
# Build
cargo build --release

# Generate data and analyze
cargo run --release -- generate-boards --output data/boards.json
cargo run --release -- analyze-censoring data/boards.json --kaplan-meier
make generate-normalization
```

See [documentation](docs/README.md) for detailed guides.

## Documentation

Detailed documentation is available in the `docs/` directory:

- **[docs/README.md](docs/README.md)** - Documentation hub and navigation
- **[docs/](docs/)** - Full documentation
- **[AGENTS.md](AGENTS.md)** - Development guidelines

See [evaluator documentation](docs/evaluator/) for evaluation system details and [roadmap](docs/evaluator/km-feature-transform/roadmap.md) for development status.

## Contributing

Key principles:

1. **Data-driven**: Use statistics, not intuition
2. **Interpretable**: Keep transformations meaningful
3. **Well-documented**: Update docs with code changes

See [AGENTS.md](AGENTS.md) and [docs/](docs/) for detailed guidelines.

## License

[License information to be added]

## Contact

[Contact information to be added]
