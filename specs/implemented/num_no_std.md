# Plan to make `deep_causality_num` `no-std` compatible

## Current State Analysis

The `deep_causality_num` crate currently declares no external dependencies in its `[dependencies]` section of `Cargo.toml`. Most core Rust types and operations it uses (like integer primitives, `Option`, `Result`, `core::num::Wrapping`, `core::fmt` traits, `std::ops` traits, `core::cmp::Ordering`, `core::mem::size_of`) are already part of `core` and thus `no-std` compatible.

The primary area requiring attention for `no-std` compatibility are the floating-point (`f32`, `f64`) mathematical functions defined within the `Float` trait and its implementations in `src/float/float_32_impl.rs` and `src/float/float_64_impl.rs`. These include functions like `sin`, `cos`, `sqrt`, `exp`, `log`, `atan2`, etc., which are usually provided by the standard library's `libm` dependency or an external `no-std` compatible `libm` crate.

Another minor `std` dependency is `std::num::FpCategory`, which has a `core` equivalent (`core::num::FpCategory`).

## Proposed Plan

The plan involves modifying `Cargo.toml` to introduce a `no-std` feature and conditionally integrate the `libm` crate for floating-point mathematical operations.

### 1. Modify `Cargo.toml`

The `Cargo.toml` will be updated to:
*   Add `#![no_std]` in `src/lib.rs`.
*   Specify `default-features = false` for the `deep_causality_num` package.
*   Introduce a new feature `libm_math`.
*   Conditionally add `libm` as a dependency under the `libm_math` feature.

```toml
# deep_causality_num/Cargo.toml
[package]
name = "deep_causality_num"
version = "0.1.8"
edition = { workspace = true }
rust-version = { workspace = true }
license = { workspace = true }

repository = "https://github.com/deepcausality/deep_causality.rs"
authors = ["Marvin Hansen <marvin.hansen@gmail.com>", ]
description = "Number utils for for deep_causality crate."
documentation = "https://docs.rs/deep_causality"
categories = ["development-tools"]
keywords = ["number-utils", "traits"]
# Exclude all bazel files as these conflict with Bazel workspace when vendored.
exclude = ["*.bazel", "*/*.bazel",  "*.bazel.*", "BUILD", "BUILD.bazel", "MODULE.bazel", ".bazelignore",".bazelrc", "tests/**/*"]

[features]
default = ["std"] # Keep std by default for convenience, can be turned off
std = [] # Explicitly enable std
libm_math = ["dep:libm"] # Feature to enable libm for no-std floating point math

[dependencies]
# No external dependencies by default
libm = { version = "0.2.7", optional = true } # Add libm as an optional dependency
```

**Note**: The `std` feature is added as default for backward compatibility and ease of use in `std` environments. Users building for `no-std` will explicitly disable default features (`default-features = false`).

### 2. Update `src/lib.rs`

*   Add `#![cfg_attr(not(feature = "std"), no_std)]` at the crate root.

```rust
// src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

// ... existing module declarations ...

// Re-exports
// ... existing pub use statements ...
```

### 3. Modify Floating-Point Implementations (`src/float/float_32_impl.rs` and `src/float/float_64_impl.rs`)

The implementations of the `Float` trait for `f32` and `f64` will be updated to conditionally use `libm` functions when the `std` feature is disabled and the `libm_math` feature is enabled. Otherwise, they will rely on the inherent methods of `f32`/`f64` (which implicitly use `std`'s `libm` if `std` is enabled).

Here's an example for `f32::sin`:

```rust
// src/float/float_32_impl.rs (similar changes for float_64_impl.rs)

// Conditional use of libm if std is not available and libm_math feature is active
#[cfg(all(not(feature = "std"), feature = "libm_math"))]
use libm;

impl Float for f32 {
    // ... other methods ...

    #[inline]
    fn sin(self) -> Self {
        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        libm::sinf(self) // Use libm for no-std, libm_math
        #[cfg(not(all(not(feature = "std"), feature = "libm_math")))]
        self.sin() // Fallback to inherent method (which uses std's libm or hardware)
    }

    // ... similar changes for cos, tan, asin, acos, atan, exp, ln, log, powf, sqrt, cbrt, hypot, sinh, cosh, tanh, asinh, acosh, atanh ...

    // For methods like `is_nan`, `is_finite`, `classify`, `floor`, `ceil`, `round`, `trunc`, `fract`, `abs`, `signum`, `is_sign_positive`, `is_sign_negative`, `mul_add`, `recip`, `powi`, `exp2`, `to_degrees`, `to_radians`, `max`, `min`, `clamp`, `integer_decode`, `copysign`:
    // These methods can often directly use their inherent `f32::` or `f64::` counterparts, as these are usually available even without the full `std` if the target supports floating-point.
    // However, it's safer to either wrap them under `#[cfg(feature = "std")]` or explicitly call `libm` equivalents for consistency in a `no-std` context when `libm_math` is enabled, if `libm` provides them. Many of these (like `is_nan`, `abs`) are inherent methods and don't need `libm`.
    // The current implementation directly uses `f32::method()` which works in `core` context on most targets, but relying on `libm` for the transcendental functions is the primary concern.
}
```
**Note**: `core::num::FpCategory` should be used instead of `std::num::FpCategory`. This requires changing the `use` statement in `src/float/mod.rs`, `src/float/float_32_impl.rs`, and `src/float/float_64_impl.rs` from `use std::num::FpCategory;` to `use core::num::FpCategory;`.

### 4. Adjust `FloatOption` Trait and Implementations (`src/float_option/mod.rs`)

The `FloatOption` trait uses `std::fmt::Debug`, `std::marker::Send`, `std::marker::Sync`. These are already `core` compatible.
The implementations also use `is_nan()`, which is an inherent float method. This module should be `no-std` compatible without changes, given `core::fmt::Debug` is used.

### 5. Review `utils_tests` Module (`src/utils_tests/mod.rs`)

The `utils_tests` module exports `utils_complex_tests` and `utils_octonion_tests`. These contain helper functions (`assert_approx_eq`, `assert_complex_approx_eq`, `assert_octonion_approx_eq`) that are used in the crate's own tests. Since they are part of `src`, they are compiled with the library.

*   **Recommendation**: wrap the entire `src/utils_tests` module and its contents under `#[cfg(feature = "std")]` or `#[cfg(test)]`. 

### 6. Verification

*   After implementing the changes, rebuild the `deep_causality_num` crate with `cargo build --target <your_no_std_target> --no-default-features --features libm_math` to ensure successful compilation.
*   Run tests conditionally if possible, or adapt existing tests to run in a `no-std` environment if necessary.

## Impact on Functionality

*   **`std` feature enabled (default)**: `deep_causality_num` will behave as it does currently, using the standard library's math functions.
*   **`std` feature disabled, `libm_math` enabled**: The crate will be `no-std` compatible and use the `libm` crate for floating-point math functions.
*   **`std` feature disabled, `libm_math` disabled**: The crate will be purely `core` compatible. Floating-point math functions will *not* be available, leading to compilation errors if they are called. This option is for environments that might not even want `libm` (e.g., if custom FPU routines are used), but it would severely limit the functionality of the `Float` trait. Therefore, enabling `libm_math` is strongly recommended for `no-std` builds that require floating-point math.

This plan aims to provide a robust `no-std` compatible version of `deep_causality_num` while maintaining its mathematical functionality.

## Sub-task List for Implementation

1.  **Modify `Cargo.toml`**: Add `std` and `libm_math` features and `libm` dependency.
2.  **Add `#![cfg_attr(not(feature = "std"), no_std)]` to `src/lib.rs`**.
3.  **Change `std::num::FpCategory` to `core::num::FpCategory`**: Update `use` statements in `src/float/mod.rs`, `src/float/float_32_impl.rs`, and `src/float/float_64_impl.rs`.
4.  **Make `src/utils_tests` work for std and non-std
5.  **Modify Floating-Point Implementations (`src/float/float_32_impl.rs` and `src/float/float_64_impl.rs`)**: Conditionally use `libm` functions for transcendental operations.
6.  **Feature flag `std`-only tests**: Identify and feature flag any tests that rely on `std` functionality, making them conditional on the `std` feature.