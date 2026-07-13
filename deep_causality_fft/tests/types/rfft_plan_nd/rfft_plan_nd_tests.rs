/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_fft::{FftError, FftPlanNd, RfftPlanNd};
use deep_causality_num_complex::Complex;

fn rbuf(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| ((i as f64) * 0.37).sin() + 0.25 * ((i as f64) * 1.7).cos())
        .collect()
}

/// The half-spectrum of the real N-D transform must match the
/// corresponding bins of the full complex N-D transform.
fn assert_matches_complex_nd(shape: &[usize], tol: f64) {
    let n: usize = shape.iter().product();
    let d = shape.len();
    let n_last = shape[d - 1];
    let hl = n_last / 2 + 1;

    let rplan = RfftPlanNd::<f64>::new(shape).unwrap();
    assert_eq!(rplan.shape(), shape);
    assert_eq!(rplan.len(), n);
    assert!(!rplan.is_empty());
    assert_eq!(rplan.spectrum_shape()[d - 1], hl);
    assert_eq!(rplan.spectrum_len(), n / n_last * hl);

    let input = rbuf(n);
    let mut spec = vec![Complex::new(0.0, 0.0); rplan.spectrum_len()];
    let mut scratch = vec![Complex::new(0.0, 0.0); rplan.scratch_len()];
    rplan.execute(&input, &mut spec, &mut scratch).unwrap();

    // Reference: complex N-D transform of the realified input.
    let cplan = FftPlanNd::<f64>::new(shape).unwrap();
    let mut full: Vec<Complex<f64>> = input.iter().map(|&x| Complex::new(x, 0.0)).collect();
    let mut cscratch = vec![Complex::new(0.0, 0.0); cplan.scratch_len()];
    cplan.execute(&mut full, &mut cscratch).unwrap();

    for (line_idx, (spec_line, full_line)) in spec
        .chunks_exact(hl)
        .zip(full.chunks_exact(n_last))
        .enumerate()
    {
        for (k, (a, b)) in spec_line.iter().zip(full_line.iter().take(hl)).enumerate() {
            assert!(
                (a.re - b.re).abs() < tol && (a.im - b.im).abs() < tol,
                "shape {shape:?}, line {line_idx}, bin {k}: {a:?} vs {b:?}"
            );
        }
    }
}

#[test]
fn test_2d_matches_complex_nd() {
    assert_matches_complex_nd(&[4, 8], 1e-10);
    assert_matches_complex_nd(&[8, 4], 1e-10);
    assert_matches_complex_nd(&[5, 6], 1e-10);
}

#[test]
fn test_3d_matches_complex_nd() {
    assert_matches_complex_nd(&[4, 4, 4], 1e-10);
    assert_matches_complex_nd(&[8, 4, 16], 1e-10);
}

#[test]
fn test_1d_shape() {
    assert_matches_complex_nd(&[16], 1e-10);
    assert_matches_complex_nd(&[9], 1e-10);
}

#[test]
// Ignored under Miri: the 32x32x32 = 32768-point real N-D round-trip is ~72 s
// under Miri (large buffer + allocation churn). The real N-D walk's UB is
// covered under Miri by the smaller-shape tests; full correctness runs in normal CI.
#[cfg_attr(miri, ignore)]
fn test_round_trip_3d() {
    for shape in [[16usize, 16, 16], [8, 4, 32], [32, 32, 32]] {
        let n: usize = shape.iter().product();
        let rplan = RfftPlanNd::<f64>::new(&shape).unwrap();
        let input = rbuf(n);
        let mut spec = vec![Complex::new(0.0, 0.0); rplan.spectrum_len()];
        let mut scratch = vec![Complex::new(0.0, 0.0); rplan.scratch_len()];
        rplan.execute(&input, &mut spec, &mut scratch).unwrap();
        let mut back = vec![0.0f64; n];
        rplan
            .execute_inverse(&mut spec, &mut back, &mut scratch)
            .unwrap();
        for (a, b) in input.iter().zip(back.iter()) {
            assert!((a - b).abs() < 1e-11, "shape {shape:?}: {a} vs {b}");
        }
    }
}

#[test]
fn test_odd_last_axis_round_trip() {
    let shape = [4usize, 9];
    let n = 36;
    let rplan = RfftPlanNd::<f64>::new(&shape).unwrap();
    let input = rbuf(n);
    let mut spec = vec![Complex::new(0.0, 0.0); rplan.spectrum_len()];
    let mut scratch = vec![Complex::new(0.0, 0.0); rplan.scratch_len()];
    rplan.execute(&input, &mut spec, &mut scratch).unwrap();
    let mut back = vec![0.0f64; n];
    rplan
        .execute_inverse(&mut spec, &mut back, &mut scratch)
        .unwrap();
    for (a, b) in input.iter().zip(back.iter()) {
        assert!((a - b).abs() < 1e-11);
    }
}

/// A non-last axis of length 1 is a no-op: it must be skipped both when the
/// plan is built and when the complex axes are transformed. Exercises the
/// `len == 1 { continue }` branches in `new` and `complex_axes`.
#[test]
fn test_unit_leading_axis() {
    // Forward half-spectrum still matches the full complex transform.
    assert_matches_complex_nd(&[1, 8], 1e-10);
    assert_matches_complex_nd(&[1, 1, 8], 1e-10);

    // And the transform round-trips through the inverse.
    let shape = [1usize, 16];
    let n = 16;
    let rplan = RfftPlanNd::<f64>::new(&shape).unwrap();
    let input = rbuf(n);
    let mut spec = vec![Complex::new(0.0, 0.0); rplan.spectrum_len()];
    let mut scratch = vec![Complex::new(0.0, 0.0); rplan.scratch_len()];
    rplan.execute(&input, &mut spec, &mut scratch).unwrap();
    let mut back = vec![0.0f64; n];
    rplan
        .execute_inverse(&mut spec, &mut back, &mut scratch)
        .unwrap();
    for (a, b) in input.iter().zip(back.iter()) {
        assert!((a - b).abs() < 1e-11, "shape {shape:?}: {a} vs {b}");
    }
}

#[test]
fn test_invalid_shapes_rejected() {
    assert_eq!(
        RfftPlanNd::<f64>::new(&[]).unwrap_err(),
        FftError::InvalidLength(0)
    );
    assert_eq!(
        RfftPlanNd::<f64>::new(&[4, 0]).unwrap_err(),
        FftError::InvalidLength(0)
    );
}

#[test]
fn test_buffer_validation() {
    let plan = RfftPlanNd::<f64>::new(&[4, 8]).unwrap();
    let input = rbuf(32);
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];

    let mut bad_spec = vec![Complex::new(0.0, 0.0); 3];
    assert_eq!(
        plan.execute(&input, &mut bad_spec, &mut scratch)
            .unwrap_err(),
        FftError::LengthMismatch {
            expected: 20,
            got: 3
        }
    );

    let mut spec = vec![Complex::new(0.0, 0.0); 20];
    assert_eq!(
        plan.execute(&rbuf(8), &mut spec, &mut scratch).unwrap_err(),
        FftError::LengthMismatch {
            expected: 32,
            got: 8
        }
    );

    let mut tiny: Vec<Complex<f64>> = Vec::new();
    let err = plan.execute(&input, &mut spec, &mut tiny).unwrap_err();
    assert!(matches!(err, FftError::ScratchTooSmall { .. }));

    let mut out = vec![0.0f64; 16];
    assert_eq!(
        plan.execute_inverse(&mut spec, &mut out, &mut scratch)
            .unwrap_err(),
        FftError::LengthMismatch {
            expected: 32,
            got: 16
        }
    );
}

#[test]
fn test_plan_is_cloneable_and_debuggable() {
    let plan = RfftPlanNd::<f64>::new(&[4, 8]).unwrap();
    let cloned = plan.clone();
    assert_eq!(cloned.spectrum_len(), 20);
    assert!(format!("{plan:?}").contains("RfftPlanNd"));
}
