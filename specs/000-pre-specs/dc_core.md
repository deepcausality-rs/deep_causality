# Plan: Create `deep_causality_core` Crate

## Objective
Create a new `deep_causality_core` crate to house core functional types, replicating them from the main `deep_causality` crate. This new crate must be `no-std` compatible.

## Scope
Replicate the following types and their dependencies:
1.  **Monad Types**: `deep_causality/src/types/monad_types`
2.  **Effect Log**: `deep_causality/src/types/reasoning_types/effect_log`
3.  **Effect Value**: `deep_causality/src/types/reasoning_types/effect_value`
4.  **Propagating Effect**: `deep_causality/src/types/reasoning_types/propagating_effect`

### Identified Dependencies to Replicate
*   `Alias Primitives`: `deep_causality/src/alias/alias_primitives.rs` (for `ContextoidId`, `IdentificationValue`)
*   `Causality Error`: `deep_causality/src/errors/causality_error.rs`
*   `Numeric Value`: `deep_causality/src/types/reasoning_types/numeric_value`
*   `Propagating Value`: `deep_causality/src/traits/propagating_value`
*   `Intervenable`: `deep_causality/src/traits/intervenable`

## Implementation Plan

### 1. Crate Initialization
*   [ ] Create directory `deep_causality_core`.
*   [ ] Create `deep_causality_core/Cargo.toml`:
    *   Name: `deep_causality_core`
    *   Features: `default = ["std"]`, `std = ["alloc"]`, `alloc = []`.
    *   Dependencies:
        *   `deep_causality_num` (default-features = false)
        *   `deep_causality_haft` (default-features = false, features = ["alloc"])
        *   `deep_causality_tensor` (optional = true, std-only for now)
        *   `deep_causality_uncertain` (optional = true, std-only for now)
*   [ ] Create `deep_causality_core/src/lib.rs`:
    *   Add `#![cfg_attr(not(feature = "std"), no_std)]`.
    *   Add `extern crate alloc;`.
    *   Define modules.

### 2. File Replication & Refactoring
Copy files from `deep_causality` to `deep_causality_core` and refactor imports/usage for `no-std`.

*   **Alias**:
    *   [ ] Copy `src/alias/alias_primitives.rs` -> `deep_causality_core/src/alias.rs`.
    *   [ ] Refactor: Ensure `String` uses `alloc::string::String`.

*   **Errors**:
    *   [ ] Copy `src/errors/causality_error.rs` -> `deep_causality_core/src/errors.rs`.
    *   [ ] Refactor: `std::error::Error` might need `std` feature or `core::error::Error` (if on recent Rust). For now, gate `Error` impl behind `std` or use `alloc`.

*   **Traits**:
    *   [ ] Copy `src/traits/propagating_value` -> `deep_causality_core/src/traits/propagating_value`.
    *   [ ] Copy `src/traits/intervenable` -> `deep_causality_core/src/traits/intervenable`.

*   **Types**:
    *   [ ] Copy `src/types/reasoning_types/numeric_value` -> `deep_causality_core/src/types/numeric_value`.
    *   [ ] Copy `src/types/reasoning_types/effect_log` -> `deep_causality_core/src/types/effect_log`.
        *   Refactor: `SystemTime` usage in `CausalEffectLog` is `std` only. Gate timestamping behind `std` or make it optional.
    *   [ ] Copy `src/types/reasoning_types/effect_value` -> `deep_causality_core/src/types/effect_value`.
        *   Refactor: Gate `HashMap` (Map variant) behind `std`.
        *   Refactor: Gate `Tensor`, `ComplexTensor`, `QuaternionTensor` behind `std` (and `deep_causality_tensor` feature).
        *   Refactor: Gate `Uncertain*`, `MaybeUncertain*` behind `std` (and `deep_causality_uncertain` feature).
    *   [ ] Copy `src/types/reasoning_types/propagating_effect` -> `deep_causality_core/src/types/propagating_effect`.
    *   [ ] Copy `src/types/monad_types` -> `deep_causality_core/src/types/monad_types`.

### 3. Build Configuration
*   [ ] Add `deep_causality_core` to root `Cargo.toml` workspace members.
*   [ ] Create `deep_causality_core/BUILD.bazel` (similar to `haft`/`num`).

### 4. Verification
*   [ ] Run `cargo check -p deep_causality_core --no-default-features` (Core only).
*   [ ] Run `cargo check -p deep_causality_core --no-default-features --features alloc` (Alloc support).
*   [ ] Run `cargo check -p deep_causality_core --all-features` (Std support).
