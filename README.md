# Oxidris

A playable Tetris game with AI players that learn through statistical analysis and genetic algorithms.

> **Note for AI Assistants:** This README is for human users. AI assistants should start with [AGENTS.md](AGENTS.md) for project-specific guidelines and documentation structure.

## Overview

This project implements a playable Tetris game with an AI training and evaluation system. You can play the game yourself or watch trained AI models play automatically. The AI learns to play Tetris by optimizing feature weights through genetic algorithms. What constitutes "skilled play" depends on the training objective - models can be optimized for survival time, game score, or a balance of both.

## Features

- **Human Play**: Play Tetris with keyboard controls
- **AI Auto-Play**: Watch trained AI models play automatically
- **AI Training**: Train new models using genetic algorithms on gameplay data
- **Statistical Analysis**: Analyze feature distributions, visualize board states, and perform survival analysis (Kaplan-Meier)

## Design Principles

Key principles guiding this project:

1. **Data-driven**: Use statistics, not intuition
2. **Interpretable**: Keep transformations meaningful
3. **Well-documented**: Update docs with code changes

## Quick Start

```sh
# Build the project
cargo build --release

# Play the game yourself
make play

# Watch AI play automatically
make auto-play-aggro
```

For more commands (training, analysis, data generation), run `make help`.

See [documentation](docs/README.md) for detailed guides.

## Documentation

For detailed documentation, architecture guides, and development status:

- **[Documentation Hub](docs/README.md)** - Complete documentation navigation and current project status
- **[AGENTS.md](AGENTS.md)** - Development guidelines for AI assistants

## Contributing

We welcome contributions! Please see:

- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute (for humans)
- **[AGENTS.md](AGENTS.md)** - Development guidelines (for AI assistants)
- **[Documentation](docs/)** - Full project documentation

## Technical Overview

### Architecture

Oxidris is built around three core systems:

- **Game Engine** - Tetris implementation with standard mechanics (some simplifications)
- **Evaluator** - Board state evaluation and AI decision-making
- **Training** - Genetic algorithm for optimizing AI behavior

See [Architecture Documentation](docs/architecture/README.md) for detailed system design and component interactions.

### Project Structure

```text
oxidris/
├── crates/         # Rust workspace with engine, evaluator, training, stats, and CLI
├── docs/           # Architecture and development documentation
├── models/         # Trained AI model weights
└── Makefile        # Common commands (play, train, analyze)
```

See [Documentation Hub](docs/README.md) for detailed code structure and documentation navigation.

### Game Mechanics

This implementation uses standard Tetris mechanics (10×20 grid, 7-bag randomizer, hold system) with a simplified rotation system. See [Engine Documentation](docs/architecture/engine/README.md) for implementation details.

## License

[License information to be added]

## Contact

[Contact information to be added]
