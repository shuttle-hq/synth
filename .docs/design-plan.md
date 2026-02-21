# Implementation Plan: Synth Nightly Rust Migration

**Status**: Draft
**Research Doc**: `/Users/alex/projects/terraphim/synth/.docs/research-findings.md`
**Author**: Claude Code
**Date**: 2026-02-21
**Estimated Effort**: 2-3 hours

## Overview

### Summary
Remove the `concat_idents` feature flag from `synth/src/lib.rs` to fix compilation on current Rust nightly (1.95.0). The feature was removed in Rust 1.90.0 and is not actually used in the codebase. Additionally, address clippy warnings to ensure CI passes with `-D warnings` flag.

### Approach
Minimal change approach: remove the unused feature declaration and fix clippy warnings that would cause CI failures. No functional code changes required.

### Scope

**In Scope:**
1. Remove `concat_idents` from feature declaration in `synth/src/lib.rs`
2. Fix clippy warnings that would fail CI (9 warnings in synth-core, 1 in synth-gen, 1 in synth)
3. Verify compilation passes
4. Verify all tests pass

**Out of Scope:**
- Migrating `box_patterns` to stable (still works on nightly)
- Migrating `try_blocks` to stable (still works on nightly)
- Migrating `error_iter` to stable (still works on nightly)
- Major dependency updates
- Feature additions or enhancements

**Avoid At All Cost** (from 5/25 analysis):
- Refactoring working code to avoid `box_patterns` (unnecessary complexity)
- Refactoring working code to avoid `try_blocks` (unnecessary complexity)
- Updating dependencies beyond patch versions (risk of breaking changes)
- Adding new features or capabilities (scope creep)
- Pinning to older Rust nightly (technical debt)

## Architecture

### Component Diagram
```
Before:
synth/src/lib.rs
  └── #![feature(box_patterns, concat_idents, error_iter)]
                           ^^^^^^^^^^^^^ REMOVE THIS

After:
synth/src/lib.rs
  └── #![feature(box_patterns, error_iter)]
```

### Data Flow
No changes to data flow. This is a compilation fix only.

### Key Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Remove only `concat_idents` | Feature is not used anywhere in codebase; simplest fix | Replace with `${concat(..)}` (unnecessary since unused) |
| Fix clippy warnings now | CI uses `-D warnings`; warnings would fail build | Fix later (would fail CI immediately) |
| Keep `box_patterns` | Feature still available on nightly; refactoring would be significant | Migrate to stable (high effort, low value) |
| Keep `try_blocks` | Feature still available on nightly; used in format.rs | Migrate to stable (high effort, low value) |

### Eliminated Options (Essentialism)

| Option Rejected | Why Rejected | Risk of Including |
|-----------------|--------------|-------------------|
| Replace `concat_idents` with `${concat(..)}` | Not actually used; would be dead code | Adds complexity for no benefit |
| Migrate `box_patterns` to stable | Would require rewriting 4 pattern matches; feature still works | Significant refactoring risk; no current benefit |
| Migrate `try_blocks` to stable | Would require rewriting error handling in format.rs | Significant refactoring risk; no current benefit |
| Pin to older nightly | Creates technical debt; prevents security updates | Security risk; future migration harder |
| Update all dependencies | Out of scope; may introduce breaking changes | Unnecessary risk for compilation fix |
| Rewrite using `paste!` macro | `paste` crate already available but unnecessary | Over-engineering; feature not used |

### Simplicity Check

> "Minimum code that solves the problem. Nothing speculative."

**Answer: What if this could be easy?**

The simplest design is to delete 15 characters (`concat_idents, `) from line 1 of `synth/src/lib.rs`. This is exactly what this plan specifies.

**Senior Engineer Test**: Would a senior engineer call this overcomplicated?
**Answer**: No. This is the absolute minimum change required.

**Nothing Speculative Checklist**:
- [x] No features the user didn't request
- [x] No abstractions "in case we need them later"
- [x] No flexibility "just in case"
- [x] No error handling for scenarios that cannot occur
- [x] No premature optimization

## File Changes

### New Files
None.

### Modified Files

| File | Changes |
|------|---------|
| `synth/src/lib.rs` | Remove `concat_idents, ` from line 1 feature declaration |
| `core/src/schema/content/object.rs` | Fix `mismatched_lifetime_syntaxes` warning (line 35) |
| `core/src/compile/state.rs` | Fix `mismatched_lifetime_syntaxes` warning (line 44) |
| `core/src/graph/series.rs` | Address `dead_code` warning for `AutoCorrelatedSeries` (line 262) |
| `core/src/schema/content/date_time.rs` | Fix `derivable_impls` warning (line 121) |
| `core/src/graph/string/faker.rs` | Fix `derivable_impls` warning (line 25) |
| `core/src/compile/state.rs` | Fix `mem_replace_option_with_some` warning (line 435) |
| `gen/src/generator/mod.rs` | Fix `non_local_definitions` warning (line 1272) |
| `synth/src/datasource/mysql_datasource.rs` | Fix `empty_line_after_doc_comments` warning (line 23) |

### Deleted Files
None.

## API Design

No API changes. All changes are internal to compilation and linting.

## Test Strategy

### Unit Tests
No new unit tests required. Existing tests should continue to pass.

### Integration Tests
Run full test suite to verify no regressions:

| Test | Command | Purpose |
|------|---------|---------|
| Full test suite | `cargo test` | Verify no functional regressions |
| Clippy check | `cargo clippy --tests --all-targets --all-features` | Verify no warnings |
| Format check | `cargo fmt --all -- --check` | Verify formatting |

### Verification Steps

```bash
# 1. Verify compilation
$ cargo check
# Expected: success, no errors

# 2. Verify tests pass
$ cargo test
# Expected: all tests pass

# 3. Verify clippy passes
$ cargo clippy --tests --all-targets --all-features
# Expected: no warnings (with -D warnings)

# 4. Verify formatting
$ cargo fmt --all -- --check
# Expected: no formatting issues
```

## Implementation Steps

### Step 1: Remove concat_idents Feature
**Files:** `synth/src/lib.rs`
**Description:** Remove `concat_idents, ` from the feature declaration
**Tests:** `cargo check` should pass
**Dependencies:** None
**Estimated:** 5 minutes

```rust
// BEFORE (line 1):
#![feature(box_patterns, concat_idents, error_iter)]

// AFTER (line 1):
#![feature(box_patterns, error_iter)]
```

### Step 2: Fix Clippy Warnings in synth-core
**Files:**
- `core/src/schema/content/object.rs` (line 35)
- `core/src/compile/state.rs` (lines 44, 435)
- `core/src/graph/series.rs` (line 262)
- `core/src/schema/content/date_time.rs` (line 121)
- `core/src/graph/string/faker.rs` (line 25)

**Description:** Fix 9 clippy warnings
**Tests:** `cargo clippy -p synth-core` should pass
**Dependencies:** Step 1
**Estimated:** 30 minutes

**Specific fixes:**

1. `core/src/schema/content/object.rs:35` - Add explicit lifetime:
```rust
// BEFORE:
fn add_reserved_underscores(key: &str) -> Cow<str> {

// AFTER:
fn add_reserved_underscores(key: &str) -> Cow<'_, str> {
```

2. `core/src/compile/state.rs:44` - Add explicit lifetime:
```rust
// BEFORE:
pub(super) fn iter_ordered(&self) -> std::slice::Iter<String> {

// AFTER:
pub(super) fn iter_ordered(&self) -> std::slice::Iter<'_, String> {
```

3. `core/src/compile/state.rs:435` - Use `Option::replace()`:
```rust
// BEFORE:
std::mem::replace(&mut self.src, Some(inner))

// AFTER:
self.src.replace(inner)
```

4. `core/src/graph/series.rs:262` - Add `#[allow(dead_code)]` or remove:
```rust
// Option A - suppress warning:
#[allow(dead_code)]
pub struct AutoCorrelatedSeries {

// Option B - remove if truly unused (verify first)
```

5. `core/src/schema/content/date_time.rs:121` - Derive Default:
```rust
// BEFORE:
pub enum ChronoValueType {
    NaiveDate,
    NaiveTime,
    NaiveDateTime,
    DateTime,
}

impl Default for ChronoValueType {
    fn default() -> Self {
        Self::DateTime
    }
}

// AFTER:
#[derive(Default)]
pub enum ChronoValueType {
    NaiveDate,
    NaiveTime,
    NaiveDateTime,
    #[default]
    DateTime,
}
```

6. `core/src/graph/string/faker.rs:25` - Derive Default (similar pattern)

### Step 3: Fix Clippy Warnings in synth-gen
**Files:** `gen/src/generator/mod.rs` (line 1272)
**Description:** Fix `non_local_definitions` warning
**Tests:** `cargo clippy -p synth-gen` should pass
**Dependencies:** Step 1
**Estimated:** 15 minutes

The warning is about an impl block inside a test function. The fix is to move the impl outside the function or use a different approach.

### Step 4: Fix Clippy Warnings in synth
**Files:** `synth/src/datasource/mysql_datasource.rs` (line 23)
**Description:** Fix `empty_line_after_doc_comments` warning
**Tests:** `cargo clippy -p synth` should pass
**Dependencies:** Step 1
**Estimated:** 5 minutes

Remove empty line between doc comment and struct definition.

### Step 5: Full Verification
**Files:** All
**Description:** Run full verification suite
**Tests:**
- `cargo check --all`
- `cargo test --all`
- `cargo clippy --tests --all-targets --all-features`
- `cargo fmt --all -- --check`
**Dependencies:** Steps 1-4
**Estimated:** 10 minutes

## Rollback Plan

If issues discovered:
1. Revert changes using git: `git checkout -- <files>`
2. Re-apply selectively if only some changes caused issues

No feature flags needed - this is a compilation fix.

## Migration

No database or data migration required.

## Dependencies

### New Dependencies
None.

### Dependency Updates
None. This plan intentionally avoids dependency updates to minimize risk.

## Performance Considerations

No performance impact expected. Changes are:
1. Removing an unused feature flag (no runtime impact)
2. Fixing lint warnings (no runtime impact)

## Open Items

| Item | Status | Owner |
|------|--------|-------|
| Verify AutoCorrelatedSeries is truly unused | Pending | Implementation phase |
| Determine best fix for non_local_definitions | Pending | Implementation phase |

## Approval

- [ ] Technical review complete
- [ ] Test strategy approved
- [ ] Human approval received

---

## Appendix: Detailed Clippy Warning Analysis

### synth-core Warnings (9 total)

1. **dead_code** - `AutoCorrelatedSeries` in `core/src/graph/series.rs:262`
   - Risk: Low
   - Fix: Add `#[allow(dead_code)]` or remove struct

2. **mismatched_lifetime_syntaxes** (2 occurrences)
   - `core/src/schema/content/object.rs:35`
   - `core/src/compile/state.rs:44`
   - Risk: Low
   - Fix: Add explicit `'_` lifetime

3. **derivable_impls** (2 occurrences)
   - `core/src/schema/content/date_time.rs:121`
   - `core/src/graph/string/faker.rs:25`
   - Risk: Low
   - Fix: Add `#[derive(Default)]` and `#[default]` attribute

4. **mem_replace_option_with_some** - `core/src/compile/state.rs:435`
   - Risk: Low
   - Fix: Use `Option::replace()` method

### synth-gen Warnings (1 total)

1. **non_local_definitions** - `gen/src/generator/mod.rs:1272`
   - Risk: Medium
   - Fix: Move impl block outside function or refactor test

### synth Warnings (1 total)

1. **empty_line_after_doc_comments** - `synth/src/datasource/mysql_datasource.rs:23`
   - Risk: Low
   - Fix: Remove empty line
