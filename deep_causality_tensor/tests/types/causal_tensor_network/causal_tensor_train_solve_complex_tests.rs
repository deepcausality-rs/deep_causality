/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage-4 complex solvers: the AMEn linear solve, ALS fit, and two-site TDVP over `Complex<f64>`
//! (modulus-based pivoting, real residuals). The eigensolver stays real/dual-only (it needs a
//! complex Hermitian eigensolver), so it is not exercised here.

use deep_causality_num::{Complex, ConjugateScalar, Zero};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, SolveConfig, TensorTrain,
    TensorTrainOperator, Truncation, solve,
};

type C = Complex<f64>;

fn cabs(z: C) -> f64 {
    z.modulus_squared().sqrt()
}

fn full() -> Truncation<f64> {
    Truncation::by_bond(4096).unwrap()
}

fn interleaved_index(out: usize, inx: usize, dims: &[usize]) -> usize {
    let d = dims.len();
    let (mut o, mut i) = (out, inx);
    let mut o_dig = vec![0usize; d];
    let mut i_dig = vec![0usize; d];
    for k in (0..d).rev() {
        o_dig[k] = o % dims[k];
        o /= dims[k];
        i_dig[k] = i % dims[k];
        i /= dims[k];
    }
    let mut idx = 0usize;
    for k in 0..d {
        idx = idx * dims[k] + o_dig[k];
        idx = idx * dims[k] + i_dig[k];
    }
    idx
}

fn operator_from_matrix(mmat: &[C], dims: &[usize]) -> CausalTensorTrainOperator<C> {
    let nn: usize = dims.iter().product();
    let mut inter = vec![C::zero(); nn * nn];
    for out in 0..nn {
        for inx in 0..nn {
            inter[interleaved_index(out, inx, dims)] = mmat[out * nn + inx];
        }
    }
    let shape: Vec<usize> = dims.iter().flat_map(|&d| [d, d]).collect();
    let dense = CausalTensor::new(inter, shape).unwrap();
    CausalTensorTrainOperator::<C>::from_dense(&dense, dims, dims, &full()).unwrap()
}

fn complex_state(shape: &[usize], seed: f64) -> CausalTensorTrain<C> {
    let total: usize = shape.iter().product();
    let data: Vec<C> = (0..total)
        .map(|i| {
            Complex::new(
                (i as f64 * seed).sin() + 0.5,
                (i as f64 * seed * 0.7).cos() - 0.3,
            )
        })
        .collect();
    CausalTensorTrain::<C>::from_dense(&CausalTensor::new(data, shape.to_vec()).unwrap(), &full())
        .unwrap()
}

#[test]
fn test_complex_linear_solve_recovers_x() {
    // Diagonally-dominant complex operator A; b = A·x*; AMEn recovers x with A·x ≈ b.
    let dims = [2usize, 2];
    let nn = 4usize;
    let amat: Vec<C> = (0..nn * nn)
        .map(|k| {
            let (i, j) = (k / nn, k % nn);
            if i == j {
                Complex::new(4.0, 0.5)
            } else {
                Complex::new((k as f64 * 0.3).sin() * 0.2, (k as f64 * 0.4).cos() * 0.2)
            }
        })
        .collect();
    let a = operator_from_matrix(&amat, &dims);
    let xstar = complex_state(&dims, 0.6);
    let b = a.apply(&xstar, &full()).unwrap();

    let cfg = SolveConfig::<f64>::new(100, 1e-11, 1e-14).unwrap();
    let x = solve::linear(&a, &b, 4, &cfg).unwrap();

    let ax = a.apply(&x, &full()).unwrap().to_dense().unwrap();
    let bd = b.to_dense().unwrap();
    for (got, want) in ax.as_slice().iter().zip(bd.as_slice().iter()) {
        assert!(cabs(*got - *want) <= 1e-7, "complex A·x ≠ b");
    }
}

#[test]
fn test_complex_fit_recovers_train() {
    // Fully sample a known complex train, fit, and compare densely.
    let shape = [3usize, 3, 3];
    let target = complex_state(&shape, 0.4);
    let dense = target.to_dense().unwrap();

    let mut samples = Vec::new();
    let mut idx = vec![0usize; 3];
    for i0 in 0..3 {
        for i1 in 0..3 {
            for i2 in 0..3 {
                idx[0] = i0;
                idx[1] = i1;
                idx[2] = i2;
                samples.push((idx.clone(), *dense.get(&idx).unwrap()));
            }
        }
    }
    let cfg = SolveConfig::<f64>::new(200, 1e-10, 1e-14).unwrap();
    let fitted = solve::fit(&shape, 4, &samples, &cfg).unwrap();
    let fd = fitted.to_dense().unwrap();
    for (got, want) in fd.as_slice().iter().zip(dense.as_slice().iter()) {
        assert!(cabs(*got - *want) <= 1e-6, "complex fit mismatch");
    }
}

#[test]
fn test_complex_tdvp_conserves_norm_under_skew_hermitian() {
    // A skew-Hermitian generator A (Aᴴ = −A) ⇒ exp(A·dt) is unitary ⇒ the TDVP step conserves norm
    // (the quantum Schrödinger setting A = −iH). Build A = K − Kᴴ.
    let dims = [2usize, 2, 2];
    let nn = 8usize;
    let kmat: Vec<C> = (0..nn * nn)
        .map(|k| Complex::new((k as f64 * 0.21).sin(), (k as f64 * 0.13).cos()))
        .collect();
    let mut amat = vec![C::zero(); nn * nn];
    for i in 0..nn {
        for j in 0..nn {
            // A[i][j] = K[i][j] − conj(K[j][i]).
            amat[i * nn + j] = kmat[i * nn + j] - kmat[j * nn + i].conjugate();
        }
    }
    let a = operator_from_matrix(&amat, &dims);

    let mut x = complex_state(&dims, 0.5);
    let n0 = x.norm().unwrap().re();
    solve::tdvp_step(&a, &mut x, Complex::new(0.05, 0.0), &full()).unwrap();
    let n1 = x.norm().unwrap().re();
    assert!(
        (n1 - n0).abs() <= 1e-6 * (n0 + 1.0),
        "complex TDVP did not conserve norm: {n0} -> {n1}"
    );
}
