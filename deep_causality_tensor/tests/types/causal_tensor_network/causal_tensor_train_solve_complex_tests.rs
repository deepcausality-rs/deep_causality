/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage-4 complex solvers over `Complex<f64>` (modulus-based pivoting, real residuals): the AMEn
//! linear solve, ALS fit, two-site TDVP (norm-conserving under a skew-Hermitian generator), and
//! DMRG3S ground state via a complex **Hermitian** eigensolver.

use deep_causality_algebra::ConjugateScalar;
use deep_causality_num::Zero;
use deep_causality_num_complex::Complex;
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

#[test]
fn test_complex_eigen_recovers_hermitian_ground_state() {
    // A Hermitian operator A = s·I + (g − s)·v·vᴴ with g < s, so the complex unit vector v is the
    // ground state with eigenvalue g. DMRG3S with a complex Hermitian eigensolver recovers it.
    let dims = [3usize, 3];
    let nn = 9usize;
    let mut v: Vec<C> = (0..nn)
        .map(|i| Complex::new((i as f64 * 0.7).sin() + 0.4, (i as f64 * 0.5).cos() - 0.2))
        .collect();
    let nrm = v.iter().map(|z| z.modulus_squared()).sum::<f64>().sqrt();
    for z in v.iter_mut() {
        *z *= Complex::new(1.0 / nrm, 0.0);
    }
    let (g, s) = (-3.0f64, 1.0f64);

    let mut amat = vec![C::zero(); nn * nn];
    for out in 0..nn {
        for inx in 0..nn {
            // (g − s)·v[out]·conj(v[in]) + s·δ — Hermitian by construction.
            let mut val = v[out] * v[inx].conjugate() * Complex::new(g - s, 0.0);
            if out == inx {
                val += Complex::new(s, 0.0);
            }
            amat[out * nn + inx] = val;
        }
    }
    let a = operator_from_matrix(&amat, &dims);

    let cfg = SolveConfig::<f64>::new(200, 1e-9, 1e-13).unwrap();
    let (lambda, vtt) = solve::eigen(&a, 3, &cfg).unwrap();

    // Eigenvalue is real and equals the planted ground-state value.
    assert!(lambda.im().abs() <= 1e-6, "eigenvalue not real");
    assert!(
        (lambda.re() - g).abs() <= 1e-5,
        "eigenvalue off: {}",
        lambda.re()
    );

    // Eigenvector aligns with v (up to a global phase): |⟨v|vtt⟩| / ‖vtt‖ ≈ 1.
    let vd = vtt.to_dense().unwrap();
    let mut dot = C::zero();
    let mut vn = 0.0;
    for (k, &vk) in v.iter().enumerate() {
        dot += vk.conjugate() * vd.as_slice()[k];
        vn += vd.as_slice()[k].modulus_squared();
    }
    let cosine = cabs(dot) / vn.sqrt();
    assert!(
        (cosine - 1.0).abs() <= 1e-5,
        "eigenvector misaligned: {cosine}"
    );

    // Residual A·v ≈ λ·v.
    let av = a.apply(&vtt, &full()).unwrap();
    let resid = av.add(&vtt.scale(-lambda)).unwrap();
    let rrel = resid.norm().unwrap().re() / vtt.norm().unwrap().re();
    assert!(rrel <= 1e-5, "complex eigen residual too large: {rrel}");
}
