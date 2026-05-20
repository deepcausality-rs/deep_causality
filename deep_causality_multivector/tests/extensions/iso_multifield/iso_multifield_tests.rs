/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the structural pack/unpack iso between
//! [`CausalMultiField<T>`] and its carrier tuple
//! `(CausalTensor<T>, Metric, [T; 3], [usize; 3])`.

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, MultiFieldCarrier};
use deep_causality_num::iso::witness::StandardIso;
use deep_causality_num::iso::witness::test_support::assert_witness_iso_round_trip;
use deep_causality_tensor::CausalTensor;

fn make_field() -> CausalMultiField<f32> {
    let shape = [2_usize, 2, 2];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1_f32, 0.2, 0.3];
    CausalMultiField::zeros(shape, metric, dx)
}

// =============================================================================
// Forward: CausalMultiField -> tuple
// =============================================================================

#[test]
fn forward_unpacks_into_carrier_tuple() {
    let field = make_field();
    let metric_before = Metric::from_signature(2, 0, 0);
    let expected_dx = [0.1_f32, 0.2, 0.3];
    let expected_shape = [2_usize, 2, 2];

    let carrier: MultiFieldCarrier<f32> = field.into();
    let (_tensor, metric, dx, shape) = carrier;
    assert_eq!(metric, metric_before);
    assert_eq!(dx, expected_dx);
    assert_eq!(shape, expected_shape);
}

#[test]
fn forward_preserves_tensor_data() {
    let field = make_field();
    let tensor_clone = field.data().clone();

    let carrier: MultiFieldCarrier<f32> = field.into();
    let (tensor, _, _, _) = carrier;
    assert_eq!(tensor, tensor_clone);
}

// =============================================================================
// Reverse: tuple -> CausalMultiField
// =============================================================================

#[test]
fn reverse_packs_tuple_into_multifield() {
    let shape = [2_usize, 2, 2];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.4_f32, 0.5, 0.6];
    let tensor = CausalTensor::new(vec![1.0_f32; 32], vec![2, 2, 2, 2, 2]).unwrap();

    let carrier: MultiFieldCarrier<f32> = (tensor.clone(), metric, dx, shape);
    let field: CausalMultiField<f32> = carrier.into();

    assert_eq!(field.metric(), metric);
    assert_eq!(field.dx(), &dx);
    assert_eq!(field.shape(), &shape);
    assert_eq!(field.data(), &tensor);
}

// =============================================================================
// Round-trip via StandardIso witness
// =============================================================================

#[test]
fn round_trip_via_standard_iso_holds() {
    // Two independent inputs of matching shape, per the Tier 2
    // round-trip contract.
    let field = make_field();
    let shape = [2_usize, 2, 2];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.7_f32, 0.8, 0.9];
    let tensor = CausalTensor::new(vec![2.5_f32; 32], vec![2, 2, 2, 2, 2]).unwrap();
    let carrier: MultiFieldCarrier<f32> = (tensor, metric, dx, shape);

    assert_witness_iso_round_trip::<
        StandardIso<CausalMultiField<f32>, MultiFieldCarrier<f32>>,
        CausalMultiField<f32>,
        MultiFieldCarrier<f32>,
    >(field, carrier);
}

#[test]
fn round_trip_byte_identical_for_self_pair() {
    let field = make_field();
    let carrier: MultiFieldCarrier<f32> = field.clone().into();
    let back: CausalMultiField<f32> = carrier.into();
    assert_eq!(field, back);
}
