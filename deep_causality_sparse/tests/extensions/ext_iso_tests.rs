/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the mixed-tier `CausalTensor<F>` <-> `CsrMatrix<F>` iso.

use deep_causality_num::iso::witness::Iso;
use deep_causality_num::iso::witness::test_support::assert_witness_iso_round_trip;
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;

// =============================================================================
// Forward: CausalTensor (rank 2) -> CsrMatrix via `From`
// =============================================================================

#[test]
fn forward_from_dense_to_sparse_drops_zeros() {
    let dense = CausalTensor::new(vec![1.0_f64, 0.0, 0.0, 4.0, 0.0, 6.0], vec![2, 3]).unwrap();

    let sparse: CsrMatrix<f64> = dense.into();

    assert_eq!(sparse.shape(), (2, 3));
    assert_eq!(sparse.values(), &vec![1.0, 4.0, 6.0]);
    assert_eq!(sparse.col_indices(), &vec![0, 0, 2]);
}

#[test]
fn forward_from_all_zeros_yields_empty_sparse() {
    let dense = CausalTensor::new(vec![0.0_f64; 6], vec![2, 3]).unwrap();
    let sparse: CsrMatrix<f64> = dense.into();
    assert_eq!(sparse.shape(), (2, 3));
    assert_eq!(sparse.values().len(), 0);
}

#[test]
#[should_panic(expected = "CausalTensor -> CsrMatrix requires rank 2")]
fn forward_panics_on_rank_one() {
    let dense = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap();
    let _: CsrMatrix<f64> = dense.into();
}

#[test]
#[should_panic(expected = "CausalTensor -> CsrMatrix requires rank 2")]
fn forward_panics_on_rank_three() {
    let dense = CausalTensor::new(vec![1.0_f64; 8], vec![2, 2, 2]).unwrap();
    let _: CsrMatrix<f64> = dense.into();
}

// =============================================================================
// Reverse: CsrMatrix -> CausalTensor via Iso + inherent to_dense()
// =============================================================================

#[test]
fn reverse_to_dense_via_iso_method_materialises_zeros() {
    let triplets = vec![(0_usize, 0_usize, 1.0_f64), (1, 0, 4.0), (1, 2, 6.0)];
    let sparse = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();

    let dense = <CsrMatrix<f64> as Iso<CsrMatrix<f64>, CausalTensor<f64>>>::to_target(sparse);

    assert_eq!(dense.shape(), &[2, 3]);
    assert_eq!(dense.data(), &vec![1.0, 0.0, 0.0, 4.0, 0.0, 6.0]);
}

#[test]
fn reverse_to_dense_inherent_alias_agrees_with_iso() {
    let triplets = vec![(0_usize, 0_usize, 1.0_f64), (1, 0, 4.0), (1, 2, 6.0)];
    let sparse_a = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();
    let sparse_b = sparse_a.clone();

    let via_iso = <CsrMatrix<f64> as Iso<CsrMatrix<f64>, CausalTensor<f64>>>::to_target(sparse_a);
    let via_alias = sparse_b.to_dense();

    assert_eq!(via_iso.shape(), via_alias.shape());
    assert_eq!(via_iso.data(), via_alias.data());
}

#[test]
fn reverse_to_dense_handles_empty_sparse() {
    let sparse = CsrMatrix::<f64>::from_triplets(2, 3, &[]).unwrap();
    let dense = sparse.to_dense();
    assert_eq!(dense.shape(), &[2, 3]);
    assert_eq!(dense.data(), &vec![0.0_f64; 6]);
}

// =============================================================================
// Round-trip (independent inputs, per the Tier 2 helper contract)
// =============================================================================

#[test]
fn round_trip_holds_with_independent_inputs() {
    // Independent (s, t) pair: a sparse matrix and an unrelated dense
    // tensor of the SAME shape. The helper checks both
    // F->G->F (sparse->dense->sparse) and G->F->G (dense->sparse->dense).
    let sparse =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0_f64), (1, 0, 4.0), (1, 2, 6.0)]).unwrap();
    let dense = CausalTensor::new(vec![1.0_f64, 0.0, 0.0, 4.0, 0.0, 6.0], vec![2, 3]).unwrap();

    assert_witness_iso_round_trip::<CsrMatrix<f64>, CsrMatrix<f64>, CausalTensor<f64>>(
        sparse, dense,
    );
}

#[test]
fn round_trip_holds_for_all_zero_inputs() {
    let sparse = CsrMatrix::<f64>::from_triplets(2, 3, &[]).unwrap();
    let dense = CausalTensor::new(vec![0.0_f64; 6], vec![2, 3]).unwrap();

    assert_witness_iso_round_trip::<CsrMatrix<f64>, CsrMatrix<f64>, CausalTensor<f64>>(
        sparse, dense,
    );
}
