# Oxidris Documentation

This is the documentation hub for Oxidris. For project introduction and quick start, see the main [README](../README.md).

- **Document type**: Reference
- **Purpose**: Central navigation point for all Oxidris documentation
- **Audience**: Developers, AI assistants, contributors
- **When to read**: Starting point for finding any documentation
- **Prerequisites**: None (this is the entry point)
- **Related documents**: [README](../README.md) (project introduction), [AGENTS.md](../AGENTS.md) (development guidelines), [CONTRIBUTING.md](../CONTRIBUTING.md) (contribution guide)

## Documentation Structure

### Architecture Documentation

System design documentation for all three core systems.

- **[Architecture Overview](./architecture/README.md)** - System boundaries, data flow, and design decisions
- **[Evaluator System](./architecture/evaluator/README.md)** - Board evaluation (features, normalization)
- **[Training System](./architecture/training/README.md)** - Weight optimization (GA, fitness functions)
- **[Engine Implementation](./architecture/engine/README.md)** - Game mechanics (simplified SRS)

### AI Assistant Guidelines

Guidelines for AI assistants working on this project.

- **[Documentation Guidelines](./ai/documentation-guidelines.md)** - How to organize and maintain documentation
- **[Review Process](./ai/review-process.md)** - How to present changes for review
- **[When to Ask](./ai/when-to-ask.md)** - When to ask for confirmation before making changes

### Active Projects

Time-limited projects currently in progress.

- **[KM-Based Feature Transform](./projects/km-feature-transform/)** - Survival feature normalization using Kaplan-Meier analysis
  - [Overview](./projects/km-feature-transform/README.md) - Goals and approach
  - [Design](./projects/km-feature-transform/design.md) - Architecture details
  - [Roadmap](./projects/km-feature-transform/roadmap.md) - Phase-by-phase plan

### Future Improvements

- **[Future Projects](./future-projects.md)** - Improvement proposals across all systems

## Current Development Status

**Active Project**: KM-Based Feature Transform (Phase 3-4)

The project is currently integrating Kaplan-Meier survival analysis for feature normalization:

- Integrating KM-based normalization into the `BoardFeatureSource` trait
- Applying survival-time-based transformations to survival features (holes, height)
- Validating improvements over linear normalization

See the [KM Feature Transform Roadmap](./projects/km-feature-transform/roadmap.md) for detailed phase-by-phase status.

## Getting Started

For installation, quick start commands, and usage examples, see the main [README](../README.md).

For development guidelines and project structure, see [AGENTS.md](../AGENTS.md).

## Contributing

- **[Contributing Guide](../CONTRIBUTING.md)** - How to contribute (for humans)
- **[AGENTS.md](../AGENTS.md)** - Main entry point for AI assistants
- **[AI Guidelines](./ai/)** - Detailed guidelines for AI assistants

## External References

- [Kaplan-Meier Estimator](https://en.wikipedia.org/wiki/Kaplan%E2%80%93Meier_estimator) - Survival analysis for censored data
- [Survival Analysis](https://en.wikipedia.org/wiki/Survival_analysis) - Statistical methods for time-to-event data
- [Tetris SRS](https://tetris.wiki/Super_Rotation_System) - Official rotation system (note: we use simplified version)
- [Genetic Algorithms](https://en.wikipedia.org/wiki/Genetic_algorithm) - Evolutionary optimization techniques
