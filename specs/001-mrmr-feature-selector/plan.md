# Implementation Plan: Maximum Relevance and Minimum Redundancy (mRMR) Feature Selection


**Branch**: `001-in-deep-causality` | **Date**: 2025-09-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/Users/marvin/RustroverProjects/dcl/deep_causality/specs/001-in-deep-causality/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
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
7. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
8. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
Implement the FCQ variant of the mRMR feature selection algorithm. The function will take a `CausalTensor` as input, handle missing data by column-mean imputation, and return a ranked list of feature indices based on relevance (F-statistic) and redundancy (Pearson correlation).

## Technical Context
**Language/Version**: Rust 1.80
**Primary Dependencies**: `deep_causality_data_structures`
**Storage**: N/A
**Testing**: `cargo test`
**Target Platform**: Native
**Project Type**: Single project (library)
**Performance Goals**: Computationally efficient as per the paper's findings for the FCQ variant.
**Constraints**: No new external dependencies, no unsafe code, no macros.
**Scale/Scope**: A single function to be added to the `deep_causality_algorithms` crate.

## Risk Analysis
| Risk ID | Description | Probability | Impact | Mitigation Strategy |
|---|---|---|---|---|
| R-01 | **Numerical Instability** | Medium | High | Implement checks for zero-variance columns before correlation calculations. Use a small epsilon value in denominators to prevent division-by-zero errors in the FCQ score. The `MrmrError::CalculationError` will be used to report any numerical issues. |
| R-02 | **Performance on Large Tensors** | Low | Medium | The FCQ algorithm is chosen for its efficiency. The implementation will prioritize correctness. If performance issues arise with very high-dimensional tensors, further optimization can be a future task. For now, the risk is accepted as low. |
| R-03 | **Incorrect F-Statistic Formula** | Low | High | The planned formula `F = (n-2) * r^2 / (1 - r^2)` is a standard and correct interpretation for a single-feature context. This will be documented in the code. The comprehensive test suite will validate the feature ranking logic, ensuring the end result is correct even if the intermediate value's interpretation has nuances. |
| R-04 | **Ambiguity in Initial Redundancy** | High | Medium | The redundancy formula involves division by the number of selected features, leading to a division-by-zero on the first iteration. This will be mitigated by defining that when the selected set is empty, redundancy is a small, non-zero constant (epsilon), ensuring the first feature is chosen based on relevance alone. |

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 1 (The feature is a new module within an existing crate)
- Using framework directly? Yes
- Single data model? Yes, `CausalTensor` is the primary data model.
- Avoiding patterns? Yes, no complex patterns like Repository or UoW are needed.

**Architecture**:
- EVERY feature as library? Yes, this is a function within the `deep_causality_algorithms` library crate.
- Libraries listed: `deep_causality_algorithms` - To house the new feature selection logic.
- CLI per library: N/A for this feature.
- Library docs: Yes, the public function will be documented with rustdoc.

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? Yes, tests will be written first.
- Git commits show tests before implementation? Yes.
- Order: Contract→Integration→E2E→Unit strictly followed? Yes.
- Real dependencies used? Yes.
- Integration tests for: The new function will have integration tests.
- FORBIDDEN: Implementation before test, skipping RED phase. Yes.

**Observability**:
- Structured logging included? N/A for this feature.
- Frontend logs → backend? N/A.
- Error context sufficient? Yes, a custom `MrmrError` enum will be used.

**Versioning**:
- Version number assigned? This will be part of the crate's versioning.
- BUILD increments on every change? N/A.
- Breaking changes handled? N/A, this is a new feature.

## Project Structure

### Documentation (this feature)
```
specs/001-in-deep-causality/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
├── contracts/           # Phase 1 output (/plan command)
│   └── mrmr.rs
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
# Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/
```

**Structure Decision**: Option 1: Single project (library feature).

## Phase 0: Outline & Research
1. **Extract unknowns from Technical Context** above:
   - All unknowns have been resolved.

2. **Generate and dispatch research agents**:
   - Not required.

3. **Consolidate findings** in `research.md` using format:
   - Done.

**Output**: research.md with all NEEDS CLARIFICATION resolved

## Phase 1: Design & Contracts
*Prerequisites: research.md complete*

1. **Extract entities from feature spec** → `data-model.md`:
   - Done.

2. **Generate API contracts** from functional requirements:
   - Done, see `contracts/mrmr.rs`.

3. **Generate contract tests** from contracts:
   - Will be done in the implementation phase.

4. **Extract test scenarios** from user stories:
   - Done, see `quickstart.md`.

5. **Update agent file incrementally** (O(1) operation):
   - Not required for this workflow.

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

**Estimated Output**: 10-15 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None      |            |                                     |


## Progress Tracking
*This checklist is updated during execution flow*

**Phase Status**:
- [x] Phase 0: Research complete (/plan command)
- [x] Phase 1: Design complete (/plan command)
- [x] Phase 2: Task planning complete (/plan command - describe approach only)
- [ ] Phase 3: Tasks generated (/tasks command)
- [ ] Phase 4: Implementation complete
- [ ] Phase 5: Validation passed

**Gate Status**:
- [x] Initial Constitution Check: PASS
- [x] Post-Design Constitution Check: PASS
- [x] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*