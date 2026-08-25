/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The read-trait seam that lets `deep_causality_linear` run against a tensor's own buffer.

use deep_causality_linear::{
    LinearError, LinearErrorEnum, MatrixView, matrix_norm_frobenius, matrix_norm_inf,
    matrix_norm_l1,
};
use deep_causality_tensor::CausalTensor;

fn m23() -> CausalTensor<f64> {
    // [1 2 3]
    // [4 5 6]
    CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap()
}

#[test]
fn test_a_two_dimensional_tensor_presents_its_shape() {
    let t = m23();
    assert_eq!(MatrixView::rows(&t), 2);
    assert_eq!(MatrixView::cols(&t), 3);
    assert_eq!(t.shape(), &[2, 3]);
    assert!(!MatrixView::is_empty(&t));
    assert!(!MatrixView::is_square(&t));
}

#[test]
fn test_entries_read_in_row_major_order() {
    let t = m23();
    // The tensor's own row-major layout is what the read trait exposes; no transpose, no copy.
    for (i, expected) in [(0usize, 1.0), (1, 2.0), (2, 3.0)] {
        assert_eq!(MatrixView::get(&t, 0, i).unwrap(), expected);
    }
    assert_eq!(MatrixView::get(&t, 1, 0).unwrap(), 4.0);
    assert_eq!(MatrixView::get(&t, 1, 2).unwrap(), 6.0);
}

#[test]
fn test_a_position_outside_the_shape_is_rejected() {
    let t = m23();
    assert!(matches!(
        MatrixView::get(&t, 2, 0),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds {
            index: (2, 0),
            shape: (2, 3)
        }))
    ));
    assert!(matches!(
        MatrixView::get(&t, 0, 3),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds {
            index: (0, 3),
            shape: (2, 3)
        }))
    ));
}

#[test]
fn test_a_tensor_of_rank_other_than_two_is_not_a_matrix() {
    // rows/cols return usize and cannot report an error, so a non-2-D tensor has to present as
    // some shape. It presents as 0x1: empty, and not square, so the operations refuse it.
    let rank3: CausalTensor<f64> =
        CausalTensor::new((0..24).map(|i| i as f64).collect(), vec![2, 3, 4]).unwrap();
    assert_eq!(MatrixView::rows(&rank3), 0);
    assert_eq!(MatrixView::cols(&rank3), 1);
    assert!(MatrixView::is_empty(&rank3));
    assert!(
        !MatrixView::is_square(&rank3),
        "0x0 would be square, and the determinant of the empty matrix is one -- a rank-3 tensor \
         must not get a confident answer back"
    );

    let rank1: CausalTensor<f64> = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    assert_eq!(MatrixView::rows(&rank1), 0);
    assert_eq!(MatrixView::cols(&rank1), 1);
}

#[test]
fn test_every_position_of_a_non_matrix_tensor_is_rejected() {
    let rank3: CausalTensor<f64> =
        CausalTensor::new((0..24).map(|i| i as f64).collect(), vec![2, 3, 4]).unwrap();
    // Not merely the positions outside 0x1 -- (0, 0) is refused too, because the tensor has no
    // position that `(row, col)` names.
    for (r, c) in [(0usize, 0usize), (1, 2), (0, 1)] {
        assert!(
            matches!(
                MatrixView::get(&rank3, r, c),
                Err(LinearError(LinearErrorEnum::IndexOutOfBounds {
                    shape: (0, 1),
                    ..
                }))
            ),
            "({r}, {c}) must be refused"
        );
    }
}

#[test]
fn test_a_linear_algorithm_runs_against_the_tensor_directly() {
    // The point of the seam: the norms are generic over the read trait, so they run against the
    // tensor's own buffer with no dense matrix built to hold a copy.
    //
    // [1 2 3; 4 5 6]: column sums of moduli 5, 7, 9 -> l1 = 9. Row sums 6 and 15 -> inf = 15.
    // Frobenius = sqrt(1 + 4 + 9 + 16 + 25 + 36) = sqrt(91).
    let t = m23();
    assert_eq!(matrix_norm_l1(&t).unwrap(), 9.0);
    assert_eq!(matrix_norm_inf(&t).unwrap(), 15.0);
    assert!((matrix_norm_frobenius(&t).unwrap() - 91.0f64.sqrt()).abs() < 1e-12);
}

#[test]
fn test_a_norm_of_a_tensor_that_is_not_a_matrix_is_the_empty_norm() {
    // 0x1 has no entries, so the sums are empty and every norm is zero. That is the honest answer
    // for the shape presented; the operations that can refuse -- the ones needing squareness --
    // are what catch the misuse.
    let rank3: CausalTensor<f64> =
        CausalTensor::new((0..8).map(|i| i as f64).collect(), vec![2, 2, 2]).unwrap();
    assert_eq!(matrix_norm_l1(&rank3).unwrap(), 0.0);
    assert_eq!(matrix_norm_inf(&rank3).unwrap(), 0.0);
}

#[test]
fn test_a_degenerate_two_dimensional_shape_is_empty_not_broken() {
    // A 0xN tensor is a genuine 2-D shape that holds nothing, distinct from a non-matrix.
    let empty: CausalTensor<f64> = CausalTensor::new(vec![], vec![0, 3]).unwrap();
    assert_eq!(MatrixView::rows(&empty), 0);
    assert_eq!(MatrixView::cols(&empty), 3);
    assert!(MatrixView::is_empty(&empty));
    assert!(matches!(
        MatrixView::get(&empty, 0, 0),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds {
            shape: (0, 3),
            ..
        }))
    ));
}

#[test]
fn test_row_major_copy_is_the_tensor_buffer_in_order() {
    // The override exists to skip the per-entry path; it must still deliver rows * cols entries in
    // the order the default would have produced them.
    let t = m23();
    let buf = MatrixView::to_row_major(&t).unwrap();
    assert_eq!(buf.len(), MatrixView::rows(&t) * MatrixView::cols(&t));
    assert_eq!(buf, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);

    let by_position: Vec<f64> = (0..MatrixView::rows(&t))
        .flat_map(|r| (0..MatrixView::cols(&t)).map(move |c| (r, c)))
        .map(|(r, c)| MatrixView::get(&t, r, c).unwrap())
        .collect();
    assert_eq!(buf, by_position);
}

#[test]
fn test_row_major_copy_of_a_non_matrix_tensor_is_empty() {
    // A rank-3 tensor presents as 0x1, so the buffer the view owes is rows * cols = 0 entries. A
    // copy of the tensor's 24 entries would be a buffer whose shape disagrees with the reported
    // one, and the kernels index it by the reported shape.
    let rank3: CausalTensor<f64> =
        CausalTensor::new((0..24).map(|i| i as f64).collect(), vec![2, 3, 4]).unwrap();
    let buf = MatrixView::to_row_major(&rank3).unwrap();
    assert_eq!(
        buf.len(),
        MatrixView::rows(&rank3) * MatrixView::cols(&rank3)
    );
    assert!(buf.is_empty());

    let rank1: CausalTensor<f64> = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    assert!(MatrixView::to_row_major(&rank1).unwrap().is_empty());
}

#[test]
fn test_row_major_copy_of_a_degenerate_two_dimensional_shape_is_empty() {
    // 0x3 is a genuine 2-D shape holding nothing, and it takes the copy branch rather than the
    // non-matrix one. Both owe zero entries, for different reasons.
    let empty: CausalTensor<f64> = CausalTensor::new(vec![], vec![0, 3]).unwrap();
    assert!(MatrixView::to_row_major(&empty).unwrap().is_empty());
}
