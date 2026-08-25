/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::{
    CsrMatrix, DenseMatrix, LinearError, LinearErrorEnum, MatrixBuild, MatrixView, PackedGf2,
    csr_to_dense, csr_to_packed_gf2_mod2, csr_to_packed_gf2_strict, dense_gf2_to_packed,
    dense_to_csr, packed_to_dense_gf2,
};
use deep_causality_num::Gf2;

#[test]
fn test_sparse_to_dense_and_back_round_trips_including_the_shape() {
    let m = CsrMatrix::from_triplets(3, 4, &[(0, 0, 1.0), (2, 3, 5.0)]).unwrap();
    let dense = csr_to_dense(&m);
    assert_eq!(dense.shape(), (3, 4));
    let back = dense_to_csr(&dense);
    assert_eq!(back.shape(), (3, 4));
    for i in 0..3 {
        for j in 0..4 {
            assert_eq!(
                back.get(i, j).unwrap(),
                m.get(i, j).unwrap(),
                "at ({i}, {j})"
            );
        }
    }
}

#[test]
fn test_densifying_materialises_the_structural_zeros() {
    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 7.0)]).unwrap();
    let dense = csr_to_dense(&m);
    assert_eq!(dense.get(0, 0).unwrap(), 7.0);
    assert_eq!(dense.get(1, 1).unwrap(), 0.0);
}

#[test]
fn test_a_sparse_matrix_with_an_entirely_empty_row() {
    let m = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1.0), (2, 1, 1.0)]).unwrap();
    // Row 1 stores nothing at all.
    assert_eq!(m.get(1, 0).unwrap(), 0.0);
    assert_eq!(m.get(1, 1).unwrap(), 0.0);
    assert_eq!(csr_to_dense(&m).shape(), (3, 2));
}

#[test]
fn test_packing_rejects_an_entry_outside_zero_one_and_names_the_position() {
    let m = CsrMatrix::from_triplets(2, 2, &[(1, 0, 5i8)]).unwrap();
    let e = csr_to_packed_gf2_strict::<u64>(&m).unwrap_err();
    assert!(
        matches!(e, LinearError(LinearErrorEnum::NotBinary { at: (1, 0) })),
        "got {e:?}"
    );
}

#[test]
fn test_packing_accepts_the_boundary_operator_alphabet_by_reducing() {
    // -1 and 1 both map to the F2 one; the reducing conversion is total.
    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, -1i8), (1, 1, 1i8)]).unwrap();
    let packed: PackedGf2<u64> = csr_to_packed_gf2_mod2(&m);
    assert_eq!(packed.get(0, 0).unwrap(), Gf2::ONE, "-1 must map to one");
    assert_eq!(packed.get(1, 1).unwrap(), Gf2::ONE);
    assert_eq!(packed.get(0, 1).unwrap(), Gf2::ZERO);
}

#[test]
fn test_the_strict_conversion_accepts_what_is_already_binary() {
    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1i8), (1, 1, 0i8)]).unwrap();
    let packed: PackedGf2<u64> = csr_to_packed_gf2_strict(&m).unwrap();
    assert_eq!(packed.get(0, 0).unwrap(), Gf2::ONE);
}

#[test]
fn test_dense_gf2_packs_and_unpacks() {
    let mut d: DenseMatrix<Gf2> = DenseMatrix::zeros(2, 3);
    d.set(0, 2, Gf2::ONE).unwrap();
    d.set(1, 0, Gf2::ONE).unwrap();
    let packed: PackedGf2<u8> = dense_gf2_to_packed(&d).unwrap();
    let back = packed_to_dense_gf2(&packed);
    assert_eq!(back.shape(), (2, 3));
    for i in 0..2 {
        for j in 0..3 {
            assert_eq!(
                back.get(i, j).unwrap(),
                d.get(i, j).unwrap(),
                "at ({i}, {j})"
            );
        }
    }
}

#[test]
fn test_a_conversion_preserves_an_empty_shape() {
    let m: CsrMatrix<f64> = CsrMatrix::zeros(0, 0);
    assert_eq!(csr_to_dense(&m).shape(), (0, 0));
}
