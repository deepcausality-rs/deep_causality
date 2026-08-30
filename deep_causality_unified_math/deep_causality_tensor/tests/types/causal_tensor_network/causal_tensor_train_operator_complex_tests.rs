/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage-4 complex matrix-product operators: the MPO surface over `Complex<f64>` — operator TT-SVD
//! round-trip, `apply` (MPO·MPS) against a dense complex matrix–vector product, and `compose`.

use deep_causality_algebra::ConjugateScalar;
use deep_causality_num::Zero;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
};

type C = Complex<f64>;

fn cabs(z: C) -> f64 {
    z.modulus_squared().sqrt()
}

fn full() -> Truncation<f64> {
    Truncation::by_bond(4096).unwrap()
}

const TOL: f64 = 1e-9;

/// Row-major interleaved index `[o0,i0,o1,i1,…]` for matrix entry `M[out, in]`.
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

/// A complex operator from a flat `nn×nn` matrix in site-interleaved layout.
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

#[test]
fn test_complex_mpo_apply_matches_dense_matvec() {
    let dims = [2usize, 2];
    let nn = 4usize;
    let mmat: Vec<C> = (0..nn * nn)
        .map(|i| Complex::new((i as f64 * 0.5).sin() + 0.2, (i as f64 * 0.31).cos() - 0.1))
        .collect();
    let a = operator_from_matrix(&mmat, &dims);

    let xdata: Vec<C> = (0..nn)
        .map(|i| Complex::new((i as f64 * 0.7).cos(), (i as f64 * 0.9).sin() + 0.3))
        .collect();
    let x = CausalTensorTrain::<C>::from_dense(
        &CausalTensor::new(xdata.clone(), vec![2, 2]).unwrap(),
        &full(),
    )
    .unwrap();

    let y = a.apply(&x, &full()).unwrap();
    let yd = y.to_dense().unwrap();

    // Reference: y[out] = Σ_in M[out][in]·x[in].
    for (out, got) in yd.as_slice().iter().enumerate() {
        let mut acc = C::zero();
        for (inx, &xv) in xdata.iter().enumerate() {
            acc += mmat[out * nn + inx] * xv;
        }
        assert!(cabs(*got - acc) <= TOL, "complex MPO·MPS off");
    }
}

#[test]
fn test_complex_mpo_compose_equivalence() {
    let dims = [2usize, 2];
    let nn = 4usize;
    let mmat: Vec<C> = (0..nn * nn)
        .map(|i| Complex::new((i as f64 * 0.4).sin(), (i as f64 * 0.6).cos()))
        .collect();
    let nmat: Vec<C> = (0..nn * nn)
        .map(|i| Complex::new((i as f64 * 0.8).cos() + 0.1, (i as f64 * 0.2).sin() - 0.2))
        .collect();
    let m = operator_from_matrix(&mmat, &dims);
    let n = operator_from_matrix(&nmat, &dims);

    let xdata: Vec<C> = (0..nn)
        .map(|i| Complex::new((i as f64 * 1.1).cos(), (i as f64 * 0.5).sin()))
        .collect();
    let x =
        CausalTensorTrain::<C>::from_dense(&CausalTensor::new(xdata, vec![2, 2]).unwrap(), &full())
            .unwrap();

    // (M∘N)·x == M·(N·x).
    let lhs = m.compose(&n, &full()).unwrap().apply(&x, &full()).unwrap();
    let rhs = m.apply(&n.apply(&x, &full()).unwrap(), &full()).unwrap();
    let (ld, rd) = (lhs.to_dense().unwrap(), rhs.to_dense().unwrap());
    for (a, b) in ld.as_slice().iter().zip(rd.as_slice().iter()) {
        assert!(cabs(*a - *b) <= TOL, "complex compose equivalence off");
    }
}

#[test]
fn test_complex_mpo_to_dense_roundtrip() {
    let dims = [2usize, 2];
    let nn = 4usize;
    let mmat: Vec<C> = (0..nn * nn)
        .map(|i| Complex::new((i as f64 * 0.33).sin() - 0.4, (i as f64 * 0.77).cos() + 0.5))
        .collect();
    let mut inter = vec![C::zero(); nn * nn];
    for out in 0..nn {
        for inx in 0..nn {
            inter[interleaved_index(out, inx, &dims)] = mmat[out * nn + inx];
        }
    }
    let a = operator_from_matrix(&mmat, &dims);
    let back = a.to_dense().unwrap();
    for (got, want) in back.as_slice().iter().zip(inter.iter()) {
        assert!(cabs(*got - *want) <= TOL, "complex MPO round-trip off");
    }
}
