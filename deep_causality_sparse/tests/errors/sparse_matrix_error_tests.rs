use deep_causality_sparse::SparseMatrixError;
use std::error::Error;

#[test]
fn test_shape_mismatch_error_display() {
    let error = SparseMatrixError::ShapeMismatch((2, 3), (3, 2));
    assert_eq!(
        format!("{}", error),
        "Shape mismatch: Cannot perform operation on matrices with different shapes. Left: (2, 3), Right: (3, 2)"
    );
}

#[test]
fn test_dimension_mismatch_error_display() {
    let error = SparseMatrixError::DimensionMismatch(3, 2);
    assert_eq!(
        format!("{}", error),
        "Dimension mismatch: Incompatible dimensions for matrix multiplication. Left columns: 3, Right rows: 2"
    );
}

#[test]
fn test_index_out_of_bounds_error_display() {
    let error = SparseMatrixError::IndexOutOfBounds(5, 3);
    assert_eq!(
        format!("{}", error),
        "Index out of bounds: Index 5 is out of bounds for dimension of size 3."
    );
}

#[test]
fn test_empty_matrix_error_display() {
    let error = SparseMatrixError::EmptyMatrix;
    assert_eq!(
        format!("{}", error),
        "Empty matrix: Operation not supported on empty matrix."
    );
}

#[test]
fn test_sparse_matrix_error_is_error_trait() {
    let error = SparseMatrixError::EmptyMatrix;
    // Check if it implements the Error trait
    assert!(error.source().is_none()); // If it compiles and runs, it implements Error
}

#[test]
fn test_sparse_matrix_error_debug_trait() {
    let error = SparseMatrixError::ShapeMismatch((1, 1), (2, 2));
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ShapeMismatch"));
    assert!(debug_str.contains("(1, 1)"));
    assert!(debug_str.contains("(2, 2)"));
}

#[test]
fn test_sparse_matrix_error_clone_trait() {
    let error = SparseMatrixError::DimensionMismatch(10, 5);
    let cloned_error = error.clone();
    assert_eq!(error, cloned_error);
}

#[test]
fn test_sparse_matrix_error_partial_eq_trait() {
    let error1 = SparseMatrixError::IndexOutOfBounds(0, 10);
    let error2 = SparseMatrixError::IndexOutOfBounds(0, 10);
    let error3 = SparseMatrixError::IndexOutOfBounds(1, 10);
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
}

#[test]
fn test_sparse_matrix_error_eq_trait() {
    let error1 = SparseMatrixError::EmptyMatrix;
    let error2 = SparseMatrixError::EmptyMatrix;
    assert!(error1 == error2);
}
