use deep_causality_sparse::CsrMatrix;
use deep_causality_sparse::SparseMatrixError;

#[test]
fn test_new_matrix_is_empty() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(matrix.shape(), (0, 0));
    assert!(matrix.row_indices().is_empty());
    assert!(matrix.col_indices().is_empty());
    assert!(matrix.values().is_empty());
}

#[test]
fn test_with_capacity_initializes_correctly() {
    let rows = 5;
    let cols = 10;
    let capacity = 20;
    let matrix: CsrMatrix<f64> = CsrMatrix::with_capacity(rows, cols, capacity);
    assert_eq!(matrix.shape(), (rows, cols));
    assert_eq!(matrix.row_indices().len(), rows + 1);
    assert_eq!(matrix.row_indices(), &vec![0; rows + 1]); // All zeros initially
    assert!(matrix.col_indices().capacity() >= capacity);
    assert!(matrix.values().capacity() >= capacity);
    assert!(matrix.col_indices().is_empty());
    assert!(matrix.values().is_empty());
}

#[test]
fn test_from_triplets_basic_construction() {
    let triplets = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();
    // Expected:
    // [1.0, 0.0, 2.0]
    // [0.0, 3.0, 0.0]
    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]); // Sorted by row, then col
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]); // First row has 2 elements (0 and 2), second has 1 (1)
}

#[test]
fn test_from_triplets_with_duplicate_entries_summed() {
    let triplets = vec![(0, 0, 1.0), (0, 0, 0.5), (1, 1, 3.0), (0, 1, 1.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    // Expected:
    // [1.5, 1.0]
    // [0.0, 3.0]
    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![1.5, 1.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 1, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}

#[test]
fn test_from_triplets_with_zero_valued_entries_after_summation() {
    let triplets = vec![(0, 0, 1.0), (0, 0, -1.0), (1, 1, 3.0), (0, 1, 0.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    // Expected:
    // [0.0, 0.0]
    // [0.0, 3.0]
    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![3.0]);
    assert_eq!(matrix.col_indices(), &vec![1]);
    // First row has no non-zeros, so row_ptrs[0] and row_ptrs[1] should be the same
    assert_eq!(matrix.row_indices(), &vec![0, 0, 1]);
}

#[test]
fn test_from_triplets_empty_triplets() {
    let triplets: Vec<(usize, usize, f64)> = Vec::new();
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    assert_eq!(matrix.shape(), (2, 2));
    assert!(matrix.values().is_empty());
    assert!(matrix.col_indices().is_empty());
    assert_eq!(matrix.row_indices(), &vec![0, 0, 0]);
}

#[test]
fn test_from_triplets_index_out_of_bounds_row() {
    let triplets = vec![(0, 0, 1.0), (2, 1, 2.0)]; // Row 2 is out of bounds for 2x2
    let err = CsrMatrix::from_triplets(2, 2, &triplets).unwrap_err();
    assert!(matches!(err, SparseMatrixError::IndexOutOfBounds(2, 2)));
}

#[test]
fn test_from_triplets_index_out_of_bounds_col() {
    let triplets = vec![(0, 0, 1.0), (1, 2, 2.0)]; // Col 2 is out of bounds for 2x2
    let err = CsrMatrix::from_triplets(2, 2, &triplets).unwrap_err();
    assert!(matches!(err, SparseMatrixError::IndexOutOfBounds(2, 2)));
}

#[test]
fn test_from_triplets_only_zero_values() {
    let triplets = vec![(0, 0, 0.0), (1, 1, 0.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    assert_eq!(matrix.shape(), (2, 2));
    assert!(matrix.values().is_empty());
    assert!(matrix.col_indices().is_empty());
    assert_eq!(matrix.row_indices(), &vec![0, 0, 0]);
}

#[test]
fn test_from_triplets_mixed_valid_and_zero_values() {
    let triplets = vec![(0, 0, 1.0), (0, 1, 0.0), (1, 1, 2.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![1.0, 2.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 1, 2]);
}

#[test]
fn test_from_triplets_unordered_input() {
    let triplets = vec![(1, 1, 3.0), (0, 0, 1.0), (0, 2, 2.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();
    // Should be sorted internally to: (0,0,1.0), (0,2,2.0), (1,1,3.0)
    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}
