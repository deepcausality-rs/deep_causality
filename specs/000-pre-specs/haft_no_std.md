# Plan to make `deep_causality_haft` `no-std` compatible

## Current Status Overview

The `deep_causality_haft` crate currently relies on `std` library features primarily through its `extensions` module and in the `utils_tests.rs` file. The core `algebra` and `core` modules, which contain trait definitions and HKT machinery, appear to be largely `no-std` compatible themselves. The primary areas of concern are dynamic collections (`Vec`, `LinkedList`, `HashMap`, `BTreeMap`), `Box`, and `String`, as well as `std::cell::RefCell`.

### Identified `std` Usages:

*   **`lib.rs`**: No direct `std` usage.
*   **`Cargo.toml`**: Depends on `deep_causality_num`. No direct `std` dependencies for `deep_causality_haft` itself.
*   **`core` module (`hkt.rs`, `hkt_unbound.rs`)**: Appears `no-std` compatible. Uses traits and `PhantomData`.
*   **`algebra` module**: Contains trait definitions. Generally `no-std` compatible. `deep_causality_num::Zero` used in `comonad.rs` is assumed to be `no-std` compatible.
*   **`effect_system` module**: Contains trait definitions. Appears `no-std` compatible. Examples in documentation mention `String` and `Vec<String>`, indicating implementations might use them.
*   **`extensions` module**: **Primary area of `std` dependency.**
    *   `std::collections::Vec`, `VecDeque`, `LinkedList`, `HashMap`, `BTreeMap` are used for HKT implementations.
    *   `std::boxed::Box` is used for `BoxWitness`.
    *   `std::string::String` is implicitly used as part of `HashMap`/`BTreeMap` keys/values and in error types for `ResultWitness` examples.
    *   `std::cell::RefCell` is used in some examples within `profunctor.rs` and `unbound_haft.rs` for `FnMut` closures.
*   **`utils_tests.rs`**: Uses `Vec<String>` and `Option<String>` in its custom effect types (`MyCustomEffectType`, `MyCustomEffectType4`, `MyCustomEffectType5`) for logging and error reporting. This file is for testing/demonstration purposes.

## Proposed Plan for `no-std` Compatibility

The plan will focus on migrating or removing `std`-dependent components, leveraging the `alloc` crate for dynamic memory allocation when desired, or offering a purely `core` environment.

### 1. Configure `Cargo.toml` for `no-std`

*   Add `#![no_std]` at the top of `src/lib.rs`.
*   Modify `Cargo.toml` to:
    *   Specify `edition = "2021"` (already done).
    *   Add `default-features = false` to the `deep_causality_haft` package.
    *   Introduce a feature flag `alloc` to enable heap-allocated types (`Box`, `Vec`, `String`, `collections`).
    *   Conditional dependency on `deep_causality_num` with `default-features = false` and ensure `deep_causality_num` is also `no-std` compatible (this is an assumption for this plan).

```toml
[package]
name = "deep_causality_haft"
# ... existing fields ...

[features]
default = []
alloc = [] # Enables `alloc` crate features for dynamic collections and Box/String

[dependencies]
deep_causality_num = { path = "../deep_causality_num", version = "0.1", default-features = false } # Ensure this is also no-std compatible
```

### 2. Handle `alloc` Crate Introduction

*   Conditionally compile `alloc` related imports and code using `#[cfg(feature = "alloc")]`.
*   Replace `std::` paths with `alloc::` where appropriate (e.g., `std::vec::Vec` -> `alloc::vec::Vec`).

### 3. Migrate `extensions` Module

This module requires significant changes to either remove `std` collections or wrap them under an `alloc` feature.

*   **Heap-Allocated Collections (`Vec`, `VecDeque`, `LinkedList`, `HashMap`, `BTreeMap`)**:
    *   For `VecWitness`, `VecDequeWitness`, `LinkedListWitness`, `HashMapWitness`, `BTreeMapWitness` implementations, change `std::collections` types to their `alloc::collections` counterparts (`alloc::vec::Vec`, `alloc::collections::VecDeque`, `alloc::collections::LinkedList`, `alloc::collections::BTreeMap`, `alloc::collections::HashMap`).
    *   Wrap these implementations entirely under `#[cfg(feature = "alloc")]`. If the `alloc` feature is not enabled, these witnesses and their implementations will not be available, ensuring pure `no-std` compatibility.
*   **`BoxWitness`**:
    *   Replace `std::boxed::Box` with `alloc::boxed::Box`.
    *   Implement `BoxWitness` and its associated trait implementations (Functor, Applicative, Monad, CoMonad) under `#[cfg(feature = "alloc")]`.
*   **`String` Usage**:
    *   If `String` is used as a generic parameter (e.g., error types in `ResultWitness` implementations or `HashMap`/`BTreeMap` keys/values), it should be replaced with `alloc::string::String` and fall under the `alloc` feature. Alternatively, provide implementations using `&'static str` or a custom fixed-size string type for truly `no-alloc` scenarios.
*   **`std::cell::RefCell`**:
    *   `std::cell::RefCell` is not available in `no-std`. The examples in `profunctor.rs` and `unbound_haft.rs` that use `RefCell` should either be:
        *   **Removed or commented out**: As they are examples and not core library functionality.
        *   **Refactored**: If interior mutability is absolutely critical in a `no-std` context, an alternative like `spin::Mutex` from the `spin` crate (a `no-std` compatible locking primitive) could be considered, but this adds another dependency. Given the context, removal or conditional compilation is preferred.

### 4. Address `utils_tests.rs`

This file is a test utility and not part of the core library.

*   **Option 1 (Recommended)**: Move `src/utils_tests.rs` to the `tests/` directory (e.g., `tests/utils/effect_tests.rs`) and rename it accordingly. This removes it from the main `lib.rs` compilation path for `no-std` builds.
*   **Option 2**: Refactor `utils_tests.rs` itself to be `no-std` compatible by:
    *   Replacing `Vec<String>` with `alloc::vec::Vec<alloc::string::String>` under the `alloc` feature.
    *   Ensuring `MyCustomEffectType`, `MyCustomEffectType4`, `MyCustomEffectType5` use `alloc` types conditionally.

### 5. Review Trait Bounds

*   **`Default` bound**: Traits like `MonadEffect5` have a `Default` bound on generic type `U`. While `Default` is in `core`, complex types (e.g., `Vec<String>`) often derive their `Default` implementation via `std`. Ensure that for types used with these traits in a `no-std` environment, a `Default` implementation is either available via `core` or explicitly provided without `std` dependencies. This is generally manageable as `Default` for primitive types and `Option`/`Result` is `no-std` compatible.

## Impact on Functionality

*   **`alloc` feature enabled**: The `deep_causality_haft` crate will provide full functionality, including implementations for dynamic collections, `Box`, and `String`, making it usable in `no-std` environments with an allocator.
*   **`alloc` feature disabled**: The crate will be purely `core` compatible. Implementations for `Vec`, `VecDeque`, `LinkedList`, `HashMap`, `BTreeMap`, and `Box` will be unavailable. Users will need to provide their own HKT witnesses for `no-alloc` compatible collection types (e.g., `arrayvec` or custom fixed-size types). `OptionWitness` and `ResultWitness` will remain fully functional as they rely on `core` types.

This plan aims to provide flexibility for consumers of `deep_causality_haft` to choose between a minimal `no-std` build (without `alloc`) or a more feature-rich `no-std` build (with `alloc`).
