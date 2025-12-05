# Pathway to `no-std`: A Feasibility Report

## UPDATE

Dec/5 2025

Following an in-depth assessment, it was decided to split the deep_causality crate into a core crate and the remaining main crate. Subsequently, the core crate and its dependencies were refactored to be no-std, whereas the deep_causality crate was preserved as is because of its complex data structures.

## 1. Executive Summary

This report assesses the feasibility of porting the `deep_causality` ecosystem to a `no-std` + `alloc` environment,
suitable for use in regulated industries (automotive, avionics, etc.) that rely on certified Rust toolchains.

**Conclusion: The port is highly feasible.** The ecosystem's zero-dependency policy and clean architecture are
significant enablers. The required effort is moderate and consists of well-understood, mechanical changes and key
architectural improvements.

A major finding of this analysis is that the primary `no-std` blockers (concurrency primitives and float math) can be
cleanly managed. The `Float` trait provides a perfect abstraction for `no-std` math, and dependency injection can
eliminate the need for most `std`-only synchronization primitives.

Upon completion, `deep_causality` would likely be the **first and only computational causality library architected for
high-integrity, safety-critical systems**, creating a unique and powerful strategic advantage.

## 2. Motivation & Core Strategy

The primary driver for this port is the increasing use of certified Rust compilers in regulated industries. These
toolchains typically certify `core` and `alloc` but not the full `std` library due to its OS dependencies. To be usable
in these environments, the library must be `no-std` compatible.

The core strategy involves two main principles:

1. **Use of the `alloc` Crate:** All heap-allocated types (`Vec`, `String`, `HashMap`, `BTreeSet`, `Box`, `Arc`) will be
   sourced from the `alloc` crate instead of `std`.
2. **Feature Gating:** All functionality that truly depends on an operating system (e.g., the `std::error::Error` trait,
   threading primitives, file I/O) will be conditionally compiled using a `std` feature flag.

## 3. Crate-by-Crate Analysis & Action Plan

### `deep_causality_num`

This crate is the most critical for the `no-std` transition as it provides the core numerical abstractions.

* **Challenge 1:** The `Float` trait's methods (`sin`, `cos`, `sqrt`, etc.) rely on `std`.
    * **Solution:** This is the central point of the `no-std` math strategy. The `impl Float for f64` and
      `impl Float for f32` blocks will be updated to use `#[cfg]` attributes. When the `std` feature is present, they
      will call the standard library functions. When it is absent, they will call the `libm` crate. This centralizes all
      `no-std` float logic in one place.
* **Challenge 2:** The `Float::classify` method returns `std::num::FpCategory`.
    * **Solution:** This non-essential method will be feature-gated with `#[cfg(feature = "std")]`.

### `deep_causality_data_structures`

* **Challenge 1:** `VectorStorage` uses `Vec`.
    * **Solution:** Use `alloc::vec::Vec`.
* **Challenge 2:** `Grid` uses `std::cell::RefCell`.
    * **Solution:** Replace with the `no-std` compatible `core::cell::Cell`.

### `deep_causality_tensor`

* **Challenge 1:** Uses `Vec` and `String`.
    * **Solution:** Switch to `alloc::vec::Vec` and `alloc::string::String`.
* **Challenge 2:** Implements `std::error::Error`.
    * **Solution:** Feature-gate the `impl` block with `#[cfg(feature = "std")]`.
* **Challenge 3:** `CausalTensorMathExt` uses float functions.
    * **Solution:** Ensure all calls use the `Float` trait from `deep_causality_num`. No other changes are needed, as
      the `no-std` logic is handled in the `num` crate.

### `deep_causality_rand`

* **Challenge 1:** Error types implement `std::error::Error`.
    * **Solution:** Feature-gate the `impl` blocks.
* **Challenge 2:** The default `ThreadRng` uses `thread_local!`.
    * **Solution:** Gate the `ThreadRng` implementation behind the `std` feature.

### `deep_causality_algorithms`

* **Challenge 1:** Uses `HashMap` and `Vec`.
    * **Solution:** Switch to `alloc` counterparts.
* **Challenge 2 (Major):** The optional `parallel` feature depends on `rayon`.
    * **Solution:** The `parallel` feature must be made dependent on the `std` feature. A `no-std` build cannot support
      this parallelism.

### `deep_causality_uncertain`

* **Challenge 1:** Uses `Arc`, `Box`, `Vec`, `HashMap`, `String`.
    * **Solution:** Switch to `alloc` counterparts.
* **Challenge 2 (Major):** Relies on a global static cache using `std::sync::{RwLock, OnceLock}`.
    * **Solution (Architectural Improvement):** Eliminate the global cache. Refactor the sampling mechanism to use
      dependency injection via a new `SampleCache` trait. This removes the `std::sync` dependency entirely.

### `deep_causality` (Core Crate)

* **Challenge 1:** Pervasive use of `std` collections.
    * **Solution:** Systematically replace all imports with their `alloc` counterparts.
* **Challenge 2 (Major):** Use of `std::sync::RwLock` in `Causaloid` and `Assumption`.
    * **Solution:** Replace with `spin::RwLock` when the `std` feature is not present. This requires one small,
      foundational dependency.
* **Challenge 3:** The `time_execution` utility uses `std::time::Instant`.
    * **Solution:** Gate the entire utility with `#[cfg(feature = "std")]`.

## 4. Key Challenges & Strategic Decisions

1. **Centralized Floating-Point Abstraction (`Float` Trait):** This is the cornerstone of the `no-std` strategy. By
   implementing the `std`/`libm` switch inside the `Float` trait in `deep_causality_num`, all other crates can perform
   float math without being aware of the `no-std` context. This is a clean, maintainable, and robust design.

2. **Refactoring the Global Cache (A Major Win):** The decision to eliminate the global cache in
   `deep_causality_uncertain` via dependency injection is a significant architectural improvement. It removes the need
   for `std::sync` primitives, which in turn removes the need for `spin` or `once_cell` as dependencies for that crate,
   making the library even more self-contained.

3. **Essential Concurrency (`RwLock` in `deep_causality`):** For the remaining essential use of `RwLock` in the core
   crate, the "no external dependencies" policy should be amended. The risk of implementing a custom lock is too high.
    * **Recommendation:** Add the `spin` crate as a dependency, enabled only for `no-std` builds. It is the standard,
      audited solution for this problem.

4. **Conditional Parallelism (`rayon`):** The `parallel` feature in `deep_causality_algorithms` is inherently `std`
   -only. The `Cargo.toml` must be configured to make this feature dependent on the `std` feature to prevent invalid
   build configurations.

## 5. Proposed Implementation Plan

The port should be executed incrementally, starting from the bottom of the dependency stack.

1. **Phase 1: Foundational Crates.**
    * Convert `deep_causality_num` (add `libm` and implement the `Float` trait abstraction).
    * Convert `deep_causality_data_structures` (replace `RefCell`).
    * Convert `deep_causality_tensor`.

2. **Phase 2: Core Logic & Utilities.**
    * Convert `deep_causality_rand` (gate `ThreadRng`).
    * Refactor and convert `deep_causality_uncertain` (dependency-inject `SampleCache`).
    * Convert `ultragraph`.

3. **Phase 3: Top-Level Crates.**
    * Convert `deep_causality_algorithms` (gate `parallel` feature).
    * Convert the main `deep_causality` crate (introduce `spin::RwLock` for `no-std`).

4. **Phase 4: Workspace Integration & Validation.**
    * Create a top-level workspace feature to enable `no-std` builds across all crates.
    * Build and test the entire ecosystem for a known `no-std` target (e.g., `thumbv7em-none-eabihf`).

## 6. Risk Assessment and Mitigation

### Risk: Incorrect Crate Conversion Order

* **Risk:** Attempting to convert crates without first converting their local dependencies will lead to cascading build
  failures, making incremental testing impossible.
* **Mitigation:** Strictly follow the phased implementation plan. By starting with foundational crates and moving up the
  dependency tree, we ensure that at each stage, a crate's dependencies are already `no-std` compliant.

### Risk: Flawed Concurrency Primitives

* **Risk:** Incorrectly implementing or using concurrency primitives can introduce subtle data races or deadlocks.
* **Mitigation:** This risk is managed with a two-pronged approach:
    1. **Eliminate:** Refactor the `deep_causality_uncertain` cache to use dependency injection, completely removing the
       need for a lock there.
    2. **Use a Trusted Crate:** For the remaining essential use in `deep_causality`, use the battle-tested `spin` crate.
       The risk of using this small, audited dependency is far lower than the risk of a custom implementation.

### Risk: Incomplete `std` Feature Gating

* **Risk:** Accidentally leaving a `std`-only item (e.g., `println!`, `std::error::Error`) outside of a
  `#[cfg(feature = "std")]` block will break the `no-std` build.
* **Mitigation:** The CI/CD pipeline must be updated with a dedicated job that compiles the entire workspace for a
  `no-std` target with default features disabled (`--no-default-features`). This provides immediate, automated feedback.

### Risk: Performance Regression in `no-std` Mode

* **Risk:** The `libm` crate (software-based float math) may be slower than the hardware-backed intrinsics used by
  `std`.
* **Mitigation:** The `Float` trait provides an abstraction point. While `libm` will be the default for `no-std`, this
  architecture allows for adding target-specific features in the future to enable hardware-specific math libraries if
  needed.

## 7. Conclusion

The path to making `deep_causality` a `no-std` library is clear and achievable. The project's clean architecture and
your strategy to centralize float math via the `Float` trait make the porting process robust and maintainable. By
executing this plan, `deep_causality` will be positioned as the only library of its kind ready for the next generation
of safety-critical and high-integrity systems.
