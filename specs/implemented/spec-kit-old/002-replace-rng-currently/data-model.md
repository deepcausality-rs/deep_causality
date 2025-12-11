# Data Model: Replace RNG

## Entities

### deep_causality_rand Crate
*   **Description**: A new internal Rust crate providing random number generation functionality. It will encapsulate the PRNG logic and the `Rng` trait implementation.
*   **Key Attributes**:
    *   **Internal State**: A struct (e.g., `SipHash13Rng`) holding the current state of the pseudo-random number generator.
        *   Fields: `v0: u64`, `v1: u64`, `v2: u64`, `v3: u64` (example for SipHash13).
        *   Validation: Initialized with a seed, state transitions deterministically.
*   **Relationships**:
    *   Implements the `Rng` trait.
    *   Consumed by `deep_causality_uncertain` crate.

### Rng Trait
*   **Description**: The trait defining the interface for random number generation. This trait, along with its dependencies, will be moved into the `deep_causality_rand` crate.
*   **Key Methods (examples)**:
    *   `fn next_u64(&mut self) -> u64`: Generates a random `u64`.
    *   `fn gen_range<T: SampleUniform>(&mut self, low: T, high: T) -> T`: Generates a random value within a specified range.
*   **Relationships**:
    *   Implemented by the `deep_causality_rand` crate's RNG state struct.

### deep_causality_uncertain Crate
*   **Description**: An existing Rust crate that will be updated to consume the `Rng` trait from `deep_causality_rand` instead of the external `rand` crate.
*   **Relationships**:
    *   Depends on the `deep_causality_rand` crate.

### RngError Enum
*   **Description**: Custom error type for random number generation operations.
*   **Variants**:
    *   `OsRandomGenerator(String)`: Wraps OS-specific errors from the `getrandom` crate.
    *   Other variants as needed for internal RNG errors (e.g., invalid range).
*   **Relationships**:
    *   Returned by methods in the `Rng` trait and `UniformSampler` trait where errors can occur.
