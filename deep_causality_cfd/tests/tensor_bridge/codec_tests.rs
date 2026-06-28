/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{dequantize, quantize};
use deep_causality_tensor::{CausalTensor, Truncation};

const TOL: f64 = 1e-12;

fn field(data: Vec<f64>) -> CausalTensor<f64> {
    let n = data.len();
    CausalTensor::new(data, vec![n]).unwrap()
}

#[test]
fn round_trips_to_tolerance() {
    let u = field(vec![1.0, -2.0, 3.5, 0.0, 4.0, -1.5, 2.0, 7.0]); // N = 8, L = 3
    let exact = Truncation::<f64>::by_bond(4096).unwrap();
    let q = quantize(&u, &exact).unwrap();
    let back = dequantize(&q).unwrap();
    assert_eq!(back.shape(), &[8]);
    for (a, b) in back.as_slice().iter().zip(u.as_slice()) {
        assert!((a - b).abs() <= TOL, "round-trip differs: {a} vs {b}");
    }
}

#[test]
fn smooth_field_compresses() {
    // A single sine over N = 64 is QTT-rank 2 — the bond is far below the dense size.
    let n = 64usize;
    let data: Vec<f64> = (0..n)
        .map(|i| (core::f64::consts::TAU * i as f64 / n as f64).sin())
        .collect();
    let u = field(data);
    let q = quantize(&u, &Truncation::<f64>::by_tol(1e-10).unwrap()).unwrap();
    let max_bond = q.cores().iter().map(|c| c.shape()[2]).max().unwrap();
    assert!(
        max_bond < n,
        "expected compression, max bond {max_bond} vs N {n}"
    );
    let back = dequantize(&q).unwrap();
    for (a, b) in back.as_slice().iter().zip(u.as_slice()) {
        assert!(
            (a - b).abs() <= 1e-9,
            "compressed round-trip differs: {a} vs {b}"
        );
    }
}

#[test]
fn non_power_of_two_is_rejected() {
    let u = field(vec![1.0; 6]);
    assert!(quantize(&u, &Truncation::<f64>::by_bond(16).unwrap()).is_err());
}
