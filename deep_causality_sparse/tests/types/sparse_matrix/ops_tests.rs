use deep_causality_sparse::CsrMatrix;
use deep_causality_sparse::SparseMatrixError;

#[test]
fn test_add_matrix_success() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    // A = [[1.0, 0.0], [0.0, 2.0]]

    let b = CsrMatrix::from_triplets(2, 2, &[(0, 1, 3.0), (1, 0, 4.0)]).unwrap();
    // B = [[0.0, 3.0], [4.0, 0.0]]

    let c = a.add_matrix(&b).unwrap();
    // C = A + B = [[1.0, 3.0], [4.0, 2.0]]

    assert_eq!(c.get_value_at(0, 0), 1.0);
    assert_eq!(c.get_value_at(0, 1), 3.0);
    assert_eq!(c.get_value_at(1, 0), 4.0);
    assert_eq!(c.get_value_at(1, 1), 2.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_add_matrix_with_overlapping_elements() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 2.0)]).unwrap();
    // A = [[1.0, 2.0], [0.0, 0.0]]

    let b = CsrMatrix::from_triplets(2, 2, &[(0, 0, 3.0), (1, 1, 4.0)]).unwrap();
    // B = [[3.0, 0.0], [0.0, 4.0]]

    let c = a.add_matrix(&b).unwrap();
    // C = A + B = [[4.0, 2.0], [0.0, 4.0]]

    assert_eq!(c.get_value_at(0, 0), 4.0);
    assert_eq!(c.get_value_at(0, 1), 2.0);
    assert_eq!(c.get_value_at(1, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 4.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_add_matrix_resulting_in_zero_element() {
    let a = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(1, 1, &[(0, 0, -1.0)]).unwrap();
    let c = a.add_matrix(&b).unwrap();
    assert_eq!(c.get_value_at(0, 0), 0.0);
    assert!(c.values().is_empty());
}

#[test]
fn test_add_matrix_shape_mismatch() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();
    let err = a.add_matrix(&b).unwrap_err();
    assert!(matches!(
        err,
        SparseMatrixError::ShapeMismatch((2, 2), (2, 3))
    ));
}

#[test]
fn test_sub_matrix_success() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 5.0), (0, 1, 2.0), (1, 1, 3.0)]).unwrap();
    // A = [[5.0, 2.0], [0.0, 3.0]]

    let b = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 1.0)]).unwrap();
    // B = [[1.0, 0.0], [0.0, 1.0]]

    let c = a.sub_matrix(&b).unwrap();
    // C = A - B = [[4.0, 2.0], [0.0, 2.0]]

    assert_eq!(c.get_value_at(0, 0), 4.0);
    assert_eq!(c.get_value_at(0, 1), 2.0);
    assert_eq!(c.get_value_at(1, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 2.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_sub_matrix_resulting_in_zero_element() {
    let a = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap();
    let c = a.sub_matrix(&b).unwrap();
    assert_eq!(c.get_value_at(0, 0), 0.0);
    assert!(c.values().is_empty());
}

#[test]
fn test_sub_matrix_shape_mismatch() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();
    let err = a.sub_matrix(&b).unwrap_err();
    assert!(matches!(
        err,
        SparseMatrixError::ShapeMismatch((2, 2), (2, 3))
    ));
}

#[test]
fn test_scalar_mult() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    // A = [[1.0, 0.0], [0.0, 2.0]]

    let c = a.scalar_mult(3.0);
    // C = 3 * A = [[3.0, 0.0], [0.0, 6.0]]

    assert_eq!(c.get_value_at(0, 0), 3.0);
    assert_eq!(c.get_value_at(0, 1), 0.0);
    assert_eq!(c.get_value_at(1, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 6.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_scalar_mult_by_zero() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let c = a.scalar_mult(0.0);
    assert_eq!(c.get_value_at(0, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 0.0);
    // Scalar multiplication by zero should not change sparsity structure,
    // although all actual values become zero. This is a design choice.
    assert_eq!(c.values(), &vec![0.0, 0.0]); // Values are now all zero
    assert_eq!(c.col_indices(), &vec![0, 1]);
    assert_eq!(c.row_indices(), &vec![0, 1, 2]);
}

#[test]
fn test_vec_mult_success() {
    let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    // A = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]

    let x = vec![1.0, 2.0, 3.0];
    let y = a.vec_mult(&x);
    // y = Ax = [(1.0*1.0 + 0.0*2.0 + 2.0*3.0), (0.0*1.0 + 3.0*2.0 + 0.0*3.0)] = [7.0, 6.0]

    assert_eq!(y, vec![7.0, 6.0]);
}

// #[test]
// #[should_panic]
// fn test_vec_mult_panic_dimension_mismatch() {
//     let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();
//     let x = vec![1.0, 2.0]; // Incorrect length
//     a.vec_mult(&x); // Should panic
// }

#[test]
fn test_mat_mult_success() {
    let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    // A (2x3) = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]

    let b = CsrMatrix::from_triplets(3, 2, &[(0, 0, 4.0), (1, 1, 5.0), (2, 0, 6.0)]).unwrap();
    // B (3x2) = [[4.0, 0.0], [0.0, 5.0], [6.0, 0.0]]

    let c = a.mat_mult(&b).unwrap();
    // C = A * B (2x2) = [[(1*4+0*0+2*6), (1*0+0*5+2*0)], [(0*4+3*0+0*6), (0*0+3*5+0*0)]]
    //                 = [[16.0, 0.0], [0.0, 15.0]]

    assert_eq!(c.get_value_at(0, 0), 16.0);
    assert_eq!(c.get_value_at(0, 1), 0.0);
    assert_eq!(c.get_value_at(1, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 15.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_mat_mult_dimension_mismatch() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap(); // 2x2
    let b = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1.0)]).unwrap(); // 3x2
    let err = a.mat_mult(&b).unwrap_err();
    assert!(matches!(err, SparseMatrixError::DimensionMismatch(2, 3)));
}

#[test]
fn test_transpose_success() {
    let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    // A (2x3) = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]

    let a_t = a.transpose();
    // A^T (3x2) = [[1.0, 0.0], [0.0, 3.0], [2.0, 0.0]]

    assert_eq!(a_t.shape(), (3, 2));
    assert_eq!(a_t.get_value_at(0, 0), 1.0);
    assert_eq!(a_t.get_value_at(0, 1), 0.0);
    assert_eq!(a_t.get_value_at(1, 0), 0.0);
    assert_eq!(a_t.get_value_at(1, 1), 3.0);
    assert_eq!(a_t.get_value_at(2, 0), 2.0);
    assert_eq!(a_t.get_value_at(2, 1), 0.0);

    // Also test original col indices become new row indices (transposed)
    assert_eq!(a_t.row_indices(), &vec![0, 1, 2, 3]);
    assert_eq!(a_t.col_indices(), &vec![0, 1, 0]);
    assert_eq!(a_t.values(), &vec![1.0, 3.0, 2.0]);
}

#[test]
fn test_transpose_empty_matrix() {
    let a: CsrMatrix<f64> = CsrMatrix::new();
    let a_t = a.transpose();
    assert_eq!(a_t.shape(), (0, 0));
    assert!(a_t.values().is_empty());
}
