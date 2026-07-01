/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::EndoArrow;
use deep_causality_num::{ConjugateScalar, Float106, FromPrimitive, RealField};
use deep_causality_tensor::{
    CausalTensor, CausalTensorError, CausalTensorTrain, CausalTensorTrainOperator, Tensor,
    TensorTrain, TensorTrainOperator, Truncation,
};

fn v<T: FromPrimitive>(x: f64) -> T {
    T::from_f64(x).unwrap()
}

fn tensor<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    data: &[f64],
    shape: &[usize],
) -> CausalTensor<T> {
    CausalTensor::new(data.iter().map(|&x| v::<T>(x)).collect(), shape.to_vec()).unwrap()
}

fn tol<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> T {
    T::epsilon().sqrt() * v::<T>(64.0)
}

fn assert_dense_eq<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    a: &CausalTensor<T>,
    b: &CausalTensor<T>,
) {
    assert_eq!(a.shape(), b.shape());
    for (x, y) in a.as_slice().iter().zip(b.as_slice().iter()) {
        assert!((*x - *y).abs() <= tol::<T>(), "differ beyond tolerance");
    }
}

fn full<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() -> Truncation<T> {
    Truncation::by_bond(4096).unwrap()
}

// A 2×2×2×2 (site-interleaved [out0,in0,out1,in1]) operator with distinct entries.
fn op_dense<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    seed: f64,
) -> CausalTensor<T> {
    let data: Vec<f64> = (0..16).map(|i| (i as f64 + seed).sin()).collect();
    tensor::<T>(&data, &[2, 2, 2, 2])
}

fn state_dense<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    seed: f64,
) -> CausalTensor<T> {
    let data: Vec<f64> = (0..4).map(|i| (i as f64) * 0.5 + seed).collect();
    tensor::<T>(&data, &[2, 2])
}

/// Brute-force dense action of a 2-site operator on a 2-site state.
fn dense_apply<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>(
    op: &CausalTensor<T>,
    state: &CausalTensor<T>,
) -> CausalTensor<T> {
    // Reorder [o0,i0,o1,i1] → [o0,o1,i0,i1], view as a 4×4 matrix, multiply the length-4 vector.
    let m = op
        .permute_axes(&[0, 2, 1, 3])
        .unwrap()
        .reshape(&[4, 4])
        .unwrap();
    let md = m.as_slice();
    let xd = state.as_slice();
    let mut out = vec![T::zero(); 4];
    for (o, slot) in out.iter_mut().enumerate() {
        let mut acc = T::zero();
        for (i, &xi) in xd.iter().enumerate() {
            acc += md[o * 4 + i] * xi;
        }
        *slot = acc;
    }
    CausalTensor::new(out, vec![2, 2]).unwrap()
}

fn check_identity<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let x = CausalTensorTrain::from_dense(&state_dense::<T>(0.0), &full::<T>()).unwrap();
    let id = CausalTensorTrainOperator::<T>::identity(&[2, 2]);
    let applied = id.apply(&x, &full::<T>()).unwrap();
    assert_dense_eq(&applied.to_dense().unwrap(), &x.to_dense().unwrap());
}

fn check_from_dense_roundtrip<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let dense = op_dense::<T>(1.0);
    let op = CausalTensorTrainOperator::from_dense(&dense, &[2, 2], &[2, 2], &full::<T>()).unwrap();
    assert_eq!(op.out_dims(), &[2, 2]);
    assert_eq!(op.in_dims(), &[2, 2]);
    assert_dense_eq(&op.to_dense().unwrap(), &dense);
}

fn check_apply_matches_dense<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let op_d = op_dense::<T>(0.7);
    let st_d = state_dense::<T>(-0.3);
    let op = CausalTensorTrainOperator::from_dense(&op_d, &[2, 2], &[2, 2], &full::<T>()).unwrap();
    let st = CausalTensorTrain::from_dense(&st_d, &full::<T>()).unwrap();

    let got = op.apply(&st, &full::<T>()).unwrap().to_dense().unwrap();
    let want = dense_apply(&op_d, &st_d);
    assert_dense_eq(&got, &want);
}

fn check_compose<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let m =
        CausalTensorTrainOperator::from_dense(&op_dense::<T>(0.2), &[2, 2], &[2, 2], &full::<T>())
            .unwrap();
    let n =
        CausalTensorTrainOperator::from_dense(&op_dense::<T>(1.3), &[2, 2], &[2, 2], &full::<T>())
            .unwrap();
    let x = CausalTensorTrain::from_dense(&state_dense::<T>(0.1), &full::<T>()).unwrap();

    // (M∘N)·x == M·(N·x).
    let composed = m.compose(&n, &full::<T>()).unwrap();
    let via_compose = composed
        .apply(&x, &full::<T>())
        .unwrap()
        .to_dense()
        .unwrap();
    let via_chain = m
        .apply(&n.apply(&x, &full::<T>()).unwrap(), &full::<T>())
        .unwrap()
        .to_dense()
        .unwrap();
    assert_dense_eq(&via_compose, &via_chain);
}

fn check_transpose<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let dense = op_dense::<T>(0.9);
    let op = CausalTensorTrainOperator::from_dense(&dense, &[2, 2], &[2, 2], &full::<T>()).unwrap();
    let t = op.transpose();
    assert_eq!(t.out_dims(), &[2, 2]);
    assert_eq!(t.in_dims(), &[2, 2]);
    // Transposing swaps the out/in legs: dense [o0,i0,o1,i1] → [i0,o0,i1,o1].
    let dd = dense.as_slice();
    let mut wd = vec![T::zero(); 16];
    for o0 in 0..2 {
        for i0 in 0..2 {
            for o1 in 0..2 {
                for i1 in 0..2 {
                    let old = ((o0 * 2 + i0) * 2 + o1) * 2 + i1;
                    let new = ((i0 * 2 + o0) * 2 + i1) * 2 + o1;
                    wd[new] = dd[old];
                }
            }
        }
    }
    let want = CausalTensor::new(wd, vec![2, 2, 2, 2]).unwrap();
    assert_dense_eq(&t.to_dense().unwrap(), &want);
}

fn check_round<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let op =
        CausalTensorTrainOperator::from_dense(&op_dense::<T>(0.4), &[2, 2], &[2, 2], &full::<T>())
            .unwrap();
    // Compose with identity inflates the bond; rounding restores it while preserving the operator.
    let id = CausalTensorTrainOperator::<T>::identity(&[2, 2]);
    let inflated = op
        .compose(&id, &Truncation::by_bond(4096).unwrap())
        .unwrap();
    let rounded = inflated.round(&full::<T>()).unwrap();
    assert_dense_eq(&rounded.to_dense().unwrap(), &op.to_dense().unwrap());
}

/// `add` / `scale` / `neg` / `sub` complete the operator algebra: they densify to the elementwise
/// real-space operations, are linear under `apply`, and reject mismatched dimensions.
fn check_operator_algebra<T: RealField + FromPrimitive + ConjugateScalar<Real = T>>() {
    let a_d = op_dense::<T>(0.2);
    let b_d = op_dense::<T>(1.1);
    let mk = |d: &CausalTensor<T>| {
        CausalTensorTrainOperator::from_dense(d, &[2, 2], &[2, 2], &full::<T>()).unwrap()
    };
    let (a, b) = (mk(&a_d), mk(&b_d));
    let zip = |x: &CausalTensor<T>, y: &CausalTensor<T>, f: fn(T, T) -> T| {
        CausalTensor::new(
            x.as_slice()
                .iter()
                .zip(y.as_slice())
                .map(|(p, q)| f(*p, *q))
                .collect(),
            x.shape().to_vec(),
        )
        .unwrap()
    };

    // add / sub densify to the elementwise sum / difference.
    assert_dense_eq(
        &a.add(&b).unwrap().to_dense().unwrap(),
        &zip(&a_d, &b_d, |x, y| x + y),
    );
    assert_dense_eq(
        &a.sub(&b).unwrap().to_dense().unwrap(),
        &zip(&a_d, &b_d, |x, y| x - y),
    );

    // scale / neg.
    let s = v::<T>(2.5);
    let scaled = CausalTensor::new(
        a_d.as_slice().iter().map(|x| *x * s).collect(),
        a_d.shape().to_vec(),
    )
    .unwrap();
    assert_dense_eq(&a.scale(s).to_dense().unwrap(), &scaled);
    let negd = CausalTensor::new(
        a_d.as_slice().iter().map(|x| T::zero() - *x).collect(),
        a_d.shape().to_vec(),
    )
    .unwrap();
    assert_dense_eq(&a.neg().to_dense().unwrap(), &negd);

    // Linearity of apply: (a + b)·x == a·x + b·x.
    let x = CausalTensorTrain::from_dense(&state_dense::<T>(0.3), &full::<T>()).unwrap();
    let ax = a.apply(&x, &full::<T>()).unwrap().to_dense().unwrap();
    let bx = b.apply(&x, &full::<T>()).unwrap().to_dense().unwrap();
    let lhs = a
        .add(&b)
        .unwrap()
        .apply(&x, &full::<T>())
        .unwrap()
        .to_dense()
        .unwrap();
    assert_dense_eq(&lhs, &zip(&ax, &bx, |p, q| p + q));

    // Mismatched dimensions are rejected.
    let single = CausalTensorTrainOperator::from_dense(
        &tensor::<T>(&[1.0, 2.0, 3.0, 4.0], &[2, 2]),
        &[2],
        &[2],
        &full::<T>(),
    )
    .unwrap();
    assert!(matches!(
        a.add(&single),
        Err(CausalTensorError::ShapeMismatch)
    ));
    assert!(matches!(
        a.sub(&single),
        Err(CausalTensorError::ShapeMismatch)
    ));
}

#[test]
fn test_operator_algebra_f64() {
    check_operator_algebra::<f64>();
}
#[test]
fn test_operator_algebra_float106() {
    check_operator_algebra::<Float106>();
}

#[test]
fn test_identity_f32() {
    check_identity::<f32>();
}
#[test]
fn test_identity_f64() {
    check_identity::<f64>();
}
#[test]
fn test_identity_float106() {
    check_identity::<Float106>();
}

#[test]
fn test_from_dense_roundtrip_f32() {
    check_from_dense_roundtrip::<f32>();
}
#[test]
fn test_from_dense_roundtrip_f64() {
    check_from_dense_roundtrip::<f64>();
}
#[test]
fn test_from_dense_roundtrip_float106() {
    check_from_dense_roundtrip::<Float106>();
}

#[test]
fn test_apply_matches_dense_f32() {
    check_apply_matches_dense::<f32>();
}
#[test]
fn test_apply_matches_dense_f64() {
    check_apply_matches_dense::<f64>();
}
#[test]
fn test_apply_matches_dense_float106() {
    check_apply_matches_dense::<Float106>();
}

#[test]
fn test_compose_f32() {
    check_compose::<f32>();
}
#[test]
fn test_compose_f64() {
    check_compose::<f64>();
}
#[test]
fn test_compose_float106() {
    check_compose::<Float106>();
}

#[test]
fn test_transpose_f64() {
    check_transpose::<f64>();
}

#[test]
fn test_round_f64() {
    check_round::<f64>();
}

#[test]
fn test_endoarrow_iterate_identity() {
    // The identity operator as an endo-arrow: iterate_n leaves the state unchanged.
    let x = CausalTensorTrain::from_dense(&state_dense::<f64>(0.0), &full::<f64>()).unwrap();
    let id = CausalTensorTrainOperator::<f64>::identity(&[2, 2]).with_rounding(full::<f64>());
    let out = id.iterate_n(x.clone(), 5);
    assert_dense_eq(&out.to_dense().unwrap(), &x.to_dense().unwrap());
}

#[test]
fn test_errors() {
    let op = CausalTensorTrainOperator::from_dense(
        &op_dense::<f64>(0.0),
        &[2, 2],
        &[2, 2],
        &full::<f64>(),
    )
    .unwrap();
    // apply with a state whose dims differ from in_dims.
    let bad_state =
        CausalTensorTrain::from_dense(&tensor::<f64>(&[1.0, 2.0, 3.0], &[3]), &full::<f64>())
            .unwrap();
    assert!(matches!(
        op.apply(&bad_state, &full::<f64>()),
        Err(CausalTensorError::ShapeMismatch)
    ));
    // compose with mismatched mid dims: op.in_dims = [2,2] but other.out_dims = [3,3].
    let other = CausalTensorTrainOperator::from_dense(
        &tensor::<f64>(
            &(0..36).map(|i| i as f64).collect::<Vec<_>>(),
            &[3, 2, 3, 2],
        ),
        &[3, 3],
        &[2, 2],
        &full::<f64>(),
    )
    .unwrap();
    assert!(matches!(
        op.compose(&other, &full::<f64>()),
        Err(CausalTensorError::ShapeMismatch)
    ));
    // from_dense with wrong shape.
    assert!(matches!(
        CausalTensorTrainOperator::from_dense(
            &op_dense::<f64>(0.0),
            &[2, 2],
            &[3, 3],
            &full::<f64>()
        ),
        Err(CausalTensorError::ShapeMismatch)
    ));
    // from_cores rejects non-rank-4.
    let bad = CausalTensor::new(vec![0.0; 4], vec![1, 2, 2]).unwrap();
    assert!(matches!(
        CausalTensorTrainOperator::<f64>::from_cores(vec![bad]),
        Err(CausalTensorError::DimensionMismatch)
    ));
}
