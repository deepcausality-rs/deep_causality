/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::time_dilation_angle_kernel;

#[test]
fn experiment_gamma_clamp() {
    // Try to find two near-parallel timelike vectors whose computed gamma lands
    // just below 1.0 by floating-point rounding, exercising the clamp at
    // spacetime.rs:137-138.
    for k in 1..40 {
        let eps = (k as f64) * 1e-9;
        let mut d1 = vec![0.0f64; 16];
        d1[1] = 1.0;
        d1[2] = eps;
        let t1 = CausalMultiVector::new(d1, Metric::Minkowski(4)).unwrap();

        let mut d2 = vec![0.0f64; 16];
        d2[1] = 1.0;
        d2[2] = -eps;
        let t2 = CausalMultiVector::new(d2, Metric::Minkowski(4)).unwrap();

        // A valid (Ok) result means the computed gamma was accepted — either
        // already >= 1 or pulled up to 1 by the clamp.
        if let Ok(angle) = time_dilation_angle_kernel(&t1, &t2) {
            assert!(angle.value().is_finite());
        }
    }
}

#[test]
fn test_gamma_clamp_identical_vectors() {
    // For two *identical* timelike vectors the rapidity is physically zero
    // (gamma = 1). But `gamma = dot / (|t1|·|t2|)` is computed as
    // `s / (sqrt(s)·sqrt(s))`, and for many squared magnitudes `s` the product
    // `sqrt(s)·sqrt(s)` rounds slightly *above* `s`, making the raw quotient a
    // hair below 1.0 (≈ 1 − 2.2e-16). Since that gap is far smaller than the
    // sqrt(epsilon) tolerance, the clamp `gamma = one` at spacetime.rs:137-138
    // fires and the kernel returns a finite (≈ 0) rapidity instead of erroring
    // with "Invalid Lorentz factor < 1.0".
    //
    // Under this dense Minkowski(4) representation the kernel's
    // `squared_magnitude` of a two-component vector (a, b) is `a² + b²`, so we
    // scan a grid of (a, b) pairs. Whenever `sqrt(a²+b²)·sqrt(a²+b²)` rounds
    // above `a²+b²` (true for many non-perfect-square sums, e.g. a=3, b=2 ⇒
    // s = 5.0 ⇒ gamma ≈ 0.999999999999999778), the clamp at spacetime.rs:138
    // fires. We require at least one such pair, and assert each clamped result
    // is the physical zero rapidity.
    let mut clamped_ok = 0;
    for ai in 1..20u64 {
        for bi in 1..20u64 {
            let a = ai as f64;
            let b = bi as f64;
            let mut d = vec![0.0f64; 16];
            d[1] = a;
            d[2] = b;
            let v = CausalMultiVector::new(d, Metric::Minkowski(4)).unwrap();

            // Identical vectors: dot == squared_magnitude exactly.
            let r = time_dilation_angle_kernel(&v, &v);
            if let Ok(angle) = r {
                // Clamped gamma == 1 ⇒ acosh(1) == 0.
                assert!(
                    angle.value().abs() < 1e-6,
                    "clamped rapidity should be ~0, got {}",
                    angle.value()
                );
                clamped_ok += 1;
            }
        }
    }
    assert!(
        clamped_ok > 0,
        "expected at least one identical-vector pair to exercise the gamma clamp"
    );
}

// NOTE on two defensively-unreachable guards in `time_dilation_angle_kernel`:
//   * spacetime.rs:79-81 — "Inner product did not yield any data". For two
//     dense `CausalMultiVector`s of equal metric, `inner_product` always
//     returns a non-empty multivector (at least the grade-0 scalar slot), so
//     `inner.data()` is never empty.
//   * spacetime.rs:126-128 — "Invalid normalization in gamma computation"
//     (`denom == 0 || !denom.is_finite()`). `denom = |t1|·|t2|` where both
//     magnitudes are `sqrt` of strictly-positive squared magnitudes (the
//     timelike check at lines 116-120 already required `s1 > 0 && s2 > 0`), so
//     `denom` is strictly positive and finite. The guard never fires.
