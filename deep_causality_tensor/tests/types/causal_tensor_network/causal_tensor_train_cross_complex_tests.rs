/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage-4 complex TT-cross: building a `Complex<f64>` tensor train from a complex oracle without
//! materializing the dense tensor, with modulus-based pivoting and a real residual estimate.

use deep_causality_num::{Complex, ConjugateScalar, Zero};
use deep_causality_tensor::{CausalTensorTrain, CrossConfig, TensorTrain};

type C = Complex<f64>;

fn cabs(z: C) -> f64 {
    z.modulus_squared().sqrt()
}

#[test]
fn test_complex_cross_recovers_rank_one_oracle() {
    // A separable (rank-1) complex oracle f(i,j,k) = a[i]·b[j]·c[k]; TT-cross must recover it
    // exactly with bond dimension 1.
    let shape = [3usize, 3, 3];
    let a = [C::new(1.0, 0.5), C::new(-0.3, 0.7), C::new(0.8, -0.2)];
    let b = [C::new(0.4, -0.6), C::new(1.1, 0.1), C::new(-0.5, 0.9)];
    let c = [C::new(0.7, 0.3), C::new(-0.2, -0.8), C::new(0.6, 0.4)];
    let oracle = |idx: &[usize]| a[idx[0]] * b[idx[1]] * c[idx[2]];

    let cfg = CrossConfig::<f64>::with_rank_cap(4, 1e-12).unwrap();
    let (train, residual) = CausalTensorTrain::<C>::cross(&shape, oracle, &cfg).unwrap();

    // Residual is a real, small number.
    assert!(residual >= 0.0, "residual must be real and non-negative");
    assert!(residual <= 1e-9, "cross residual too large: {residual}");

    // The recovered train matches the oracle at every index.
    for (i0, &av) in a.iter().enumerate() {
        for (i1, &bv) in b.iter().enumerate() {
            for (i2, &cv) in c.iter().enumerate() {
                let idx = [i0, i1, i2];
                let got = train.eval(&idx).unwrap();
                assert!(
                    cabs(got - av * bv * cv) <= 1e-9,
                    "complex cross mismatch at {idx:?}"
                );
            }
        }
    }

    // A rank-1 oracle yields bond dimensions 1.
    assert!(train.bond_dims().iter().all(|&r| r == 1), "expected bond 1");
}

#[test]
fn test_complex_cross_nonfinite_oracle_errors() {
    let shape = [2usize, 2];
    let oracle = |idx: &[usize]| {
        if idx == [0, 0] {
            C::new(f64::NAN, 0.0)
        } else {
            C::zero()
        }
    };
    let cfg = CrossConfig::<f64>::with_rank_cap(2, 1e-10).unwrap();
    assert!(matches!(
        CausalTensorTrain::<C>::cross(&shape, oracle, &cfg),
        Err(deep_causality_tensor::CausalTensorError::CrossSampleFailure)
    ));
}
