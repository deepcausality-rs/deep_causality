# Sparse Matrix Specification for `deep_causality_sparse`

## 1. Introduction

This document specifies the design and proposed mathematical operations for the `CsrMatrix` (Compressed Sparse Row Matrix) data structure within the `deep_causality_sparse` crate. The goal is to provide an efficient and robust foundation for handling sparse matrices, which are common in scientific computing, machine learning, and graph theory. Sparse matrices are matrices in which most of the elements are zero. Storing only the non-zero elements can significantly save memory and computation time.

## 2. Core Data Structure: `CsrMatrix<T>`

The `CsrMatrix<T>` is designed to store sparse matrices using the Compressed Sparse Row (CSR) format. This format is particularly efficient for row-wise operations, such as matrix-vector multiplication.

### 2.1 Structure Definition

```rust
#[derive(Clone, Debug)]
pub struct CsrMatrix<T> {
    pub(crate) row_indices: Vec<usize>,
    pub(crate) col_indices: Vec<usize>,
    pub(crate) values: Vec<T>,
    pub(crate) shape: (usize, usize), // (Rows, Cols)
}
```

### 2.2 Fields

*   **`row_indices: Vec<usize>`**: An array of pointers (indices) into the `col_indices` and `values` arrays. `row_indices[i]` stores the index in `col_indices` and `values` where the `i`-th row starts. `row_indices[rows]` stores the total number of non-zero elements.
*   **`col_indices: Vec<usize>`**: Stores the column indices of the non-zero elements. The column indices for row `i` are found in `col_indices[row_indices[i]..row_indices[i+1]]`.
*   **`values: Vec<T>`**: Stores the actual non-zero values of the matrix. The values for row `i` are found in `values[row_indices[i]..row_indices[i+1]]`, corresponding to the column indices in `col_indices`.
*   **`shape: (usize, usize)`**: A tuple `(rows, cols)` representing the dimensions of the matrix.

### 2.3 Existing Constructors and Accessors

*   **`CsrMatrix::new()`**: Creates an empty `CsrMatrix`.
*   **`CsrMatrix::with_capacity(rows: usize, cols: usize, capacity: usize)`**: Creates a `CsrMatrix` with a specified shape and pre-allocated capacity for non-zero elements.
*   **`Default` implementation**: Provides a default empty matrix.
*   **`row_ptrs(&self) -> &Vec<usize>`**: Returns a reference to the `row_indices`.
*   **`col_indices(&self) -> &Vec<usize>`**: Returns a reference to the `col_indices`.
*   **`values(&self) -> &Vec<T>`**: Returns a reference to the `values`.
*   **`shape(&self) -> (usize, usize)`**: Returns the shape `(rows, cols)` of the matrix.

## 3. Proposed Mathematical Operations

This section outlines common mathematical operations for `CsrMatrix`, detailing their purpose, mathematical notation, constraints, and proposed Rust signatures. All operations return a `Result` type to handle potential errors such as dimension mismatches.

### 3.1 Error Handling

A new error type, `SparseMatrixError`, will be introduced to handle specific failure conditions:

```rust
#[derive(Debug, Error)]
pub enum SparseMatrixError {
    #[error("Shape mismatch: Cannot perform operation on matrices with different shapes. Left: {0:?}, Right: {1:?}")]
    ShapeMismatch((usize, usize), (usize, usize)),
    #[error("Dimension mismatch: Incompatible dimensions for matrix multiplication. Left columns: {0}, Right rows: {1}")]
    DimensionMismatch(usize, usize),
    #[error("Operation not yet implemented for sparse matrices.")]
    NotImplemented,
    #[error("Index out of bounds: Index {0} is out of bounds for dimension of size {1}.")]
    IndexOutOfBounds(usize, usize),
    #[error("Empty matrix: Operation not supported on empty matrix.")]
    EmptyMatrix,
}
```

### 3.2 Matrix-Matrix Addition

*   **Description**: Adds two matrices element-wise.
*   **Mathematical Notation**: `C = A + B`
*   **Input Constraints**: Both matrices `A` and `B` must have the same shape `(rows, cols)`.
*   **Output**: A new `CsrMatrix` `C` with the same shape as `A` and `B`.
*   **Rust Signature (Proposed)**:
    ```rust
    pub fn add_matrix(&self, other: &Self) -> Result<Self, SparseMatrixError>
    ```
*   **Complexity**:
    *   Time: O(rows + nnzA + nnzB), where nnzA and nnzB are the number of non-zero elements in matrices A and B respectively. This involves merging the sorted column indices for each row.
    *   Space: O(rows + nnzC) for the resulting matrix C.

### 3.3 Matrix-Matrix Subtraction

*   **Description**: Subtracts one matrix from another element-wise.
*   **Mathematical Notation**: `C = A - B`
*   **Input Constraints**: Both matrices `A` and `B` must have the same shape `(rows, cols)`.
*   **Output**: A new `CsrMatrix` `C` with the same shape as `A` and `B`.
*   **Rust Signature (Proposed)**:
    ```rust
    pub fn sub_matrix(&self, other: &Self) -> Result<Self, SparseMatrixError>
    ```
*   **Complexity**:
    *   Time: Similar to matrix addition, O(rows + nnzA + nnzB).
    *   Space: O(rows + nnzC) for the resulting matrix C.

### 3.4 Scalar Multiplication

*   **Description**: Multiplies every element of the matrix by a scalar value.
*   **Mathematical Notation**: `C = k * A`
*   **Input Constraints**: None.
*   **Output**: A new `CsrMatrix` `C` with the same shape as `A`.
*   **Rust Signature (Proposed)**:
    ```rust
    pub fn scalar_mult(&self, scalar: T) -> Self
    ```
*   **Complexity**:
    *   Time: O(nnzA), where nnzA is the number of non-zero elements.
    *   Space: O(rows + nnzA) for the new matrix.

### 3.5 Matrix Transpose

*   **Description**: Swaps the row and column indices of the matrix.
*   **Mathematical Notation**: `A^T`
*   **Input Constraints**: None.
*   **Output**: A new `CsrMatrix` `A^T` with shape `(cols, rows)`.
*   **Rust Signature (Proposed)**:
    ```rust
    pub fn transpose(&self) -> Self
    ```
*   **Complexity**:
    *   Time: O(rows + cols + nnzA). Requires iterating through the original matrix to count elements per column and then reconstructing the CSR structure.
    *   Space: O(cols + nnzA) for the new transposed matrix.

### 3.6 Matrix-Matrix Multiplication

*   **Description**: Performs the dot product of two matrices.
*   **Mathematical Notation**: `C = A * B`
*   **Input Constraints**: The number of columns in matrix `A` must equal the number of rows in matrix `B` (`A.cols == B.rows`).
*   **Output**: A new `CsrMatrix` `C` with shape `(A.rows, B.cols)`.
*   **Rust Signature (Proposed)**:
    ```rust
    pub fn mat_mult(&self, other: &Self) -> Result<Self, SparseMatrixError>
    ```
*   **Complexity**:
    *   This is a highly complex operation for CSR matrices. A common approach involves converting the second matrix (`B`) to CSC (Compressed Sparse Column) format or transposing `B` and then performing a dot product of rows from `A` with rows from `B^T`.
    *   Time: Can range from O(nnzA * B.cols) in naive implementations to O(nnzA * nnzB / rows) or more complex depending on sparsity patterns and algorithms.
    *   Space: O(A.rows + B.cols + nnzC) for the resulting matrix and intermediate structures.
*   **Note**: A robust and efficient implementation for CSR matrix multiplication is intricate and will require careful consideration of optimized algorithms (e.g., Gustavson's algorithm or variations). The initial implementation may use a simpler, less optimized approach or serve as a placeholder for future refinement.

## 4. Further functionlity 

### 4.1 Iterators

*   **Description**: Efficiently traverse the sparse matrix's elements. Iterators provide an idiomatic Rust way to process matrix data without exposing internal representation details directly, offering views into rows, columns, or non-zero entries.
*   **Proposed Iterators**:
    *   **`NonZeroIterator`**: Iterates over `(row_idx, col_idx, &value)` for all non-zero elements. This is the most natural and efficient iteration for a CSR matrix.
        *   **Rust Signature (Proposed)**:
            ```rust
            pub fn iter_non_zeros(&self) -> NonZeroIterator<'_, T>
            ```
        *   **Complexity**: O(nnz) time (where nnz is the number of non-zero elements) and O(1) additional space.
    *   **`RowIterator`**: Iterates over rows, yielding a structure or an iterator that provides access to `(col_idx, &value)` pairs within that specific row.
        *   **Rust Signature (Proposed)**:
            ```rust
            pub fn iter_rows(&self) -> RowIterator<'_, T>
            ```
        *   **Complexity**: O(rows + nnz) time and O(1) additional space. Each row is visited once, and its non-zero elements are yielded.
    *   **`ColIterator` (Conceptual, less efficient in CSR)**: Iterates over columns, yielding an iterator for `(row_idx, &value)` pairs within that column. Due to CSR's row-major storage, iterating efficiently over columns typically requires a preliminary scan or transposing the matrix. An efficient column iterator might involve constructing a temporary CSC (Compressed Sparse Column) representation or a dedicated algorithm.
        *   **Rust Signature (Proposed)**:
            ```rust
            // pub fn iter_cols(&self) -> ColIterator<'_, T> // More complex to implement efficiently
            ```
        *   **Complexity**: Without building an intermediate structure (like CSC), iterating over each column independently would be O(nnz) per column. Building a CSC for column iteration would be O(rows + cols + nnz) for setup, then O(nnz) for iteration.

### 4.2 Operator Overloading

*   **Description**: Enhance readability and ergonomics by allowing standard arithmetic operators (`+`, `-`, `*`) to be used directly with `CsrMatrix` instances. This will delegate to the explicitly named functions (e.g., `add_matrix`, `sub_matrix`, `scalar_mult`, `mat_mult`) already defined.
*   **Proposed Overloads**:
    *   **`Add` (`+`)**: For matrix-matrix addition.
        *   **Rust Signature (Proposed)**:
            ```rust
            impl<T> std::ops::Add for CsrMatrix<T> { /* ... */ }
            // Allows `matrix_a + matrix_b`
            ```
    *   **`Sub` (`-`)**: For matrix-matrix subtraction.
        *   **Rust Signature (Proposed)**:
            ```rust
            impl<T> std::ops::Sub for CsrMatrix<T> { /* ... */ }
            // Allows `matrix_a - matrix_b`
            ```
    *   **`Mul` (`*`)**:
        *   For matrix-matrix multiplication.
            *   **Rust Signature (Proposed)**:
                ```rust
                impl<T> std::ops::Mul for CsrMatrix<T> { /* ... */ }
                // Allows `matrix_a * matrix_b`
                ```
        *   For scalar-matrix multiplication (e.g., `scalar * matrix`). This often requires a blanket `impl` or a wrapper for the scalar type.
            *   **Rust Signature (Proposed)**:
                ```rust
                impl<T> std::ops::Mul<CsrMatrix<T>> for T { /* ... */ }
                // Allows `scalar * matrix`
                ```
        *   For matrix-scalar multiplication (e.g., `matrix * scalar`).
            *   **Rust Signature (Proposed)**:
                ```rust
                impl<T> std::ops::Mul<T> for CsrMatrix<T> { /* ... */ }
                // Allows `matrix * scalar`
                ```
*   **Error Handling**: Overloaded operators should preferably return `Result` types (e.g., `Result<Self, SparseMatrixError>`) or panic if the operation is invalid (e.g., shape mismatch), consistent with Rust's common practices for `std::ops` implementations in numerical libraries where panicking on invalid input is sometimes accepted for brevity if the operation is fundamentally unsound.

### 4.3 Build from Triplets

*   **Description**: A convenient constructor to create a `CsrMatrix` from a list of `(row_index, col_index, value)` tuples, often referred to as "triplets" or Coordinate (COO) format. This is a common and flexible input format for sparse matrix data.
*   **Input**: A vector of `(usize, usize, T)` tuples, representing `(row_index, col_index, value)`, and the `(rows, cols)` shape of the resulting matrix.
*   **Process**:
    1.  **Validation**: Ensure all triplet `row_index` and `col_index` values are within the specified `rows` and `cols` bounds. Return a `SparseMatrixError::IndexOutOfBounds` if any triplet is invalid.
    2.  **Sorting**: Sort the triplets primarily by row index, then secondarily by column index. This is crucial for correctly building the `row_indices`, `col_indices`, and `values` arrays in CSR format.
    3.  **Duplicate Handling**: If multiple triplets specify the same `(row, col)` entry, their values should be summed.
    4.  **CSR Construction**:
        *   Initialize `row_indices` to all zeros, with `rows + 1` elements.
        *   Iterate through the sorted and aggregated triplets to populate `col_indices` and `values`.
        *   Increment `row_indices[row_idx + 1]` for each non-zero element encountered in `row_idx`.
        *   Perform a cumulative sum on `row_indices` to get the final pointers.
*   **Rust Signature (Proposed)**:
    ```rust
    pub fn from_triplets(
        rows: usize,
        cols: usize,
        triplets: &[(usize, usize, T)]
    ) -> Result<Self, SparseMatrixError>
    ```
*   **Complexity**:
    *   Time: Dominated by sorting the triplets, O(N log N) where N is the number of triplets.
    *   Space: O(N) for storing the triplets and intermediate structures during construction.

### Other Future Considerations

*   **Element Access/Modification**: Methods for getting and setting individual elements, although these are typically less efficient in CSR format for random access.
*   **Conversion to/from dense matrices**: Utility functions for converting between `CsrMatrix` and dense matrix representations.
*   **BLAS/LAPACK Interoperability**: Consider integration with optimized linear algebra libraries for higher performance.
