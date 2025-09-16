# Tasks: Replace RNG

**Input**: Design documents from `/specs/002-replace-rng-currently/`
**Prerequisites**: plan.md (required), research.md, data-model.md, quickstart.md

## Execution Flow (main)
```
1. Load plan.md from feature directory
   → If not found: ERROR "No implementation plan found"
   → Extract: tech stack, libraries, structure
2. Load optional design documents:
   → data-model.md: Extract entities → model tasks
   → contracts/: Each file → contract test task
   → research.md: Extract decisions → setup tasks
3. Generate tasks by category:
   → Setup: project init, dependencies, linting
   → Tests: contract tests, integration tests
   → Core: models, services, CLI commands
   → Integration: DB, middleware, logging
   → Polish: unit tests, performance, docs
4. Apply task rules:
   → Different files = mark [P] for parallel
   → Same file = sequential (no [P])
   → Tests before implementation (TDD)
5. Number tasks sequentially (T001, T002...)
6. Generate dependency graph
7. Create parallel execution examples
8. Validate task completeness:
   → All contracts have tests?
   → All entities have models?
   → All endpoints implemented?
9. Return: SUCCESS (tasks ready for execution)
```

## Format: `[ID] [P?] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- Include exact file paths in descriptions

## Path Conventions
- **Single project**: `src/`, `tests/` at repository root
- Paths shown below assume single project - adjust based on plan.md structure

## Phase 3.1: Setup
## Phase 3.1: Setup
- [ ] T001 Create `deep_causality_rand` Crate Structure: Create a new Rust library crate named `deep_causality_rand` in the monorepo. Create the initial `src/lib.rs` and the specified subfolders (`src/errors`, `src/traits`, `src/types`).
    - Reference: `plan.md` (Summary, Technical Context, Project Structure), `data-model.md` (deep_causality_rand Crate)
- [ ] T002 Configure `Cargo.toml` for `deep_causality_rand`: Set up basic crate metadata, ensure no external dependencies beyond `std`.
    - Reference: `plan.md` (Constraints), `data-model.md` (deep_causality_rand Crate)
    - File: `deep_causality_rand/Cargo.toml`
- [ ] T003 Configure Linting and Formatting: Ensure `clippy` and `rustfmt` are configured for the new crate.
    - Reference: Project conventions (from `GEMINI.md` context)
    - File: `deep_causality_rand/.clippy.toml`, `deep_causality_rand/rustfmt.toml` (or similar project-level configs)
- [ ] T004 Define `RngError` Enum: Define the `RngError` enum in `deep_causality_rand/src/errors` with variants as needed, including `OsRandomGenerator(String)`.
    - Reference: `002-replace-rng-currently/002-replace-rng-currently.md` (FR-012), `data-model.md` (RngError Enum)
    - File: `deep_causality_rand/src/errors/mod.rs`
- [ ] T005 Develop Unit Tests for `RngError`: Write comprehensive unit tests for the `RngError` enum, ensuring all variants and error conversions are tested.
    - Reference: `002-replace-rng-currently/002-replace-rng-currently.md` (FR-013)
    - File: `deep_causality_rand/tests/errors/test_rng_error.rs`

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**
- [ ] T006 [P] Develop Unit Tests for `SipHash13Rng` (Initial Failing): Write comprehensive unit tests for the `SipHash13Rng` implementation, focusing on the `Rng` trait methods. These tests should initially fail.
    - Reference: `plan.md` (Technical Context - Testing, Constitution Check - Testing), `research.md` (Testing Strategy), `data-model.md` (Rng Trait, deep_causality_rand Crate - Internal State)
    - File: `deep_causality_rand/tests/types/test_siphash13_rng.rs`
- [ ] T007 [P] Develop Unit Tests for `os-random` RNG (Initial Failing): Write comprehensive unit tests for the `getrandom`-based RNG implementation, ensuring these tests initially fail when the `os-random` feature is enabled.
    - Reference: `plan.md` (Technical Context - Testing), `research.md` (Testing Strategy), `data-model.md` (Rng Trait)
    - File: `deep_causality_rand/tests/types/test_os_random_rng.rs`

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T008 Implement `SipHash13` PRNG Struct: Create the `SipHash13Rng` struct and its initialization logic.
    - Reference: `data-model.md` (deep_causality_rand Crate - Internal State)
    - File: `deep_causality_rand/src/types/siphash13_rng.rs`
- [ ] T009 Implement `Rng` Trait for `SipHash13Rng`: Implement the `Rng` trait for `SipHash13Rng`, making the tests from T006 pass.
    - Reference: `data-model.md` (Rng Trait)
    - File: `deep_causality_rand/src/types/siphash13_rng.rs`
- [ ] T010 Implement `getrandom`-based RNG: Create the `OsRandomRng` struct and its initialization logic, conditionally compiled with the `os-random` feature.
    - Reference: `plan.md` (Summary, Technical Context - Primary Dependencies)
    - File: `deep_causality_rand/src/types/os_random_rng.rs`
- [ ] T011 Implement `Rng` Trait for `OsRandomRng`: Implement the `Rng` trait for `OsRandomRng`, making the tests from T007 pass.
    - Reference: `data-model.md` (Rng Trait)
    - File: `deep_causality_rand/src/types/os_random_rng.rs`
- [ ] T012 Migrate `Rng` Trait and Dependencies: Move the `Rng` trait and all its depending traits from `@ctx/rng_traits.rs` into `deep_causality_rand/src/traits`.
    - Reference: `plan.md` (Summary, Technical Context, Constitution Check - Architecture), `data-model.md` (Rng Trait)
    - File: `deep_causality_rand/src/traits/rng.rs`

## Phase 3.4: Integration
- [ ] T013 Update `deep_causality_uncertain` `Cargo.toml`: Add `deep_causality_rand` as a dependency to `deep_causality_uncertain/Cargo.toml`.
    - Reference: `plan.md` (Summary), `data-model.md` (deep_causality_uncertain Crate)
    - File: `deep_causality_uncertain/Cargo.toml`
- [ ] T014 Update `deep_causality_uncertain` to use `deep_causality_rand`: Modify `deep_causality_uncertain` to use the `Rng` trait from `deep_causality_rand`.
    - Reference: `plan.md` (Summary), `data-model.md` (deep_causality_uncertain Crate)
    - File: `deep_causality_uncertain/src/**/*.rs` (example path)
- [ ] T015 Update `deep_causality_uncertain` Tests: Ensure all existing tests in `deep_causality_uncertain` pass with the new RNG.
    - Reference: `plan.md` (Constitution Check - Testing), `quickstart.md` (Run Tests for `deep_causality_uncertain`)
    - File: `deep_causality_uncertain/tests/**/*.rs` (example path)

## Phase 3.5: Polish
- [ ] T016 Add Standard Rust Docstrings: Add comprehensive Rust docstrings to all public methods and types in `deep_causality_rand`.
    - Reference: `plan.md` (Constitution Check - Library docs), `research.md` (Library Documentation Format)
    - File: `deep_causality_rand/src/**/*.rs`
- [ ] T017 Manual Constraint Verification: Manually verify that `deep_causality_rand` adheres to zero external dependencies, zero unsafe code, and zero macros.
    - Reference: `quickstart.md` (Verify `deep_causality_rand` Constraints)
- [ ] T018 Run `quickstart.md` Verification Steps: Execute all steps in `quickstart.md` to ensure the feature is fully functional and verified.
    - Reference: `quickstart.md`

## Dependencies
- T001, T002, T003, T004, T005 must complete before T006, T007.
- T006, T007 must complete before T008, T009, T010, T011, T012.
- T008, T009 are sequential.
- T010, T011 are sequential.
- T012 can be done in parallel with T008-T011, but must be done before T014.
- T008-T012 must complete before T013, T014, T015.
- T013, T014, T015 can be parallel [P].
- All tasks T001-T015 must complete before T016, T017, T018.

## Parallel Example
```bash
# Phase 3.1: Setup
Task: "T001 Create deep_causality_rand Crate Structure"
Task: "T002 Configure Cargo.toml for deep_causality_rand"
Task: "T003 Configure Linting and Formatting"
Task: "T004 Define RngError Enum"
Task: "T005 Develop Unit Tests for RngError"

# Phase 3.2: Tests First (TDD) - Can run in parallel
Task: "T006 [P] Develop Unit Tests for SipHash13Rng (Initial Failing)"
Task: "T007 [P] Develop Unit Tests for os-random RNG (Initial Failing)"

# Phase 3.3: Core Implementation - Sequential within each RNG type, T012 can be parallel
# After T006, T007 are done:
Task: "T008 Implement SipHash13 PRNG Struct"
Task: "T009 Implement Rng Trait for SipHash13Rng"
# After T006, T007 are done:
Task: "T010 Implement getrandom-based RNG"
Task: "T011 Implement Rng Trait for OsRandomRng"
# Can run in parallel with T008-T011
Task: "T012 Migrate Rng Trait and Dependencies"

# Phase 3.4: Integration - Can run in parallel
# After T008-T012 are done:
Task: "T013 [P] Update deep_causality_uncertain Cargo.toml"
Task: "T014 [P] Update deep_causality_uncertain to use deep_causality_rand"
Task: "T015 [P] Update deep_causality_uncertain Tests"

# Phase 3.5: Polish - Can run in parallel
# After T013-T015 are done:
Task: "T016 [P] Add Standard Rust Docstrings"
Task: "T017 [P] Manual Constraint Verification"
Task: "T018 [P] Run quickstart.md Verification Steps"
```
- [ ] T004 Define `RngError` Enum: Define the `RngError` enum in `deep_causality_rand/src/errors` with variants as needed, including `OsRandomGenerator(String)`.
    - Reference: `002-replace-rng-currently/002-replace-rng-currently.md` (FR-012), `data-model.md` (RngError Enum)
    - File: `deep_causality_rand/src/errors/mod.rs`
- [ ] T005 Develop Unit Tests for `RngError`: Write comprehensive unit tests for the `RngError` enum, ensuring all variants and error conversions are tested.
    - Reference: `002-replace-rng-currently/002-replace-rng-currently.md` (FR-013)
    - File: `deep_causality_rand/tests/errors/test_rng_error.rs`

## Phase 3.2: Tests First (TDD) ⚠️ MUST COMPLETE BEFORE 3.3
**CRITICAL: These tests MUST be written and MUST FAIL before ANY implementation**
- [ ] T006 [P] Develop Unit Tests for `SipHash13Rng` (Initial Failing): Write comprehensive unit tests for the `SipHash13Rng` implementation, focusing on the `Rng` trait methods. These tests should initially fail.
    - Reference: `plan.md` (Technical Context - Testing, Constitution Check - Testing), `research.md` (Testing Strategy), `data-model.md` (Rng Trait, deep_causality_rand Crate - Internal State)
    - File: `deep_causality_rand/tests/types/test_siphash13_rng.rs`
- [ ] T007 [P] Develop Unit Tests for `os-random` RNG (Initial Failing): Write comprehensive unit tests for the `getrandom`-based RNG implementation, ensuring these tests initially fail when the `os-random` feature is enabled.
    - Reference: `plan.md` (Technical Context - Testing), `research.md` (Testing Strategy), `data-model.md` (Rng Trait)
    - File: `deep_causality_rand/tests/types/test_os_random_rng.rs`

## Phase 3.3: Core Implementation (ONLY after tests are failing)
- [ ] T008 Implement `SipHash13` PRNG Struct: Create the `SipHash13Rng` struct and its initialization logic.
    - Reference: `data-model.md` (deep_causality_rand Crate - Internal State)
    - File: `deep_causality_rand/src/types/siphash13_rng.rs`
- [ ] T009 Implement `Rng` Trait for `SipHash13Rng`: Implement the `Rng` trait for `SipHash13Rng`, making the tests from T006 pass.
    - Reference: `data-model.md` (Rng Trait)
    - File: `deep_causality_rand/src/types/siphash13_rng.rs`
- [ ] T010 Implement `getrandom`-based RNG: Create the `OsRandomRng` struct and its initialization logic, conditionally compiled with the `os-random` feature.
    - Reference: `plan.md` (Summary, Technical Context - Primary Dependencies)
    - File: `deep_causality_rand/src/types/os_random_rng.rs`
- [ ] T011 Implement `Rng` Trait for `OsRandomRng`: Implement the `Rng` trait for `OsRandomRng`, making the tests from T007 pass.
    - Reference: `data-model.md` (Rng Trait)
    - File: `deep_causality_rand/src/types/os_random_rng.rs`
- [ ] T012 Migrate `Rng` Trait and Dependencies: Move the `Rng` trait and all its depending traits from `@ctx/rng_traits.rs` into `deep_causality_rand/src/traits`.
    - Reference: `plan.md` (Summary, Technical Context, Constitution Check - Architecture), `data-model.md` (Rng Trait)
    - File: `deep_causality_rand/src/traits/rng.rs`

## Phase 3.4: Integration
- [ ] T013 Update `deep_causality_uncertain` `Cargo.toml`: Add `deep_causality_rand` as a dependency to `deep_causality_uncertain/Cargo.toml`.
    - Reference: `plan.md` (Summary), `data-model.md` (deep_causality_uncertain Crate)
    - File: `deep_causality_uncertain/Cargo.toml`
- [ ] T014 Update `deep_causality_uncertain` to use `deep_causality_rand`: Modify `deep_causality_uncertain` to use the `Rng` trait from `deep_causality_rand`.
    - Reference: `plan.md` (Summary), `data-model.md` (deep_causality_uncertain Crate)
    - File: `deep_causality_uncertain/src/**/*.rs` (example path)
- [ ] T015 Update `deep_causality_uncertain` Tests: Ensure all existing tests in `deep_causality_uncertain` pass with the new RNG.
    - Reference: `plan.md` (Constitution Check - Testing), `quickstart.md` (Run Tests for `deep_causality_uncertain`)
    - File: `deep_causality_uncertain/tests/**/*.rs` (example path)

## Phase 3.5: Polish
- [ ] T016 Add Standard Rust Docstrings: Add comprehensive Rust docstrings to all public methods and types in `deep_causality_rand`.
    - Reference: `plan.md` (Constitution Check - Library docs), `research.md` (Library Documentation Format)
    - File: `deep_causality_rand/src/**/*.rs`
- [ ] T017 Manual Constraint Verification: Manually verify that `deep_causality_rand` adheres to zero external dependencies, zero unsafe code, and zero macros.
    - Reference: `quickstart.md` (Verify `deep_causality_rand` Constraints)
- [ ] T018 Run `quickstart.md` Verification Steps: Execute all steps in `quickstart.md` to ensure the feature is fully functional and verified.
    - Reference: `quickstart.md`

## Dependencies
- T001, T002, T003, T004, T005 must complete before T006, T007.
- T006, T007 must complete before T008, T009, T010, T011, T012.
- T008, T009 are sequential.
- T010, T011 are sequential.
- T012 can be done in parallel with T008-T011, but must be done before T014.
- T008-T012 must complete before T013, T014, T015.
- T013, T014, T015 can be parallel [P].
- All tasks T001-T015 must complete before T016, T017, T018.

## Parallel Example
```bash
# Phase 3.1: Setup
Task: "T001 Create deep_causality_rand Crate Structure"
Task: "T002 Configure Cargo.toml for deep_causality_rand"
Task: "T003 Configure Linting and Formatting"
Task: "T004 Define RngError Enum"
Task: "T005 Develop Unit Tests for RngError"

# Phase 3.2: Tests First (TDD) - Can run in parallel
Task: "T006 [P] Develop Unit Tests for SipHash13Rng (Initial Failing)"
Task: "T007 [P] Develop Unit Tests for os-random RNG (Initial Failing)"

# Phase 3.3: Core Implementation - Sequential within each RNG type, T012 can be parallel
# After T006, T007 are done:
Task: "T008 Implement SipHash13 PRNG Struct"
Task: "T009 Implement Rng Trait for SipHash13Rng"
# After T006, T007 are done:
Task: "T010 Implement getrandom-based RNG"
Task: "T011 Implement Rng Trait for OsRandomRng"
# Can run in parallel with T008-T011
Task: "T012 Migrate Rng Trait and Dependencies"

# Phase 3.4: Integration - Can run in parallel
# After T008-T012 are done:
Task: "T013 [P] Update deep_causality_uncertain Cargo.toml"
Task: "T014 [P] Update deep_causality_uncertain to use deep_causality_rand"
Task: "T015 [P] Update deep_causality_uncertain Tests"

# Phase 3.5: Polish - Can run in parallel
# After T013-T015 are done:
Task: "T016 [P] Add Standard Rust Docstrings"
Task: "T017 [P] Manual Constraint Verification"
Task: "T018 [P] Run quickstart.md Verification Steps"
```