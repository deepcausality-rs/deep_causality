/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the mixed-tier `CausalTensor<F>` <-> `CsrMatrix<F>` iso.

use deep_causality_algebra::iso::witness::Iso;
use deep_causality_algebra::iso::witness::test_support::assert_witness_iso_round_trip;
use deep_causality_linear::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_tensor::{CsrFromTensorError, ToDenseTensor};

// =============================================================================
// Forward: CausalTensor (rank 2) -> CsrMatrix via `TryFrom`
// =============================================================================

#[test]
fn forward_try_from_dense_to_sparse_drops_zeros() {
    let dense = CausalTensor::new(vec![1.0_f64, 0.0, 0.0, 4.0, 0.0, 6.0], vec![2, 3]).unwrap();

    let sparse: CsrMatrix<f64> = CsrMatrix::try_from(dense).unwrap();

    assert_eq!(sparse.shape(), (2, 3));
    assert_eq!(sparse.values(), &vec![1.0, 4.0, 6.0]);
    assert_eq!(sparse.col_indices(), &vec![0, 0, 2]);
}

#[test]
fn forward_try_from_all_zeros_yields_empty_sparse() {
    let dense = CausalTensor::new(vec![0.0_f64; 6], vec![2, 3]).unwrap();
    let sparse: CsrMatrix<f64> = CsrMatrix::try_from(dense).unwrap();
    assert_eq!(sparse.shape(), (2, 3));
    assert_eq!(sparse.values().len(), 0);
}

#[test]
fn forward_try_from_returns_err_on_rank_one() {
    let dense = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap();
    let err = CsrMatrix::try_from(dense).unwrap_err();
    assert_eq!(err, CsrFromTensorError { rank: 1 });
}

#[test]
fn forward_try_from_returns_err_on_rank_three() {
    let dense = CausalTensor::new(vec![1.0_f64; 8], vec![2, 2, 2]).unwrap();
    let err = CsrMatrix::try_from(dense).unwrap_err();
    assert_eq!(err, CsrFromTensorError { rank: 3 });
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
// Iso::to_source still panics (Tier 2 trait demands infallible)
// =============================================================================

#[test]
#[should_panic(expected = "Iso::to_source requires a rank-2 CausalTensor")]
fn iso_to_source_panics_on_wrong_rank() {
    // The trait `Iso<S, T>::to_source(t: T) -> S` is infallible by
    // contract. The forward direction is intrinsically partial; the iso
    // surface therefore panics when handed a non-rank-2 tensor. Callers
    // wanting graceful failure use `CsrMatrix::try_from(...)` directly.
    let dense = CausalTensor::new(vec![1.0_f64; 8], vec![2, 2, 2]).unwrap();
    let _: CsrMatrix<f64> =
        <CsrMatrix<f64> as Iso<CsrMatrix<f64>, CausalTensor<f64>>>::to_source(dense);
}

// =============================================================================
// Round-trip with genuinely independent inputs
// =============================================================================

#[test]
fn round_trip_holds_with_independent_inputs() {
    // `sparse` and `dense` are independent matrices of the same shape.
    // They represent DIFFERENT matrices, so the helper genuinely tests:
    //   F->G->F: sparsify the sparse (round-trip its data via dense), and
    //   G->F->G: densify the dense (round-trip its data via sparse).
    // Using the same matrix in both slots would collapse the two
    // branches into one data path.
    let sparse =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0_f64), (1, 0, 4.0), (1, 2, 6.0)]).unwrap();
    let dense = CausalTensor::new(vec![0.0_f64, 7.0, 0.0, 0.0, 0.0, 8.0], vec![2, 3]).unwrap();

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

// =============================================================================
// Display / Error surface of CsrFromTensorError
// =============================================================================

#[test]
fn error_display_states_the_required_rank_and_the_rank_given() {
    // The message has to carry both halves: what the conversion needs (rank 2) and what it was
    // handed, so a caller reading a log knows why the tensor was refused without the value.
    let err = CsrFromTensorError { rank: 3 };
    assert_eq!(
        err.to_string(),
        "CausalTensor -> CsrMatrix requires rank 2, got rank 3"
    );
}

#[test]
fn error_display_reports_the_rank_of_the_tensor_that_was_refused() {
    // The rank in the message is the one the caller actually handed over, so refusals of
    // different ranks are distinguishable from the message alone.
    let rank1 = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap();
    let rank4 = CausalTensor::new(vec![0.0_f64; 16], vec![2, 2, 2, 2]).unwrap();

    let msg1 = CsrMatrix::try_from(rank1).unwrap_err().to_string();
    let msg4 = CsrMatrix::try_from(rank4).unwrap_err().to_string();

    assert_eq!(
        msg1,
        "CausalTensor -> CsrMatrix requires rank 2, got rank 1"
    );
    assert_eq!(
        msg4,
        "CausalTensor -> CsrMatrix requires rank 2, got rank 4"
    );
}

#[test]
fn error_carries_no_cause_of_its_own() {
    // The refusal originates here -- nothing underneath failed -- so the error chain ends at it.
    let err: Box<dyn std::error::Error> = Box::new(CsrFromTensorError { rank: 0 });
    assert!(err.source().is_none());
    assert_eq!(
        err.to_string(),
        "CausalTensor -> CsrMatrix requires rank 2, got rank 0"
    );
}
