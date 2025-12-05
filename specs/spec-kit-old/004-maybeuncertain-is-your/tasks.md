# Tasks: MaybeUncertain<T>

**Input**: Design documents from `/specs/004-maybeuncertain-is-your/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

## Convention Adherence
This plan adheres to the conventions outlined in `AGENTS.md`:
- **File Structure**: Follows the `src/types/my_type.rs` and `tests/types/my_type_tests.rs` structure.
- **Crate-Specific Operations**: Testing and builds will target the `deep_causality_uncertain` crate to ensure efficiency.
- **Code Style**: All code will be formatted with `make format` and linted with `make fix`.
- **Safety & Dependencies**: The implementation will not introduce `unsafe` code, macros in library code, or new external dependencies.

## Path Conventions
- New type will be in `deep_causality_uncertain/src/types/maybe_uncertain.rs`
- Tests will be in `deep_causality_uncertain/tests/types/maybe_uncertain_tests.rs`
- New error variant will be in `deep_causality_uncertain/src/errors/uncertain_error.rs`

## Phase 3.1: Setup
- [ ] T001: Create new file `deep_causality_uncertain/src/types/maybe_uncertain.rs` and its test file `deep_causality_uncertain/tests/types/maybe_uncertain_tests.rs`.
- [ ] T002: Add the new error variant `PresenceError` to `deep_causality_uncertain/src/errors/uncertain_error.rs`.
- [ ] T003: Add a public module `maybe_uncertain` in `deep_causality_uncertain/src/types/mod.rs` and export the new type from `deep_causality_uncertain/src/lib.rs`.

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**
- [ ] T004 [P]: In `maybe_uncertain_tests.rs`, write failing tests for `from_value()`, `from_uncertain()`, and `always_none()` constructors. Verify with `cargo test -p deep_causality_uncertain --test maybe_uncertain_tests -- --nocapture`.
- [ ] T005 [P]: In `maybe_uncertain_tests.rs`, write a failing test for the `from_bernoulli_and_uncertain()` constructor. Verify with `cargo test -p deep_causality_uncertain --test maybe_uncertain_tests -- --nocapture`.
- [ ] T006 [P]: In `maybe_uncertain_tests.rs`, write failing tests for `is_some()` and `is_none()` methods. Verify with `cargo test -p deep_causality_uncertain --test maybe_uncertain_tests -- --nocapture`.
- [ ] T007: In `maybe_uncertain_tests.rs`, write a failing test for the `sample()` method. Verify with `cargo test -p deep_causality_uncertain --test maybe_uncertain_tests -- --nocapture`.
- [ ] T008: In `maybe_uncertain_tests.rs`, write failing tests for `lift_to_uncertain()` covering all scenarios, including the new error variant. Verify with `cargo test -p deep_causality_uncertain --test maybe_uncertain_tests -- --nocapture`.

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T009: In `maybe_uncertain.rs`, define the `MaybeUncertain<T>` struct and implement the `from_value()`, `from_uncertain()`, and `always_none()` constructors to pass tests from T004.
- [ ] T010: Implement the `from_bernoulli_and_uncertain()` constructor to pass test T005.
- [ ] T011: Implement the `is_some()` and `is_none()` methods to pass tests from T006.
- [ ] T012: Implement the `sample()` method to pass test T007.
- [ ] T013: Implement the `lift_to_uncertain()` method to pass test T008.

## Phase 3.4: Integration & Polish
- [ ] T014: [P] Implement `std::ops` traits (e.g., `Add`, `Sub`) for `MaybeUncertain<T>`. Write failing tests first and verify with `cargo test -p deep_causality_uncertain`.
- [ ] T015: [P] Add comprehensive Rustdoc comments to the `MaybeUncertain<T>` type and all its public methods.
- [ ] T016: Ensure the new type is properly exported and usable from the `deep_causality` crate by adding a simple usage example in an integration test.
- [ ] T017: Run `make format && make fix` and ensure `cargo test -p deep_causality_uncertain` passes cleanly.

## Dependencies
- **T001-T003** must be done first.
- Tests **T004-T008** must be completed before implementation tasks **T009-T013**.
- **T009** blocks **T010, T011, T012, T013**.

## Parallel Example
```
# The initial test creation can be done in parallel:
Task: "T004: Write failing test for new() and sample()"
Task: "T005: Write failing tests for always_some() and always_none()"
Task: "T006: Write failing test for from_bernoulli_and_uncertain()"
```
