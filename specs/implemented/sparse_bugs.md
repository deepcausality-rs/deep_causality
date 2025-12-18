# Summary
- **Context**: The `from_triplets` and `from_triplets_with_zero` functions construct sparse matrices from triplet lists and validate that row and column indices are within bounds.
- **Bug**: When indices are out of bounds, the error message reports `max(row_idx, col_idx)` as the index and `max(rows, cols)` as the size, which produces misleading error information.
- **Actual vs. expected**: The error should report which specific dimension (row or column) is out of bounds with its corresponding size, but instead it conflates both dimensions using `max()`.
- **Impact**: Developers receive confusing error messages that make it extremely difficult to debug index errors, especially with non-square matrices.

# Code with bug

In `deep_causality_sparse/src/types/sparse_matrix/mod.rs`:

```rust
// Line 162-167 in from_triplets
for &(r, c, v) in triplets.iter() {
    if r >= rows || c >= cols {
        return Err(SparseMatrixError::IndexOutOfBounds(
            r.max(c),      // <-- BUG 🔴 Uses max of row and col indices
            rows.max(cols), // <-- BUG 🔴 Uses max of matrix dimensions
        ));
    }
```

```rust
// Line 254-259 in from_triplets_with_zero (same bug)
for &(r, c, v) in triplets.iter() {
    if r >= rows || c >= cols {
        return Err(SparseMatrixError::IndexOutOfBounds(
            r.max(c),      // <-- BUG 🔴 Same incorrect pattern
            rows.max(cols), // <-- BUG 🔴 Same incorrect pattern
        ));
    }
```

# Evidence

## Example

**Scenario 1: Row out of bounds in a 3×10 matrix**
- Matrix dimensions: 3 rows × 10 columns
- Triplet: `(row=5, col=1, value=1.0)`
- Row 5 >= 3 rows (out of bounds)
- **Expected error**: `IndexOutOfBounds(5, 3)` indicating row 5 is invalid for 3 rows
- **Actual error**: `IndexOutOfBounds(5, 10)` where 10 = max(3 rows, 10 cols)
- **Problem**: The size is 10 instead of 3, making it unclear whether this is a row or column issue

**Scenario 2: Column out of bounds in a 10×3 matrix**
- Matrix dimensions: 10 rows × 3 columns
- Triplet: `(row=1, col=5, value=1.0)`
- Column 5 >= 3 cols (out of bounds)
- **Expected error**: `IndexOutOfBounds(5, 3)` indicating col 5 is invalid for 3 cols
- **Actual error**: `IndexOutOfBounds(5, 10)` where 10 = max(10 rows, 3 cols)
- **Problem**: The size is 10 instead of 3, completely wrong for a column error

**Scenario 3: Extremely misleading case (5×100 matrix)**
- Matrix dimensions: 5 rows × 100 columns
- Triplet: `(row=6, col=50, value=1.0)`
- Row 6 >= 5 rows (out of bounds)
- **Expected error**: `IndexOutOfBounds(6, 5)` indicating row 6 is invalid for 5 rows
- **Actual error**: `IndexOutOfBounds(50, 100)` where 50 = max(6, 50) and 100 = max(5, 100)
- **Problem**: The error reports index 50 (the column) when row 6 is the actual problem! A developer would think there are 100 rows and look for an issue with column 50, when the real problem is row 6 in a 5-row matrix.

**Scenario 4: Both indices out of bounds (3×10 matrix)**
- Matrix dimensions: 3 rows × 10 columns
- Triplet: `(row=5, col=15, value=1.0)`
- Row 5 >= 3 rows AND col 15 >= 10 cols (both out of bounds)
- **Expected error**: Either `IndexOutOfBounds(5, 3)` or `IndexOutOfBounds(15, 10)`
- **Actual error**: `IndexOutOfBounds(15, 10)` where 15 = max(5, 15) and 10 = max(3, 10)
- **Problem**: Reports the column violation but doesn't mention the row violation at all

## Failing test

### Test script
```rust
use deep_causality_sparse::{CsrMatrix, SparseMatrixError};

fn main() {
    println!("=== Test 1: Row out of bounds in non-square matrix ===");
    // 3x10 matrix: 3 rows, 10 columns
    // Triplet has row=5 (>= 3), col=1 (< 10)
    // Should report: IndexOutOfBounds(5, 3) for row dimension
    // Actually reports: IndexOutOfBounds(5, 10) - WRONG SIZE
    let result1 = CsrMatrix::from_triplets(3, 10, &[(5, 1, 1.0)]);
    match result1 {
        Err(SparseMatrixError::IndexOutOfBounds(idx, size)) => {
            println!("Actual error: IndexOutOfBounds({}, {})", idx, size);
            println!("Expected: IndexOutOfBounds(5, 3)");
            assert_eq!(idx, 5, "Index should be 5");
            assert_eq!(size, 3, "Size should be 3 (number of rows), but got {}", size);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }

    println!("\n=== Test 2: Col out of bounds in non-square matrix ===");
    // 10x3 matrix: 10 rows, 3 columns
    // Triplet has row=1 (< 10), col=5 (>= 3)
    // Should report: IndexOutOfBounds(5, 3) for column dimension
    // Actually reports: IndexOutOfBounds(5, 10) - WRONG SIZE
    let result2 = CsrMatrix::from_triplets(10, 3, &[(1, 5, 1.0)]);
    match result2 {
        Err(SparseMatrixError::IndexOutOfBounds(idx, size)) => {
            println!("Actual error: IndexOutOfBounds({}, {})", idx, size);
            println!("Expected: IndexOutOfBounds(5, 3)");
            assert_eq!(idx, 5, "Index should be 5");
            assert_eq!(size, 3, "Size should be 3 (number of columns), but got {}", size);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }

    println!("\n=== Test 3: Extremely misleading case ===");
    // 5x100 matrix: 5 rows, 100 columns
    // Triplet has row=6 (>= 5), col=50 (< 100)
    // Should report: IndexOutOfBounds(6, 5) for row dimension
    // Actually reports: IndexOutOfBounds(50, 100) - COMPLETELY WRONG
    let result3 = CsrMatrix::from_triplets(5, 100, &[(6, 50, 1.0)]);
    match result3 {
        Err(SparseMatrixError::IndexOutOfBounds(idx, size)) => {
            println!("Actual error: IndexOutOfBounds({}, {})", idx, size);
            println!("Expected: IndexOutOfBounds(6, 5)");
            println!("BUG: Reports column 50 when row 6 is the problem!");
            assert_eq!(idx, 6, "Index should be 6 (the row), but got {}", idx);
            assert_eq!(size, 5, "Size should be 5 (number of rows), but got {}", size);
        }
        _ => panic!("Expected IndexOutOfBounds error"),
    }
}
```

### Test output
```
=== Test 1: Row out of bounds in non-square matrix ===
Actual error: IndexOutOfBounds(5, 10)
Expected: IndexOutOfBounds(5, 3)
thread 'main' panicked at test_index_bug.rs:14:13:
Size should be 3 (number of rows), but got 10
note: run with `RUST_BACKTRACE=1` for a backtrace

=== Test 2: Col out of bounds in non-square matrix ===
Actual error: IndexOutOfBounds(5, 10)
Expected: IndexOutOfBounds(5, 3)
thread 'main' panicked at test_index_bug.rs:28:13:
Size should be 3 (number of columns), but got 10

=== Test 3: Extremely misleading case ===
Actual error: IndexOutOfBounds(50, 100)
Expected: IndexOutOfBounds(6, 5)
BUG: Reports column 50 when row 6 is the problem!
thread 'main' panicked at test_index_bug.rs:42:13:
Index should be 6 (the row), but got 50
```

# Full context

The `CsrMatrix` type represents a Compressed Sparse Row matrix, which is a core data structure in the `deep_causality_sparse` crate. The `from_triplets` method is one of the primary ways users construct sparse matrices by providing a list of `(row, col, value)` triplets.

This function is called extensively throughout the codebase:
- In test suites across `deep_causality_sparse` and `deep_causality_topology`
- In examples demonstrating matrix operations
- In production code for physics simulations and topology computations
- By users of the library through the public API

The function validates that all row and column indices are within the specified matrix dimensions. When validation fails, it should provide clear error messages to help users identify which index is out of bounds. However, the current implementation conflates the row and column dimensions by using `max()`, which:

1. Makes it impossible to determine if a row or column is out of bounds
2. Reports incorrect dimension sizes for non-square matrices
3. Can report the wrong index entirely (e.g., reporting col 50 when row 6 is the problem)

The error information flows through the `SparseMatrixError::IndexOutOfBounds` variant, which is documented in `deep_causality_sparse/src/errors/sparse_matrix_error.rs`. This error type is meant to provide clear debugging information to users, with the format: "Index {index} is out of bounds for dimension of size {size}."

# Why has this bug gone undetected?

The bug has gone undetected because:

1. **All existing tests use square matrices**: The test suite in `deep_causality_sparse/tests/types/sparse_matrix/from_triplets_tests.rs` only tests with square matrices (2×2, 3×3, etc.). For square matrices, `max(rows, cols)` equals both `rows` and `cols`, so the error message happens to be correct by coincidence.

2. **Validation tests use `matches!` pattern**: Tests like `test_from_triplets_index_out_of_bounds_row` and `test_from_triplets_index_out_of_bounds_col` use:
   ```rust
   assert!(matches!(err, SparseMatrixError::IndexOutOfBounds(2, 2)));
   ```
   This only checks that the error variant matches, but with square matrices (2×2), the values happen to be "correct enough" to pass even with the bug.

3. **Real-world usage likely uses square matrices**: Most examples and production code appear to use square matrices or identity matrices, where the bug doesn't manifest.

4. **Error messages may not be carefully inspected**: When errors occur, users might focus on fixing their input rather than carefully examining whether the error message is precisely accurate, especially if the index value is close to correct.

# Recommended fix

The correct implementation should check each dimension separately and report the specific violation:

```rust
for &(r, c, v) in triplets.iter() {
    if r >= rows {
        return Err(SparseMatrixError::IndexOutOfBounds(r, rows)); // <-- FIX 🟢
    }
    if c >= cols {
        return Err(SparseMatrixError::IndexOutOfBounds(c, cols)); // <-- FIX 🟢
    }
    if v != T::zero() {
        processed_triplets.push((r, c, v));
    }
}
```

This fix should be applied to both:
1. `from_triplets` (around line 162-167)
2. `from_triplets_with_zero` (around line 254-259)

Alternative fix if reporting both violations is preferred:
- Introduce a new error variant like `IndexOutOfBounds { row: Option<(usize, usize)>, col: Option<(usize, usize)> }` to report both row and column violations when both are out of bounds
- This would require more extensive refactoring but would provide the most complete error information
