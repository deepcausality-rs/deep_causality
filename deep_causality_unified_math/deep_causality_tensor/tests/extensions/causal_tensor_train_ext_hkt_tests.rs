/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Foldable, Functor, HKT, Pure};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainWitness, TensorTrain,
};

fn sample_f64() -> CausalTensorTrain<f64> {
    let core0 = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![1, 2, 2]).unwrap();
    let core1 = CausalTensor::new(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2, 1]).unwrap();
    CausalTensorTrain::from_cores(vec![core0, core1]).unwrap()
}

#[test]
fn test_witness_type() {
    let value: <CausalTensorTrainWitness as HKT>::Type<f64> = sample_f64();
    assert_eq!(value.order(), 2);
}

#[test]
fn test_fmap_precision_conversion() {
    // The storage functor maps the scalar type of every core: f64 → f32.
    let tt = sample_f64();
    let as_f32: CausalTensorTrain<f32> = CausalTensorTrainWitness::fmap(tt, |x| x as f32);
    let cores = as_f32.cores();
    assert_eq!(cores[0].as_slice(), &[1.0f32, 2.0, 3.0, 4.0]);
    assert_eq!(cores[1].as_slice(), &[5.0f32, 6.0, 7.0, 8.0]);
    assert_eq!(as_f32.phys_dims(), &[2, 2]);
}

#[test]
fn test_functor_identity_law() {
    // Structural equality on the whole value. Comparing `as_slice()` through a `zip` reads only
    // the core payloads, skips silently if the core counts differ, and cannot see `phys_dims`,
    // `order` or the tracked canonical form.
    let tt = sample_f64();
    assert_eq!(CausalTensorTrainWitness::fmap(tt.clone(), |x: f64| x), tt);
}

#[test]
fn test_functor_identity_law_preserves_the_canonical_form() {
    // `fmap` sets `CanonicalForm::None` unconditionally, which is right for a general `f`
    // because mapping entries does not preserve orthogonality. Under the identity it makes
    // `fmap(id) != id`, so the functor identity law fails on any canonicalized train.
    let tt = sample_f64().left_canonicalize().unwrap();
    assert_eq!(CausalTensorTrainWitness::fmap(tt.clone(), |x: f64| x), tt);
}

#[test]
fn test_functor_composition_law() {
    // Both sides are checked against an independently computed expectation, not only against
    // each other: two sides that share a defect agree while both being wrong.
    let tt = sample_f64();
    let f = |x: f64| x + 1.0;
    let g = |x: f64| x * 2.0;

    let composed = CausalTensorTrainWitness::fmap(tt.clone(), move |x| g(f(x)));
    let staged = CausalTensorTrainWitness::fmap(CausalTensorTrainWitness::fmap(tt.clone(), f), g);

    // g(f(x)) = (x + 1) * 2, cores written out by hand.
    let expected = CausalTensorTrain::from_cores(vec![
        CausalTensor::new(vec![4.0, 6.0, 8.0, 10.0], vec![1, 2, 2]).unwrap(),
        CausalTensor::new(vec![12.0, 14.0, 16.0, 18.0], vec![2, 2, 1]).unwrap(),
    ])
    .unwrap();
    assert_eq!(composed, expected);
    assert_eq!(staged, expected);
    assert_eq!(composed.phys_dims(), tt.phys_dims());
    assert_eq!(composed.order(), tt.order());
}

#[test]
fn test_fold_over_core_entries() {
    // Folds over the factors (all core entries), not the logical tensor.
    let tt = sample_f64();
    let sum = CausalTensorTrainWitness::fold(tt, 0.0, |acc, x| acc + x);
    assert_eq!(sum, 1.0 + 2.0 + 3.0 + 4.0 + 5.0 + 6.0 + 7.0 + 8.0);
}

#[test]
fn test_pure_is_rank_one_scalar() {
    let tt: CausalTensorTrain<f64> = CausalTensorTrainWitness::pure(42.0);
    assert_eq!(tt.order(), 1);
    assert_eq!(tt.phys_dims(), &[1]);
    assert_eq!(tt.to_dense().unwrap().as_slice(), &[42.0]);
}
