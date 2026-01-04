# Contributing to Oxidris

Thank you for your interest in contributing to Oxidris!

This is a **hobby project** focused on exploring statistical analysis and AI techniques for Tetris. Practical utility and academic rigor are not primary goals - the focus is on learning and experimentation.

This document provides guidelines for contributing to Oxidris, including how to build the project, documentation standards, and contribution workflow.

- **Document type**: Reference
- **Purpose**: Guidelines for contributing to Oxidris project
- **Audience**: Human contributors (primary), AI assistants (reference)
- **When to read**: Before making your first contribution
- **Prerequisites**: None
- **Related documents**: [AGENTS.md](AGENTS.md) (for AI assistants), [README.md](README.md) (project overview)

## Getting Started

### Prerequisites

- Rust (latest stable)
- Basic understanding of Tetris mechanics
- Familiarity with Git

### Building the Project

```bash
# Clone the repository
git clone <repository-url>
cd tetris

# Build
cargo build --release

# Run tests
cargo test

# Play the game
make play

# Auto-play with AI
make auto-play-aggro
```

## Project Structure

```text
tetris/
├── crates/
│   ├── oxidris-engine/    # Tetris game engine
│   ├── oxidris-evaluator/ # AI evaluation system
│   ├── oxidris-training/  # Training system (GA, weights)
│   ├── oxidris-stats/     # Statistical analysis tools
│   └── oxidris-cli/       # Command-line interface
├── docs/
│   ├── architecture/      # Design documentation
│   ├── projects/          # Active project documentation
│   └── future-projects.md # Improvement proposals
├── models/ai/             # Trained AI models
└── data/                  # Generated datasets
```

## How to Contribute

### Reporting Issues

- Check existing issues first
- Provide clear description and reproduction steps
- Include relevant code snippets or error messages

### Suggesting Improvements

New improvement ideas are welcome! Before proposing:

1. Check `docs/future-projects.md` to see if it's already listed
2. Consider which system it affects (evaluator/training/engine)
3. Think about scope and feasibility

Proposals can be as simple as:

- What problem does it solve?
- What's the proposed approach?
- Estimated complexity (Small/Medium/Large)

### Code Contributions

#### Before Starting

For non-trivial changes:

1. Open an issue to discuss the approach
2. Understand which system you're modifying
3. Read relevant architecture documentation

#### Development Workflow

1. **Fork and branch**

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes**
   - Follow existing code style
   - Add tests for new functionality
   - Update documentation

3. **Test locally**

   ```bash
   cargo test
   cargo clippy
   ```

4. **Commit**
   - Write clear commit messages
   - Reference issues if applicable
   - Keep commits focused

5. **Submit Pull Request**
   - Describe what changed and why
   - Link to related issues
   - Be open to feedback

### Documentation Contributions

Documentation improvements are highly valued!

- Fix typos, broken links, unclear explanations
- Add examples or clarifications
- Update outdated information

**Documentation structure:**

- `docs/architecture/` - System design (evaluator, training, engine)
- `docs/projects/` - Active project documentation
- `docs/future-projects.md` - Improvement proposals

When updating docs, keep each system's documentation in its proper location (don't mix evaluator and training content, for example).

## Current Focus

**Active Project:** KM-Based Survival Feature Normalization

We're currently working on improving survival feature normalization using Kaplan-Meier survival analysis. See `docs/projects/km-feature-transform/` for details.

Contributions related to this project are especially welcome!

## Code Style

- Follow Rust conventions (rustfmt, clippy)
- Prefer clarity over cleverness
- Add comments for non-obvious logic
- Keep functions focused and small

## Documentation Style

### Documentation Validation

Before committing documentation changes, run the validation script:

```bash
./scripts/validate-docs.sh
```

**What it checks:**

1. **Markdown style** - Enforces [markdownlint](https://github.com/DavidAnson/markdownlint) rules (headings, lists, code blocks, spacing)
2. **Metadata blocks** - Verifies all documentation files have required metadata
3. **Document types** - Confirms types match official taxonomy (Reference, How-to guide, Tutorial, Explanation)
4. **Internal links** - Detects broken links between markdown files

**Exit codes:**

- `0` - All checks passed (or only warnings)
- `1` - One or more errors found

**When checks fail:**

- Fix reported errors
- Warnings are acceptable but should be reviewed
- Re-run the script to confirm fixes

**Individual tools (if needed):**

```bash
# Run only markdownlint
markdownlint .

# Fix auto-fixable markdown issues
markdownlint . --fix
```

**Configuration:** The project uses `.markdownlint.json` for linting rules and `.markdownlintignore` to exclude build artifacts.

**Editor integration:** Consider using a Markdown linter plugin:

- VS Code: [markdownlint extension](https://marketplace.visualstudio.com/items?itemName=DavidAnson.vscode-markdownlint)
- Vim/Neovim: [ale](https://github.com/dense-analysis/ale) with markdownlint
- Other editors: Check for markdownlint integration

### Documentation Metadata

Each documentation file should include a metadata block immediately after the title:

- **Document type**: [Reference / How-to guide / Tutorial / Explanation]
- **Purpose**: [One-line description of what this document does]
- **Audience**: [Who should read this - AI assistants, contributors, users, etc.]
- **When to read**: [Specific situations when this document becomes relevant]
- **Prerequisites**: [What to read first, or "None"]
- **Related documents**: [Links to related documents with brief descriptions]

Follow the metadata block with a one-paragraph summary of the document's contents.

**Guidelines**:

- Keep "Purpose" to one line
- Be specific about "Audience" and "When to read"
- Link to prerequisites and related documents using relative paths
- Choose one document type based on the [Divio Documentation System](https://documentation.divio.com/):
  - **Reference**: Information-oriented (rules, APIs, specifications)
  - **How-to guide**: Problem-oriented (step-by-step instructions)
  - **Tutorial**: Learning-oriented (lessons for beginners)
  - **Explanation**: Understanding-oriented (concepts, design decisions)

**Example**:

```markdown
# Documentation Guidelines

This document defines where to place different types of documentation, how to avoid duplication, and which files to update when making changes.

- **Document type**: Reference
- **Purpose**: Rules for organizing and maintaining Oxidris documentation
- **Audience**: AI assistants and human contributors
- **When to read**: Before creating, moving, or reorganizing documentation
- **Prerequisites**: Read [AGENTS.md](AGENTS.md) for project structure overview
- **Related documents**: [when-to-ask.md](docs/ai/when-to-ask.md) (when to ask about documentation changes)
```

## Design Philosophy

1. **Data-driven**: Use statistical analysis, not hand-tuned parameters
2. **Interpretable**: Keep transformations understandable
3. **Experimental**: Try new approaches, learn from results
4. **Simple first**: Start with simple solutions, add complexity only when needed

## Getting Help

- Read architecture documentation for system understanding
- Check existing issues and discussions
- Open an issue with your question

## License

[To be determined]

## Acknowledgments

This project is for learning and experimentation. Contributions of all sizes are appreciated!
