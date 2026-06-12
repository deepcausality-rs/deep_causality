/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_fft::{FftError, RfftPlan, naive_dft};
use deep_causality_num::Complex;

fn rbuf(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| ((i as f64) * 0.37).sin() + 0.25 * ((i as f64) * 1.7).cos())
        .collect()
}

/// rFFT output must equal the first n/2+1 bins of the complex DFT of the
/// realified signal.
fn assert_matches_complex_dft(n: usize, tol: f64) {
    let plan = RfftPlan::<f64>::new(n).unwrap();
    assert_eq!(plan.len(), n);
    assert!(!plan.is_empty());
    assert_eq!(plan.spectrum_len(), n / 2 + 1);

    let input = rbuf(n);
    let complexified: Vec<Complex<f64>> = input.iter().map(|&x| Complex::new(x, 0.0)).collect();
    let reference = naive_dft(&complexified);

    let mut output = vec![Complex::new(0.0, 0.0); plan.spectrum_len()];
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
    plan.execute(&input, &mut output, &mut scratch).unwrap();

    for (k, (a, b)) in output.iter().zip(reference.iter()).enumerate() {
        assert!(
            (a.re - b.re).abs() < tol && (a.im - b.im).abs() < tol,
            "n={n}, bin {k}: {a:?} vs {b:?}"
        );
    }
}

#[test]
fn test_even_lengths_match_complex_dft() {
    for n in [2usize, 4, 8, 16, 32, 64, 128, 100] {
        assert_matches_complex_dft(n, 1e-10);
    }
}

#[test]
fn test_odd_lengths_match_complex_dft() {
    for n in [1usize, 3, 5, 9, 15, 31, 101] {
        assert_matches_complex_dft(n, 1e-10);
    }
}

#[test]
fn test_edge_bins_are_real() {
    // Bin 0 (DC) and bin n/2 (Nyquist, even n) of a real signal are real.
    let n = 64usize;
    let plan = RfftPlan::<f64>::new(n).unwrap();
    let input = rbuf(n);
    let mut output = vec![Complex::new(0.0, 0.0); plan.spectrum_len()];
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
    plan.execute(&input, &mut output, &mut scratch).unwrap();
    assert!(output[0].im.abs() < 1e-12);
    assert!(output[n / 2].im.abs() < 1e-12);
}

#[test]
fn test_round_trip_identity() {
    for n in [2usize, 4, 16, 64, 100, 1, 3, 31, 101] {
        let plan = RfftPlan::<f64>::new(n).unwrap();
        let input = rbuf(n);
        let mut spectrum = vec![Complex::new(0.0, 0.0); plan.spectrum_len()];
        let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];
        plan.execute(&input, &mut spectrum, &mut scratch).unwrap();
        let mut back = vec![0.0f64; n];
        plan.execute_inverse(&spectrum, &mut back, &mut scratch)
            .unwrap();
        for (a, b) in input.iter().zip(back.iter()) {
            assert!((a - b).abs() < 1e-10, "n={n}: {a} vs {b}");
        }
    }
}

#[test]
fn test_zero_length_rejected() {
    assert_eq!(
        RfftPlan::<f64>::new(0).unwrap_err(),
        FftError::InvalidLength(0)
    );
}

#[test]
fn test_buffer_validation() {
    let plan = RfftPlan::<f64>::new(8).unwrap();
    let input = rbuf(8);
    let mut scratch = vec![Complex::new(0.0, 0.0); plan.scratch_len()];

    // Wrong spectrum length.
    let mut bad_spec = vec![Complex::new(0.0, 0.0); 3];
    assert_eq!(
        plan.execute(&input, &mut bad_spec, &mut scratch)
            .unwrap_err(),
        FftError::LengthMismatch {
            expected: 5,
            got: 3
        }
    );

    // Wrong input length.
    let mut spec = vec![Complex::new(0.0, 0.0); 5];
    assert_eq!(
        plan.execute(&rbuf(4), &mut spec, &mut scratch).unwrap_err(),
        FftError::LengthMismatch {
            expected: 8,
            got: 4
        }
    );

    // Scratch too small.
    let mut tiny: Vec<Complex<f64>> = Vec::new();
    let err = plan.execute(&input, &mut spec, &mut tiny).unwrap_err();
    assert!(matches!(err, FftError::ScratchTooSmall { .. }));

    // Inverse-side validation.
    let mut out = vec![0.0f64; 4];
    assert_eq!(
        plan.execute_inverse(&spec, &mut out, &mut scratch)
            .unwrap_err(),
        FftError::LengthMismatch {
            expected: 8,
            got: 4
        }
    );
}

#[test]
fn test_plan_is_cloneable_and_debuggable() {
    let plan = RfftPlan::<f64>::new(16).unwrap();
    let cloned = plan.clone();
    assert_eq!(cloned.spectrum_len(), 9);
    assert!(format!("{plan:?}").contains("RfftPlan"));
}
