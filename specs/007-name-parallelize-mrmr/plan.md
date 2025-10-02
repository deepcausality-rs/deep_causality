
# Implementation Plan: Parallelize mRMR algorithm with Rayon

**Branch**: `007-name-parallelize-mrmr` | **Date**: October 2, 2025 | **Spec**: [./spec.md]
**Input**: Feature specification from `/specs/[###-feature-name]/spec.md`

## Execution Flow (/plan command scope)
```
1. Load feature spec from Input path
   → If not found: ERROR "No feature spec at {path}"
2. Fill Technical Context (scan for NEEDS CLARIFICATION)
   → Detect Project Type from file system structure or context (web=frontend+backend, mobile=app+api)
   → Set Structure Decision based on project type
3. Fill the Constitution Check section based on the content of the constitution document.
4. Evaluate Constitution Check section below
   → If violations exist: Document in Complexity Tracking
   → If no justification possible: ERROR "Simplify approach first"
   → Update Progress Tracking: Initial Constitution Check
5. Execute Phase 0 → research.md
   → If NEEDS CLARIFICATION remain: ERROR "Resolve unknowns"
6. Execute Phase 1 → contracts, data-model.md, quickstart.md, agent-specific template file (e.g., `CLAUDE.md` for Claude Code, `.github/copilot-instructions.md` for GitHub Copilot, `GEMINI.md` for Gemini CLI, `QWEN.md` for Qwen Code or `AGENTS.md` for opencode).
7. Re-evaluate Constitution Check section
   → If new violations: Refactor design, return to Phase 1
   → Update Progress Tracking: Post-Design Constitution Check
8. Plan Phase 2 → Describe task generation approach (DO NOT create tasks.md)
9. STOP - Ready for /tasks command
```

**IMPORTANT**: The /plan command STOPS at step 7. Phases 2-4 are executed by other commands:
- Phase 2: /tasks command creates tasks.md
- Phase 3-4: Implementation execution (manual or via tools)

## Summary
This feature aims to significantly reduce the execution time of the mRMR feature selection algorithm by parallelizing its most computationally intensive loops using the `rayon` crate. The parallelization will be guarded by a `parallel` feature flag, allowing users to opt-in for performance gains on large datasets. The technical approach involves conditionally importing `rayon` and replacing sequential iterations with parallel iterators in the initial and iterative feature selection phases of both `mrmr_features_selector` and `mrmr_features_selector_cdl` functions.

## Technical Context
**Language/Version**: Rust 1.75+ (MSRV 1.89, Edition 2024)  
**Primary Dependencies**: `deep_causality_tensor`, `rayon` (conditional via `parallel` feature)  
**Storage**: N/A  
**Testing**: `cargo test`, `criterion` (for benchmarks)  
**Target Platform**: Cross-platform (Rust)  
**Project Type**: Library (crate within a monorepo)  
**Performance Goals**: Reduced execution time for mRMR on large datasets, to be quantified by benchmarks (non-parallel version takes ~3 minutes on full sepsis data set; some speedup expected).
**Observability**: N/A (no observation or telemetry needed).  
**Constraints**: Adherence to existing codebase conventions, no `unsafe` code, maintain numerical stability.  
**Scale/Scope**: Parallelize mRMR algorithm for improved performance in `deep_causality_algorithms` crate.

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **Principle I (Convention/Structure)**: Yes, the plan adheres to the existing structure by modifying existing algorithm files and following the established pattern for parallelization.
- [x] **Principle II (Testing)**: Yes, the plan includes running existing unit tests and benchmarking to ensure correctness and performance.
- [x] **Principle III (Performance)**: Yes, `rayon` uses zero-cost abstractions and static dispatch, aligning with the performance principle.
- [x] **Principle IV (Safety)**: Yes, `rayon` is a widely used and vetted external dependency. No `unsafe` code will be introduced. The plan includes running `make check`.
- [x] **Principle V (API Design)**: N/A. This feature modifies existing functions and does not introduce new public-facing types with fields.

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
src/
├── causal_discovery/
│   └── surd/
│       └── surd_algo.rs
│       └── surd_algo_cdl.rs
├── feature_selection/
│   └── mrmr/
│       └── mrmr_algo.rs
│       └── mrmr_algo_cdl.rs
└── lib.rs

tests/
├── causal_discovery/
│   └── surd/
└── feature_selection/
    └── mrmr/

**Structure Decision**: This feature involves optimizing existing algorithm implementations within the `deep_causality_algorithms` crate. No new top-level directories or significant structural changes are introduced. The modifications will be contained within the `src/feature_selection/mrmr/` and `tests/feature_selection/mrmr/` directories.

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
   - Run `.specify/scripts/bash/update-agent-context.sh gemini`
     **IMPORTANT**: Execute it exactly as specified above. Do not add or remove any arguments.
   - If exists: Add only NEW tech from current plan
   - Preserve manual additions between markers
   - Update recent changes (keep last 3)
   - Keep under 150 lines for token efficiency
   - Output to repository root

**Output**: data-model.md, /contracts/*, failing tests, quickstart.md, agent-specific file

## Phase 2: Task Planning Approach
*This section describes what the /tasks command will do - DO NOT execute during /plan*

**Task Generation Strategy**:
- Load `.specify/templates/tasks-template.md` as base
- Generate tasks from Phase 1 design docs (research.md, spec.md)
- Each parallelization point identified in research.md → implementation task [P]
- Each functional requirement in spec.md → test task
- Implementation tasks to make tests pass

**Ordering Strategy**:
- TDD order: Tests before implementation
- Dependency order: Modify `mrmr_algo.rs` then `mrmr_algo_cdl.rs`
- Mark [P] for parallel execution (independent files)

**Estimated Output**: 5-10 numbered, ordered tasks in tasks.md

**IMPORTANT**: This phase is executed by the /tasks command, NOT by /plan

## Phase 3+: Future Implementation
*These phases are beyond the scope of the /plan command*

**Phase 3**: Task execution (/tasks command creates tasks.md)  
**Phase 4**: Implementation (execute tasks.md following constitutional principles)  
**Phase 5**: Validation (run tests, execute quickstart.md, performance validation)

## Risk Assessment and Mitigation

### Risk 1: Data Races/Incorrect Results due to Shared Mutable State
-   **Assessment**: Parallelizing operations that directly modify shared mutable data structures (`CausalTensor`, `all_features`, `selected_features_with_scores`) can introduce data races and lead to incorrect or inconsistent results.
-   **Mitigation**:
    -   **Imputation**: The `impute_missing_values` function (in `mrmr_algo.rs`) is a pre-processing step that modifies the `CausalTensor` sequentially *before* any parallel computations begin. This ensures thread safety for the tensor during parallel read operations.
    -   **Feature Sets**: The parallel sections will primarily involve *reading* from `all_features` (to get candidate feature indices) and *calculating* scores. The critical operations of *selecting* a feature, *removing* it from `all_features`, and *adding* it to `selected_features_with_scores` will remain sequential. This design ensures that mutable shared state is accessed and modified by only one thread at a time, preventing data races.
    -   **`CausalTensor` Read Access**: All `CausalTensor` operations within the parallel loops will be read-only, which is inherently thread-safe.

### Risk 2: Performance Overhead for Small Datasets
-   **Assessment**: The overhead associated with thread management and synchronization in parallel execution can negate performance gains or even lead to slower execution for small input datasets.
-   **Mitigation**:
    -   **Feature Gate**: The existing `parallel` feature flag directly addresses this. Users can opt to compile the library without this feature if their use case primarily involves small datasets or if parallelization is not desired.
    -   **Runtime Thresholding (Consideration)**: While not part of the initial plan, a future enhancement could involve a runtime check for the number of elements to be processed. If the count falls below a certain threshold, the algorithm could dynamically switch to the sequential path, even if the `parallel` feature is enabled, to avoid unnecessary overhead. For this plan, we rely on the compile-time feature flag.

### Risk 3: Increased Code Complexity and Maintainability
-   **Assessment**: Introducing parallel constructs can make the codebase more complex, potentially increasing the difficulty of understanding, debugging, and maintaining the code.
-   **Mitigation**:
    -   **Clear Delineation**: The use of `#[cfg(feature = "parallel")]` blocks will clearly separate the parallel and sequential implementations, making it easier to follow each logic path.
    -   **Idiomatic `rayon`**: Adhering to `rayon`'s recommended patterns and methods will ensure the parallel code is as clean and readable as possible.
    -   **Comprehensive Testing**: Existing unit tests will be run to verify correctness. The parallel logic will be designed to produce the same functional output as the sequential version, minimizing the need for separate parallel-specific tests.

### Risk 4: Numerical Stability/Floating-Point Differences
-   **Assessment**: The non-associative nature of floating-point arithmetic means that the order of operations in parallel reductions (e.g., `sum`, `max`) can sometimes lead to minute differences in results compared to sequential execution. This is generally not a functional bug but can cause issues with strict equality checks in tests.
-   **Mitigation**:
    -   **Floating-Point Comparison Tolerance**: Existing and future tests involving `f64` comparisons should use an epsilon-based tolerance (e.g., `assert!((a - b).abs() < epsilon)`) rather than direct `assert_eq!`.
    -   **Tie-Breaking**: The mRMR algorithm's specification already notes that the selection order for features with identical scores is not guaranteed. `rayon`'s `max_by_key` behavior for ties is generally consistent but may differ from a specific sequential order. Given the existing ambiguity, this is an acceptable characteristic.

## Code Security Risk Assessment and Safe Code Guidelines

### Code Security Risk Assessment

1.  **Unsafe Code Introduction**:
    -   **Assessment**: Incorrect usage of `rayon` or interaction with other parts of the codebase could inadvertently introduce `unsafe` blocks or lead to memory unsafety.
    -   **Mitigation**: Adhere strictly to Rust's safety guidelines. Avoid `unsafe` blocks entirely. Rely on `rayon`'s safe abstractions. The project's `Constitution` already prohibits `unsafe` code in library crates.

2.  **Dependency Vulnerabilities**:
    -   **Assessment**: `rayon` is a well-maintained and widely used library, but any external dependency can potentially have vulnerabilities.
    -   **Mitigation**: Regularly audit dependencies using tools like `cargo audit`. Keep `rayon` updated to the latest stable version. The project's `Constitution` emphasizes vetting external dependencies.

3.  **Denial of Service (Resource Exhaustion)**:
    -   **Assessment**: Excessive parallelization on resource-constrained systems could lead to CPU or memory exhaustion, potentially causing a denial of service. While `rayon` adapts to available cores, very large inputs could still strain resources.
    -   **Mitigation**: `rayon`'s default behavior is generally robust. The existing feature gate allows users to opt-out of parallelization. For extremely large datasets, consider adding a configurable limit to the number of threads `rayon` uses, or a mechanism to fall back to sequential execution if resource limits are approached (more of a performance/stability concern).

4.  **Numerical Precision Attacks**:
    -   **Assessment**: Subtle differences in floating-point arithmetic due to parallelization could, in highly specific and contrived scenarios, lead to different outcomes that might be exploited if the results were used in a decision-making process with security implications. This is highly unlikely for mRMR.
    -   **Mitigation**: Continue to use floating-point comparison tolerances in tests. Document any known precision differences if they are significant.

### Safe Code Guidelines for Implementation

1.  **No `unsafe` code**: Strictly avoid `unsafe` blocks or functions.
2.  **Dependency Management**: Ensure `rayon` is kept up-to-date and regularly scanned for vulnerabilities (`cargo audit`).
3.  **Input Validation**: Ensure that any new code paths introduced by parallelization do not bypass or weaken existing validation checks (e.g., for tensor dimensions, `num_features`, `target_col`).
4.  **Error Handling**: Maintain robust error handling, propagating `MrmrError` variants consistently, especially for numerical issues (`NaN`, `Infinity`). Parallel execution should not mask or alter error conditions.
5.  **Resource Management**: Rely on `rayon`'s default work-stealing scheduler for efficient resource utilization. Avoid manual thread management.
6.  **Deterministic Behavior (where critical)**: Ensure that the core numerical calculations remain consistent across parallel and sequential runs (within floating-point tolerance).

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
- [ ] Complexity deviations documented

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*
