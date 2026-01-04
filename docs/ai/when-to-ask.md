# When to Ask for Confirmation

Before making certain types of changes, **ask the user first** instead of proceeding directly.

- **Document type**: Reference
- **Purpose**: Decision criteria for when to ask user confirmation before making changes
- **Audience**: AI assistants before taking action
- **When to read**: Before making any non-trivial changes to code or documentation
- **Prerequisites**: Read [AGENTS.md](../../AGENTS.md) for project context (especially Development Status)
- **Related documents**: [review-process.md](review-process.md) (how to present questions), [documentation-guidelines.md](documentation-guidelines.md) (documentation-specific rules)

## When to Ask

### Active Projects

Ask before:

- Changing scope of active projects
- Removing or merging active project documentation
- Proposing alternative approaches to active work
- Making design decisions that affect project roadmap

Example: "This would change the current project scope to include structure features. Should we expand the scope?"

### Code Architecture

Ask before:

- Changing core data structures or traits
- Refactoring system boundaries (evaluator/training/engine)
- Introducing new architectural patterns
- Major refactoring that affects multiple systems
- Removing substantial amounts of code

Example: "This change would split `PlacementEvaluator` into two traits. Should I proceed with this refactoring?"

### Documentation Structure

Ask before:

- Adding or removing documentation directories or files
- Reorganizing documentation hierarchy
- Creating new documentation categories
- Moving documentation between directories

Example: "I'd like to create a new `docs/data-analysis/` directory for statistical analysis documentation. Should I proceed?"

### Dependencies

Ask before:

- Adding new external dependencies (crates)
- Changing major dependency versions
- Introducing new system requirements
- Adding new build tools or processes

Example: "This would add the `nalgebra` crate for matrix operations. Should I add this dependency?"

## Don't Need to Ask

You can proceed directly with:

- Bug fixes (unless they require architectural changes)
- Documentation updates for existing files (see [Documentation Guidelines](documentation-guidelines.md))
- Small, focused code changes within one system
- Formatting or style improvements
- Adding tests for existing functionality

## How to Ask

When asking for confirmation, provide:

> [!NOTE]
> Follow the language matching rules in [AGENTS.md](../../AGENTS.md#communication) - present all questions and templates in the same language the user is using.

**Scope:** What system/feature is being changed
**Why:** Reason the change is needed
**Impact:** [High/Medium/Low] - affects [N] files/systems
**Alternatives:** [If applicable]

**Impact level definitions:**

- **High**: Affects multiple systems, changes architecture, or impacts >10 files
- **Medium**: Affects single system significantly, or impacts 3-10 files
- **Low**: Localized changes, affects 1-2 files within one system

**Example format:**

```text
## Proposed Change

**Scope:** [What system/feature is being changed]
**Why:** [Reason the change is needed]
**Impact:** [High/Medium/Low] - affects [N] files/systems
**Alternatives:** [If applicable]

Should I proceed?
```

## When in Doubt

**If a change seems large, complex, or affects multiple systems - ask first.**

Better to ask unnecessarily than to make unwanted changes.
