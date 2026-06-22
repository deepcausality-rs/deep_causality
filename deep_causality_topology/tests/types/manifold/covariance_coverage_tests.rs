/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for the under-sampled error path of the Manifold covariance
//! analysis: sample variance needs at least two observations, so a manifold
//! whose field carries fewer than two samples must return `InvalidInput`.

use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Simplex;
use deep_causality_topology::{Manifold, SimplicialComplex, Skeleton, TopologyErrorEnum};

/// A minimal valid manifold carrying a single 0-simplex (one sample).
fn single_vertex_manifold() -> Manifold<SimplicialComplex<f64>, f64> {
    let skeletons = vec![Skeleton::new(0, vec![Simplex::new(vec![0])])];
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(skeletons, vec![], vec![], Vec::new());
    let data = CausalTensor::new(vec![42.0], vec![1]).unwrap();
    Manifold::new(complex, data, 0).unwrap()
}

/// A valid 1-D line manifold (two 0-simplices + one 1-simplex) whose field
/// therefore carries three samples — enough for the sample covariance to
/// succeed.
fn line_manifold() -> Manifold<SimplicialComplex<f64>, f64> {
    let skeleton_0 = Skeleton::new(0, vec![Simplex::new(vec![0]), Simplex::new(vec![1])]);
    let skeleton_1 = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    let d1 = CsrMatrix::from_triplets(2, 1, &[(1, 0, 1i8), (0, 0, -1)]).unwrap();
    let complex = SimplicialComplex::new(vec![skeleton_0, skeleton_1], vec![d1], vec![], vec![]);
    let data = CausalTensor::new(vec![1.0, 2.0, 5.0], vec![3]).unwrap();
    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn covariance_with_one_sample_is_rejected() {
    let manifold = single_vertex_manifold();
    let err = manifold.covariance_matrix().unwrap_err();
    match err.0 {
        TopologyErrorEnum::InvalidInput(ref msg) => {
            assert!(
                msg.contains("at least 2"),
                "error should mention the two-sample minimum: {msg}"
            );
        }
        ref other => panic!("expected InvalidInput, got {other:?}"),
    }
}

#[test]
fn eigen_covariance_with_one_sample_propagates_the_error() {
    let manifold = single_vertex_manifold();
    let err = manifold.eigen_covariance().unwrap_err();
    assert!(matches!(err.0, TopologyErrorEnum::InvalidInput(_)));
}

#[test]
fn covariance_with_several_samples_returns_one_by_one_matrix() {
    // Three samples make `sample_covariance` succeed; the field is treated as
    // `n` observations of a single variable, so the covariance is the 1×1
    // variance. This drives the `cov.get(&[0, 0])` success read.
    let manifold = line_manifold();
    let cov = manifold.covariance_matrix().expect("covariance succeeds");
    assert_eq!(cov.len(), 1);
    assert_eq!(cov[0].len(), 1);
    assert!(cov[0][0] > 0.0, "variance of distinct samples is positive");
}

#[test]
fn eigen_covariance_one_by_one_returns_single_eigenvalue() {
    // With a successful 1×1 covariance, `eigen_covariance_impl` takes the
    // `cov.len() == 1 && cov[0].len() == 1` fast path and returns the lone
    // variance as the single eigenvalue.
    let manifold = line_manifold();
    let eig = manifold
        .eigen_covariance()
        .expect("eigen covariance succeeds");
    assert_eq!(eig.len(), 1);
    assert!(eig[0] > 0.0);
}
