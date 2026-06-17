/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_fft::{naive_dft, naive_idft};
use deep_causality_num::Complex;

fn buf(n: usize) -> Vec<Complex<f64>> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            Complex::new((x * 0.7).sin(), (x * 0.3).cos())
        })
        .collect()
}

#[test]
fn test_dft_impulse_is_constant() {
    // FFT of the unit impulse is all-ones.
    let mut input: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); 8];
    input[0] = Complex::new(1.0, 0.0);
    let out = naive_dft(&input);
    for z in out {
        assert!((z.re - 1.0).abs() < 1e-12);
        assert!(z.im.abs() < 1e-12);
    }
}

#[test]
fn test_dft_constant_is_impulse() {
    // FFT of all-ones is n at bin zero, zero elsewhere (unnormalized).
    let input: Vec<Complex<f64>> = vec![Complex::new(1.0, 0.0); 8];
    let out = naive_dft(&input);
    assert!((out[0].re - 8.0).abs() < 1e-12);
    assert!(out[0].im.abs() < 1e-12);
    for z in &out[1..] {
        assert!(z.re.abs() < 1e-12 && z.im.abs() < 1e-12);
    }
}

#[test]
fn test_dft_single_tone() {
    // x_j = e^{2πi·j·k0/n} concentrates on bin k0 with weight n.
    let n = 16usize;
    let k0 = 3usize;
    let input: Vec<Complex<f64>> = (0..n)
        .map(|j| {
            let theta = 2.0 * std::f64::consts::PI * (j * k0) as f64 / n as f64;
            Complex::new(theta.cos(), theta.sin())
        })
        .collect();
    let out = naive_dft(&input);
    for (k, z) in out.iter().enumerate() {
        let expected = if k == k0 { n as f64 } else { 0.0 };
        assert!(
            (z.re - expected).abs() < 1e-10 && z.im.abs() < 1e-10,
            "bin {k}: {z:?}"
        );
    }
}

#[test]
fn test_idft_round_trip() {
    let input = buf(12);
    let spec = naive_dft(&input);
    let back = naive_idft(&spec);
    for (a, b) in input.iter().zip(back.iter()) {
        assert!((a.re - b.re).abs() < 1e-12);
        assert!((a.im - b.im).abs() < 1e-12);
    }
}

#[test]
fn test_dft_empty() {
    let input: Vec<Complex<f64>> = Vec::new();
    assert!(naive_dft(&input).is_empty());
    assert!(naive_idft(&input).is_empty());
}
