
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
This feature integrates Higher-Kinded Types (HKT) with `Uncertain<T>` and `MaybeUncertain<T>` types in the `deep_causality_uncertain` crate. It specifically focuses on implementing the Functor, Applicative, and Monad traits from `deep_causality_haft` by leveraging the `ConstTree` data container from `deep_causality_ast`. This approach enables more composable, generic, and abstract code for probabilistic computations. The implementation will ensure proper handling of panics by returning `Err` variants and will store Witness types in the `src/extensions/` directory.

## Technical Context
**Language/Version**: Rust 1.75
**Primary Dependencies**: deep_causality_haft, deep_causality_ast
**Storage**: N/A
**Testing**: cargo test
**Target Platform**: Rust environments
**Project Type**: Library
**Performance Goals**: No major changes are expected
**Constraints**: Use `deep_causality_haft` crate, leverage `ConstTree` from `deep_causality_ast` as the underlying data container for HKT implementations, strictly limited to Functor, Applicative, Monad traits, panics must be caught and `Err` variant returned.
**Scale/Scope**: No specific scalability targets

### Architectural Rationale
The original internal representation of `Uncertain<T>` relies on an `Arc<ComputationNode>`, which is a specialized graph structure designed for specific probabilistic computations. This "compute node" design inherently limits the ability to implement generic Higher-Kinded Type (HKT) traits such as `Functor`, `Applicative`, and `Monad` in an idiomatic and efficient manner. The specialized nature of `ComputationNode` makes it difficult to perform arbitrary type transformations (`A -> B` for Functor) or to chain and flatten computations (`A -> F<B>` for Monad) without complex and potentially error-prone graph traversal and reconstruction logic.

`ConstTree` from `deep_causality_ast`, being a persistent, immutable tree structure, provides a more suitable foundation for HKT implementations. Its immutability and generic nature align perfectly with functional programming principles, allowing for:
-   **Pure Transformations**: `fmap` operations can produce new `ConstTree` instances without modifying the original.
-   **Generic Containment**: `ConstTree` can generically hold the probabilistic values of `Uncertain<T>`, facilitating `A -> B` type transformations.
-   **Structured Composition**: The tree structure offers a clear and recursive mechanism for implementing `pure`, `apply`, and `bind` operations, enabling the composition and sequencing of uncertain computations.

However, a critical obstacle remains: the `Functor`, `Applicative`, and `Monad` traits from `deep_causality_haft` define operations with generic type parameters `A` and `B` (e.g., `Func: FnMut(A) -> B`). For `Uncertain<T>`, where `T` represents a probability distribution, the types `A` and `B` cannot be arbitrary. They must be types for which `Uncertain<T>` can meaningfully define and transform probability distributions (i.e., `ProbabilisticType`). This inherent constraint conflicts with the unconstrained generic parameters of the `deep_causality_haft` HKT traits.

To resolve this, `Uncertain<T>` will *not* directly implement the `deep_causality_haft` HKT traits. Instead, a set of *independent HKT-like traits* (`UncertainFunctor`, `UncertainApplicative`, `UncertainMonad`) will be defined. These independent traits will mirror the method signatures of their `deep_causality_haft` counterparts but will explicitly include the `ProbabilisticType` bounds on their generic type parameters (`A` and `B`). This allows `Uncertain<T>` to provide monadic functionality with the necessary type constraints, while still adhering to the HKT pattern.

**Practical Solution for Monadic `Uncertain` and `MaybeUncertain`:**
To bridge this gap and enable robust HKT implementations, the following architectural changes are proposed:
1.  **Introduce `ProbabilisticType` Trait**: A new trait `ProbabilisticType` will be defined in `deep_causality_uncertain/src/traits/probabilistic_type.rs`. This trait will be implemented by types that `Uncertain<T>` can meaningfully operate on (initially `f64` and `bool`), providing methods for conversion to/from `SampledValue` and a `default_value`.
2.  **Constrain `Uncertain<T>`**: The `Uncertain<T>` struct will be constrained with `T: ProbabilisticType`, ensuring type compatibility throughout its operations.
3.  **Redesign `ComputationNode` to `UncertainNodeContent` using `ConstTree`**: The existing `ComputationNode` enum will be **deleted and replaced**. The new internal representation for `Uncertain<T>`'s computation graph will be `ConstTree<UncertainNodeContent<T>>`.
    *   `UncertainNodeContent<T>` will be a new enum (generic over `T: ProbabilisticType`) that represents the nodes in the computation graph. It will contain variants for:
        *   `Value(SampledValue)` (for leaf nodes/point distributions).
        *   `Distribution(DistributionEnum<T>)` (for probabilistic leaf nodes).
        *   `PureOp { value: SampledValue }` (for `pure` operations).
        *   `FmapOp { func: Arc<dyn SampledFmapFn>, operand: ConstTree<UncertainNodeContent<T>> }` (for `fmap`-like operations).
            *   `ApplyOp { func: Arc<dyn SampledFmapFn>, arg: ConstTree<UncertainNodeContent<T>> }` (for `apply`-like operations).
            *   `BindOp { func: Arc<dyn SampledBindFn>, operand: ConstTree<UncertainNodeContent<T>> }` (for `bind`-like operations).
            *   Other symbolic operations (arithmetic, logical, conditional) will be adapted to use `SampledValue` and `ConstTree` operands.
            *   `SampledFmapFn` and `SampledBindFn` traits (operating on `SampledValue`) will be defined to handle type erasure for closures.    *   The `root_node` in `Uncertain<T>` will become `ConstTree<UncertainNodeContent<T>>`.
4.  **Rewrite Core Logic**: All `Uncertain<T>` constructors, operator overloads, and the `SequentialSampler`'s `evaluate_node` will be rewritten to build, traverse, and interpret this new `ConstTree<UncertainNodeContent<T>>` structure, leveraging `ProbabilisticType` for type conversions during sampling.
5.  **Implement HKT-like Traits**: `UncertainWitness` and `MaybeUncertainWitness` will **not** directly implement `deep_causality_haft`'s `Functor`, `Applicative`, and `Monad` traits. Instead, they will implement *independent HKT-like traits* (`UncertainFunctor`, `UncertainApplicative`, `UncertainMonad`) that mirror the `deep_causality_haft` signatures but explicitly include `ProbabilisticType` bounds on their methods. If `deep_causality_haft`'s traits are ever needed for `UncertainWitness` (e.g., for generic functions that expect *any* `Functor`), they would be implemented by **dispatching** to these independent traits, potentially with additional `where` clauses to ensure `A` and `B` adhere to `ProbabilisticType`.

This comprehensive approach allows `Uncertain<T>` and `MaybeUncertain<T>` to become fully monadic by representing their computations as a symbolic `ConstTree` and explicitly managing type compatibility through `ProbabilisticType`, thus enabling their use as monadic subtypes within `Propagating Effect`.

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



-   **R1: Complexity of HKT Implementation in Rust**

    -   **Description**: Implementing HKT traits (Functor, Applicative, Monad) in Rust is inherently complex due to type system limitations, the need for witness types, and the requirement for `Send + Sync + 'static` bounds for closures. This is exacerbated by the probabilistic nature of `Uncertain<T>`.

    -   **Mitigation**: Leverage existing patterns and best practices from `deep_causality_haft`. Start with simpler traits (Functor) and progressively move to more complex ones (Monad). Conduct thorough code reviews. Explicitly consider `Send + Sync + 'static` requirements during design and implementation of closures. Break down HKT implementations into smaller, testable units.



-   **R2: Performance Overhead of HKT Abstractions**

    -   **Description**: The shift to `ConstTree` (immutable data structures, cloning), symbolic function wrapping, and the new sampling logic might introduce unexpected performance overhead compared to the optimized `ComputationNode`.

    -   **Mitigation**: Establish performance benchmarks *before* the rewrite for key operations. Continuously monitor performance during and after the rewrite. Profile critical paths and optimize where necessary. Explore memoization strategies within `ConstTree` evaluation and sampling.



-   **R3: Panic Handling Complexity**

    -   **Description**: Catching panics from user-provided functions (`Func` in HKT traits) and converting them to `Err` variants can be tricky and might not cover all edge cases or introduce its own overhead.

    -   **Mitigation**: Carefully design and test the panic-to-error conversion mechanism. Use Rust's `catch_unwind` where appropriate and ensure proper error propagation. Document expected panic behavior clearly.



-   **R4: Incorrect Trait Implementations**

    -   **Description**: Risk of subtle bugs in `Functor`, `Applicative`, and `Monad` implementations leading to incorrect behavior, especially given the symbolic nature of the `ConstTree` and the probabilistic semantics.

    -   **Mitigation**: Extensive unit and integration testing, especially property-based testing for HKT laws (identity, composition, associativity). Thorough code reviews by individuals familiar with functional programming and probabilistic concepts.



-   **R10: `ProbabilisticType` Trait Design and Implementation**

    -   **Description**: The design of `ProbabilisticType` and its implementations for `f64` and `bool` (and potentially future types) might be insufficient, incorrect, or lead to type mismatches or incorrect probabilistic behavior.

    -   **Mitigation**: Thorough design review of the `ProbabilisticType` trait, its methods (`to_sampled_value`, `from_sampled_value`, `default_value`), and its implementations. Extensive unit tests for all conversion and default value logic. Ensure clear error handling for `from_sampled_value`.



-   **R11: `ConstTree` Node Content Redesign (`UncertainNodeContent<T>`)**

    -   **Description**: The new `UncertainNodeContent<T>` enum must accurately represent all necessary operations (values, distributions, function applications, monadic binds, arithmetic, logical, conditional) and be generic over `T: ProbabilisticType`. Errors in its design could break the entire computation graph and its probabilistic semantics.

    -   **Mitigation**: Modular design of `UncertainNodeContent` with clear separation of concerns for each variant. Comprehensive unit tests for each node type's construction, cloning, and symbolic representation. Leverage existing `ComputationNode` logic as a reference for required operations.



-   **R12: Rewrite of `Uncertain<T>` Constructors and Operators**

    -   **Description**: Rewriting all existing constructors (`point`, `normal`, `bernoulli`) and operator overloads (`Add`, `Sub`, `Mul`, `Div`, `Not`, `BitAnd`, `BitOr`, `BitXor`) to build `ConstTree<UncertainNodeContent<T>>` instances is a massive task prone to errors, potentially altering existing behavior or introducing new bugs.

    -   **Mitigation**: Adopt a strict test-driven development (TDD) approach. Port existing tests for `Uncertain<T>` first, then rewrite the implementation to pass them. Implement new integration tests to ensure parity with old behavior and correctness of new `ConstTree`-based graph construction.



-   **R13: Rewrite of Sampling Logic (`SequentialSampler`)**

    -   **Description**: The `evaluate_node` function is the core of the probabilistic computation. Rewriting it to correctly traverse and interpret the new `ConstTree<UncertainNodeContent<T>>` (including symbolic function applications) is highly complex and critical for the correctness of all probabilistic results.

    -   **Mitigation**: Break down `evaluate_node` into smaller, testable components. Use property-based testing (e.g., with `proptest`) to cover a wide range of computation graphs and input distributions. Compare sampled outputs with known statistical properties for simple, analytically solvable cases. Implement robust error handling for type mismatches during evaluation.



-   **R14: Correctness of HKT Implementations with `ProbabilisticType` Constraints**

    -   **Description**: Ensuring `Functor`, `Applicative`, and `Monad` implementations correctly handle the symbolic embedding of functions and respect the `ProbabilisticType` constraints, especially for `bind` (flattening nested `Uncertain<Uncertain<T>>`), is very challenging. Incorrect implementations could violate HKT laws.

    -   **Mitigation**: Focus on strict adherence to HKT laws (identity, composition, associativity) through dedicated unit tests. Develop specific test cases that verify the probabilistic semantics are preserved across transformations. Use examples from `deep_causality_haft`'s existing implementations (e.g., `OptionWitness`, `ResultWitness`) as a guide for structural correctness.



### Schedule Risks



-   **R5: Underestimation of Implementation Effort**

    -   **Description**: The extensive architectural changes, including the introduction of `ProbabilisticType`, the `ConstTree` redesign, and the complete rewrite of core `Uncertain<T>` logic and sampling, significantly increase the overall implementation effort.

    -   **Mitigation**: Break down the project into very small, manageable sub-tasks with clear definitions of done. Regularly track progress and re-evaluate estimates. Prioritize core functionality first. Allocate buffer time for unforeseen complexities.



-   **R6: Delays due to External Dependencies**

    -   **Description**: `deep_causality_ast` is a new dependency. Issues or API changes within this crate could impact the development schedule.

    -   **Mitigation**: Maintain good communication with `deep_causality_ast` maintainers. Ensure `deep_causality_ast` is stable and well-tested. Abstract away direct dependencies where possible to minimize impact of upstream changes.



### Quality Risks



-   **R7: Incomplete Test Coverage**

    -   **Description**: Despite the plan for tests, the sheer volume and complexity of changes increase the risk that edge cases or subtle interactions might be missed by tests, leading to undetected bugs.

    -   **Mitigation**: Focus on comprehensive test coverage: unit tests for individual components (`ProbabilisticType`, `UncertainNodeContent`), integration tests for constructors and operators, and end-to-end tests for HKT trait implementations. Prioritize tests that verify probabilistic correctness.



-   **R8: Code Quality Degradation**

    -   **Description**: The extensive refactoring and new code might not adhere to project conventions or introduce technical debt, making future maintenance difficult.

    -   **Mitigation**: Enforce `make format` and `make fix` regularly. Conduct thorough code reviews for every pull request. Adhere strictly to the project's Constitution and established coding style.



### Dependency Risks



-   **R9: `deep_causality_haft` API Changes**

    -   **Description**: Future changes in `deep_causality_haft` could break the HKT implementation for `Uncertain<T>`.

    -   **Mitigation**: Monitor `deep_causality_haft` development. Abstract away direct dependencies where possible. Use versioning to manage compatibility.



-   **R16: Compatibility with Existing `Deep Causality` Crate Usage**



    -   **Description**: Changes to `Uncertain<T>` and `MaybeUncertain<T>`'s internal structure and API might inadvertently break existing code in the main `Deep Causality` crate, specifically the `Propagating Effect` type and its associated reasoning logic (e.g., `_evaluate_uncertain_logic` in `collection_reasoning_uncertain.rs`). This includes ensuring continued support for existing constructors, operator overloads, and methods like `greater_than` and `to_bool`. Furthermore, the `EffectEthos` evaluation logic for uncertain norms, which relies on `UncertainActivationPredicate` returning `Uncertain<bool>` and then resolving it via `probability_exceeds` (or `to_bool`) using `UncertainParameter`, is a critical usage pattern that must be preserved.



    -   **Mitigation**: Ensure strict backward compatibility for the public APIs of `Uncertain<T>` and `MaybeUncertain<T>`. Use the `_evaluate_uncertain_logic` function and the `EffectEthos::evaluate_action` method (especially for uncertain norms) as primary integration test cases. Clearly document any unavoidable breaking changes. Run full integration tests of the `Deep Causality` crate after the rewrite to catch regressions early and validate that existing logic functions identically.



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
