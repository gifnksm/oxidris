# Documentation Guidelines

This document defines where to place different types of documentation, how to avoid duplication, and which files to update when making changes.

- **Document type**: Reference
- **Purpose**: Rules for organizing and maintaining Oxidris documentation
- **Audience**: AI assistants and human contributors
- **When to read**: Before creating, moving, or reorganizing documentation
- **Prerequisites**: Read [AGENTS.md](../../AGENTS.md) for project structure overview
- **Related documents**: [when-to-ask.md](when-to-ask.md) (when to ask about documentation changes)

## Documentation Distribution: Rustdoc vs Markdown

Oxidris uses two documentation systems with distinct roles:

### Rustdoc (Source of Truth for Implementation)

Rustdoc is the authoritative source for:

- **What**: Current implementation details
- **Why**: Design decisions and rationale
- **How**: API usage and code examples
- **Trade-offs**: Design trade-offs and limitations
- **Scope**: Single crate or module

**Location:** Module-level comments (`//!`) and item-level comments (`///`) in Rust source files.

**When to use:**

- Explaining how a module/function works
- Documenting design decisions and their reasons
- Providing API usage examples
- Describing current limitations and trade-offs

### Markdown (docs/)

Markdown documentation in `docs/` is for:

- **Architecture**: System-wide architecture across multiple crates
- **Context**: Project-specific context and status
- **Navigation**: Guide to where information lives (links to rustdoc)
- **Future**: Future improvements and proposals
- **NOT**: Implementation details or design rationale (those go in rustdoc)

**When to use:**

- Describing system architecture that spans multiple crates
- Providing project context and development status
- Creating navigation hubs that guide readers to rustdoc
- Documenting future improvements

### No Duplication Rule

**Markdown should link to rustdoc, not duplicate implementation details.**

```markdown
<!-- Good: Link to rustdoc -->
For detailed feature documentation and design decisions, see the 
[board_feature module documentation](../../../crates/oxidris-evaluator/src/board_feature/mod.rs).

<!-- Bad: Duplicate implementation details -->
Features use P05-P95 normalization because...
[long explanation that's already in rustdoc]
```

### Example Distribution

**In rustdoc** (`crates/oxidris-evaluator/src/board_feature/mod.rs`):

```rust
//! # Why Percentile-Based Normalization?
//!
//! - **Data-driven**: Grounded in actual gameplay behavior
//! - **Robust to outliers**: P05-P95 range clips extremes
//! - **Simple and fast**: Linear scaling, no complex computation
```

**In Markdown** (`docs/architecture/evaluator/README.md`):

```markdown
## Implementation Documentation

For detailed feature documentation, normalization design decisions,
and current limitations, see:

cargo doc --open --package oxidris-evaluator
```

## Documentation Structure

Oxidris documentation is organized by system and project:

- **System documentation** - `docs/architecture/` - Each system (Evaluator, Training, Engine) has its own subdirectory
- **Active projects** - `docs/projects/` - Time-limited project documentation in separate subdirectories
- **Future improvements** - `docs/future-projects.md` - Improvement proposals for all systems

**For the complete documentation structure tree, see the "Documentation Structure" section in AGENTS.md.**

## Organization Rules

### Scope Separation

Each system's documentation must stay within its own directory.

Do NOT mix concerns:

- Training content (GA, fitness functions) in `docs/architecture/evaluator/`
- Evaluator details (feature normalization) in `docs/architecture/training/`
- Engine mechanics in evaluator or training docs
- Duplicate future-projects.md content in project subdirectories
- Active project docs outside of `docs/projects/`

DO keep documentation organized:

- Keep each system's documentation in its own directory
- Use cross-references via links, not duplication
- Put future improvement proposals in `docs/future-projects.md`
- Put active projects in `docs/projects/[project-name]/`

## Cross-System References

When one system depends on another, link to the relevant documentation instead of duplicating content:

```markdown
<!-- Good: In docs/architecture/evaluator/README.md -->
Weights are learned through training. See [Training System](../architecture/training/README.md) for details.

<!-- Bad: Duplicating training content in evaluator docs -->
Weights are learned using a genetic algorithm with population size 30...
```

## Active Project Documentation

Active projects get their own subdirectory in `docs/projects/`:

```text
docs/projects/[project-name]/
├── README.md      # Project overview and goals
├── design.md      # Design decisions (optional)
└── roadmap.md     # Implementation plan (optional)
```

**Rules:**

- Keep all project-related docs in the project subdirectory
- Don't duplicate "Future Work" sections - use `docs/future-projects.md`
- When project completes, move relevant content to architecture docs and remove project directory

## Maintenance Guidelines

### Update the Appropriate File

Each file has a specific purpose. Update the right file based on what changed:

**System architecture docs** (`docs/architecture/[system]/`) when:

- Changing implementation details
- Discovering new issues or limitations
- Adding/removing features
- Updating system-specific design

**Active project docs** (`docs/projects/[project-name]/`) when:

- Working on that specific project
- Updating project design, roadmap, or status
- Documenting project-specific decisions

**Cross-cutting documentation** when:

- `docs/future-projects.md` - Proposing new improvement projects
- `docs/README.md` - Adding documentation sections or completing major milestones
- `docs/architecture/README.md` - Changing system boundaries or architecture overview

### Synchronization Principle

**Keep documentation synchronized with code changes in the same commit.**

- Document design decisions when you make them
- Update relevant docs in the same commit as code changes
- Add new issues to appropriate documentation when discovered

### Lint Before Committing

**Always run the linting script after markdown documentation changes (files in `docs/` directory):**

```bash
# Check and auto-fix markdown documentation
./scripts/lint docs --fix

# Or just check without auto-fix
./scripts/lint docs
```

**What it does:**

- **Typos**: Detects and automatically fixes typos in markdown files (with --fix)
- **Markdownlint**: Validates markdown style and auto-fixes issues where possible (with --fix)
- **Metadata blocks**: Verifies all docs have required metadata
- **Document types**: Confirms valid document type values
- **Internal links**: Checks for broken links between docs
- **Git changes**: Reports any modifications made by the linter (only in fix mode)

The `--fix` flag enables automatic fixes. Without it, the script only checks and reports issues. Always review changes with `git diff` before committing.

**Important:** This linter is for **markdown files** in the `docs/` directory. If you're updating **rustdoc comments** in `.rs` files, use `./scripts/lint rust --fix` instead (rustdoc is part of code, not separate documentation).

**Common Errors and Fixes:**

1. **MD032: Lists should be surrounded by blank lines**

   Bad: List without blank lines before/after

   Good: Add blank line before list

2. **MD040: Fenced code blocks should have a language**

   Bad: Three backticks without language

   Good: Use three backticks with language (rust, bash, text, etc)

3. **MD036: Emphasis used instead of a heading**

   Bad: Using bold text as section title

   Good: Use proper heading levels (####)

4. **MD031: Fenced code blocks should be surrounded by blank lines**

   Bad: Code block immediately after text without blank line

   Good: Add blank lines before and after code blocks

**Most errors are automatically fixed by the linting script.** If errors remain that couldn't be auto-fixed, address them manually using the guidance above.

See [CONTRIBUTING.md](../../CONTRIBUTING.md#linting-scripts) for details.

## Redundancy Check

Before adding content, ask yourself:

1. **Is this already documented elsewhere?**
   - Search existing docs before writing
   - If found, link to it instead of duplicating

2. **Does this belong in this directory's scope?**
   - Check the scope definition above
   - If it's about a different system, put it there

3. **Is this "future work"?**
   - If yes, it belongs in `docs/future-projects.md`
   - Don't add "Future Work" sections in project docs

4. **Is this about an active project?**
   - If yes, put it in `docs/projects/[project-name]/`
   - Don't mix project docs into architecture docs

## Documentation Metadata

All documentation files should include a metadata block immediately after the title.

For the standard metadata format and guidelines, see the [Documentation Metadata section in CONTRIBUTING.md](../../CONTRIBUTING.md#documentation-metadata).

## When Unsure

If you're unsure where to document something:

1. Check the "Documentation Structure" in AGENTS.md
2. Ask: "Which system does this primarily affect?"
3. If it affects multiple systems equally, put it in `docs/README.md` or `docs/architecture/README.md`
4. **If still unsure, see [When to Ask](when-to-ask.md) for guidance on when to ask before making changes**
