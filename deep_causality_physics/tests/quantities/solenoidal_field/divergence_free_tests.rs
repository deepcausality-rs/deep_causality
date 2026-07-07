/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_physics::utils_tests::{divergence, random_cochain, sup_norm, unit_manifold};
use deep_causality_physics::{SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::ChainComplex;

/// Both construction paths produce divergence-free fields, per precision.
fn assert_both_paths_divergence_free<R>(rel_tol: R)
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let manifold = unit_manifold::<R>(5);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<R>(n1, 67), vec![n1]).unwrap();
    let scale = sup_norm(raw.as_slice());

    // Path 1: per-step Leray projection.
    let velocity = VelocityOneForm::new(raw.clone(), &manifold).unwrap();
    let (leray_field, potential) =
        SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();
    assert_eq!(potential.len(), manifold.complex().num_cells(0));
    let div = divergence(&manifold, leray_field.as_one_form().as_slice());
    assert!(
        sup_norm(&div) < rel_tol * scale,
        "Leray path divergence {} (scale {scale})",
        sup_norm(&div)
    );

    // Path 2: per-snapshot Hodge projection.
    let decomposition = manifold.hodge_decompose(&raw, 1).unwrap();
    let hodge_field = SolenoidalField::from_hodge_projection(&decomposition).unwrap();
    let div = divergence(&manifold, hodge_field.as_one_form().as_slice());
    assert!(
        sup_norm(&div) < rel_tol * scale,
        "Hodge path divergence {} (scale {scale})",
        sup_norm(&div)
    );

    // The two paths agree on the divergence-free field itself.
    let mut max_gap = R::zero();
    for (a, b) in leray_field
        .as_one_form()
        .as_slice()
        .iter()
        .zip(hodge_field.as_one_form().as_slice().iter())
    {
        let d = (*a - *b).abs();
        if d > max_gap {
            max_gap = d;
        }
    }
    assert!(
        max_gap < rel_tol * scale,
        "construction paths disagree by {max_gap}"
    );
}

#[test]
fn both_paths_divergence_free_f64() {
    assert_both_paths_divergence_free::<f64>(1e-7);
}

#[test]
fn both_paths_divergence_free_f32() {
    assert_both_paths_divergence_free::<f32>(1e-3_f32);
}

#[test]
fn both_paths_divergence_free_float106() {
    assert_both_paths_divergence_free::<Float106>(Float106::from_f64(1e-7));
}
