/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// ---------------------------------------------------------------------------
// Open-boundary weighted Leray projection
// ---------------------------------------------------------------------------

use deep_causality_physics::utils_tests::{divergence, random_cochain, sup_norm, unit_manifold};
use deep_causality_physics::{SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::ChainComplex;

#[test]
fn from_open_leray_projection_weighted_opts_empty_rows_reduces_to_closed() {
    // With empty zeroed/prescribed/reference/rows, the weighted open path reduces
    // to the constrained projection: a divergence-free field plus the φ potential.
    let manifold = unit_manifold::<f64>(5);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<f64>(n1, 101), vec![n1]).unwrap();
    let scale = sup_norm(raw.as_slice());
    let velocity = VelocityOneForm::new(raw, &manifold).unwrap();

    let opts = deep_causality_topology::HodgeDecomposeOptions::default();
    let (field, potential) = SolenoidalField::from_open_leray_projection_weighted_opts(
        &velocity,
        &manifold,
        &[],
        &[],
        &[],
        &[],
        &opts,
        None,
    )
    .unwrap();

    assert_eq!(potential.len(), manifold.complex().num_cells(0));
    let div = divergence(&manifold, field.as_one_form().as_slice());
    assert!(
        sup_norm(&div) < 1e-7 * scale,
        "weighted open path divergence {} (scale {scale})",
        sup_norm(&div)
    );
}

#[test]
fn from_open_leray_projection_weighted_opts_propagates_failure() {
    // A weighted-row edge index out of range surfaces as a typed TopologyError.
    let manifold = unit_manifold::<f64>(4);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(vec![1.0; n1], vec![n1]).unwrap();
    let velocity = VelocityOneForm::new(raw, &manifold).unwrap();

    let bad_row = deep_causality_topology::CutFaceConstraint::new(
        vec![(n1 + 10, 1.0_f64)], // out-of-range edge index
        0.0,
        1.0,
        deep_causality_topology::CutConstraintKind::Tangential,
    );
    let opts = deep_causality_topology::HodgeDecomposeOptions::default();
    let err = SolenoidalField::from_open_leray_projection_weighted_opts(
        &velocity,
        &manifold,
        &[],
        &[],
        &[],
        core::slice::from_ref(&bad_row),
        &opts,
        None,
    )
    .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("Topology Error") && msg.contains("open weighted Leray projection failed"),
        "got: {msg}"
    );
}

#[test]
fn read_only_accessors_and_derives() {
    let manifold = unit_manifold::<f64>(3);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<f64>(n1, 73), vec![n1]).unwrap();
    let velocity = VelocityOneForm::new(raw, &manifold).unwrap();
    let (field, _potential) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();

    assert_eq!(field.len(), n1);
    assert!(!field.is_empty());
    assert_eq!(field.as_one_form().len(), n1);

    let clone = field.clone();
    assert_eq!(clone, field);
    assert!(format!("{field:?}").contains("SolenoidalField"));
}
