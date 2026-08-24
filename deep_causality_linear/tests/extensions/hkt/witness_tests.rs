/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The HKT witnesses, and the one container that structurally cannot have one.

use deep_causality_haft::HKT;
use deep_causality_linear::MatrixBuild;
use deep_causality_linear::{
    CsrMatrix, CsrMatrixWitness, DenseMatrix, DenseMatrixWitness, DenseVector, DenseVectorWitness,
};

#[test]
fn test_the_dense_matrix_witness_projects_to_the_dense_matrix() {
    fn projects<W: HKT>() {}
    projects::<DenseMatrixWitness>();
    // The projection is the container itself, checked by assigning through it.
    let _: <DenseMatrixWitness as HKT>::Type<f64> = DenseMatrix::from_vec(vec![1.0], 1, 1).unwrap();
}

#[test]
fn test_the_vector_witness_projects_to_the_vector() {
    let _: <DenseVectorWitness as HKT>::Type<f64> = DenseVector::from_vec(vec![1.0]);
}

#[test]
fn test_the_sparse_witness_projects_to_the_sparse_matrix() {
    let _: <CsrMatrixWitness as HKT>::Type<f64> = CsrMatrix::new();
}

#[test]
fn test_the_witnesses_are_zero_sized_and_defaultable() {
    // A witness is a stand-in for a type constructor, so it carries no data.
    assert_eq!(core::mem::size_of::<DenseMatrixWitness>(), 0);
    assert_eq!(core::mem::size_of::<DenseVectorWitness>(), 0);
    assert_eq!(core::mem::size_of::<CsrMatrixWitness>(), 0);
    let _ = DenseMatrixWitness;
    let _ = DenseVectorWitness;
}

#[test]
fn test_the_packed_gf2_matrix_has_no_witness_and_that_is_structural() {
    // PackedGf2 is generic in its *word* and fixed to Gf2 in its element, so there is no
    // PackedGf2<T> for HKT's Type<T> to name. The route to the HKT surface is the conversion to
    // DenseMatrix<Gf2>, which does have a witness.
    //
    // The absence cannot be asserted directly -- this MSRV has no negative impls -- so what is
    // asserted is that the documented route exists.
    let packed: deep_causality_linear::PackedGf2<u64> =
        deep_causality_linear::PackedGf2::zeros(1, 1);
    let _: <DenseMatrixWitness as HKT>::Type<deep_causality_num::Gf2> =
        deep_causality_linear::packed_to_dense_gf2(&packed);
}
