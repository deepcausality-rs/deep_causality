# Research Findings: Replace RNG

## Performance Goals
*   **Decision**: No specific performance goals or benchmarks will be set for the `deep_causality_rand` crate.
*   **Rationale**: The primary goal is cross-platform compatibility with zero external dependencies and zero unsafe code. The default `SipHash13` PRNG is considered sufficient for general-purpose use. Users requiring higher performance or stronger randomness can opt-in to the `os-random` feature flag, which leverages the `getrandom` crate.
*   **Alternatives considered**: Setting specific throughput/latency targets. Rejected to prioritize portability and simplicity for the default implementation.

## Library Documentation Format
*   **Decision**: Standard Rust docstring documentation will be used for all public methods in `deep_causality_rand`.
*   **Rationale**: This aligns with common Rust development practices and provides clear, accessible documentation within the code.
*   **Alternatives considered**: Using `llms.txt` format. Rejected as it was not explicitly required and standard docstrings are sufficient.

## Testing Strategy for `deep_causality_rand`
*   **Decision**: Only unit testing with 100% branch coverage will be implemented for the `deep_causality_rand` crate.
*   **Rationale**: This ensures the internal logic of the RNG is thoroughly tested while maintaining focus on the library's core functionality. Integration testing for how `deep_causality_rand` interacts with `deep_causality_uncertain` will be handled within the `deep_causality_uncertain` crate's test suite.
*   **Alternatives considered**: Implementing contract, integration, or E2E tests within `deep_causality_rand`. Rejected to keep the library focused and to avoid redundant testing efforts with consuming crates.

## Structured Logging
*   **Decision**: No structured logging will be included in the `deep_causality_rand` crate.
*   **Rationale**: The library is a low-level random number generator, and logging is not considered a core requirement for its functionality. This also helps maintain the "zero external dependencies" constraint.
*   **Alternatives considered**: Implementing structured logging. Rejected to maintain simplicity and avoid unnecessary overhead.
