# Review Process Guidelines

This document defines the standard process for presenting, discussing, and completing reviews in a structured and interruptible manner.

- **Document type**: How-to guide
- **Purpose**: Step-by-step process for conducting code and documentation reviews
- **Audience**: AI assistants conducting reviews
- **When to read**: When reviewing proposed changes or existing content
- **Prerequisites**: None
- **Related documents**: [when-to-ask.md](when-to-ask.md) (when to ask during reviews), [documentation-guidelines.md](documentation-guidelines.md) (for documentation reviews)

## Principles

- **Small batches**: Present changes in digestible chunks, not all at once
- **Progressive disclosure**: Show overview first, then details step-by-step
- **Clear progress**: Always indicate where we are in the review process
- **Interruptible**: Support pausing and resuming reviews across conversations
- **User confirmation**: Wait for user approval before proceeding to next step
- **Language matching**: Follow the language matching rules in [AGENTS.md](../../AGENTS.md#communication) - present all templates and conversations in the same language the user is using

## Review Process

### Step 1: Present Overview

Start with a high-level summary of what will be reviewed.

> [!NOTE]
> Present this template in the user's language (see [Language matching in AGENTS.md](../../AGENTS.md#communication))

```text
## Review Overview

**Review type:** [Proposed changes / Existing content review / Other]
**Scope:** [What is being reviewed]
**Impact:** [High/Medium/Low - see definitions below]
**Total items:** [N items to review]

### Items at a glance

1. [Item 1 - brief description]
2. [Item 2 - brief description]
3. [Item 3 - brief description]
...

Ready to proceed with detailed review?
```

**Impact level definitions:**

- **High**: Affects multiple systems, changes architecture, or impacts >10 files
- **Medium**: Affects single system significantly, or impacts 3-10 files
- **Low**: Localized changes, affects 1-2 files within one system

**Examples:**

- Proposed changes: "3 files to modify in training system"
- Existing content review: "5 consistency issues found in documentation-guidelines.md"

**⚠️ Wait for user approval** before proceeding to details. Do not continue to Step 2 until the user confirms.

### Step 2: Review Items Step-by-Step

Present each item individually.

> [!NOTE]
> Present this template in the user's language (see [Language matching in AGENTS.md](../../AGENTS.md#communication))

```text
## Review: Item X/N - [Item Name]

**File:** [file path if applicable]
**Progress:** 3/5 items reviewed

**What's being reviewed:**
- [Brief description of the item]
- [Why this matters / needs attention]

**Key points:**
- [Important detail 1]
- [Important detail 2]

**Details:**
[Code diff, documentation excerpt, or issue description - keep it focused]

---
Ready to proceed? Or any changes needed here?
```

**⚠️ Wait for confirmation** before moving to next item. Do not proceed to the next item until the user confirms.

### Step 3: Handle Feedback

When user requests changes:

1. Note the feedback clearly
2. Ask: "Should I fix this now, or continue reviewing remaining items?"
3. Keep a list of all requested changes
4. Summarize all changes before implementing

### Step 4: Summary After Completion

```text
## Review Complete ✓

**Reviewed:** All N items
**Approved:** X items
**Modified:** Y items (based on feedback)
**Next steps:** [Implementation / Further review / etc.]
```

> [!NOTE]
> Present this template in the user's language (see [Language matching in AGENTS.md](../../AGENTS.md#communication))

## Best Practices

### Handling Discovered Issues

While reviewing, if you discover issues in existing content (not part of the primary review focus):

#### Minor Issues (Fix Silently)

**Fix immediately without asking:**

- Typos in documentation or comments
- Formatting inconsistencies
- Broken links
- Obvious syntax errors in markdown

**Note in review summary:**

```text
## Issues Fixed During Review

- Fixed typo in docs/architecture/evaluator/README.md
- Fixed broken link to training documentation
```

#### Medium Issues (Ask First)

**Pause and ask the user:**

- Design inconsistencies between documents
- Contradictions in documentation
- Outdated information that might be intentional
- Unclear code that could be improved

**Example:**

```text
⚠️ Discovered inconsistency:

docs/architecture/evaluator/README.md says feature count is 15,
but docs/projects/km-feature-transform/README.md mentions removing duplicates.

Should I:
1. Update both to clarify current vs. future state?
2. Continue review and address separately?
3. Ignore (intentional difference)?
```

#### Major Issues (Stop Review)

**Stop and report immediately:**

- Architectural problems requiring design discussion
- Scope creep (requires work outside intended scope)
- Fundamental flaws in approach

**Example:**

```text
🛑 Review Paused - Major Issue Discovered

While reviewing the evaluator changes, I discovered that the
proposed KM transform approach conflicts with the existing
BoardFeatureSource trait design in a fundamental way.

This requires architectural discussion before proceeding.

Issue details:
[Explain the problem]

Recommendation:
[Suggest approach to resolve]

Should we address this first, or continue review?
```

### Grouping Large Reviews

For reviews with many items (more than 5), group by logical phases:

```text
## Review Plan

Total: 12 items across 3 phases

### Phase 1: Documentation Updates (4 items)
### Phase 2: Code Changes (6 items)
### Phase 3: Tests (2 items)

Let's start with Phase 1. Ready?
```

Review one phase at a time. Show phase progress after completing each phase:

```text
Phase 1/3 complete ✓
Moving to Phase 2...
```

### Progress Indicators

Use clear visual progress tracking:

#### Navigation Breadcrumbs

```text
📍 You are here → docs/architecture/evaluator/README.md → Section 2.3
```

#### Checklists

```text
✓ Item 1 - Done
✓ Item 2 - Done
🔄 Item 3 - Current
⬜ Item 4 - Pending
⬜ Item 5 - Pending
```

### Handling Interruptions

If conversation must pause or split across sessions:

```text
## Session Checkpoint 🔖

**Completed:** Items 1-3 of 10
**Current:** About to review item 4 (docs/architecture/evaluator/README.md)
**Remaining:** Items 5-10
**Pending feedback:** [List any unresolved questions]

**To resume next session:** Say "Continue review from checkpoint"
```

This allows the user to:

- See exactly where we stopped
- Resume easily in a new conversation
- Track any pending feedback

### Token Limit Awareness

Monitor token usage proactively:

- If approaching token limit (~80%), pause and create checkpoint
- Provide clear resume instructions
- Offer to continue in new conversation
- Don't wait until hitting the limit

Example:

```text
⚠️ Approaching token limit. Creating checkpoint...

[Checkpoint details here]

Should we continue in a new conversation?
```

## Checkpoint Management

### Checkpoint File Location

Review checkpoint files should be saved to:

```text
.review-checkpoints/
```

This is a project-root directory (not under `docs/`) because checkpoints may cover:

- Documentation reviews
- Code reviews
- Configuration file reviews
- Cross-system reviews

### Checkpoint File Naming

Use descriptive names with dates:

```text
YYYY-MM-description-checkpoint.md
```

**Examples:**

- `.review-checkpoints/2025-01-evaluator-rustdoc-checkpoint.md`
- `.review-checkpoints/2025-01-review-complete-checkpoint.md`
- `.review-checkpoints/2025-01-training-system-checkpoint.md`

### Checkpoint File Content

Include in checkpoint files:

1. **Date and Status** - When created, current state
2. **Summary** - What was reviewed/changed
3. **Completed Work** - Detailed list of completed items
4. **Pending Work** - What remains to be done
5. **Next Steps** - How to resume

**Template:**

```markdown
# [Review Name] Checkpoint

**Date**: YYYY-MM-DD
**Status**: [In Progress / Complete / Paused]

## Summary

[Brief description of review scope and progress]

## Completed Work

- ✅ Item 1 - Description
- ✅ Item 2 - Description

## Pending Work

- ⬜ Item 3 - Description
- ⬜ Item 4 - Description

## Next Steps

[Instructions for resuming the review]
```

### Version Control

Checkpoint files should be:

- ✅ **Included in `.markdownlintignore`** (already configured)
- ✅ **Included in `.gitignore`** (already configured)

Checkpoint files are temporary working files for AI-assisted review sessions and should not be committed to version control. They represent personal work-in-progress state, similar to editor-specific files like `.vscode/` or `.idea/`.

**Final review outcomes** should be documented through:

- Commit messages describing the changes
- Updated project documentation
- Pull request descriptions (if applicable)

### Checkpoint Cleanup

**When to delete checkpoint files:**

1. **After review completion** - When all changes are committed and the review is fully complete
2. **Session concluded** - When the AI-assisted work session has ended and won't be resumed
3. **Regular cleanup** - Periodically remove old checkpoints (e.g., older than 1 week)

**How to clean up:**

```bash
# Remove specific checkpoint
rm .review-checkpoints/2025-01-specific-checkpoint.md

# Remove all checkpoints (when all work is complete)
rm -rf .review-checkpoints/

# Remove old checkpoints (Unix/Linux/macOS)
find .review-checkpoints -name "*.md" -mtime +7 -delete
```

**Best practice:** Delete checkpoint files as soon as the review work is committed to version control. They serve no purpose after the final changes are applied and documented.

### Cross-File Impact Tracking

When reviewing, note if issues affect multiple files:

```text
⚠️ Cross-file Impact Detected

This issue also appears in:

- docs/ai/when-to-ask.md (same problem)
- docs/README.md (related issue)

Should I:

1. Fix all related files in this session?
2. Note for separate review?
3. Create consistency checklist?
```

This helps identify systemic issues early.

## Example Flow

**User Request:**

```text
User: "Update documentation for new training system"
```

**Agent Response:**

```text
## Review Overview

**Review type:** Proposed changes
**Scope:** Training system documentation
**Impact:** Medium (3 documentation files)
**Total items:** 3 files to modify

### Items at a glance

1. docs/architecture/training/README.md - Add fitness function formulas
2. docs/architecture/evaluator/README.md - Update cross-reference
3. docs/README.md - Add training system to hub

Ready to proceed with detailed review?
```

**After User Approval:**

```text
## Review: Item 1/3 - Add fitness function formulas

**File:** docs/architecture/training/README.md
**Progress:** 1/3 items

**What's being reviewed:**

- Adding new "Fitness Function Design" section
- Explaining AggroSessionEvaluator and DefensiveSessionEvaluator formulas

**Key points:**

- Documents the mathematical formulas used
- Explains the coefficients (currently ad-hoc)
- Notes that these are not systematically optimized

**Details:**

[Shows the new section content]

---
Ready to proceed to item 2?
```

**Continue Until Complete:**

```text
## Review Complete ✓

**Reviewed:** All 3 items
**Approved:** 3 items
**Modified:** 0 items

## Issues Fixed During Review

- Fixed typo in docs/architecture/evaluator/README.md (line 42)

**Next steps:** Implementing approved changes now...
```

### Review Type Comparison

The same process applies to both review types:

| Aspect | Proposed Changes | Existing Content Review |
| --- | --- | --- |
| **Items** | Changes to apply | Issues found |
| **Overview** | Files to modify | Problems to fix |
| **Step 2** | Show proposed changes | Show issue + suggested fix |
| **Summary** | Changes implemented | Issues resolved |

**Example: Existing content review overview:**

```text
**Review type:** Existing content review
**Scope:** docs/ai/documentation-guidelines.md
**Total items:** 5 issues found (2 High, 2 Medium, 1 Low)

### Items at a glance
1. Duplicate structure tree with AGENTS.md
2. Project-specific examples need generalization
3. Missing cross-references
...
```

## Tips for Effective Reviews

1. **Clarify review type upfront** - Change review or document review? Set expectations early
2. **Keep each item focused** - One file, one issue, or one logical change at a time
3. **Show impact and scope** - Not just what, but why and how much is affected
4. **Track cross-file impacts** - Note when issues appear in multiple files
5. **Provide relevant context** - But don't overwhelm with entire files
6. **Be consistent** - Use the same format for each item
7. **Respect cognitive load** - If user seems overwhelmed, break into smaller pieces
8. **Track everything** - Progress, feedback, pending items, discovered issues
9. **Be interruptible** - Always ready to pause and resume
10. **Fix minor issues proactively** - Don't bother user with trivial typos
11. **Ask about medium issues** - When in doubt, ask
12. **Stop for major issues** - Don't continue if fundamental problems exist
13. **Visualize progress** - Use checklists, phase indicators, and completion percentages
14. **Note related work** - Identify files that may need similar review

## Cross-Document Consistency Checks

After large-scale changes (renaming, refactoring, architecture updates), perform cross-document consistency checks.

> **Note:** The compiler validates code correctness, but not documentation content. These checks focus on documentation (Markdown files and rustdoc comments), configuration files (Makefile, .gitignore, etc.) where consistency must be verified manually.

**When to run:**

- After renaming crates, modules, or major components
- After moving or reorganizing documentation
- After updating terminology or conventions
- Periodically (e.g., after completing a major feature)

**What to check:**

1. **Naming consistency** - Crate names, module names, file paths
2. **Link validity** - All cross-references point to existing files
3. **Terminology** - Same concepts use same terms across documents
4. **Fixed values** - No hard-coded counts or values that may change
5. **Metadata completeness** - All docs have required metadata blocks

**Tools to use:**

```bash
# Find all occurrences of old crate name
grep -r "old-crate-name" --include="*.md" .

# Find broken links (example pattern)
grep -r "path/that/moved\.md" --include="*.md" .

# Find files without metadata
find docs -name "*.md" -exec grep -L "Document type:" {} \;

# Run markdownlint
markdownlint .
```

**Process:**

1. **Plan** - List what to check based on recent changes
2. **Explore** - Use grep/find to discover issues
3. **Categorize** - Group by severity (critical/medium/minor)
4. **Fix** - Address issues in priority order
5. **Validate** - Re-run checks to confirm fixes

## Validation Tools

Use automated tools to catch issues early.

**Primary tool:**

```bash
# Check all documentation (recommended)
./scripts/lint docs

# Check and auto-fix documentation issues
./scripts/lint docs --fix
```

This checks:

1. **Typos** - Detects and fixes typos (with --fix)
2. **Markdown style** - markdownlint compliance (with auto-fix using --fix)
3. **Metadata blocks** - All docs have required metadata
4. **Document types** - Types match official taxonomy (Reference, How-to guide, Tutorial, Explanation)
5. **Internal links** - No broken links between markdown files

**Additional tools:**

| Tool       | When                                 | Command                                  |
|------------|--------------------------------------|------------------------------------------|
| cargo test | After doc changes with code examples | `cargo test --doc`                       |
| cargo doc  | After rustdoc changes                | `cargo doc --no-deps`                    |
| grep       | Custom cross-document searches       | `grep -r "pattern" --include="*.md"`     |

**Individual checks (if needed):**

```bash
# Run only markdownlint (included in lint script)
markdownlint .

# Fix auto-fixable markdown issues
markdownlint . --fix

# Verify doctests compile
cargo test --doc --package oxidris-evaluator

# Generate rustdoc
cargo doc --no-deps --package oxidris-evaluator

# Custom pattern search
grep -r "pattern" --include="*.md" . --exclude-dir=.review-checkpoints
```

## Review Reflection

After completing a review, reflect on the process to improve future reviews.

**Questions to ask:**

1. **What worked well?** - Which techniques were effective?
2. **What was difficult?** - Where did we get stuck or confused?
3. **What was missed?** - Issues discovered later that should have been caught?
4. **What patterns emerged?** - Recurring issues that suggest systemic problems?
5. **Should the process change?** - Do these guidelines need updates?

**When to update this document:**

- Discovered a useful technique not documented here
- Found that existing guidance was unclear or incorrect
- Identified a common pitfall to warn about
- Learned a more efficient approach

**How to update:**

1. Note the improvement during or after review
2. Discuss with user if it should be added
3. Update this document with the new guidance
4. Keep changes minimal and focused

**Example reflection notes:**

```text
## Review Reflection - 2025-01-XX

**Review:** Cross-document consistency check after crate rename

**What worked:**
- grep + find commands found all issues efficiently
- Categorizing by severity helped prioritize
- markdownlint caught formatting issues we missed

**What to improve:**
- Should have run markdownlint earlier
- Need better checklist for "what to verify after X type of change"

**Process updates:**
- Added "Cross-Document Consistency Checks" section
- Added "Validation Tools" reference
```

**Result:** This reflection led to the sections you're reading now.
