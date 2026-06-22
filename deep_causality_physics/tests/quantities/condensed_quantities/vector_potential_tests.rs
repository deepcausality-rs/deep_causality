/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[test]
fn test_vector_potential_default() {
    let vp = deep_causality_physics::VectorPotential::default();
    // Default is a single-component zero multivector.
    assert_eq!(vp.inner().data().len(), 1);
    assert_eq!(vp.inner().data()[0], 0.0);
}

#[test]
fn test_vector_potential_new() {
    let mv = deep_causality_multivector::CausalMultiVector::new(
        vec![1.0],
        deep_causality_multivector::Metric::Euclidean(0),
    )
    .unwrap();
    let vp = deep_causality_physics::VectorPotential::new(mv.clone());
    assert_eq!(vp.inner().data(), mv.data());
}
