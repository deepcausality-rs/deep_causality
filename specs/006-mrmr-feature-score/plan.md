# Implementation Plan: MRMR Feature Score


**Branch**: `006-mrmr-feature-score` | **Date**: 2025-09-30 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/Users/marvin/RustroverProjects/dcl/deep_causality/specs/006-mrmr-feature-score/spec.md`

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
The MRMR feature selection algorithms (`mrmr_features_selector` and `mrmr_features_selector_cdl`) will be updated to return not only the selected feature indices but also their corresponding importance scores. This allows for a better understanding of the relative contribution of each feature.

## Technical Context
**Language/Version**: Rust 1.75
**Primary Dependencies**: deep_causality_tensor
**Storage**: N/A
**Testing**: cargo test
**Target Platform**: N/A
**Project Type**: single
**Performance Goals**: N/A
**Constraints**: N/A
**Scale/Scope**: N/A

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Simplicity**:
- Projects: 1
- Using framework directly? Yes
- Single data model? Yes
- Avoiding patterns? Yes

**Architecture**:
- EVERY feature as library? Yes
- Libraries listed: deep_causality_algorithms
- CLI per library: N/A
- Library docs: N/A

**Testing (NON-NEGOTIABLE)**:
- RED-GREEN-Refactor cycle enforced? Yes
- Git commits show tests before implementation? Yes
- Order: Contract→Integration→E2E→Unit strictly followed? N/A
- Real dependencies used? Yes
- Integration tests for: new libraries, contract changes, shared schemas? N/A
- FORBIDDEN: Implementation before test, skipping RED phase

**Observability**:
- Structured logging included? N/A
- Frontend logs → backend? N/A
- Error context sufficient? Yes

**Versioning**:
- Version number assigned? Yes
- BUILD increments on every change? Yes
- Breaking changes handled? Yes

## Project Structure

### Documentation (this feature)
```
specs/006-mrmr-feature-score/
├── plan.md              # This file (/plan command output)
├── research.md          # Phase 0 output (/plan command)
├── data-model.md        # Phase 1 output (/plan command)
├── quickstart.md        # Phase 1 output (/plan command)
└── tasks.md             # Phase 2 output (/tasks command - NOT created by /plan)
```

### Source Code (repository root)
```
# Option 1: Single project (DEFAULT)
src/
src/
├── errors/
├── traits/
└── types/

tests/
├── errors/
├── traits/
└── types/

# Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/
```

**Structure Decision**: Option 1

## Phase 0: Outline & Research
Completed. See [research.md](research.md).

## Phase 1: Design & Contracts
Completed. See [data-model.md](data-model.md) and [quickstart.md](quickstart.md).

## Phase 2: Task Planning Approach

1. **Update Error Type**: Add the `FeatureScoreError(String)` variant to the `MrmrError` enum in `deep_causality_algorithms/src/feature_selection/mrmr/mrmr_error.rs` and add a corresponding test case in `deep_causality_algorithms/tests/feature_selection/mrmr/mrmr_error_tests.rs`.

2. **Modify Algorithm Implementations**:
    * For both `mrmr_features_selector` and `mrmr_features_selector_cdl`, change the return type to `Result<Vec<(usize, f64)>, MrmrError>`.
    * The selection logic will be updated to store and return the feature index along with its corresponding score (F-statistic for the first feature, mRMR score for subsequent ones).
    * Add a check to return the new `FeatureScoreError` if a division by zero would occur during the redundancy calculation.

3. **Update Tests**: Update all tests in `deep_causality_algorithms/tests/feature_selection/mrmr/mrmr_algo_tests.rs` and `deep_causality_algorithms/tests/feature_selection/mrmr/mrmr_algo_cdl_tests.rs` to match the new `Vec<(usize, f64)>` return type and ensure all assertions are correct.

4. **Verification**: After all changes, run `make format && make fix` to format the code and fix any linting issues. Then, run `cargo test -p deep_causality_algorithms` to ensure everything compiles and all tests pass.

5. **Documentation (Post-Verification)**:
    * Add a new file `example_mrmr.rs` in `deep_causality_algorithms/examples/` to demonstrate the new functionality.
    * Update the main `deep_causality_algorithms/README.md` to reflect the changes and include usage instructions for the new example.

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| | | |


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