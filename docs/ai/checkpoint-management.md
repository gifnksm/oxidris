# Checkpoint Management Guidelines

This document describes how to create and manage checkpoint files during AI-assisted work sessions.

- **Document type**: Process Guidelines
- **Purpose**: Define checkpoint file format and management practices
- **Audience**: AI assistants (primary), human contributors (reference)
- **Related documents**: [Review Process](review-process.md), [AGENTS.md](../../AGENTS.md)

## What Are Checkpoints?

Checkpoints are temporary working files that capture the current state of ongoing work. They serve as resumption points for AI-assisted sessions that may span multiple conversations or be interrupted.

**Use checkpoints for:**

- Long-running tasks (reviews, refactoring, implementation)
- Work that spans multiple sessions
- Complex changes affecting multiple files
- When interruption is likely or expected

**Checkpoints are not:**

- Permanent documentation (outcomes should be in commit messages and docs)
- Substitutes for version control
- Required for simple, single-session tasks

## When to Create Checkpoints

Create a checkpoint when:

1. **Work is paused mid-task** - Session ending before completion
2. **User requests it explicitly** - "Create a checkpoint" or "Save progress"
3. **Complex work in progress** - Multiple steps remaining, high interruption risk
4. **Before risky operations** - Major refactoring, large-scale changes

**Do not create checkpoints when:**

- Task is complete (commit and document instead)
- Work is trivial or nearly done
- User asks for a summary only (provide inline summary)

## Checkpoint File Location

Save checkpoint files to:

```text
.checkpoints/
```

This directory is at the project root and is configured to be:

- ✅ Excluded from `.gitignore` (not committed)
- ✅ Excluded from `.markdownlintignore` (no linting required)

Checkpoints are personal work-in-progress files, similar to editor-specific files like `.vscode/` or `.idea/`.

## Checkpoint File Naming

Use descriptive names with dates:

```text
YYYY-MM-DD-description-checkpoint.md
```

**Examples:**

- `.checkpoints/2025-01-06-evaluator-refactor-checkpoint.md`
- `.checkpoints/2025-01-06-km-normalization-impl-checkpoint.md`
- `.checkpoints/2025-01-06-doc-review-checkpoint.md`

**Naming guidelines:**

- Include date for easy cleanup
- Use descriptive keywords (what is being worked on)
- Keep it concise but clear
- Use lowercase with hyphens

## Checkpoint File Template

**Always use this exact template format:**

```markdown
# [Work Name] Checkpoint

**Date**: YYYY-MM-DD
**Status**: [In Progress / Paused / Nearly Complete]

## Summary

[Brief description of the work scope and current progress - 2-3 sentences]

## Completed Work

- ✅ Item 1 - Brief description
- ✅ Item 2 - Brief description
- ✅ Item 3 - Brief description

## Pending Work

- ⬜ Item 4 - What needs to be done
- ⬜ Item 5 - What needs to be done
- ⬜ Item 6 - What needs to be done

## Deferred Work

- 🔄 Item 7 - Task deferred for later (reason why deferred)
- 🔄 Item 8 - Optional improvement to consider (reason)

## Next Steps

[Specific instructions for resuming the work - what to do first, what to check, etc.]

## Notes

[Optional: Any important context, decisions made, issues encountered, trade-offs, etc.]
```

**Template rules:**

- Use exact structure (don't improvise)
- Include all required sections (Summary, Completed, Pending, Next Steps)
- Use ✅ for completed items, ⬜ for pending items, 🔄 for deferred items
- Deferred Work section is optional (include only if tasks were postponed)
- Be specific and actionable in "Next Steps"
- Keep "Summary" concise

## Creating Checkpoints

**Step-by-step process:**

1. **Read this template** before creating the checkpoint
2. **Fill in all required sections** with current state
3. **Save to** `.checkpoints/YYYY-MM-DD-description-checkpoint.md`
4. **Inform the user** that checkpoint was created and where

**Example interaction:**

```text
AI: I've created a checkpoint file at:
    .checkpoints/2025-01-06-evaluator-refactor-checkpoint.md

    This captures the completed refactoring of the evaluator module
    and the remaining work for the training integration.

    You can resume by reading this checkpoint in a new session.
```

## Checkpoint Cleanup

**When to delete checkpoint files:**

Delete a checkpoint only when **ALL** of the following are true:

1. ✅ **All Completed Work is committed** - Changes are in version control
2. ✅ **All Pending Work is done** - No remaining tasks
3. ✅ **All Deferred Work is handled** - Either completed, moved elsewhere, or explicitly decided to discard

**Do NOT delete if:**

- ❌ Deferred Work section has items (they would be lost)
- ❌ Pending Work section has items (work is incomplete)
- ❌ You're unsure if work is truly complete

**Special case - Deferred Work only:**

If a checkpoint has only Deferred Work (no Pending Work), you can:

- Keep it as-is for future reference
- Move deferred items to issue tracker or `docs/future-projects.md` and then delete
- Update checkpoint status to "Deferred" and keep for later review

**How to clean up:**

```bash
# Remove specific checkpoint
rm .checkpoints/2025-01-06-specific-checkpoint.md

# Remove all checkpoints (when all work is complete)
rm -rf .checkpoints/
```

**AI assistants should:**

- **Check for Deferred Work before deleting** - Never delete if Deferred Work exists
- Ask user if they want to handle deferred items before deletion
- Remind users to delete checkpoints only after ALL work (including deferred) is addressed
- Not delete checkpoints automatically (always user decision)

**Example of proper cleanup check:**

```text
AI: This checkpoint has:
    - ✅ All completed work is committed
    - ✅ No pending work remains
    - ⚠️  2 deferred work items remain
    
    The deferred items are:
    1. Add unit tests (deferred to separate task)
    2. Performance optimization (deferred until validation complete)
    
    Would you like to:
    - Keep the checkpoint for future reference
    - Move deferred items to issue tracker and delete checkpoint
    - Create a new checkpoint with only deferred items
```

## Version Control

Checkpoint files should **never be committed** to version control because:

- They are temporary working files
- They represent personal session state
- They are already in `.gitignore`

**Final work outcomes** should be documented through:

- Commit messages describing changes
- Updated project documentation (Markdown files in `docs/`)
- Pull request descriptions (if applicable)

## Use Case Examples

### Example 1: Code Review (Multi-Session)

**Scenario:** Reviewing 20 files of rustdoc comments, paused after 12 files.

**Checkpoint content:**

```markdown
# Evaluator Rustdoc Review Checkpoint

**Date**: 2025-01-06
**Status**: In Progress

## Summary

Reviewing rustdoc comments across evaluator crate for consistency,
completeness, and accuracy. Completed 12 of 20 modules.

## Completed Work

- ✅ board_feature/mod.rs - Fixed formatting, added examples
- ✅ board_feature/holes.rs - Clarified survival feature concept
- ✅ board_analysis.rs - Added performance notes
- ... (list all 12)

## Pending Work

- ⬜ placement_evaluator.rs - Review method docs
- ⬜ session_evaluator.rs - Check example code
- ... (list remaining 8)

## Deferred Work

- 🔄 Add integration tests - Deferred to separate task (scope too large)
- 🔄 Performance benchmarks - Optional, user requested later

## Next Steps

Continue with `placement_evaluator.rs`. Focus on:
- Method documentation completeness
- Example code accuracy
- Cross-references to related modules
```

### Example 2: Refactoring (Single Session, Interrupted)

**Scenario:** Refactoring analyze-censoring module, interrupted mid-work.

**Checkpoint content:**

```markdown
# Analyze-Censoring Refactor Checkpoint

**Date**: 2025-01-06
**Status**: Paused

## Summary

Refactoring analyze-censoring to eliminate duplicate calculations
and separate concerns. Feature statistics computation complete,
normalization integration in progress.

## Completed Work

- ✅ Split analyze_feature into compute/display/save functions
- ✅ Added collect_phase_data and collect_evaluator_data helpers
- ✅ Updated feature.rs with new structure

## Pending Work

- ⬜ Update mod.rs to use new feature module functions
- ⬜ Test with actual data
- ⬜ Update function documentation

## Deferred Work

- 🔄 Add unit tests for helper functions - Will add after main refactor
- 🔄 Consider splitting into more modules - User suggested revisiting later

## Next Steps

1. Modify mod.rs `run()` function to call feature::compute_feature_statistics
2. Pass results to display/save functions separately
3. Update generate_normalization_params to reuse computed stats
4. Run `./scripts/lint rust --fix` and test
```

### Example 3: Implementation (Multi-Day)

**Scenario:** Implementing KM-based normalization, Phase 4 of project.

**Checkpoint content:**

```markdown
# KM Normalization Implementation Checkpoint

**Date**: 2025-01-06
**Status**: In Progress

## Summary

Implementing Phase 4 of KM-based normalization project. Type design
complete, FeatureBuilder integration started.

## Completed Work

- ✅ Designed MappedNormalized<S> type (Phase 3)
- ✅ Defined coexistence strategy with RawNormalized
- ✅ Created feature naming convention
- ✅ Updated roadmap.md with decisions

## Pending Work

- ⬜ Implement MappedNormalized type in evaluator crate
- ⬜ Integrate with FeatureBuilder
- ⬜ Add training tool support
- ⬜ Validation and testing

## Deferred Work

- 🔄 Explore non-KM mappings (log, sqrt) - Future project after Phase 4
- 🔄 Performance optimization - Defer until correctness validated

## Next Steps

Start with implementing MappedNormalized<S> in:
`crates/oxidris-evaluator/src/board_feature/normalized.rs`

Reference Phase 4 tasks in:
`docs/projects/km-feature-transform/roadmap.md`

## Notes

Decision: Use MappedNormalized<S> name instead of KMNormalized
to allow future non-KM mappings (log, sqrt, etc.)
```

## Tips for Effective Checkpoints

### Be Specific

❌ Bad: "Fixed some files"
✅ Good: "Updated board_feature/holes.rs rustdoc with survival feature explanation"

### Include Context

Add notes about:

- Decisions made and why
- Issues encountered
- Alternative approaches considered
- Important discoveries

### Track Deferred Work

When user says "let's do that later" or "save that for another time":

- Add to "Deferred Work" section
- Include reason for deferral
- This prevents forgotten improvements or scope creep

### Make Next Steps Actionable

❌ Bad: "Continue work"
✅ Good: "Start with mod.rs line 45, update feature::compute call, then test with data/boards.json"

### Update as You Go

If work scope changes mid-session:

- Update the checkpoint before pausing
- Add new items to Pending Work
- Note changes in Notes section

### Keep It Current

- One active checkpoint per work stream
- Archive or delete old checkpoints
- Don't let checkpoints accumulate

## Integration with Other Processes

### Review Process

When conducting reviews (see [Review Process](review-process.md)):

- Use checkpoints for multi-session reviews
- Include review-specific context (files reviewed, issues found)
- Reference review process steps in "Next Steps"

### Documentation Updates

When updating documentation (see [Documentation Guidelines](documentation-guidelines.md)):

- Checkpoint after completing each major section
- List remaining docs to update in "Pending Work"
- Include cross-file consistency checks in "Next Steps"

### Implementation Work

For implementation tasks:

- Checkpoint after completing each logical unit (module, feature, test suite)
- Include test status and validation results
- Reference project documentation (e.g., `docs/projects/*/roadmap.md`)

## Summary

**Remember:**

1. ✅ Use checkpoints for long-running, interruptible work
2. ✅ Always follow the template format exactly
3. ✅ Be specific and actionable in all sections
4. ✅ Save to `.checkpoints/YYYY-MM-DD-description-checkpoint.md`
5. ✅ Track deferred work with 🔄 to avoid forgetting tasks
6. ✅ **Only delete checkpoints when ALL work is addressed** (including Deferred Work)

**Before deleting a checkpoint, verify:**

- ✅ All Completed Work is committed
- ✅ All Pending Work is done
- ✅ All Deferred Work is handled (completed, moved elsewhere, or explicitly discarded)

**If Deferred Work remains, do NOT delete the checkpoint.**

**Checkpoints are tools for continuity, not permanent documentation.**
