# Quickstart Guide: Replace RNG

This guide outlines the steps to quickly verify the successful implementation of the `deep_causality_rand` crate and its integration into `deep_causality_uncertain`.

## Prerequisites
*   Rust toolchain installed.
*   Project cloned and set up.

## Verification Steps

### 1. Build the Project
Ensure the entire project builds successfully after the changes.
```bash
cargo build --workspace
```

### 2. Run Unit Tests for `deep_causality_rand`
Verify that all unit tests for the new `deep_causality_rand` crate pass and achieve 100% branch coverage.
```bash
cargo test -p deep_causality_rand -- --show-output
# You may need a tool like `grcov` or `tarpaulin` to check branch coverage
# cargo tarpaulin --workspace --features "os-random" --ignore-tests --output-dir target/tarpaulin --out Html
```

### 3. Verify `deep_causality_rand` Constraints
Manually inspect the `deep_causality_rand` crate to ensure it adheres to the specified constraints:
*   **Zero external dependencies**: Check `Cargo.toml` for `[dependencies]` section (should only list `std` implicitly).
*   **Zero unsafe code**: Search for `unsafe` keyword in the source code.
*   **Zero macros**: Search for `macro_rules!` or other macro definitions/invocations.

### 4. Run Tests for `deep_causality_uncertain`
Ensure that the `deep_causality_uncertain` crate, now using `deep_causality_rand`, still functions correctly and its tests pass.
```bash
cargo test -p deep_causality_uncertain -- --show-output
```

### 5. Test `os-random` Feature Flag (Optional)
If the `os-random` feature is implemented, verify its functionality.
```bash
cargo test -p deep_causality_rand --features "os-random" -- --show-output
cargo test -p deep_causality_uncertain --features "os-random" -- --show-output
```

### Expected Outcome
*   All `cargo build` and `cargo test` commands complete successfully with zero errors.
*   `deep_causality_rand` adheres to the zero dependency, zero unsafe, zero macro constraints.
*   `deep_causality_rand` unit tests achieve 100% branch coverage.
*   `deep_causality_uncertain` functions correctly with the new RNG.
