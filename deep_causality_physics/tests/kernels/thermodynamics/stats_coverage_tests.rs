/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for error branches in `kernels::thermodynamics::stats`.

use deep_causality_physics::{
    PhysicsErrorEnum, Temperature, carnot_efficiency_kernel, partition_function_kernel,
};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// carnot_efficiency_kernel: ZeroKelvinViolation (stats.rs:99)
// =============================================================================

#[test]
fn test_carnot_efficiency_zero_hot_reservoir() {
    // T_H = 0 K triggers `th <= 0` ⇒ ZeroKelvinViolation (stats.rs:98-99).
    // Temperature::new(0) is permitted (only strictly-negative values are
    // rejected), so this exercises the kernel's own guard.
    let th = Temperature::<f64>::new(0.0).unwrap();
    let tc = Temperature::<f64>::new(0.0).unwrap();

    let result = carnot_efficiency_kernel(th, tc);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::ZeroKelvinViolation => {}
        e => panic!("Expected ZeroKelvinViolation, got {e:?}"),
    }
}

// =============================================================================
// partition_function_kernel: non-finite beta (stats.rs:233-235)
// =============================================================================

#[test]
fn test_partition_function_non_finite_beta() {
    // For a sub-denormal temperature, `k_B · T` underflows to exactly 0 in
    // f64, so `beta = 1 / (k_B · T)` becomes +∞, hitting the
    // `!beta.is_finite()` guard (stats.rs:232-236).
    let tiny = 1.0e-320_f64; // sub-normal; k_B (~1.38e-23) · tiny underflows to 0
    let temp = Temperature::<f64>::new(tiny).unwrap();
    let energies = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap();

    let result = partition_function_kernel(&energies, temp);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::NumericalInstability(_) => {}
        e => panic!("Expected NumericalInstability (non-finite beta), got {e:?}"),
    }
}

// =============================================================================
// partition_function_kernel: non-finite Z via overflow (stats.rs:253-255)
// =============================================================================

#[test]
fn test_partition_function_non_finite_z_overflow() {
    // Each exponent is clamped to at most 700, so every term is at most
    // e^700 ≈ 1.01e304. Summing enough such terms overflows the f64 sum to
    // +∞, hitting the `!z.is_finite()` guard (stats.rs:252-256).
    //
    // beta = 1/(k_B·T); with a normal temperature and large negative energies,
    // `-beta·e` saturates the clamp at +700 for every entry.
    let temp = Temperature::<f64>::new(1.0).unwrap();
    let n = 25_000usize; // > 1.8e308 / 1.01e304 ≈ 1.78e4 terms to overflow
    let energies = CausalTensor::new(vec![-1.0e30_f64; n], vec![n]).unwrap();

    let result = partition_function_kernel(&energies, temp);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::NumericalInstability(_) => {}
        e => panic!("Expected NumericalInstability (non-finite Z), got {e:?}"),
    }
}

// NOTE on stats.rs:137-138 and 224-225 — the `ok_or_else` closure bodies for
// `R::from_f64(BOLTZMANN_CONSTANT)` in `boltzmann_factor_kernel` and
// `partition_function_kernel`. `from_f64` is infallible for f64, so the
// conversion never returns `None` and these defensive error closures can never
// run for the f64 monomorphisation exercised here. The kernels' reachable error
// paths (ZeroKelvinViolation, non-finite beta, non-finite Z) are covered above.
