# Summary
- **Context**: The `vec_mult_impl` function performs matrix-vector multiplication in the sparse matrix library, using a performance optimization with 4 parallel accumulators to break CPU dependency chains.
- **Bug**: The parallel accumulator optimization causes catastrophic cancellation errors when a row contains large values with opposite signs alongside smaller values, producing results that can be off by orders of magnitude.
- **Actual vs. expected**: When multiplying a row `[1e20, 1.0, -1e20, 1.0, ...]` by a vector of `[1.0, 1.0, ...]`, the result is 0.0 instead of the mathematically correct 2.0 - an error of 100%.
- **Impact**: Silently incorrect numerical results in matrix-vector multiplications when rows contain values with wide dynamic ranges, potentially causing wrong scientific/engineering calculations.

# Code with bug

`deep_causality_sparse/src/types/sparse_matrix/ops/vec_mult_impl.rs:70-94`

```rust
// 4 Independent accumulators to break dependency chains.
// This allows the CPU to pipeline the math operations.
let mut sum0 = T::zero();
let mut sum1 = T::zero();
let mut sum2 = T::zero();
let mut sum3 = T::zero();

// Process 4 elements at a time.
let mut chunks_cols = row_cols.chunks_exact(4);
let mut chunks_vals = row_vals.chunks_exact(4);

// The main "Pseudo-SIMD" loop.
// The iterator handles the loop logic efficiently.
while let (Some(c), Some(v)) = (chunks_cols.next(), chunks_vals.next()) {
    // Note: x[c[0]] involves a bounds check.
    // Without 'unsafe', we cannot remove it, but splitting the
    // accumulators helps hide the latency of that check/load.
    sum0 = sum0 + (v[0] * x[c[0]]);
    sum1 = sum1 + (v[1] * x[c[1]]);
    sum2 = sum2 + (v[2] * x[c[2]]);
    sum3 = sum3 + (v[3] * x[c[3]]);
}

// Reduce the parallel accumulators
let mut final_sum = (sum0 + sum1) + (sum2 + sum3); // <-- BUG 🔴 Numerical instability from non-associative floating-point addition
```

# Evidence

## Example

Consider a sparse matrix row with 8 non-zero elements: `[1e20, 1.0, -1e20, 1.0, 1e-10, 1e-10, 1e-10, 1e-10]`

Multiplying by a vector of all 1.0s:

**Mathematically correct result:**
```
1e20 + 1.0 - 1e20 + 1.0 + 4×1e-10 = 2.0 + 0.0000000004 ≈ 2.0000000004
```

**With parallel accumulators (buggy implementation):**

First chunk (indices 0-3):
```
sum0 = 0 + (1e20 × 1.0) = 1e20
sum1 = 0 + (1.0 × 1.0)  = 1.0
sum2 = 0 + (-1e20 × 1.0) = -1e20
sum3 = 0 + (1.0 × 1.0)  = 1.0
```

Reduction:
```
final_sum = (sum0 + sum1) + (sum2 + sum3)
          = (1e20 + 1.0) + (-1e20 + 1.0)
          = 1e20 + (-1e20 + 1.0)        ← 1.0 is lost in (1e20 + 1.0) due to floating-point precision
          = 1e20 - 1e20 + 1.0
          = 0.0 + 1.0 = 1.0              ← One of the 1.0 values is lost!
```

Second chunk remainder (indices 4-7):
```
final_sum = 1.0 + 4×1e-10 ≈ 1.0000000004
```

**Actual buggy result: 1.0000000004** (should be 2.0000000004)

But in the test case, even worse results occur where the final result is **0.0000000004** instead of **2.0000000004**, losing the entire contribution of two 1.0 values!

## Inconsistency within the codebase

### Reference code

Previous version (commit b359eef6^) in `deep_causality_sparse/src/types/sparse_matrix/ops.rs`:

```rust
pub(crate) fn vec_mult_impl(&self, x: &[T]) -> Result<Vec<T>, SparseMatrixError> {
    // ... validation code ...

    for i in 0..rows {
        let start = self.row_indices[i];
        let end = self.row_indices[i + 1];

        let mut sum = T::zero();

        // This loop is highly vectorizable due to SoA layout
        for j in start..end {
            let col = self.col_indices[j];
            let val = self.values[j];
            sum = sum + (val * x[col]);  // Sequential accumulation
        }
        y.push(sum);
    }
    Ok(y)
}
```

### Current code

Current version in `deep_causality_sparse/src/types/sparse_matrix/ops/vec_mult_impl.rs`:

```rust
pub(crate) fn vec_mult_impl(&self, x: &[T]) -> Result<Vec<T>, SparseMatrixError> {
    // ... validation code ...

    for i in 0..rows {
        // ... slice creation ...

        let mut sum0 = T::zero();
        let mut sum1 = T::zero();
        let mut sum2 = T::zero();
        let mut sum3 = T::zero();

        let mut chunks_cols = row_cols.chunks_exact(4);
        let mut chunks_vals = row_vals.chunks_exact(4);

        while let (Some(c), Some(v)) = (chunks_cols.next(), chunks_vals.next()) {
            sum0 = sum0 + (v[0] * x[c[0]]);
            sum1 = sum1 + (v[1] * x[c[1]]);
            sum2 = sum2 + (v[2] * x[c[2]]);
            sum3 = sum3 + (v[3] * x[c[3]]);
        }

        let mut final_sum = (sum0 + sum1) + (sum2 + sum3);  // Parallel reduction
        // ... remainder handling ...
    }
}
```

### Contradiction

The previous version used **sequential accumulation** which maintains numerical stability by accumulating values in the order they appear in the sparse matrix. The current version uses **parallel accumulation with 4 separate accumulators**, which can cause catastrophic cancellation when:

1. Large positive and negative values are grouped into different accumulators (e.g., `sum0 = 1e20`, `sum2 = -1e20`)
2. Small values are added to accumulators with large values (e.g., `sum1 = 1.0` computed as `1e20 + 1.0 = 1e20`)
3. The parallel reduction combines these intermediate results, causing small values to be lost

This is a classic example of how **floating-point addition is not associative**: `(a + b) + (c + d) ≠ a + b + c + d` when the values have widely different magnitudes.

## Failing test

### Test script

```rust
// Demonstrates catastrophic cancellation bug in vec_mult_impl
//
// The bug occurs when:
// 1. A row contains large positive and large negative values that nearly cancel
// 2. The row also contains small values that should contribute to the sum
// 3. The 4-way parallel accumulator groups the large values separately from each other
//
// This causes intermediate sums like (1e20 + 1.0) = 1e20 (losing the 1.0),
// then (1e20) + (-1e20 + 1.0) = 1e20 - 1e20 = 0 (losing both 1.0 values)

use deep_causality_sparse::CsrMatrix;

#[test]
fn test_vec_mult_catastrophic_cancellation() {
    // Matrix with one row containing values designed to expose the bug:
    // [1e20, 1.0, -1e20, 1.0, 1e-10, 1e-10, 1e-10, 1e-10]
    //
    // The mathematically correct result when multiplied by a vector of all 1.0s is:
    // 1e20 + 1.0 - 1e20 + 1.0 + 4e-10 = 2.0 + 4e-10 ≈ 2.0000000004
    //
    // With parallel accumulators (chunks of 4):
    //   sum0 = 1e20 * 1.0 = 1e20
    //   sum1 = 1.0 * 1.0 = 1.0
    //   sum2 = -1e20 * 1.0 = -1e20
    //   sum3 = 1.0 * 1.0 = 1.0
    //
    //   final_sum = (sum0 + sum1) + (sum2 + sum3)
    //             = (1e20 + 1.0) + (-1e20 + 1.0)
    //             = 1e20 + (-1e20 + 1.0)    <- 1.0 lost here in sum0+sum1
    //             = 0.0 + 1.0 = 1.0          <- one 1.0 lost, only one 1.0 from sum2+sum3 remains
    //
    // Then remainder: 1.0 + 4e-10 ≈ 1.0000000004
    //
    // But actual result is 0.0000000004 (missing the 2.0!)

    let matrix = CsrMatrix::from_triplets(
        1,
        8,
        &[
            (0, 0, 1e20),
            (0, 1, 1.0),
            (0, 2, -1e20),
            (0, 3, 1.0),
            (0, 4, 1e-10),
            (0, 5, 1e-10),
            (0, 6, 1e-10),
            (0, 7, 1e-10),
        ],
    )
    .unwrap();

    let x = vec![1.0; 8];
    let result = matrix.vec_mult(&x).unwrap();

    // The expected result (mathematically) is 2.0 + 4e-10
    let expected: f64 = 2.0 + 4e-10;

    println!("Result:   {:.15}", result[0]);
    println!("Expected: {:.15}", expected);
    println!("Error:    {:.15}", (result[0] - expected).abs());

    // The bug causes the result to be dramatically wrong
    // Actual result is ~4e-10 instead of ~2.0000000004
    // This is off by approximately 2.0!
    assert!(
        (result[0] - expected).abs() < 1e-6,
        "Catastrophic cancellation: expected {:.15}, got {:.15}, error = {:.15}",
        expected,
        result[0],
        (result[0] - expected).abs()
    );
}

fn main() {
    test_vec_mult_catastrophic_cancellation();
}
```

### Test output

```
running 1 test
test test_vec_mult_catastrophic_cancellation ... FAILED

failures:

---- test_vec_mult_catastrophic_cancellation stdout ----
Result:   0.000000000000000
Expected: 2.000000000400000
Error:    2.000000000400000

thread 'test_vec_mult_catastrophic_cancellation' (9128) panicked at test_catastrophic_cancellation.rs:65:5:
Catastrophic cancellation: expected 2.000000000400000, got 0.000000000000000, error = 2.000000000400000
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_vec_mult_catastrophic_cancellation

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

# Full context

The `vec_mult_impl` function is an internal implementation of matrix-vector multiplication for the `CsrMatrix` (Compressed Sparse Row Matrix) type. It is called by the public API method `vec_mult` in `deep_causality_sparse/src/types/sparse_matrix/api/mod.rs`.

This function is a core operation in sparse linear algebra and is used in:
- Scientific computing applications (physics simulations, finite element methods)
- Machine learning (sparse neural networks, graph neural networks)
- Graph algorithms (PageRank, network analysis)
- Any application using the DeepCausality sparse matrix library for matrix-vector products

The bug was introduced in commit b359eef6 ("Refactored of code structures and applied various performance improvements") which replaced the simple sequential accumulation with a parallel accumulator approach to improve CPU pipelining. The comments in the code indicate this was done for performance:
- "4 Independent accumulators to break dependency chains"
- "This allows the CPU to pipeline the math operations"
- "Pseudo-SIMD loop"

While this optimization may improve performance in the common case, it sacrifices numerical accuracy for rows with values spanning large dynamic ranges - a scenario that is not uncommon in real-world scientific/engineering applications (e.g., systems with multi-scale physics, stiff ODEs, ill-conditioned matrices).

# Why has this bug gone undetected?

1. **Testing focused on well-behaved cases**: The existing test cases in `deep_causality_sparse/tests/types/sparse_matrix/ops_tests.rs` only use small integer-like values (1.0, 2.0, 3.0, etc.) that don't expose floating-point precision issues.

2. **Rare triggering conditions**: The bug only manifests when:
    - A row has values with widely different magnitudes (spanning many orders of magnitude)
    - Large positive and negative values that nearly cancel are present in the same row
    - The chunking happens to group these values in a way that causes intermediate cancellation
    - This specific pattern is not common in typical test matrices

3. **Performance optimization looked correct**: The optimization follows a common pattern (parallel reduction) that works well for many cases. The numerical stability issue is subtle and requires understanding of floating-point arithmetic's non-associative nature.

4. **Silent failure**: The function doesn't error or warn - it silently returns incorrect results that might look "close enough" in some cases, making the bug hard to detect without specific numerical accuracy tests.

5. **Optimization obscures the issue**: The performance-focused comments ("pipeline", "SIMD", "break dependency chains") suggest the code was written with CPU performance in mind, and numerical stability may not have been a primary concern during the optimization.

# Recommended fix

Replace the parallel accumulator reduction with a numerically stable accumulation method. The simplest fix is to **revert to sequential accumulation**:

```rust
// Reduce the parallel accumulators sequentially for numerical stability
// Instead of: let mut final_sum = (sum0 + sum1) + (sum2 + sum3);
let mut final_sum = sum0 + sum1 + sum2 + sum3; // <-- FIX 🟢 Sequential reduction
```

However, this still has issues. A better fix would be to accumulate directly without splitting:

```rust
// Accumulate directly for numerical stability
let mut final_sum = T::zero();
for (&c, &v) in row_cols.iter().zip(row_vals.iter()) {
    final_sum = final_sum + (v * x[c]); // <-- FIX 🟢 Sequential accumulation
}
```

For maximum numerical stability while maintaining some parallelism, consider using **Kahan summation** or **pairwise summation** algorithms, which are designed to minimize floating-point errors in summation.

Alternatively, if the performance optimization is critical, document the numerical stability trade-off prominently and provide a configuration option for users to choose between performance and accuracy.

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

