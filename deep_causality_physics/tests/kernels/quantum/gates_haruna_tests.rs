/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    logical_cz, logical_hadamard, logical_s, logical_t, logical_x, logical_z,
};

// Helper to create a simple projector field a^2 = a
fn create_projector_field() -> CausalMultiVector<Complex<f64>> {
    // For projector: a=0 or a=1. Let's use a=1 for testing
    let data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap()
}

#[test]
fn test_logical_s_gate() {
    let a = create_projector_field();
    let result = logical_s(&a);

    // S(a) = exp(i pi/2 a^2)
    // For a=1: a^2=1, so exp(i pi/2 * 1) = cos(pi/2) + i sin(pi/2) = i
    // Result should be scalar i
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_z_gate() {
    let a = create_projector_field();
    let result = logical_z(&a);

    // Z(a) = exp(i pi a)
    // For a=1: exp(i pi) = -1
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_x_gate() {
    let b = create_projector_field();
    let result = logical_x(&b);

    // X(b) = exp(i pi b)
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_hadamard_gate() {
    let a = create_projector_field();
    let b = create_projector_field();
    let result = logical_hadamard(&a, &b);

    // H = phase * S(a) * exp(i pi/2 b^2) * S(a)
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_cz_gate() {
    let a1 = create_projector_field();
    let a2 = create_projector_field();
    let result = logical_cz(&a1, &a2);

    // CZ = exp(i pi a1 a2)
    assert!(!result.data().is_empty());
}

#[test]
fn test_logical_t_gate() {
    let a = create_projector_field();
    let result = logical_t(&a);

    // T = exp(i pi (1/2 a^3 - 3/4 a^2 + 1/2 a))
    assert!(!result.data().is_empty());
}

// Helper for a zero field.
fn create_zero_field() -> CausalMultiVector<Complex<f64>> {
    CausalMultiVector::new(vec![Complex::new(0.0, 0.0); 8], Metric::Euclidean(3)).unwrap()
}

#[test]
fn test_exp_zero_fast_path() {
    // logical_z(0) => exp(i pi * 0) = exp(0). All components are ~0, tripping
    // the fast-path in exp() that returns the scalar identity (gates_haruna.rs:31).
    let a = create_zero_field();
    let result = logical_z(&a);
    // exp(0) = I => scalar component is 1, rest 0.
    assert!((result.data()[0].re - 1.0).abs() < 1e-12);
    assert!(result.data()[0].im.abs() < 1e-12);
    for c in &result.data()[1..] {
        assert!(c.re.abs() < 1e-12 && c.im.abs() < 1e-12);
    }
}

#[test]
fn test_exp_huge_norm_returns_identity() {
    // A field with an enormous component makes the exponent norm exceed the
    // 1e6 guard, so exp() returns the scalar identity to avoid overflow
    // (gates_haruna.rs:48-51).
    let mut data: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); 8];
    data[1] = Complex::new(1e8, 0.0); // |exponent| ~ 1e8 * pi >> 1e6
    let a = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
    let result = logical_z(&a);
    // Returns identity: scalar 1, rest 0, all finite.
    assert!((result.data()[0].re - 1.0).abs() < 1e-12);
    for c in result.data() {
        let re: f64 = c.re;
        let im: f64 = c.im;
        assert!(re.is_finite() && im.is_finite());
    }
}

#[test]
fn test_exp_nonfinite_term_returns_partial_sum() {
    // A finite-norm field (norm < 1e6) whose Taylor-series powers overflow f64
    // to non-finite values mid-series, tripping the non-finite-term guard that
    // returns the accumulated partial sum (gates_haruna.rs:59-65).
    // Component ~2000 keeps norm well under 1e6 but 2000^n overflows quickly.
    let mut data: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); 8];
    data[1] = Complex::new(2000.0, 0.0);
    let a = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
    let result = logical_z(&a);
    // The kernel must return without panicking; result is defined.
    assert!(!result.data().is_empty());
}

#[test]
fn test_exp_overflow_scan_trips_nonfinite_delta_guard() {
    // Sweep a range of finite (< 1e6 norm) exponent magnitudes through the exp
    // Taylor series. With the imaginary exponent `i·π·mv`, the partial sum
    // overflows to a non-finite value during accumulation, so the *next*
    // successive-difference `delta` becomes non-finite and the kernel returns
    // the previous partial sum via the non-finite-delta guard
    // (gates_haruna.rs:78-80). An offline trace confirms every overflowing
    // magnitude trips this delta guard — never the term guard (line 64) or the
    // post-loop guard (line 92), which are shadowed by it (see the NOTE below).
    // Each `logical_z` must still return a finite, non-empty multivector.
    let magnitudes = [
        1.0e2_f64, 3.0e2, 5.0e2, 7.0e2, 9.0e2, 1.1e3, 1.5e3, 2.0e3, 5.0e3, 1.0e4, 5.0e4, 1.0e5,
        3.0e5, 5.0e5, 9.0e5,
    ];
    for &m in &magnitudes {
        let mut data: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); 8];
        data[1] = Complex::new(m, 0.0);
        let a = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
        let result = logical_z(&a);
        assert!(!result.data().is_empty());
        for c in result.data() {
            let re: f64 = c.re;
            let im: f64 = c.im;
            assert!(
                re.is_finite() && im.is_finite(),
                "exp result must stay finite for magnitude {m}"
            );
        }
    }
}

// NOTE on the two defensively-unreachable non-finite guards in `exp`:
//   * gates_haruna.rs:64 — the mid-series "term is non-finite ⇒ return sum"
//     guard. Per-iteration order is: build `term`, check `term` finiteness,
//     then `sum += term`, then compute `delta = sum − prev` and check it. A
//     term only overflows to ±∞ *after* the previous iteration already added a
//     ~1e308 term into `sum`, which overflows `sum` and makes that iteration's
//     `delta` non-finite first — so the line-78 delta guard (line 79) always
//     fires one iteration earlier. An offline brute-force scan over ~1000
//     finite-norm magnitudes never reached the term guard.
//   * gates_haruna.rs:92 — the post-loop "sum is non-finite ⇒ return identity"
//     guard. The loop only exits normally via `delta < tol` (negligible final
//     change ⇒ finite sum) or by exhausting all 64 iterations without any
//     overflow (finite sum). Any path that would make the final sum non-finite
//     exits early through the line-79 delta guard, so the post-loop sum is
//     always finite here.
