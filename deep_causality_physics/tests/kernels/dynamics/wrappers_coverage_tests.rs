/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for error branches in `kernels::dynamics::wrappers`.

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{Mass, kinetic_energy};

// =============================================================================
// kinetic_energy wrapper kernel-error branch (wrappers.rs:55)
// =============================================================================

#[test]
fn test_kinetic_energy_wrapper_kernel_error_path() {
    // A non-finite velocity component drives `kinetic_energy_kernel` into its
    // `!v_sq.is_finite()` error, which the wrapper forwards as an error effect
    // (wrappers.rs:55, the `Err(e) =>` arm of the inner kernel match).
    let mass = Mass::<f64>::new(2.0).unwrap();
    let velocity = CausalMultiVector::new(
        vec![0.0, f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let effect = kinetic_energy(&mass, &velocity);
    assert!(!effect.is_ok());
}

// NOTE on defensively-unreachable wrapper arms in `kernels::dynamics::wrappers`:
//   * wrappers.rs:53 — the inner `Energy::new(v)` `Err` arm of `kinetic_energy`.
//     `Energy::new` unconditionally returns `Ok` (Energy may be negative; it
//     performs no validation), so the inner error arm can never run.
//   * wrappers.rs:70, 72 — the inner `Energy::new` `Err` arm and the outer
//     kernel `Err` arm of `rotational_kinetic_energy`. `Energy::new` is
//     infallible (line 70), and `rotational_kinetic_energy_kernel`'s only error
//     path is `R::from_f64(0.5)` failing — infallible for f64 — so the outer
//     arm (line 72) is also unreachable for f64.
