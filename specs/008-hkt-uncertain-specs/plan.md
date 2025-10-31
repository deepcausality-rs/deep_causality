
# Implementation Plan: [FEATURE]

**Branch**: `[###-feature-name]` | **Date**: [DATE] | **Spec**: [link]
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
This feature integrates Higher-Kinded Types (HKT) with `Uncertain<T>` and `MaybeUncertain<T>` types in the `deep_causality_uncertain` crate. It specifically focuses on implementing the Functor, Applicative, and Monad traits from `deep_causality_haft` to enable more composable, generic, and abstract code for probabilistic computations. The implementation will ensure proper handling of panics by returning `Err` variants and will store Witness types in the `src/extensions/` directory.

## Technical Context
**Language/Version**: Rust 1.75
**Primary Dependencies**: deep_causality_haft
**Storage**: N/A
**Testing**: cargo test
**Target Platform**: Rust environments
**Project Type**: Library
**Performance Goals**: No major changes are expected
**Constraints**: Use `deep_causality_haft` crate, strictly limited to Functor, Applicative, Monad traits, panics must be caught and `Err` variant returned.
**Scale/Scope**: No specific scalability targets

## Constitution Check
*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] Principle I (Convention/Structure): Does the proposed file layout adhere to the `src/{types, traits, errors}` structure?
- [x] Principle II (Testing): Does the plan include tasks for creating unit and integration tests for all new logic?
- [x] Principle III (Performance): Does the design avoid `dyn Trait` and favor static dispatch?
- [x] Principle IV (Safety): Does the plan introduce any new external dependencies? If so, are they vetted?
- [x] Principle V (API Design): Are all proposed public-facing types designed with private fields and explicit accessors?

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
deep_causality_uncertain/
├── src/
│   ├── extensions/
│   └── types/
└── tests/
```

**Structure Decision**: Single project library structure within `deep_causality_uncertain` crate, with new HKT-related implementations residing in `src/extensions/`.

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

## Complexity Tracking
*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

## Risk Assessment and Mitigation

### Technical Risks

- **R1: Complexity of HKT Implementation in Rust**
  - **Description**: Implementing HKT traits (Functor, Applicative, Monad) in Rust can be complex due to Rust's type system limitations and the need for witness types. Additionally, closures used in these implementations require `Send + Sync + 'static` bounds for thread safety and lifetime management, which adds to the complexity.
  - **Mitigation**: Leverage existing patterns and best practices from `deep_causality_haft` crate. Start with simpler traits (Functor) and progressively move to more complex ones (Monad). Conduct thorough code reviews. Explicitly consider `Send + Sync + 'static` requirements during design and implementation of closures.

- **R2: Performance Overhead of HKT Abstractions**
  - **Description**: While zero-cost abstractions are a goal, there's a risk that the HKT implementation might introduce unexpected performance overhead.
  - **Mitigation**: Implement benchmarks for key HKT operations. Regularly profile the code during development. Optimize critical paths if performance targets are not met (current targets are "no major changes expected").

- **R3: Panic Handling Complexity**
  - **Description**: Catching panics and converting them to `Err` variants can be tricky and might not cover all edge cases or might introduce its own overhead.
  - **Mitigation**: Carefully design and test the panic-to-error conversion mechanism. Use Rust's `catch_unwind` where appropriate and ensure proper error propagation.

- **R4: Incorrect Trait Implementations**
  - **Description**: Risk of subtle bugs in `Functor`, `Applicative`, and `Monad` implementations leading to incorrect behavior.
  - **Mitigation**: Extensive unit and integration testing, especially property-based testing if feasible. Thorough code reviews by individuals familiar with functional programming concepts.

### Schedule Risks

- **R5: Underestimation of Implementation Effort**
  - **Description**: The complexity of HKT and panic handling might lead to underestimation of the time required for implementation.
  - **Mitigation**: Break down tasks into smaller, manageable sub-tasks. Regularly track progress and re-evaluate estimates. Prioritize core functionality first.

- **R6: Delays due to External Dependencies**
  - **Description**: Although `deep_causality_haft` is internal, changes or issues within it could impact this feature.
  - **Mitigation**: Maintain good communication with `deep_causality_haft` maintainers. Ensure `deep_causality_haft` is stable and well-tested.

### Quality Risks

- **R7: Incomplete Test Coverage**
  - **Description**: Despite the plan for tests, there's a risk that edge cases or subtle interactions might be missed by tests.
  - **Mitigation**: Focus on comprehensive test coverage, including unit tests for individual trait methods and integration tests for complex scenarios. Consider property-based testing.

- **R8: Code Quality Degradation**
  - **Description**: New code might not adhere to project conventions or introduce technical debt.
  - **Mitigation**: Enforce `make format` and `make fix` regularly. Conduct thorough code reviews. Adhere strictly to the project's Constitution.

### Dependency Risks

- **R9: `deep_causality_haft` API Changes**
  - **Description**: Future changes in `deep_causality_haft` could break the HKT implementation.
  - **Mitigation**: Monitor `deep_causality_haft` development. Abstract away direct dependencies where possible. Use versioning to manage compatibility.

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
- [ ] Post-Design Constitution Check: PASS
- [ ] All NEEDS CLARIFICATION resolved
- [ ] Complexity deviations documented

---
*Based on Constitution v2.1.1 - See `/memory/constitution.md`*
