/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage-4 complex tensor trains: the core MPS surface over `Complex<f64>` — TT-SVD round-trip,
//! the Hermitian inner product `⟨x|y⟩ = Σ x̄ᵢ yᵢ` (with a real, non-negative `⟨x|x⟩`), the real
//! Frobenius norm, exact `add`/`scale`, and lossless `round`/canonicalization.

use deep_causality_num::{Complex, ConjugateScalar, Zero};
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, TensorTrain, Truncation};

type C = Complex<f64>;

fn cabs(z: C) -> f64 {
    z.modulus_squared().sqrt()
}

fn full() -> Truncation<f64> {
    Truncation::by_bond(4096).unwrap()
}

/// A deterministic complex dense tensor of the given shape.
fn dense(shape: &[usize]) -> CausalTensor<C> {
    let total: usize = shape.iter().product();
    let data: Vec<C> = (0..total)
        .map(|i| Complex::new((i as f64 * 0.6).sin() + 0.5, (i as f64 * 0.35).cos() - 0.4))
        .collect();
    CausalTensor::new(data, shape.to_vec()).unwrap()
}

const TOL: f64 = 1e-9;

#[test]
fn test_complex_tt_from_dense_to_dense_roundtrip() {
    let shape = [2usize, 3, 2];
    let a = dense(&shape);
    let tt = CausalTensorTrain::<C>::from_dense(&a, &full()).unwrap();
    let back = tt.to_dense().unwrap();
    assert_eq!(back.shape(), a.shape());
    for (x, y) in back.as_slice().iter().zip(a.as_slice().iter()) {
        assert!(cabs(*x - *y) <= TOL, "complex round-trip off");
    }
}

#[test]
fn test_complex_tt_hermitian_inner_and_norm() {
    let shape = [2usize, 2, 2];
    let dx = dense(&shape);
    let dy = {
        let total: usize = shape.iter().product();
        let data: Vec<C> = (0..total)
            .map(|i| Complex::new((i as f64 * 0.9).cos(), (i as f64 * 0.5).sin() + 0.2))
            .collect();
        CausalTensor::new(data, shape.to_vec()).unwrap()
    };
    let x = CausalTensorTrain::<C>::from_dense(&dx, &full()).unwrap();
    let y = CausalTensorTrain::<C>::from_dense(&dy, &full()).unwrap();

    // Hermitian inner ⟨x|y⟩ = Σ x̄ᵢ yᵢ vs the dense reference.
    let mut want = C::zero();
    for (a, b) in dx.as_slice().iter().zip(dy.as_slice().iter()) {
        want += a.conjugate() * *b;
    }
    let got = x.inner(&y).unwrap();
    assert!(cabs(got - want) <= TOL, "Hermitian inner mismatch");

    // ⟨x|x⟩ is real and non-negative.
    let xx = x.inner(&x).unwrap();
    assert!(xx.im().abs() <= TOL, "⟨x|x⟩ not real");
    assert!(xx.re() >= -TOL, "⟨x|x⟩ negative");

    // Norm = sqrt(Σ |xᵢ|²), returned real-valued.
    let want_norm = dx
        .as_slice()
        .iter()
        .map(|z| z.modulus_squared())
        .sum::<f64>()
        .sqrt();
    let n = x.norm().unwrap();
    assert!(n.im().abs() <= TOL, "norm not real");
    assert!((n.re() - want_norm).abs() <= TOL, "norm mismatch");
}

#[test]
fn test_complex_tt_add_and_scale() {
    let shape = [2usize, 2, 2];
    let dx = dense(&shape);
    let x = CausalTensorTrain::<C>::from_dense(&dx, &full()).unwrap();

    // scale by a genuinely complex factor.
    let s = Complex::new(0.3, -1.2);
    let xs = x.scale(s).to_dense().unwrap();
    for (got, a) in xs.as_slice().iter().zip(dx.as_slice().iter()) {
        assert!(cabs(*got - *a * s) <= TOL, "complex scale off");
    }

    // x + x == 2x.
    let sum = x.add(&x).unwrap().to_dense().unwrap();
    for (got, a) in sum.as_slice().iter().zip(dx.as_slice().iter()) {
        assert!(cabs(*got - (*a + *a)) <= TOL, "complex add off");
    }
}

#[test]
fn test_complex_tt_round_and_canonicalize_lossless() {
    let shape = [2usize, 3, 2];
    let dx = dense(&shape);
    let x = CausalTensorTrain::<C>::from_dense(&dx, &full()).unwrap();

    // round with a generous policy is lossless.
    let r = x.round(&full()).unwrap().to_dense().unwrap();
    for (got, a) in r.as_slice().iter().zip(dx.as_slice().iter()) {
        assert!(cabs(*got - *a) <= TOL, "complex round changed the tensor");
    }

    // Left-canonicalization preserves the represented tensor and yields isometric cores.
    let lc = x.left_canonicalize().unwrap();
    let lcd = lc.to_dense().unwrap();
    for (got, a) in lcd.as_slice().iter().zip(dx.as_slice().iter()) {
        assert!(
            cabs(*got - *a) <= TOL,
            "left-canonicalize changed the tensor"
        );
    }
    // The first core, reshaped [r·n, r'], has orthonormal columns under the Hermitian inner product.
    let core0 = &lc.cores()[0];
    let (rl, n, rr) = (core0.shape()[0], core0.shape()[1], core0.shape()[2]);
    let cd = core0.as_slice();
    let rows = rl * n;
    for a in 0..rr {
        for b in 0..rr {
            let mut acc = C::zero();
            for i in 0..rows {
                acc += cd[i * rr + a].conjugate() * cd[i * rr + b];
            }
            let expect = if a == b { 1.0 } else { 0.0 };
            assert!(
                cabs(acc - Complex::new(expect, 0.0)) <= TOL,
                "core not isometric"
            );
        }
    }
}
