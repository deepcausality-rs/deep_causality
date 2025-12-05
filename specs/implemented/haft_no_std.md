# Plan to make `deep_causality_haft` `no-std` compatible

## Current Status Overview

The `deep_causality_haft` crate currently relies on `std` library features primarily through its `extensions` module and in the `utils_tests.rs` file. The core `algebra` and `core` modules appear to be largely `no-std` compatible. The primary areas of concern are dynamic collections (`Vec`, `LinkedList`, `HashMap`, `BTreeMap`), `Box`, and `String`.

### Identified `std` Usages:

*   **`lib.rs`**: No direct `std` usage, but needs configuration for `no_std`.
*   **`Cargo.toml`**: Needs feature flags.
*   **`extensions` module**: **Primary area of `std` dependency.**
    *   `HashMap` is NOT available in `alloc` (requires random source). Must be gated behind `std`.
    *   `Vec`, `VecDeque`, `LinkedList`, `BTreeMap`, `Box`, `String` are available in `alloc`.
*   **`utils_tests.rs`**: Uses `Vec` and `String`. Can be supported with `alloc`.

## Implementation Checklist

- [ ] **Step 1: Crate Configuration**
    - [ ] 1.1. Update `Cargo.toml`:
        - Add `[features]` section:
          ```toml
          [features]
          default = ["std"]
          std = []
          alloc = []
          ```
        - Update `deep_causality_num` dependency to `default-features = false`.
    - [ ] 1.2. Update `src/lib.rs`:
        - Add `#![cfg_attr(not(feature = "std"), no_std)]` at the top.
        - Add `#[cfg(feature = "alloc")] extern crate alloc;`.

- [ ] **Step 2: Refactor `extensions` Module**
    - [ ] 2.1. Update `src/extensions/mod.rs`:
        - Gate `mod func_fold_hash_map_ext;` behind `#[cfg(feature = "std")]`.
        - Gate other alloc-dependent modules (vec, box, linked_list, b_tree_map, etc.) behind `#[cfg(feature = "alloc")]`.
    - [ ] 2.2. Update `src/lib.rs` re-exports:
        - Gate `HashMapWitness` re-export behind `#[cfg(feature = "std")]`.
        - Gate other extension re-exports behind `#[cfg(feature = "alloc")]`.
    - [ ] 2.3. Refactor extension files to use `alloc`:
        - `src/extensions/hkt_box_ext.rs`: `std::boxed::Box` -> `alloc::boxed::Box`.
        - `src/extensions/hkt_vec_ext.rs`: `std::vec::Vec` -> `alloc::vec::Vec`.
        - `src/extensions/hkt_linked_list_ext.rs`: `std::collections::LinkedList` -> `alloc::collections::LinkedList`.
        - `src/extensions/func_fold_b_tree_map_ext.rs`: `std::collections::BTreeMap` -> `alloc::collections::BTreeMap`.
        - `src/extensions/func_fold_vec_deque_ext.rs`: `std::collections::VecDeque` -> `alloc::collections::VecDeque`.
        - `src/extensions/hkt_string_ext.rs` (if exists) or usages of `String`: `std::string::String` -> `alloc::string::String`.
        - `src/extensions/func_fold_hash_map_ext.rs`: Keep as `std` (or use `hashbrown` if desired, but `std` is safer for now).

- [ ] **Step 3: Refactor `utils_tests.rs`**
    - [ ] 3.1. Add `#[cfg(feature = "alloc")]` to the top of `src/utils_tests.rs`.
    - [ ] 3.2. Replace `std` imports with `alloc` imports where applicable (`Vec`, `String`).

- [ ] **Step 4: Update `BUILD.bazel`**
    - [ ] 4.1. Add `crate_features` to `deep_causality_haft/BUILD.bazel` with `std`, `alloc`.

- [ ] **Step 5: Verification**
    - [ ] 5.1. Run `cargo check -p deep_causality_haft --no-default-features` (should pass, only core/algebra).
    - [ ] 5.2. Run `cargo check -p deep_causality_haft --no-default-features --features alloc` (should pass, includes extensions except HashMap).
    - [ ] 5.3. Run `cargo check -p deep_causality_haft --all-features` (should pass, includes everything).
    - [ ] 5.4. Run tests: `cargo test -p deep_causality_haft --all-features`.
