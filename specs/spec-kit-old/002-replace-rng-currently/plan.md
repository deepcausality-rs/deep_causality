# Implementation Plan: Replace RNG


**Branch**: `002-replace-rng-currently` | **Date**: Tuesday, September 16, 2025 | **Spec**: /Users/marvin/RustroverProjects/dcl/deep_causality/specs/002-replace-rng/002-replace-rng.md
**Input**: Feature specification from `/Users/marvin/RustroverProjects/dcl/deep_causality/specs/002-replace-rng/002-replace-rng.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path (DONE)
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION) (DONE)
   → Detect Project Type from context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
4. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
5. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, or `GEMINI.md` for Gemini CLI).
6. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
7. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md) (DONE)
8. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Replace the `rand` crate in `deep_causality_uncertain` with a new internal crate `deep_causality_rand`. This new crate will provide random number generation with zero external dependencies (only `std`), zero unsafe code, and zero macros. It will use `SipHash13` by default and offer an `os-random` feature flag for OS-backed randomness via `getrandom`. The `Rng` trait and its dependencies will be moved into `deep_causality_rand`.

## Technical Context
**Language/Version**: Rust 1.75+
**Primary Dependencies**: `std` library, `getrandom` (conditional via feature flag)
**Storage**: N/A
**Testing**: `cargo test` (with 100% coverage goal)
**Target Platform**: Cross-platform (Rust's default targets)
**Project Type**: Library
**Performance Goals**: No specific performance goals or benchmarks.
**Constraints**: Zero external dependencies (except `std`), zero unsafe code, zero macros.
**Scale/Scope**: Internal library for `deep_causality_uncertain`.

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 1 (deep_causality_rand)
- Using framework directly? Yes (std lib)
- Single data model? Yes (RNG state)
- Avoiding patterns? Yes (no complex patterns)

**Architecture**:
- EVERY feature as library? Yes
- Libraries listed: deep_causality_rand (RNG functionality), deep_causality_uncertain (consumes RNG)
- CLI per library: N/A (RNG library)
- Library docs: Standard Rust docstring documentation on all public methods.

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? Yes (100% test coverage)
- Git commits show tests before implementation? Yes (process detail)
- Order: Contract→Integration→E2E→Unit strictly followed? Only unit testing with 100% branch coverage.
- Real dependencies used? Yes (std, getrandom)
- Integration tests for: new libraries, contract changes, shared schemas? Yes (deep_causality_uncertain will test deep_causality_rand)
- FORBIDDEN: Implementation before test, skipping RED phase? Yes

**Observability**:
- Structured logging included? No logging.
- Frontend logs → backend? N/A
- Error context sufficient? Yes (custom `RngError` enum with variants and unit tests)

**Versioning**:
- Version number assigned? Yes (project-level)
- BUILD increments on every change? Yes (project-level)
- Breaking changes handled? Yes (relevant for Rng trait changes)

## Project Structure

### Documentation (this feature)
```
specs/[###-feature]/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
# Option 1: Single project (DEFAULT)
src/
├── errors/
├── traits/
└── types/

tests/
├── errors/
├── traits/
└── types/
```

**Structure Decision**: Option 1: Single project (DEFAULT)

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context** above:
   - For each NEEDS CLARIFICATION → research task
   - For each dependency → best practices task
   - For each integration → patterns task

2. **Generate and dispatch research agents**:
   ```
   For each unknown in Technical Context:
     Task: "Research {unknown} for {feature context}"
   For each technology choice:
     Task: "Find best practices for {tech} in {domain}"
   ```

3. **Consolidate findings** in `research.md` using format:
   - Decision: [what was chosen]
   - Rationale: [why chosen]
   - Alternatives considered: [what else evaluated]

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Entity name, fields, relationships
   - Validation rules from requirements
   - State transitions if applicable

2. **Generate API contracts** from functional requirements:
   - For each user action → endpoint
   - Use standard REST/GraphQL patterns
   - Output OpenAPI/GraphQL schema to `/contracts/`

3. **Generate contract tests** from contracts:
   - One test file per endpoint
   - Assert request/response schemas
   - Tests must fail (no implementation yet)

4. **Extract test scenarios** from user stories:
   - Each story → integration test scenario
   - Quickstart test = story validation steps

5. **Update agent file incrementally** (O(1) operation):
   - Run `/scripts/bash/update-agent-context.sh gemini` for your AI assistant
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Load `/templates/tasks-template.md` as base
- Generate tasks from Phase 1 design docs (contracts, data model, quickstart)
- Each contract → contract test task [P]
- Each entity → model creation task [P] 
- Each user story → integration test task
- Implementation tasks to make tests pass

**Ordering Strategy**:
- TDD order: Tests before implementation 
- Dependency order: Models before services before UI
- Mark [P] for parallel execution (independent files)

**Estimated Output**: 25-30 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Risk Analysis

### Risk 1: Quality of `SipHash13` as a PRNG
*   **Description**: Relying on `SipHash13` (intended as a hasher) for pseudo-random number generation might lead to statistical biases or insufficient randomness for certain applications, even if it's "sufficient for general-purpose use."
*   **Mitigation**:
    *   **Clear Documentation**: Clearly document the statistical properties and limitations of the `SipHash13`-based PRNG, advising users on its suitability for different use cases.
    *   **Feature Flag for Stronger RNG**: The `os-random` feature flag, enabling `getrandom`, already provides a path for users requiring cryptographically secure or higher-quality randomness. Promote its use for sensitive applications.

### Risk 2: Complexity of `Rng` Trait Migration
*   **Description**: Moving the `Rng` trait and "all its depending traits" from `@ctx/rng_traits.rs` into `deep_causality_rand` might be more complex than anticipated, especially if there are many implicit dependencies or subtle behaviors.
*   **Mitigation**:
    *   **Dependency Analysis**: Before migration, perform a detailed analysis of the `Rng` trait and its ecosystem to identify all directly and indirectly dependent traits and types.
    *   **Incremental Migration**: Migrate the trait and its dependencies incrementally, ensuring that each step maintains compilation and passes existing tests (if any).
    *   **Dedicated Unit Tests**: Develop comprehensive unit tests for the `Rng` trait and its implementations within `deep_causality_rand` to ensure correct behavior after migration.

### Risk 3: Integration Issues with `deep_causality_uncertain`
*   **Description**: The migration of `deep_causality_uncertain` to use the new `deep_causality_rand` crate might introduce integration bugs or unexpected behavior.
*   **Mitigation**:
    *   **Comprehensive Integration Tests**: Ensure `deep_causality_uncertain` has a robust suite of integration tests that thoroughly exercise its use of the `Rng` trait.
    *   **Feature Flag for Old RNG (Temporary)**: Consider a temporary feature flag in `deep_causality_uncertain` to easily switch between the old `rand` crate and `deep_causality_rand` during the migration and testing phase, allowing for quick comparisons and rollbacks.

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |


## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [x] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [x] Complexity deviations documented

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*