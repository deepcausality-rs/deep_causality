/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{DensityMatrix, EnvironmentalPrep};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

/// The Bell preparation as a mixed one-qubit marginal (maximally mixed I/2).
fn rho_a() -> DensityMatrix<f64> {
    let m = CausalTensor::new(
        vec![c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)],
        vec![2, 2],
    )
    .unwrap();
    DensityMatrix::new(m).unwrap()
}

#[test]
fn test_prep_is_read_only_and_reproducible() {
    let prep = EnvironmentalPrep::new(rho_a());
    // Only read accessors exist; the wrapped state is unchanged and reproducible.
    assert_eq!(prep.dim(), 2);
    assert!((prep.state().purity() - 0.5).abs() < 1e-12);
    let again = prep.clone();
    assert_eq!(prep, again);
    // Reading twice yields the identical matrix (no interior mutation).
    let first = prep.matrix().clone();
    let second = prep.matrix().clone();
    assert_eq!(first.as_slice(), second.as_slice());
}

#[test]
fn test_prep_preserves_the_sealed_state() {
    let dm = rho_a();
    let prep = EnvironmentalPrep::new(dm.clone());
    assert_eq!(prep.state(), &dm);
}
