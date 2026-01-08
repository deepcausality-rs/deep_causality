/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;

#[test]
fn test_display_empty_matrix() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    let expected = "CsrMatrix (0x0) [Empty]";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_single_element_matrix() {
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap();
    let expected = "CsrMatrix (1x1)\n[   1.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_simple_matrix() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    // Expected:
    // [[1.0, 0.0, 2.0],
    //  [0.0, 3.0, 0.0]]
    let expected =
        "CsrMatrix (2x3)\n[   1.000,    0.000,    2.000]\n[   0.000,    3.000,    0.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_matrix_with_empty_row() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(3, 2, &[(0, 0, 1.0), (2, 1, 4.0)]).unwrap();
    // Expected:
    // [[1.0, 0.0],
    //  [0.0, 0.0],
    //  [0.0, 4.0]]
    let expected =
        "CsrMatrix (3x2)\n[   1.000,    0.000]\n[   0.000,    0.000]\n[   0.000,    4.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_larger_matrix() {
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(
        4,
        4,
        &[
            (0, 0, 1.123),
            (0, 3, 2.0),
            (1, 1, 3.45),
            (2, 0, 5.0),
            (2, 2, 6.789),
            (3, 3, 7.0),
        ],
    )
    .unwrap();
    let expected = "CsrMatrix (4x4)\n[   1.123,    0.000,    0.000,    2.000]\n[   0.000,    3.450,    0.000,    0.000]\n[   5.000,    0.000,    6.789,    0.000]\n[   0.000,    0.000,    0.000,    7.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_all_zeros_matrix_from_triplets() {
    let triplets = vec![(0, 0, 0.0), (1, 1, 0.0)]; // All zero values
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    // This should result in an effectively empty sparse matrix representation with 2x2 shape
    let expected = "CsrMatrix (2x2)\n[   0.000,    0.000]\n[   0.000,    0.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_float_precision() {
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.234567)]).unwrap();
    let expected = "CsrMatrix (1x1)\n[   1.235]\n"; // Rounded to 3 decimal places
    assert_eq!(format!("{}", matrix), expected);
}
