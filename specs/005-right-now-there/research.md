# Research Findings for Implement Data Cleaner and update Feature Selector for Option<f64>

## Performance Goals
**Decision**: For the `deep_causality_discovery` crate, ease of use and speed of progress are prioritized over raw performance, as it is primarily used in R&D. Therefore, no specific performance goals are set for this initial implementation.
**Rationale**: The primary focus is on correctness and successful integration of the `OptionNoneDataCleaner` and the updated `FeatureSelector` trait with `CausalTensor<Option<f64>>`. Performance optimization can be addressed in a later iteration if profiling indicates bottlenecks, but it is secondary to development velocity and ease of use in R&D contexts.
**Alternatives considered**: None, as performance is not a critical driver for this foundational change.

## Constraints
**Decision**: No specific constraints are set for this initial implementation beyond the existing project conventions and Rust's type system.
**Rationale**: The feature is about adapting existing data structures and algorithms to handle missing values more explicitly. The constraints are inherent in the Rust language and the `deep_causality` ecosystem.
**Alternatives considered**: None.