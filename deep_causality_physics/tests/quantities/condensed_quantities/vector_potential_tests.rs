/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[test]
fn test_vector_potential_default_and_new() {
    let vp = deep_causality_physics::VectorPotential::default();
    // Default is a single-component zero multivector.
    assert_eq!(vp.inner().data().len(), 1);

    let mv = deep_causality_multivector::CausalMultiVector::new(
        vec![1.0],
        deep_causality_multivector::Metric::Euclidean(0),
    )
    .unwrap();
    let vp2 = deep_causality_physics::VectorPotential::new(mv.clone());
    assert_eq!(vp2.inner().data(), mv.data());
}
