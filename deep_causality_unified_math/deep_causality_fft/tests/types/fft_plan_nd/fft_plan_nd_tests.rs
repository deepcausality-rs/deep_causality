/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_fft::{FftError, FftPlanNd, naive_dft};
use deep_causality_num_complex::Complex;

fn buf(n: usize) -> Vec<Complex<f64>> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            Complex::new((x * 0.37).sin() + 0.2, (x * 0.11).cos() - 0.4)
        })
        .collect()
}

/// Reference N-D DFT: the naïve 1-D DFT applied independently along each
/// axis of a row-major array.
fn naive_dft_nd(data: &[Complex<f64>], shape: &[usize]) -> Vec<Complex<f64>> {
    let mut out = data.to_vec();
    let d = shape.len();
    for a in 0..d {
        let len = shape[a];
        let inner: usize = shape[a + 1..].iter().product();
        let block = len * inner;
        for block_start in (0..out.len()).step_by(block) {
            for r in 0..inner {
                let line: Vec<Complex<f64>> =
                    (0..len).map(|j| out[block_start + j * inner + r]).collect();
                let spec = naive_dft(&line);
                for (j, v) in spec.into_iter().enumerate() {
                    out[block_start + j * inner + r] = v;
                }
            }
        }
    }
    out
}

fn max_err(a: &[Complex<f64>], b: &[Complex<f64>]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| ((x.re - y.re).abs()).max((x.im - y.im).abs()))
        .fold(0.0, f64::max)
}

fn assert_matches_naive_nd(shape: &[usize], tol: f64) {
    let n: usize = shape.iter().product();
    let plan = FftPlanNd::<f64>::new(shape).unwrap();
    assert_eq!(plan.len(), n);
    assert_eq!(plan.shape(), shape);
    assert!(!plan.is_empty());
    let mut data = buf(n);
    let reference = naive_dft_nd(&data, shape);
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
    plan.execute(&mut data, &mut scratch).unwrap();
    let err = max_err(&data, &reference);
    assert!(err < tol, "shape {shape:?}: max error {err:e}");
}

#[test]
fn test_2d_matches_naive() {
    assert_matches_naive_nd(&[4, 8], 1e-10);
    assert_matches_naive_nd(&[8, 4], 1e-10);
    assert_matches_naive_nd(&[16, 16], 1e-10);
}

#[test]
fn test_3d_matches_naive() {
    assert_matches_naive_nd(&[4, 4, 4], 1e-10);
    assert_matches_naive_nd(&[8, 4, 2], 1e-10);
}

#[test]
// Ignored under Miri: the N-D naive-DFT reference over a 16x32x8 grid is
// O(N^2) in the 4096-point volume (~56 s under Miri). The N-D FFT walk's UB is
// covered under Miri by the round-trip tests; full correctness runs in normal CI.
#[cfg_attr(miri, ignore)]
fn test_anisotropic_shape_matches_naive() {
    // Unequal axis lengths, including a non-power-of-two axis
    // (Bluestein inside the N-D walk).
    assert_matches_naive_nd(&[16, 32, 8], 1e-9);
    assert_matches_naive_nd(&[6, 8], 1e-10);
}

#[test]
fn test_1d_shape_degenerates_to_fft() {
    assert_matches_naive_nd(&[64], 1e-9);
}

#[test]
fn test_unit_axes_are_identity() {
    assert_matches_naive_nd(&[1, 16], 1e-10);
    assert_matches_naive_nd(&[16, 1], 1e-10);
    assert_matches_naive_nd(&[1, 1], 1e-12);
}

#[test]
fn test_round_trip_16_cubed() {
    let shape = [16usize, 16, 16];
    let n = 16 * 16 * 16;
    let plan = FftPlanNd::<f64>::new(&shape).unwrap();
    let original = buf(n);
    let mut data = original.clone();
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
    plan.execute(&mut data, &mut scratch).unwrap();
    plan.execute_inverse(&mut data, &mut scratch).unwrap();
    let err = max_err(&data, &original);
    assert!(err < 1e-11, "round-trip error {err:e}");
}

#[test]
// Ignored under Miri: a 32x32x32 = 32768-point 3-D FFT round-trip is ~117 s
// under Miri (large buffer + allocation churn). The N-D walk's UB is covered
// under Miri by the smaller-shape tests; full correctness runs in normal CI.
#[cfg_attr(miri, ignore)]
fn test_round_trip_32_cubed() {
    let shape = [32usize, 32, 32];
    let n = 32 * 32 * 32;
    let plan = FftPlanNd::<f64>::new(&shape).unwrap();
    let original = buf(n);
    let mut data = original.clone();
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
    plan.execute(&mut data, &mut scratch).unwrap();
    plan.execute_inverse(&mut data, &mut scratch).unwrap();
    let err = max_err(&data, &original);
    assert!(err < 1e-11, "round-trip error {err:e}");
}

#[test]
fn test_invalid_shapes_rejected() {
    assert_eq!(
        FftPlanNd::<f64>::new(&[]).unwrap_err(),
        FftError::InvalidLength(0)
    );
    assert_eq!(
        FftPlanNd::<f64>::new(&[4, 0, 2]).unwrap_err(),
        FftError::InvalidLength(0)
    );
}

#[test]
fn test_buffer_validation() {
    let plan = FftPlanNd::<f64>::new(&[4, 4]).unwrap();
    let mut data = buf(8);
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
    assert_eq!(
        plan.execute(&mut data, &mut scratch).unwrap_err(),
        FftError::LengthMismatch {
            expected: 16,
            got: 8
        }
    );
    let mut data = buf(16);
    let mut tiny: Vec<Complex<f64>> = Vec::new();
    let err = plan.execute(&mut data, &mut tiny).unwrap_err();
    assert!(matches!(err, FftError::ScratchTooSmall { .. }));
    let err = plan.execute_inverse(&mut data, &mut tiny).unwrap_err();
    assert!(matches!(err, FftError::ScratchTooSmall { .. }));
}

#[test]
fn test_plan_is_cloneable_and_debuggable() {
    let plan = FftPlanNd::<f64>::new(&[8, 8]).unwrap();
    let cloned = plan.clone();
    assert_eq!(cloned.len(), 64);
    assert!(format!("{plan:?}").contains("FftPlanNd"));
}
