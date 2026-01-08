# Design Review: `deep_causality_physics`
**Date:** 2025-12-08
**Target:** `specs/current/deep_causality_physics.md`

## Executive Summary

The specification provides a solid modular foundation and correctly identifies Type Safety (via Newtypes) as a priority. However, the current design suffers from a critical architectural flaw regarding **performance granularity** and a significant safety gap in **data validation**. As it stands, the library will be safe to use but too slow for high-fidelity simulations (LHC scale) and permits unphysical states (negative mass).

## Critical Findings

### 1. The "Monadic Granularity" Issue (Performance)
**Severity:** CRITICAL
**Observation:** The requirement that *all* physics functions (e.g., `kinetic_energy`, `ideal_gas_law`) return `PropagatingEffect<T>` is architecturally unsound for a physics engine.
**Impact:** `PropagatingEffect` implies tracking, error handling, and potential logging. Wrapping basic arithmetic operations ($E = 1/2 mv^2$) in this structure introduces massive overhead (allocations, pointer chasing) that destroys CPU cache locality and prevents auto-vectorization (SIMD). In a particle simulation loop running $10^9$ times, this design is non-viable.
**Recommendation:**
*   **Dual-Layer Architecture:**
    *   **Kernel Layer:** Implement standalone calculations in a `kernels` submodule or using a `_kernel` suffix.
    *   **Causal Layer:** Wraps kernels in `PropagatingEffect`.
*   **Naming Convention:**
    *   **Kernel Function:** `pub fn {name}_kernel(...) -> T` (or `Result<T>`).
    *   **Causal Wrapper:** `pub fn {name}(...) -> PropagatingEffect<T>`.
    *   *Example:* `kinetic_energy_kernel` (math) vs `kinetic_energy` (causal graph).

### 2. The `pub f64` Safety Violation
**Severity:** HIGH
**Observation:** `pub struct Mass(pub f64);`
**Impact:** This allows `let m = Mass(-50.0);`. A "Newtype" that exposes its inner primitive publicly without validation is just a type alias with extra steps. It provides syntactic sugar but zero semantic guarantee.
**Recommendation:**
*   Fields must be private: `pub struct Mass(f64);`
*   Implement `impl Mass { pub fn new(v: f64) -> Result<Self, CausalityError> { ... } }`
*   Enforce invariants (Mass > 0, Probability <= 1.0).
*   Add `fn new_unchecked(v: f64)` for performance-critical paths where validation is done upstream.

### 3. Missing Source of Truth: Physical Constants
**Severity:** MEDIUM
**Observation:** The spec mentions `$E=mc^2$` and `Boltzmann factor` but doesn't define where $c$, $k_B$, $h_{bar}$, $G$ come from.
**Impact:** "Magic numbers" will creep into implementations. Discrepancies in constants (e.g. using 2.998e8 vs 299792458) cause subtle bugs across modules.
**Recommendation:**
*   Add a `constants` module.
*   Use CODATA 2022 recommended values.
*   Constants should be typed (e.g., `pub const C: Speed = Speed(299_792_458.0);`).

### 4. Serialization Gap
**Severity:** MEDIUM
**Observation:** No mention of serialization strategy.
**Impact:** Physics simulations generate large datasets requiring efficient checkpointing. Standard `serde` introduces significant overhead (CPU/Storage) which is unacceptable for localized HPC data.
**Recommendation:** 
*   **Use `rkyv`:** Implement `rkyv::Archive, Serialize, Deserialize` as a type extension for zero-copy deserialization.
*   **Opt-In:** Must be behind a feature flag (e.g., `features = ["rkyv-support"]`) that is disabled by default to minimize compile times for users who don't need it.

## Refined Design Proposal (Example)

```rust
// src/constants.rs
pub const C: Speed = Speed::new_unchecked(299_792_458.0);

// src/dynamics/quantities.rs
// src/dynamics/quantities.rs
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize))]
// #[cfg_attr(feature = "rkyv", rkyv(check_bytes))] // Optional validation
pub struct Mass(f64); // Private field

impl Mass {
    pub fn new(val: f64) -> Result<Self, CausalityError> {
        if val < 0.0 { return Err(CausalityError::PhysicalInvariantBroken("Negative Mass")); }
        Ok(Self(val))
    }
    
    // For internal tight loops where we know inputs are valid
    pub unsafe fn new_unchecked(val: f64) -> Self { Self(val) }
    
    pub fn value(&self) -> f64 { self.0 }
}

// src/dynamics/mod.rs

// 1. Math Kernel (The "Compute")
// Naming: {function_name}_kernel
// Returns: Result<T, Error> or T. No Monad.
pub fn kinetic_energy_kernel(mass: Mass, v: Speed) -> Energy {
    // Fast, stack-based, SIMD-friendly
    let args_valid = mass > Mass(0.0); // Check invariants if needed
    let e_val = 0.5 * mass.value() * v.value().powi(2);
    Energy::new_unchecked(e_val)
}

// 2. Causal Wrapper (The "Graph")
// Naming: {function_name}
// Returns: PropagatingEffect<T>
pub fn kinetic_energy(mass: Mass, v: Speed) -> PropagatingEffect<Energy> {
    // 1. Call Kernel
    let energy = kinetic_energy_kernel(mass, v);
    
    // 2. Wrap in Effect (Logging, History, Error Propagation)
    PropagatingEffect::pure(energy)
}
```

## Conclusion
The design reflects good "Enterprise Software" practices but misses key "Scientific Computing" performance and correctness patterns. implementing the **Dual-Layer API** and **Private Fields with Validation** is mandatory before commissioning.
