# Research for MaybeUncertain<T>

## Technical Context Decisions

*   **Language/Version**: Rust 1.75. The project is a Rust monorepo, and this version is a stable, recent choice.
*   **Primary Dependencies**: The feature will be implemented in the `deep_causality_uncertain` crate and integrated into the `deep_causality` crate. No new external dependencies are required.
*   **Storage**: Not applicable. This is an in-memory data structure.
*   **Testing**: `cargo test`. Standard for Rust projects.
*   **Target Platform**: Platform-agnostic. As a library, it should compile on any platform supported by the Rust compiler.
*   **Project Type**: Single project (library).
*   **Performance Goals**: While no specific benchmarks are defined in the spec, the implementation should be a zero-cost abstraction where possible, avoiding unnecessary allocations and overhead, as stated in NFR-002.
*   **Constraints**: Must not introduce breaking changes to the `Uncertain<T>` type, as stated in NFR-001.
*   **Scale/Scope**: The scope is limited to the definition and implementation of the `MaybeUncertain<T>` type, its associated functions, and its integration within the existing `deep_causality` ecosystem.

## Research Findings

The feature is self-contained and builds upon existing patterns within the `deep_causality` library. The primary challenge is API design. The decision to create a new type `MaybeUncertain<T>` instead of modifying `Uncertain<T>` is sound, as it avoids breaking changes and maintains semantic clarity. The proposed monadic `lift_to_uncertain` function is a key element, providing a safe way to transition from probabilistic presence to a definite (but uncertain) value. No further research is needed as the specification is clear and self-contained.
