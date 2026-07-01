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
    let tt = sample_f64();
    let mapped = CausalTensorTrainWitness::fmap(tt.clone(), |x| x);
    for (a, b) in tt.cores().iter().zip(mapped.cores()) {
        assert_eq!(a.as_slice(), b.as_slice());
    }
}

#[test]
fn test_functor_composition_law() {
    let tt = sample_f64();
    let f = |x: f64| x + 1.0;
    let g = |x: f64| x * 2.0;

    let composed = CausalTensorTrainWitness::fmap(tt.clone(), move |x| g(f(x)));
    let staged = CausalTensorTrainWitness::fmap(CausalTensorTrainWitness::fmap(tt, f), g);

    for (a, b) in composed.cores().iter().zip(staged.cores()) {
        assert_eq!(a.as_slice(), b.as_slice());
    }
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
