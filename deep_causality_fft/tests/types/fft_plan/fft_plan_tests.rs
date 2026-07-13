/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_fft::{FftError, FftPlan, naive_dft};
use deep_causality_num::Float106;
use deep_causality_num_complex::Complex;

fn buf_f64(n: usize) -> Vec<Complex<f64>> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            Complex::new((x * 0.37).sin() + 0.2, (x * 0.11).cos() - 0.4)
        })
        .collect()
}

fn max_err_f64(a: &[Complex<f64>], b: &[Complex<f64>]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| ((x.re - y.re).abs()).max((x.im - y.im).abs()))
        .fold(0.0, f64::max)
}

/// Forward transform via the plan, compared against the naïve DFT.
fn assert_matches_naive_f64(n: usize, tol: f64) {
    let plan = FftPlan::<f64>::new(n).unwrap();
    assert_eq!(plan.len(), n);
    assert!(!plan.is_empty());
    let mut data = buf_f64(n);
    let reference = naive_dft(&data);
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
    plan.execute(&mut data, &mut scratch).unwrap();
    let err = max_err_f64(&data, &reference);
    assert!(err < tol, "n={n}: max error {err:e} exceeds {tol:e}");
}

#[test]
fn test_small_kernels_match_naive() {
    // The hardcoded planner base cases.
    for n in [1usize, 2, 4, 8, 16, 32] {
        assert_matches_naive_f64(n, 1e-12);
    }
}

#[test]
// Ignored under Miri: the O(n^2) naive-DFT reference at n up to 1024 is
// interpreter-pathological (>6 min). The Stockham kernel's UB is covered under
// Miri by the O(n log n) round-trip test; full correctness runs in normal CI.
#[cfg_attr(miri, ignore)]
fn test_stockham_powers_of_two_match_naive() {
    for n in [64usize, 128, 256, 512, 1024] {
        assert_matches_naive_f64(n, 1e-9);
    }
}

#[test]
// Ignored under Miri: the O(n^2) naive-DFT reference at n up to 1009 is
// interpreter-pathological (>6 min). The Bluestein kernel's UB is covered under
// Miri by the O(n log n) round-trip test; full correctness runs in normal CI.
#[cfg_attr(miri, ignore)]
fn test_bluestein_lengths_match_naive() {
    // Primes and composites that are not powers of two, including ones
    // below the small-kernel cutoff.
    for n in [3usize, 5, 6, 7, 12, 31, 100, 121, 127, 1000, 1009] {
        assert_matches_naive_f64(n, 1e-8);
    }
}

#[test]
fn test_round_trip_identity() {
    for n in [1usize, 2, 8, 32, 64, 256, 12, 127, 1000] {
        let plan = FftPlan::<f64>::new(n).unwrap();
        let original = buf_f64(n);
        let mut data = original.clone();
        let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
        plan.execute(&mut data, &mut scratch).unwrap();
        plan.execute_inverse(&mut data, &mut scratch).unwrap();
        let err = max_err_f64(&data, &original);
        assert!(err < 1e-10, "n={n}: round-trip error {err:e}");
    }
}

#[test]
fn test_inverse_matches_conjugate_reuse_definition() {
    // ifft(x) must be bit-identical to conj(fft(conj(x)))/n computed by hand.
    let n = 64usize;
    let plan = FftPlan::<f64>::new(n).unwrap();
    let original = buf_f64(n);
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];

    let mut via_inverse = original.clone();
    plan.execute_inverse(&mut via_inverse, &mut scratch)
        .unwrap();

    let mut by_hand: Vec<Complex<f64>> =
        original.iter().map(|z| Complex::new(z.re, -z.im)).collect();
    plan.execute(&mut by_hand, &mut scratch).unwrap();
    let inv_n = 1.0 / n as f64;
    for z in by_hand.iter_mut() {
        *z = Complex::new(z.re * inv_n, -z.im * inv_n);
    }

    for (a, b) in via_inverse.iter().zip(by_hand.iter()) {
        assert_eq!(a.re.to_bits(), b.re.to_bits());
        assert_eq!(a.im.to_bits(), b.im.to_bits());
    }
}

#[test]
fn test_f32_matches_naive() {
    let n = 128usize;
    let plan = FftPlan::<f32>::new(n).unwrap();
    let mut data: Vec<Complex<f32>> = (0..n)
        .map(|i| {
            let x = i as f32;
            Complex::new((x * 0.37).sin(), (x * 0.11).cos())
        })
        .collect();
    let reference = naive_dft(&data);
    let mut scratch = vec![Complex::new(0.0f32, 0.0); plan.scratch_len()];
    plan.execute(&mut data, &mut scratch).unwrap();
    for (a, b) in data.iter().zip(reference.iter()) {
        assert!((a.re - b.re).abs() < 1e-3);
        assert!((a.im - b.im).abs() < 1e-3);
    }
}

#[test]
fn test_float106_accuracy_exceeds_f64() {
    // The transform is precision-generic; at Float106 the agreement with
    // the naïve DFT must be far below f64 rounding.
    let n = 16usize;
    let plan = FftPlan::<Float106>::new(n).unwrap();
    let mut data: Vec<Complex<Float106>> = (0..n)
        .map(|i| {
            let x = Float106::from_f64(i as f64);
            let a = Float106::from_f64(0.37);
            let b = Float106::from_f64(0.11);
            Complex::new(
                deep_causality_algebra::Real::sin(x * a),
                deep_causality_algebra::Real::cos(x * b),
            )
        })
        .collect();
    let reference = naive_dft(&data);
    let mut scratch =
        vec![Complex::new(Float106::from_f64(0.0), Float106::from_f64(0.0)); plan.scratch_len()];
    plan.execute(&mut data, &mut scratch).unwrap();
    for (a, b) in data.iter().zip(reference.iter()) {
        let dre = a.re - b.re;
        let dim = a.im - b.im;
        assert!(dre.hi().abs() < 1e-28, "re err {:e}", dre.hi());
        assert!(dim.hi().abs() < 1e-28, "im err {:e}", dim.hi());
    }
}

#[test]
fn test_scratch_classes_by_planner_dispatch() {
    // Small kernels are scratch-free; the Stockham pipeline needs n; the
    // Bluestein fallback needs its convolution buffers. This pins the
    // planner's dispatch classes without exposing internals.
    for n in [2usize, 4, 8, 16, 32] {
        assert_eq!(FftPlan::<f64>::new(n).unwrap().scratch_len(), 0);
    }
    assert_eq!(FftPlan::<f64>::new(64).unwrap().scratch_len(), 64);
    // Prime length: m = next_power_of_two(2n−1) plus the inner plan's n.
    let p = FftPlan::<f64>::new(1009).unwrap();
    assert!(p.scratch_len() >= 2 * 2048);
}

#[test]
fn test_zero_length_rejected() {
    assert_eq!(
        FftPlan::<f64>::new(0).unwrap_err(),
        FftError::InvalidLength(0)
    );
}

#[test]
fn test_length_mismatch_rejected() {
    let plan = FftPlan::<f64>::new(8).unwrap();
    let mut data = buf_f64(4);
    let mut scratch: Vec<Complex<f64>> = Vec::new();
    assert_eq!(
        plan.execute(&mut data, &mut scratch).unwrap_err(),
        FftError::LengthMismatch {
            expected: 8,
            got: 4
        }
    );
    let mut data = buf_f64(8);
    assert!(plan.execute(&mut data, &mut scratch).is_ok());
}

#[test]
fn test_scratch_too_small_rejected() {
    let plan = FftPlan::<f64>::new(64).unwrap();
    let mut data = buf_f64(64);
    let mut scratch = vec![Complex::new(0.0, 0.0); 1];
    assert_eq!(
        plan.execute(&mut data, &mut scratch).unwrap_err(),
        FftError::ScratchTooSmall {
            required: 64,
            got: 1
        }
    );
    assert_eq!(
        plan.execute_inverse(&mut data, &mut scratch).unwrap_err(),
        FftError::ScratchTooSmall {
            required: 64,
            got: 1
        }
    );
}

#[test]
fn test_plan_is_cloneable_and_debuggable() {
    let plan = FftPlan::<f64>::new(16).unwrap();
    let cloned = plan.clone();
    assert_eq!(cloned.len(), 16);
    assert!(format!("{plan:?}").contains("FftPlan"));
}
