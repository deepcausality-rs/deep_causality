/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `ManifoldWitness` HKT trait impls.
//!
//! Under Option 2C (`ChainComplex::Metric` is a plain associated type; `Manifold<K, F>`
//! has no struct-level `F: RealField` bound), `ManifoldWitness<C>` implements the full
//! `deep_causality_haft` trait surface — `HKT`, `Functor`, `Foldable`, `Pure`, `Monad`,
//! `CoMonad`, `Applicative` — on stable Rust with no inherent-method shim.

use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, Monad, Pure};
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ManifoldWitness, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};

fn create_line_manifold() -> SimplicialManifold<f64, f64> {
    let vertices = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let skeleton_0 = Skeleton::new(0, vertices);

    let edges = vec![Simplex::new(vec![0, 1])];
    let skeleton_1 = Skeleton::new(1, edges);

    let d1 = CsrMatrix::from_triplets(2, 1, &[(1, 0, 1i8), (0, 0, -1)]).unwrap();

    let complex = SimplicialComplex::new(vec![skeleton_0, skeleton_1], vec![d1], vec![], vec![]);
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();

    Manifold::new(complex, data, 0).expect("Failed to create manifold")
}

#[test]
fn test_manifold_functor() {
    let manifold = create_line_manifold();
    let mapped = ManifoldWitness::<f64>::fmap(manifold, |x| x * 2.0);
    assert_eq!(mapped.data().as_slice(), &[2.0, 4.0, 6.0]);
}

#[test]
fn test_manifold_extract() {
    let complex = create_line_manifold().complex().clone();
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let manifold = Manifold::new(complex, data, 1).unwrap();

    let val = ManifoldWitness::<f64>::extract(&manifold);
    assert_eq!(val, 20.0);
}

#[test]
fn test_manifold_extend() {
    let manifold = create_line_manifold();

    let extended = ManifoldWitness::<f64>::extend(&manifold, |w| {
        let val = ManifoldWitness::<f64>::extract(w);
        val + (w.cursor() as f64)
    });

    assert_eq!(extended.data().as_slice(), &[1.0, 3.0, 5.0]);
}

#[test]
fn test_manifold_pure() {
    let manifold: SimplicialManifold<f64, f64> = ManifoldWitness::<f64>::pure(42.0);
    assert_eq!(manifold.data().as_slice(), &[42.0]);
    assert_eq!(manifold.cursor(), 0);
}

#[test]
fn test_manifold_fold() {
    let manifold = create_line_manifold();
    let sum = ManifoldWitness::<f64>::fold(manifold, 0.0, |acc, x| acc + x);
    assert_eq!(sum, 6.0);
}

#[test]
fn test_manifold_bind() {
    let manifold: SimplicialManifold<f64, f64> = ManifoldWitness::<f64>::pure(5.0);

    let bound: SimplicialManifold<f64, f64> =
        ManifoldWitness::<f64>::bind(manifold, |x| ManifoldWitness::<f64>::pure(x * 2.0));

    assert!(!bound.data().is_empty());
    assert_eq!(bound.data().as_slice()[0], 10.0);
}

#[test]
fn test_manifold_applicative_single_func() {
    // Applicative::apply works under Option 2C — `Manifold<_, Func>` is well-formed because
    // `F` has no `RealField` struct-level bound. `fn(f64) -> f64` flows through naturally.
    let func_manifold: SimplicialManifold<f64, fn(f64) -> f64> =
        ManifoldWitness::<f64>::pure(|x: f64| x * 3.0);

    let complex = create_line_manifold().complex().clone();
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let data_manifold = Manifold::new(complex, data, 0).unwrap();

    let result: SimplicialManifold<f64, f64> =
        ManifoldWitness::<f64>::apply(func_manifold, data_manifold);

    assert_eq!(result.data().as_slice(), &[3.0, 6.0, 9.0]);
}

#[test]
fn test_manifold_applicative_multi_func() {
    // When the function manifold carries MORE than one function, `apply` takes the
    // `else` branch and zips functions with arguments pairwise.
    // Covers src/extensions/hkt_manifold/mod.rs line 176.
    let complex = create_line_manifold().complex().clone();

    // Three distinct functions, one per simplex (2 vertices + 1 edge = 3 cells).
    let funcs: Vec<fn(f64) -> f64> = vec![|x| x + 1.0, |x| x * 2.0, |x| x - 3.0];
    let func_data = CausalTensor::new(funcs, vec![3]).unwrap();
    let func_manifold: SimplicialManifold<f64, fn(f64) -> f64> =
        Manifold::new(complex.clone(), func_data, 0).unwrap();

    let arg_data = CausalTensor::new(vec![10.0, 20.0, 30.0], vec![3]).unwrap();
    let data_manifold = Manifold::new(complex, arg_data, 0).unwrap();

    let result: SimplicialManifold<f64, f64> =
        ManifoldWitness::<f64>::apply(func_manifold, data_manifold);

    // f0(10)=11, f1(20)=40, f2(30)=27
    assert_eq!(result.data().as_slice(), &[11.0, 40.0, 27.0]);
}
