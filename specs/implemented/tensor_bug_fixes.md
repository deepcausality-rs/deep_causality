# Summary
- **Context**: The scalar arithmetic operations in `BackendTensor` are used to perform element-wise operations between tensors and scalar values (e.g., `tensor + 5.0`), which is a fundamental operation in tensor computation libraries.
- **Bug**: Scalar arithmetic operations on non-contiguous tensors (e.g., transposed or sliced tensors) produce incorrect results because they ignore the tensor's stride information.
- **Actual vs. expected**: When adding a scalar to a transposed tensor, the operation applies the scalar to elements in storage order rather than logical order, resulting in a tensor with values in the wrong positions.
- **Impact**: Any tensor that has been transposed, sliced, or otherwise made non-contiguous will produce mathematically incorrect results when combined with scalar operations, potentially corrupting scientific computations, machine learning models, or physics simulations.

# Code with bug

The bug is in the `impl_scalar_arithmetic!` macro (lines 316-449):

```rust
impl_scalar_arithmetic!(f64, f32, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
```

Specifically in implementations like:

```rust
// Line 333-339 in ops.rs
impl<B: TensorBackend> Add<$t> for &BackendTensor<$t, B> {
    type Output = BackendTensor<$t, B>;
    fn add(self, rhs: $t) -> Self::Output {
        let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
        // <-- BUG 🔴 B::to_vec() returns data in storage order, ignoring strides
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}
```

The problem is on line 336 where `B::to_vec(&self.inner)` is called. Looking at the CPU backend implementation:

```rust
// cpu_backend_tensor.rs:69-71
fn to_vec<T: Clone>(tensor: &Self::Tensor<T>) -> Vec<T> {
    tensor.data.clone()  // <-- Returns data in storage order, not logical order
}
```

This directly clones the underlying storage vector, which for non-contiguous tensors (transposed, sliced, etc.) does not match the logical element order defined by the tensor's shape and strides.

# Evidence

## Example

Consider a 2×3 tensor that gets transposed:

**Original tensor (2×3):**
```
[[1, 2, 3],
 [4, 5, 6]]
```
- Storage: `[1, 2, 3, 4, 5, 6]`
- Shape: `[2, 3]`
- Strides: `[3, 1]` (row-major)

**After transpose to 3×2:**
```
[[1, 4],
 [2, 5],
 [3, 6]]
```
- Storage: `[1, 2, 3, 4, 5, 6]` (unchanged)
- Shape: `[3, 2]`
- Strides: `[1, 3]` (now elements are accessed in different order)

**When adding scalar 10.0:**
- **Expected result:** Add 10 to each element in logical order
  ```
  [[11, 14],
   [12, 15],
   [13, 16]]
  ```

- **Actual buggy result:** The code does:
    1. Gets storage: `[1, 2, 3, 4, 5, 6]`
    2. Adds 10: `[11, 12, 13, 14, 15, 16]`
    3. Creates new tensor with shape `[3, 2]` and standard strides `[2, 1]`
  ```
  [[11, 12],    <-- WRONG!
   [13, 14],    <-- WRONG!
   [15, 16]]    <-- WRONG!
  ```

The issue is that the transposed tensor has strides `[1, 3]`, meaning `tensor[i, j]` maps to `storage[i*1 + j*3]`. But after the scalar operation, the result has standard strides `[2, 1]`, and the storage order doesn't match the expected logical values.

## Failing test

### Test script

```rust
// Test to demonstrate the bug with scalar operations on transposed tensors
use deep_causality_tensor::{BackendTensor, CpuBackend, Tensor, TensorBackend};

fn main() {
    // Create a 2x3 tensor: [[1, 2, 3], [4, 5, 6]]
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let tensor = <BackendTensor<f64, CpuBackend>>::from_slice(&data, &[2, 3]);

    println!("Original tensor (2x3):");
    println!("Shape: {:?}", CpuBackend::shape(tensor.inner()));
    println!("Strides: {:?}", CpuBackend::strides(tensor.inner()));
    println!("Data: {:?}", CpuBackend::to_vec(tensor.inner()));
    println!();

    // Transpose to get 3x2 tensor: [[1, 4], [2, 5], [3, 6]]
    let transposed = tensor.permute_axes(&[1, 0]).unwrap();

    println!("Transposed tensor (3x2):");
    println!("Shape: {:?}", CpuBackend::shape(transposed.inner()));
    println!("Strides: {:?}", CpuBackend::strides(transposed.inner()));
    println!("Data (storage order): {:?}", CpuBackend::to_vec(transposed.inner()));
    println!();

    // Get elements in logical order
    println!("Elements in logical order:");
    for i in 0..3 {
        for j in 0..2 {
            let val = CpuBackend::get(transposed.inner(), &[i, j]).unwrap();
            print!("{} ", val);
        }
        println!();
    }
    println!();

    // Now add a scalar to the transposed tensor
    // Expected: [[1+10, 4+10], [2+10, 5+10], [3+10, 6+10]] = [[11, 14], [12, 15], [13, 16]]
    let result = &transposed + 10.0;

    println!("After adding 10.0 to transposed tensor:");
    println!("Shape: {:?}", CpuBackend::shape(result.inner()));
    println!("Strides: {:?}", CpuBackend::strides(result.inner()));
    println!("Data: {:?}", CpuBackend::to_vec(result.inner()));
    println!();

    // Get elements in logical order
    println!("Elements in logical order (EXPECTED: [[11, 14], [12, 15], [13, 16]]):");
    for i in 0..3 {
        for j in 0..2 {
            let val = CpuBackend::get(result.inner(), &[i, j]).unwrap();
            print!("{} ", val);
        }
        println!();
    }
    println!();

    // Check if the bug exists
    // The bug would manifest as: the data is in storage order [1,2,3,4,5,6] + 10 = [11,12,13,14,15,16]
    // But interpreted with shape [3, 2], giving [[11, 12], [13, 14], [15, 16]]
    // Instead of the correct [[11, 14], [12, 15], [13, 16]]

    let expected_vals = vec![
        vec![11.0, 14.0],
        vec![12.0, 15.0],
        vec![13.0, 16.0],
    ];

    let mut all_correct = true;
    for i in 0..3 {
        for j in 0..2 {
            let val = CpuBackend::get(result.inner(), &[i, j]).unwrap();
            if val != expected_vals[i][j] {
                println!("BUG DETECTED: result[{}, {}] = {}, expected {}", i, j, val, expected_vals[i][j]);
                all_correct = false;
            }
        }
    }

    if all_correct {
        println!("✓ Test PASSED: scalar arithmetic preserves transposed layout correctly");
    } else {
        println!("✗ Test FAILED: scalar arithmetic does NOT preserve transposed layout correctly");
    }
}
```

### Test output

```
Original tensor (2x3):
Shape: [2, 3]
Strides: [3, 1]
Data: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]

Transposed tensor (3x2):
Shape: [3, 2]
Strides: [1, 3]
Data (storage order): [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]

Elements in logical order:
1 4
2 5
3 6

After adding 10.0 to transposed tensor:
Shape: [3, 2]
Strides: [2, 1]
Data: [11.0, 12.0, 13.0, 14.0, 15.0, 16.0]

Elements in logical order (EXPECTED: [[11, 14], [12, 15], [13, 16]]):
11 12
13 14
15 16

BUG DETECTED: result[0, 1] = 12, expected 14
BUG DETECTED: result[1, 0] = 13, expected 12
BUG DETECTED: result[1, 1] = 14, expected 15
BUG DETECTED: result[2, 0] = 15, expected 13
✗ Test FAILED: scalar arithmetic does NOT preserve transposed layout correctly
```

## Inconsistency within the codebase

### Reference code

`deep_causality_tensor/src/types/backend_tensor/ops.rs:221-226`

The **scalar assignment operations** (like `+=`) correctly handle non-contiguous tensors:

```rust
impl<B: TensorBackend> AddAssign<$t> for BackendTensor<$t, B> {
    fn add_assign(&mut self, rhs: $t) {
        // Create scalar tensor on same device, use broadcast add
        let scalar_tensor = B::create(&[rhs], &[1]);
        self.inner = B::add(&self.inner, &scalar_tensor);  // <-- Uses backend add with broadcasting
    }
}
```

### Current code

`deep_causality_tensor/src/types/backend_tensor/ops.rs:333-339`

The **scalar arithmetic operations** (like `+`) incorrectly bypass stride-aware logic:

```rust
impl<B: TensorBackend> Add<$t> for &BackendTensor<$t, B> {
    type Output = BackendTensor<$t, B>;
    fn add(self, rhs: $t) -> Self::Output {
        let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
        BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
    }
}
```

### Contradiction

The assignment operations correctly create a scalar tensor and use `B::add()`, which for the CPU backend delegates to `InternalCpuTensor`'s `Add` trait. This implementation uses `broadcast_op()`, which correctly handles strides by using `get_flat_index_broadcasted()` to map logical indices to storage positions.

However, the non-assignment arithmetic operations bypass this entirely by:
1. Extracting raw storage with `B::to_vec()` (which returns `tensor.data.clone()`)
2. Mapping the scalar operation over the storage vector
3. Creating a new contiguous tensor

This loses all stride information and produces incorrect results for non-contiguous tensors. The inconsistency is particularly problematic because:
- `tensor += 5.0` works correctly (uses broadcast_op)
- `tensor = tensor + 5.0` produces wrong results (bypasses broadcast_op)
- `tensor + 5.0` produces wrong results (bypasses broadcast_op)

# Full context

The `BackendTensor` type is a wrapper around backend-specific tensor implementations (CPU, MLX/GPU) that provides a unified API for tensor operations. The scalar arithmetic operations (Add, Sub, Mul, Div with scalar values) are fundamental operations used throughout the tensor library.

When users perform operations like transpose (`permute_axes`), slice, or other view operations, the resulting tensor shares the same underlying storage but has different strides that define how to interpret that storage. The stride array tells you how many positions to advance in the storage for each unit step along each dimension.

The current scalar arithmetic implementation was likely written for performance (avoiding the overhead of broadcast_op for simple scalar operations), but it incorrectly assumes tensors are always contiguous. This assumption breaks for:
- **Transposed tensors**: Different stride pattern
- **Sliced tensors**: Non-contiguous views
- **Broadcasted tensors**: May have zero strides in some dimensions
- **Reshaped tensors**: When reshape cannot be done in-place

The backend tensor is used by higher-level types like:
- `CausalTensor`: The main user-facing tensor type
- `MultiField` and `MultiVector`: Clifford algebra types used in physics simulations
- `Manifold` types in the topology module

All of these would be affected by this bug when using scalar arithmetic on non-contiguous tensors.

# Why has this bug gone undetected?

This bug has likely gone undetected for several reasons:

1. **Test coverage gap**: The existing tests in `backend_tensor_arithmetic_tests.rs` only test scalar operations on freshly created, contiguous tensors. There are no tests combining transpose/slice operations with scalar arithmetic.

2. **Common usage patterns**: Many tensor operations naturally create new contiguous tensors (like reduction operations, matrix multiplication results), so the bug only manifests in specific scenarios where:
    - A tensor is transposed/sliced AND
    - Then has scalar arithmetic applied AND
    - The result is examined element-by-element

3. **Assignment operations work**: Since the assignment operations (`+=`, `-=`, etc.) work correctly, users who primarily use those would not encounter the bug. The expression `tensor += 5.0` is correct, only `tensor = tensor + 5.0` or `result = tensor + 5.0` are wrong.

4. **Visual inspection difficulty**: When printing tensors, the Display implementation likely iterates in logical order (using the proper strides), so the bug isn't immediately visible. You need to either:
    - Access individual elements and compare to expected values, or
    - Check the underlying storage vector

5. **MLX backend may hide it**: The MLX (GPU) backend might handle this differently since it likely uses native MLX operations that respect the array layout, potentially masking the bug on GPU while it exists on CPU.

6. **Production code paths**: The bug may not have been triggered in production because:
    - Physics simulations often work with freshly computed tensors
    - Many operations that produce views are immediately followed by operations that materialize to contiguous arrays
    - The specific combination of transpose + scalar arithmetic may be rare in the actual usage patterns

# Recommended fix

Replace the scalar arithmetic implementations with the same pattern used in scalar assignment operations:

```rust
impl<B: TensorBackend> Add<$t> for &BackendTensor<$t, B> {
    type Output = BackendTensor<$t, B>;
    fn add(self, rhs: $t) -> Self::Output {
        let scalar_tensor = B::create(&[rhs], &[1]);  // <-- FIX 🟢 Create scalar tensor
        BackendTensor::from_inner(B::add(&self.inner, &scalar_tensor))  // <-- FIX 🟢 Use backend add
    }
}
```

This approach:
1. Creates a scalar tensor `[rhs]` with shape `[1]`
2. Uses the backend's `add` operation, which properly handles broadcasting and strides
3. Returns the result

The same pattern should be applied to all scalar arithmetic operations: Add, Sub, Mul, and Div, for both owned and borrowed variants.

**Note on performance**: This fix may have a slight performance cost compared to the current buggy implementation because it goes through the broadcast machinery. However:
- Correctness is more important than micro-optimizations
- The broadcast_op can be optimized to detect scalar cases
- GPU backends already use native operations that are efficient
- The performance difference is likely negligible for most use cases

# Related bugs

The same bug pattern exists for **all four arithmetic operations**:
- `Add<$t>` (lines 320-346)
- `Sub<$t>` (lines 348-378)
- `Mul<$t>` (lines 383-410)
- `Div<$t>` (lines 412-443)

Each has implementations for:
- `BackendTensor + scalar`
- `scalar + BackendTensor`
- `&BackendTensor + scalar`
- `scalar + &BackendTensor`

All of these need the same fix. The commutative operations (Add, Mul) have simpler fixes for the reverse order (scalar + tensor), but Sub and Div need careful implementation since they're non-commutative.


# Summary
- **Context**: The `inverse_impl` function in `deep_causality_tensor/src/types/causal_tensor/ops/tensor_inverse/mod.rs` computes the matrix inverse using Gaussian elimination on an augmented matrix.
- **Bug**: The function directly accesses the underlying data array without respecting the tensor's stride-based memory layout.
- **Actual vs. expected**: When called on a tensor with non-standard strides (e.g., after `permute_axes`), the function reads incorrect matrix elements instead of respecting the logical view defined by the shape and strides.
- **Impact**: Computing the inverse of a transposed or permuted matrix returns an incorrect result, making the matrix inverse operation unreliable for any tensor that has been manipulated with metadata-only operations.

# Code with bug
```rust
// Create an augmented matrix [A | I]
let mut augmented_data = Vec::with_capacity(n * 2 * n);
for r in 0..n {
    for c in 0..n {
        augmented_data.push(self.data[r * n + c]); // <-- BUG 🔴 Ignores strides, assumes row-major with stride n
    }
    for c in 0..n {
        if r == c {
            augmented_data.push(T::one());
        } else {
            augmented_data.push(T::zero());
        }
    }
}
```

Location: `deep_causality_tensor/src/types/causal_tensor/ops/tensor_inverse/mod.rs:40`

# Evidence

## Failing test

### Test script
```rust
use deep_causality_tensor::{CausalTensor, Tensor};

fn main() {
    // Create a 2x2 matrix
    // [[1.0, 2.0],
    //  [3.0, 4.0]]
    let matrix = CausalTensor::new(vec![1.0_f64, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();

    // Transpose it using permute_axes to get:
    // [[1.0, 3.0],
    //  [2.0, 4.0]]
    let transposed = matrix.permute_axes(&[1, 0]).unwrap();

    println!("Original matrix:");
    println!("{}", matrix);
    println!("\nTransposed matrix:");
    println!("{}", transposed);

    // Verify the transposed matrix has correct values via indexing
    println!("\nTransposed matrix elements via get():");
    println!("(0,0) = {}", transposed.get(&[0, 0]).unwrap());
    println!("(0,1) = {}", transposed.get(&[0, 1]).unwrap());
    println!("(1,0) = {}", transposed.get(&[1, 0]).unwrap());
    println!("(1,1) = {}", transposed.get(&[1, 1]).unwrap());

    // Try to compute the inverse of the transposed matrix
    println!("\nComputing inverse of transposed matrix...");
    match transposed.inverse() {
        Ok(inv) => {
            println!("Inverse:");
            println!("{}", inv);

            // Verify: transposed * inv should equal identity
            let product = transposed.matmul(&inv).unwrap();
            println!("\nProduct of transposed * inverse:");
            println!("{}", product);

            // Check if it's close to identity
            let id_00 = product.get(&[0, 0]).unwrap();
            let id_01 = product.get(&[0, 1]).unwrap();
            let id_10 = product.get(&[1, 0]).unwrap();
            let id_11 = product.get(&[1, 1]).unwrap();

            println!("\nExpected identity matrix:");
            println!("[[1.0, 0.0],");
            println!(" [0.0, 1.0]]");
            println!("\nActual product:");
            println!("[[{}, {}],", id_00, id_01);
            println!(" [{}, {}]]", id_10, id_11);

            let epsilon = 1e-10;
            let is_identity =
                (*id_00 - 1.0).abs() < epsilon &&
                (*id_01).abs() < epsilon &&
                (*id_10).abs() < epsilon &&
                (*id_11 - 1.0).abs() < epsilon;

            if is_identity {
                println!("\n✓ SUCCESS: Inverse is correct!");
            } else {
                println!("\n✗ FAILURE: Inverse is incorrect!");
                std::process::exit(1);
            }
        }
        Err(e) => {
            println!("Error computing inverse: {:?}", e);
            std::process::exit(1);
        }
    }
}
```

### Test output
```
Original matrix:
CausalTensor { data: [1, 2, 3, 4], shape: [2, 2], strides: [2, 1] }

Transposed matrix:
CausalTensor { data: [1, 2, 3, 4], shape: [2, 2], strides: [1, 2] }

Transposed matrix elements via get():
(0,0) = 1
(0,1) = 3
(1,0) = 2
(1,1) = 4

Computing inverse of transposed matrix...
Inverse:
CausalTensor { data: [-1.9999999999999996, 0.9999999999999998, 1.4999999999999998, -0.49999999999999994], shape: [2, 2], strides: [2, 1] }

Product of transposed * inverse:
CausalTensor { data: [2.4999999999999996, -0.5, 2, -0.0000000000000002220446049250313], shape: [2, 2], strides: [2, 1] }

Expected identity matrix:
[[1.0, 0.0],
 [0.0, 1.0]]

Actual product:
[[2.4999999999999996, -0.5],
 [2, -0.0000000000000002220446049250313]]

✗ FAILURE: Inverse is incorrect!
```

## Example

Let's trace through what happens when we transpose a 2×2 matrix and then compute its inverse:

**Step 1: Create original matrix**
```
Matrix: [[1, 2],
         [3, 4]]
Data: [1, 2, 3, 4]
Shape: [2, 2]
Strides: [2, 1]  (row-major layout)
```

**Step 2: Transpose using `permute_axes(&[1, 0])`**

After transposition, we should have:
```
Transposed: [[1, 3],
             [2, 4]]
Data: [1, 2, 3, 4]  (unchanged - metadata-only operation)
Shape: [2, 2]
Strides: [1, 2]  (now column-major layout for the original data)
```

The `permute_axes` operation is a metadata-only operation that doesn't reorder the data - it just changes the strides so that accessing element `(r, c)` now uses `data[r * strides[0] + c * strides[1]] = data[r * 1 + c * 2]`.

**Step 3: Bug occurs in `inverse_impl` at line 40**

The buggy code does:
```rust
for r in 0..n {
    for c in 0..n {
        augmented_data.push(self.data[r * n + c]);
    }
    ...
}
```

For `n = 2`, this accesses:
- `(0, 0)`: `data[0 * 2 + 0] = data[0] = 1` ✓ correct
- `(0, 1)`: `data[0 * 2 + 1] = data[1] = 2` ✗ **wrong!** (should be 3)
- `(1, 0)`: `data[1 * 2 + 0] = data[2] = 3` ✗ **wrong!** (should be 2)
- `(1, 1)`: `data[1 * 2 + 1] = data[3] = 4` ✓ correct

**The bug**: The code computes the inverse of `[[1, 2], [3, 4]]` instead of the transposed matrix `[[1, 3], [2, 4]]`.

**Expected behavior**: The code should use `data[r * self.strides[0] + c * self.strides[1]]` which would correctly access:
- `(0, 0)`: `data[0 * 1 + 0 * 2] = data[0] = 1` ✓
- `(0, 1)`: `data[0 * 1 + 1 * 2] = data[2] = 3` ✓
- `(1, 0)`: `data[1 * 1 + 0 * 2] = data[1] = 2` ✓
- `(1, 1)`: `data[1 * 1 + 1 * 2] = data[3] = 4` ✓

## Inconsistency within the codebase

### Reference code
`deep_causality_tensor/src/types/causal_tensor/ops/tensor_svd/mod.rs:42-48`
```rust
for i in 0..n {
    let mut sum = T::zero();
    for j in 0..i {
        sum += *l_matrix.get_ref(i, j)? * *z_vector.get_ref(j, 0)?;
    }
    let val = (*y_vector.get_ref(i, 0)? - sum) / *l_matrix.get_ref(i, i)?;
    z_vector.set(i, 0, val)?;
}
```

And the `get_ref` method itself at `deep_causality_tensor/src/types/causal_tensor/getters/get_ref.rs:14-20`:
```rust
pub(crate) fn get_ref(&self, row: usize, col: usize) -> Result<&T, CausalTensorError> {
    if self.num_dim() != 2 || row >= self.shape[0] || col >= self.shape[1] {
        return Err(CausalTensorError::IndexOutOfBounds);
    }
    let flat_index = row * self.strides[0] + col * self.strides[1];  // Respects strides
    Ok(&self.data[flat_index])
}
```

### Current code
`deep_causality_tensor/src/types/causal_tensor/ops/tensor_inverse/mod.rs:38-41`
```rust
for r in 0..n {
    for c in 0..n {
        augmented_data.push(self.data[r * n + c]);  // Ignores strides
    }
```

### Contradiction

The `inverse_impl` code is inconsistent with the rest of the codebase:

1. The SVD/Cholesky implementations correctly use `get_ref(row, col)` to access matrix elements, which properly respects the stride-based memory layout.

2. The `get_ref` method (which is even used extensively in the *same file* later in `inverse_impl` after line 59) explicitly uses `row * self.strides[0] + col * self.strides[1]` to compute the flat index.

3. However, line 40 of `inverse_impl` bypasses this proper accessor and directly computes `r * n + c`, which only works for tensors with standard row-major layout (where `strides == [n, 1]`).

This is particularly problematic because:
- The `get_ref` method is defined specifically to handle stride-based indexing
- The same function (`inverse_impl`) uses `get_ref` for accessing the augmented matrix elements later (lines 59, 75-76, 83, 88, 97, 102-103, 113)
- Other similar operations in the codebase (SVD, Cholesky) consistently use `get_ref`

The comment at line 7 in the file even says "Mostly used in inverse_impl", indicating that `get_ref` was designed for this purpose, yet the initial data copying doesn't use it.

# Full context

The `CausalTensor` type is the core tensor structure in the `deep_causality_tensor` crate. According to its documentation (in `deep_causality_tensor/src/types/causal_tensor/mod.rs`), it supports two types of operations:

1. **Metadata-Only Operations** (like `reshape`, `permute_axes`, `ravel`): These create a new tensor instance that clones the data but modifies the `shape` and `strides` metadata to provide a new logical view. The documentation explicitly states: "they create a *cloned* copy of the original flat data and only modify the `shape` and `strides` metadata to provide a new logical view of the data."

2. **Data-Copying Operations** (like `slice`, arithmetic operations): These create new tensors with newly allocated data.

The stride-based memory layout is fundamental to how `CausalTensor` works. The strides are calculated in the constructor (`new` and `from_vec_and_shape_unchecked`) and determine how multi-dimensional indices map to the flat data array.

The `inverse()` method is part of the public `Tensor` trait API (defined in `deep_causality_tensor/src/traits/tensor.rs`) and is exposed to users. It internally calls `inverse_impl()`.

When users call `permute_axes` to transpose or reorder dimensions, they expect subsequent operations (including `inverse()`) to work on the permuted view. However, the bug in `inverse_impl` breaks this expectation because it ignores the strides and reads from the wrong memory locations.

The matrix inverse operation is used in various linear algebra computations and is critical for applications like solving systems of equations, computing least squares solutions, and other numerical methods. Returning an incorrect inverse can lead to silently wrong results in downstream calculations.

The bug exists at line 40 where the augmented matrix is constructed. The rest of the function (lines 59-113) correctly uses `get_ref` and `set` methods which do respect strides. This suggests the bug was likely an oversight during initial implementation rather than a systematic misunderstanding of the stride system.

# Why has this bug gone undetected?

This bug has likely gone undetected for several reasons:

1. **Uncommon usage pattern**: Users typically don't call `inverse()` on permuted tensors. Most use cases involve creating a matrix directly with the correct layout and then computing its inverse, which works correctly.

2. **No tests for this scenario**: A review of the test files shows tests for `permute_axes` (in `deep_causality_tensor/tests/types/causal_tensor/op_tensor_shape_tests.rs`) and presumably tests for `inverse()`, but no tests that combine these operations. The tests don't verify that metadata-only operations compose correctly with data-processing operations.

3. **Recent addition**: The git history shows that the `inverse` operation was added relatively recently (commit `8713c6dd feat(deep_causality_tensor): Added inverse, cholesky_decomposition and solve_least_squares_cholsky operations`). Since it's a newer feature, it hasn't had as much usage and testing as other parts of the codebase.

4. **Silent failure**: The function doesn't panic or return an error - it just computes the wrong answer. The result appears plausible (it's still a 2×2 matrix with reasonable-looking numbers), so unless you verify the mathematical correctness (e.g., by checking that `A * A^(-1) = I`), the bug isn't obvious.

5. **Documentation doesn't highlight the interaction**: While the documentation mentions metadata-only operations, it doesn't explicitly state that all subsequent operations should work correctly on permuted views. Users might assume they need to materialize a permuted tensor before performing complex operations like inversion.

6. **Correct usage in rest of function**: The later parts of `inverse_impl` (after the augmented matrix is created) correctly use `get_ref` and `set`, so the function works correctly once the augmented matrix is created with standard strides. This means the test cases that use `inverse()` on normally-created matrices pass, masking the bug in the initial data access.

# Recommended fix

Replace the direct data access at line 40 with a call to `get_ref` to properly respect the tensor's strides:

```rust
// Create an augmented matrix [A | I]
let mut augmented_data = Vec::with_capacity(n * 2 * n);
for r in 0..n {
    for c in 0..n {
        augmented_data.push(*self.get_ref(r, c)?); // <-- FIX 🟢 Use get_ref to respect strides
    }
    for c in 0..n {
        if r == c {
            augmented_data.push(T::one());
        } else {
            augmented_data.push(T::zero());
        }
    }
}
```

This change makes the code consistent with:
1. The `get_ref` method which is already defined and used elsewhere in the same function
2. Other similar operations in the codebase (SVD, Cholesky decomposition) which use `get_ref`
3. The documented behavior of metadata-only operations like `permute_axes`

After this fix, the inverse of a permuted tensor will be computed correctly because the initial data will be read from the correct positions according to the stride-based layout.

# Summary
- **Context**: The `reshape` operation is used throughout the tensor library to change tensor dimensions, including within the `contract` function which is core to Einstein summation operations.
- **Bug**: `reshape` incorrectly handles tensors with non-contiguous strides (e.g., after `permute_axes`), producing results based on the original data layout rather than the permuted view.
- **Actual vs. expected**: When reshaping a permuted tensor, the operation should reorder elements according to the permuted strides before applying the new shape, but instead it ignores the permutation and uses the original data order.
- **Impact**: This bug causes silent data corruption in tensor contractions and any operations that combine `permute_axes` with `reshape`, leading to incorrect numerical results in matrix operations and Einstein summation computations.

# Code with bug
```rust
pub(in crate::types::causal_tensor) fn reshape_impl(
    &self,
    new_shape: &[usize],
) -> Result<Self, CausalTensorError> {
    let new_len: usize = new_shape.iter().product();
    if new_len != self.len() {
        return Err(CausalTensorError::ShapeMismatch);
    }
    // This is a metadata-only operation, so we clone the data but re-calculate strides.
    // BUG 🔴 This assumes self.data is in contiguous row-major order, but after permute_axes
    // the tensor has non-contiguous strides and the data is not in the logical order
    Ok(Self::from_vec_and_shape_unchecked(
        self.data.clone(),  // BUG 🔴 Clones raw data without respecting current strides
        new_shape,
    ))
}
```

Location: `deep_causality_tensor/src/types/causal_tensor/ops/tensor_shape/mod.rs` lines 12-25

The bug also affects the `contract` function which uses this pattern:
```rust
// Lines 176-186 in ein_sum_impl.rs
let permuted_lhs = lhs.permute_axes(&lhs_perm_order)?;
let permuted_rhs = rhs.permute_axes(&rhs_perm_order)?;

let contracted_dim_size: usize =
    lhs_contract_axes.iter().map(|&ax| lhs.shape[ax]).product();

let lhs_rows: usize = lhs_remaining_axes.iter().map(|&ax| lhs.shape[ax]).product();
let rhs_cols: usize = rhs_remaining_axes.iter().map(|&ax| rhs.shape[ax]).product();

let reshaped_lhs = permuted_lhs.reshape(&[lhs_rows, contracted_dim_size])?; // BUG 🔴
let reshaped_rhs = permuted_rhs.reshape(&[contracted_dim_size, rhs_cols])?; // BUG 🔴
```

# Evidence

## Example

Consider a 2×2 matrix stored as `[1, 2, 3, 4]` representing:
```
[[1, 2],
 [3, 4]]
```

After `permute_axes(&[1, 0])`, the tensor logically becomes:
```
[[1, 3],
 [2, 4]]
```

The permutation creates non-contiguous strides:
- Original: `shape=[2, 2]`, `strides=[2, 1]`, `data=[1, 2, 3, 4]`
- Permuted: `shape=[2, 2]`, `strides=[1, 2]`, `data=[1, 2, 3, 4]`

When accessing elements via the strided view:
- `get([0,0])` → index `0*1 + 0*2 = 0` → value `1` ✓
- `get([0,1])` → index `0*1 + 1*2 = 2` → value `3` ✓
- `get([1,0])` → index `1*1 + 0*2 = 1` → value `2` ✓
- `get([1,1])` → index `1*1 + 1*2 = 3` → value `4` ✓

When calling `reshape(&[4])`, the expected result is `[1, 3, 2, 4]` (reading the permuted matrix row-by-row).

However, the actual result is `[1, 2, 3, 4]` (the original data), because `reshape_impl` clones `self.data` directly without materializing the permuted view.

## Failing test

### Test script
```rust
/*
 * Test to demonstrate the bug in reshape after permute_axes
 *
 * Bug: reshape assumes contiguous data, but after permute_axes,
 * the tensor has non-contiguous strides. Calling reshape on such
 * a tensor produces incorrect results.
 */

use deep_causality_tensor::{CausalTensor, Tensor};

#[test]
fn test_reshape_after_permute_bug() {
    // Create a 2x3 matrix: [[1, 2, 3], [4, 5, 6]]
    let tensor = CausalTensor::new(
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        vec![2, 3]
    ).unwrap();

    // Transpose to 3x2: [[1, 4], [2, 5], [3, 6]]
    let transposed = tensor.permute_axes(&[1, 0]).unwrap();

    // Verify the transposed values are correct
    assert_eq!(*transposed.get(&[0, 0]).unwrap(), 1.0);
    assert_eq!(*transposed.get(&[0, 1]).unwrap(), 4.0);
    assert_eq!(*transposed.get(&[1, 0]).unwrap(), 2.0);
    assert_eq!(*transposed.get(&[1, 1]).unwrap(), 5.0);
    assert_eq!(*transposed.get(&[2, 0]).unwrap(), 3.0);
    assert_eq!(*transposed.get(&[2, 1]).unwrap(), 6.0);

    // Now reshape to 1D (should preserve the transposed order)
    let reshaped = transposed.reshape(&[6]).unwrap();

    // Expected: [1, 4, 2, 5, 3, 6] (the transposed matrix in row-major order)
    // Actual (BUG): [1, 2, 3, 4, 5, 6] (the original data, ignoring the transpose)

    println!("Reshaped data: {:?}", reshaped.data());

    // This assertion will FAIL, demonstrating the bug
    assert_eq!(*reshaped.get(&[0]).unwrap(), 1.0);
    assert_eq!(*reshaped.get(&[1]).unwrap(), 4.0); // BUG: This will be 2.0 instead of 4.0
    assert_eq!(*reshaped.get(&[2]).unwrap(), 2.0); // BUG: This will be 3.0 instead of 2.0
    assert_eq!(*reshaped.get(&[3]).unwrap(), 5.0); // BUG: This will be 4.0 instead of 5.0
    assert_eq!(*reshaped.get(&[4]).unwrap(), 3.0); // BUG: This will be 5.0 instead of 3.0
    assert_eq!(*reshaped.get(&[5]).unwrap(), 6.0);
}

#[test]
fn test_reshape_2d_after_permute_detailed() {
    // Create a simple 2x2 matrix: [[1, 2], [3, 4]]
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();

    // Transpose to [[1, 3], [2, 4]]
    let transposed = tensor.permute_axes(&[1, 0]).unwrap();

    // Verify transpose worked correctly
    assert_eq!(*transposed.get(&[0, 0]).unwrap(), 1.0);
    assert_eq!(*transposed.get(&[0, 1]).unwrap(), 3.0);
    assert_eq!(*transposed.get(&[1, 0]).unwrap(), 2.0);
    assert_eq!(*transposed.get(&[1, 1]).unwrap(), 4.0);

    // Reshape to 1D
    let reshaped = transposed.reshape(&[4]).unwrap();

    // Expected: [1, 3, 2, 4] (reading the transposed matrix row by row)
    // Actual (BUG): [1, 2, 3, 4] (the original data)
    println!("Reshaped data after transpose: {:?}", reshaped.data());

    // This will fail, showing the bug
    assert_eq!(*reshaped.get(&[0]).unwrap(), 1.0);
    assert_eq!(*reshaped.get(&[1]).unwrap(), 3.0); // BUG: This will be 2.0
    assert_eq!(*reshaped.get(&[2]).unwrap(), 2.0); // BUG: This will be 3.0
    assert_eq!(*reshaped.get(&[3]).unwrap(), 4.0);
}
```

### Test output
```
   Compiling deep_causality_tensor v0.2.0 (/home/user/deep_causality/deep_causality_tensor)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
     Running tests/bug_test_reshape_after_permute.rs (target/debug/deps/bug_test_reshape_after_permute-65e21f41ed1981e2)

running 2 tests
Reshaped data: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
Reshaped data after transpose: [1.0, 2.0, 3.0, 4.0]

thread 'test_reshape_after_permute_bug' (6798) panicked at deep_causality_tensor/tests/bug_test_reshape_after_permute.rs:40:5:
assertion `left == right` failed
  left: 2.0
 right: 4.0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'test_reshape_2d_after_permute_detailed' (6797) panicked at deep_causality_tensor/tests/bug_test_reshape_after_permute.rs:70:5:
assertion `left == right` failed
  left: 2.0
 right: 3.0
test test_reshape_after_permute_bug ... FAILED
test test_reshape_2d_after_permute_detailed ... FAILED

failures:

failures:
    test_reshape_2d_after_permute_detailed
    test_reshape_after_permute_bug

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

# Full context

The bug occurs in the tensor shape manipulation layer of the `deep_causality_tensor` crate. This crate implements tensor operations for causal analysis, including Einstein summation notation support.

## How reshape is used in the codebase

The `reshape` operation is called in several critical contexts:

1. **`contract` function** (`ein_sum_impl.rs` lines 118-197): This is the core tensor contraction implementation used by Einstein summation. It:
    - Permutes axes to arrange contracted and free dimensions
    - Reshapes the permuted tensors into 2D matrices
    - Performs matrix multiplication
    - Reshapes the result back to the final tensor shape

2. **Einstein summation operations**: The `contract` function is used by:
    - Generic `Contraction` operations
    - `DotProd` (dot product of vectors)
    - `MatMul` (when routed through contraction)

3. **User-facing API**: The `reshape` method is part of the public `Tensor` trait and can be called directly by users.

## Impact on contract function

The `contract` function implements the core algorithm for tensor contractions. The algorithm is:

1. Identify free (uncontracted) and contracted axes
2. Permute LHS to `(free_lhs, contracted_lhs)` order
3. Permute RHS to `(contracted_rhs, free_rhs)` order
4. Reshape LHS to `[M, K]` where `M = product(free_lhs_dims)`, `K = product(contracted_dims)`
5. Reshape RHS to `[K, N]` where `N = product(free_rhs_dims)`
6. Perform matrix multiplication
7. Reshape result to final shape

The bug occurs at steps 4 and 5: after permutation (steps 2-3), the tensors have non-contiguous strides, so reshape produces incorrect 2D matrices. This leads to wrong results from the matrix multiplication.

## Why the bug matters

Einstein summation is used for:
- Matrix multiplication and tensor contractions in machine learning
- Quantum state computations
- Statistical operations
- Causal graph computations

Silent data corruption in these operations can lead to:
- Incorrect model training results
- Wrong causal inferences
- Invalid statistical conclusions
- Hard-to-debug numerical errors

The bug is particularly insidious because:
- Tests may pass if they only check simple cases without permutations
- The contract function appears to work for standard matrix multiplication (which may not require complex permutations)
- Element access via `get()` works correctly on permuted tensors, masking the reshape bug

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Limited test coverage of permute+reshape combinations**: Existing tests in `ein_sum_impl_tests.rs` test individual operations but don't thoroughly test the combination of permute followed by reshape. The `contract` tests use simple cases where the permutation may not expose the bug.

2. **Correct element access masks the problem**: The `get()` method correctly handles non-contiguous strides, so permuted tensors work correctly for element-wise access. This means the bug only manifests when reshape is called on a permuted tensor.

3. **Simple contraction patterns may not trigger the bug**: Standard matrix multiplication `A[i,k] * B[k,j]` with both tensors in row-major order may not require complex permutations that expose the issue. The bug becomes apparent in more complex contraction patterns or when tensors are already in non-standard layouts.

4. **The comment suggests metadata-only operation**: The code comment in `reshape_impl` says "This is a metadata-only operation", which might have led developers to believe cloning the data verbatim was correct. However, with non-contiguous strides, a metadata-only operation is impossible without first materializing the strided view.

5. **Successful compile and unit tests**: The individual unit tests for `permute_axes` and `reshape` pass when tested separately. The bug only appears when these operations are composed, which requires integration testing.

# Recommended fix

The `reshape_impl` function must check if the tensor has contiguous strides. If strides are non-contiguous, it must materialize the data in the correct order before reshaping:

```rust
pub(in crate::types::causal_tensor) fn reshape_impl(
    &self,
    new_shape: &[usize],
) -> Result<Self, CausalTensorError> {
    let new_len: usize = new_shape.iter().product();
    if new_len != self.len() {
        return Err(CausalTensorError::ShapeMismatch);
    }

    // Check if tensor has contiguous row-major strides
    let is_contiguous = self.is_contiguous();

    if is_contiguous {
        // Fast path: just clone data and recalculate strides
        Ok(Self::from_vec_and_shape_unchecked(
            self.data.clone(),
            new_shape,
        ))
    } else {
        // FIX 🟢 Slow path: materialize the strided view into contiguous order
        let materialized_data = self.to_vec_in_logical_order();
        Ok(Self::from_vec_and_shape_unchecked(
            materialized_data,
            new_shape,
        ))
    }
}

// Helper method to check if strides are contiguous
fn is_contiguous(&self) -> bool {
    if self.shape.is_empty() {
        return true;
    }
    let mut expected_stride = 1;
    for i in (0..self.shape.len()).rev() {
        if self.strides[i] != expected_stride {
            return false;
        }
        expected_stride *= self.shape[i];
    }
    true
}

// Helper method to materialize data in logical order
fn to_vec_in_logical_order(&self) -> Vec<T> {
    let total_elements = self.len();
    let mut result = Vec::with_capacity(total_elements);

    // Iterate through all indices in row-major order
    let mut indices = vec![0; self.shape.len()];
    for _ in 0..total_elements {
        result.push(self.get(&indices).unwrap().clone());

        // Increment indices in row-major order
        let mut dim = self.shape.len() - 1;
        loop {
            indices[dim] += 1;
            if indices[dim] < self.shape[dim] {
                break;
            }
            indices[dim] = 0;
            if dim == 0 {
                break;
            }
            dim -= 1;
        }
    }

    result
}
```

Note: The exact implementation of the helper methods may need to be adjusted based on the existing codebase structure and performance requirements.


# Summary
- **Context**: The `contract` function performs tensor contraction (generalized matrix multiplication) over specified axes, and is a core operation in the Einstein summation (einsum) implementation.
- **Bug**: The function iterates through multi-dimensional indices in the wrong order when converting flat indices to tensor indices, violating row-major (C-style) memory layout conventions.
- **Actual vs. expected**: The function produces results with incorrectly ordered elements when contracting tensors with multiple free (non-contracted) axes, swapping the order of outer dimensions.
- **Impact**: Tensor contractions with multiple free axes return incorrect results, causing silent data corruption that leads to wrong numerical outputs in tensor operations and neural network computations.

# Code with bug

The bug occurs in the `contract` function where flat indices are converted to multi-dimensional tensor indices:

```rust
// Lines 250-256: Converting lhs_free_idx to multi-dimensional indices
for (i, &ax) in lhs_free.iter().enumerate() {  // <-- BUG 🔴 Iterates forward, treating first axis as fastest-varying
    let dim = lhs_free_sizes[i];
    if dim > 0 {
        lhs_index[ax] = lhs_free_remaining % dim;
        lhs_free_remaining /= dim;
    }
}
```

```rust
// Lines 269-275: Same bug for rhs_free_idx
for (i, &ax) in rhs_free.iter().enumerate() {  // <-- BUG 🔴 Iterates forward, treating first axis as fastest-varying
    let dim = rhs_free_sizes[i];
    if dim > 0 {
        rhs_index[ax] = rhs_free_remaining % dim;
        rhs_free_remaining /= dim;
    }
}
```

```rust
// Lines 258-264: Same bug for contracted indices
for &ax in lhs_axes.iter() {  // <-- BUG 🔴 Iterates forward
    let dim = lhs.shape[ax];
    if dim > 0 {
        lhs_index[ax] = contract_remaining % dim;
        contract_remaining /= dim;
    }
}
```

# Evidence

## Example

Consider contracting a 3D tensor with a 2D tensor:
- LHS shape: `[2, 2, 3]` (batch, rows, cols)
- RHS shape: `[3, 2]` (rows, cols)
- Contract over LHS axis 2 and RHS axis 0
- Expected result shape: `[2, 2, 2]` (batch, rows, cols)

The result should contain these values in row-major order:
```
result[0,0,0], result[0,0,1], result[0,1,0], result[0,1,1],
result[1,0,0], result[1,0,1], result[1,1,0], result[1,1,1]
```

With the bug, when `lhs_free_idx = 2` (meaning we want batch=0, row=1):
- The code iterates forward through free axes `[0, 1]` with sizes `[2, 2]`
- It computes: `index[0] = 2 % 2 = 0`, then `index[1] = 1 % 2 = 1`
- This gives **batch=0, row=1** ✓

Wait, let me recalculate. With `lhs_free_idx = 2`:
- Forward iteration: axis 0 first, then axis 1
- `lhs_free_remaining = 2`
- For axis 0 (size 2): `index[0] = 2 % 2 = 0`, `lhs_free_remaining = 2 / 2 = 1`
- For axis 1 (size 2): `index[1] = 1 % 2 = 1`, `lhs_free_remaining = 1 / 2 = 0`
- This gives **index = [0, 1]** (batch=0, row=1)

But for row-major layout, flat index 2 should map to:
- Total elements per batch: 2 (rows)
- Batch index: `2 / 2 = 1`
- Row index: `2 % 2 = 0`
- This should give **index = [1, 0]** (batch=1, row=0)

So the forward iteration gives `[0, 1]` when it should give `[1, 0]`.

Let me verify this with the test output:
```
Expected: [10.0, 13.0, 28.0, 40.0, 46.0, 67.0, 64.0, 94.0]
Actual:   [10.0, 13.0, 46.0, 67.0, 28.0, 40.0, 64.0, 94.0]
```

At index 2:
- Expected: 28.0 = result[0,1,0]
- Actual: 46.0 = result[1,0,0]

This confirms that indices `[0, 1]` and `[1, 0]` are being swapped.

## Inconsistency within the codebase

### Reference code

`deep_causality_tensor/src/types/cpu_tensor/mod.rs` lines 80-87:
```rust
// Calculate strides internally.
let mut strides = vec![0; shape.len()];
if !shape.is_empty() {
    let mut current_stride = 1;
    // Iterate from the last dimension to the first
    for i in (0..shape.len()).rev() {  // <-- Iterates BACKWARD
        strides[i] = current_stride;
        current_stride *= shape[i];
    }
}
```

`deep_causality_tensor/src/types/cpu_tensor/ops/tensor_ein_sum/ein_sum_impl.rs` lines 443-451 (trace function):
```rust
// Increment batch indices
let mut k = batch_axes.len();
while k > 0 {
    k -= 1;  // <-- Iterates BACKWARD
    current_batch_indices[k] += 1;
    if current_batch_indices[k] < tensor.shape[batch_axes[k]] {
        break;
    }
    current_batch_indices[k] = 0;
}
```

### Current code

`deep_causality_tensor/src/types/cpu_tensor/ops/tensor_ein_sum/ein_sum_impl.rs` lines 250-256:
```rust
for (i, &ax) in lhs_free.iter().enumerate() {  // <-- Iterates FORWARD
    let dim = lhs_free_sizes[i];
    if dim > 0 {
        lhs_index[ax] = lhs_free_remaining % dim;
        lhs_free_remaining /= dim;
    }
}
```

### Contradiction

The codebase consistently uses row-major (C-style) ordering where the **last** dimension is the **fastest-varying** (stride of 1). This is evident from:
1. Stride calculation iterates **backward** through dimensions
2. Other functions in the same file (`trace`, `diagonal`) iterate **backward** when incrementing multi-dimensional indices

However, the `contract` function iterates **forward** when converting flat indices to multi-dimensional indices. This treats the **first** dimension as fastest-varying, which is column-major (Fortran-style) ordering. This inconsistency causes elements to be placed in the wrong positions in the result tensor.

## Failing test

### Test script
```rust
#[test]
fn test_contract_index_calculation_with_multiple_free_dims() {
    // Create a 3D LHS tensor with distinct values: shape [2, 2, 3]
    // Values are arranged so we can track which elements are accessed
    // In row-major order:
    // Batch 0, Row 0: [0, 1, 2]
    // Batch 0, Row 1: [3, 4, 5]
    // Batch 1, Row 0: [6, 7, 8]
    // Batch 1, Row 1: [9, 10, 11]
    let lhs_data: Vec<f64> = (0..12).map(|x| x as f64).collect();
    let lhs = InternalCpuTensor::new(lhs_data, vec![2, 2, 3]).unwrap();

    // Create a 2D RHS tensor: shape [3, 2]
    // Row 0: [0, 1]
    // Row 1: [2, 3]
    // Row 2: [4, 5]
    let rhs_data: Vec<f64> = (0..6).map(|x| x as f64).collect();
    let rhs = InternalCpuTensor::new(rhs_data, vec![3, 2]).unwrap();

    // Contract over LHS axis 2 and RHS axis 0
    // This performs: result[b, i, j] = sum_k lhs[b, i, k] * rhs[k, j]
    // Expected result shape: [2, 2, 2]
    let result = InternalCpuTensor::contract(&lhs, &rhs, &[2], &[0]).unwrap();

    assert_eq!(result.shape(), &[2, 2, 2]);

    // Manually compute expected values for verification
    // result[0, 0, 0] = lhs[0,0,0]*rhs[0,0] + lhs[0,0,1]*rhs[1,0] + lhs[0,0,2]*rhs[2,0]
    //                 = 0*0 + 1*2 + 2*4 = 0 + 2 + 8 = 10
    let expected_0_0_0 = 0.0 * 0.0 + 1.0 * 2.0 + 2.0 * 4.0;

    // result[0, 0, 1] = lhs[0,0,0]*rhs[0,1] + lhs[0,0,1]*rhs[1,1] + lhs[0,0,2]*rhs[2,1]
    //                 = 0*1 + 1*3 + 2*5 = 0 + 3 + 10 = 13
    let expected_0_0_1 = 0.0 * 1.0 + 1.0 * 3.0 + 2.0 * 5.0;

    // result[0, 1, 0] = lhs[0,1,0]*rhs[0,0] + lhs[0,1,1]*rhs[1,0] + lhs[0,1,2]*rhs[2,0]
    //                 = 3*0 + 4*2 + 5*4 = 0 + 8 + 20 = 28
    let expected_0_1_0 = 3.0 * 0.0 + 4.0 * 2.0 + 5.0 * 4.0;

    // result[0, 1, 1] = lhs[0,1,0]*rhs[0,1] + lhs[0,1,1]*rhs[1,1] + lhs[0,1,2]*rhs[2,1]
    //                 = 3*1 + 4*3 + 5*5 = 3 + 12 + 25 = 40
    let expected_0_1_1 = 3.0 * 1.0 + 4.0 * 3.0 + 5.0 * 5.0;

    // result[1, 0, 0] = lhs[1,0,0]*rhs[0,0] + lhs[1,0,1]*rhs[1,0] + lhs[1,0,2]*rhs[2,0]
    //                 = 6*0 + 7*2 + 8*4 = 0 + 14 + 32 = 46
    let expected_1_0_0 = 6.0 * 0.0 + 7.0 * 2.0 + 8.0 * 4.0;

    // result[1, 0, 1] = lhs[1,0,0]*rhs[0,1] + lhs[1,0,1]*rhs[1,1] + lhs[1,0,2]*rhs[2,1]
    //                 = 6*1 + 7*3 + 8*5 = 6 + 21 + 40 = 67
    let expected_1_0_1 = 6.0 * 1.0 + 7.0 * 3.0 + 8.0 * 5.0;

    // result[1, 1, 0] = lhs[1,1,0]*rhs[0,0] + lhs[1,1,1]*rhs[1,0] + lhs[1,1,2]*rhs[2,0]
    //                 = 9*0 + 10*2 + 11*4 = 0 + 20 + 44 = 64
    let expected_1_1_0 = 9.0 * 0.0 + 10.0 * 2.0 + 11.0 * 4.0;

    // result[1, 1, 1] = lhs[1,1,0]*rhs[0,1] + lhs[1,1,1]*rhs[1,1] + lhs[1,1,2]*rhs[2,1]
    //                 = 9*1 + 10*3 + 11*5 = 9 + 30 + 55 = 94
    let expected_1_1_1 = 9.0 * 1.0 + 10.0 * 3.0 + 11.0 * 5.0;

    // The result should be in row-major order: [batch][row][col]
    // So result.data() should be:
    // [result[0,0,0], result[0,0,1], result[0,1,0], result[0,1,1],
    //  result[1,0,0], result[1,0,1], result[1,1,0], result[1,1,1]]
    let expected_data = vec![
        expected_0_0_0,
        expected_0_0_1,
        expected_0_1_0,
        expected_0_1_1,
        expected_1_0_0,
        expected_1_0_1,
        expected_1_1_0,
        expected_1_1_1,
    ];

    println!("Expected result data: {:?}", expected_data);
    println!("Actual result data:   {:?}", result.data());

    for (i, (&expected, &actual)) in expected_data.iter().zip(result.data().iter()).enumerate() {
        assert!(
            (expected - actual).abs() < 1e-10,
            "Mismatch at index {}: expected {}, got {}",
            i,
            expected,
            actual
        );
    }
}
```

### Test output
```
Expected result data: [10.0, 13.0, 28.0, 40.0, 46.0, 67.0, 64.0, 94.0]
Actual result data:   [10.0, 13.0, 46.0, 67.0, 28.0, 40.0, 64.0, 94.0]

thread 'types::cpu_tensor::ops::tensor_ein_sum::ein_sum_impl_tests::tests::test_contract_index_calculation_with_multiple_free_dims' panicked at deep_causality_tensor/src/types/cpu_tensor/ops/tensor_ein_sum/ein_sum_impl_tests.rs:428:13:
Mismatch at index 2: expected 28, got 46
```

The test clearly shows that:
- At index 2: expected `result[0,1,0] = 28.0`, but got `result[1,0,0] = 46.0`
- At index 3: expected `result[0,1,1] = 40.0`, but got `result[1,0,1] = 67.0`
- At index 4: expected `result[1,0,0] = 46.0`, but got `result[0,1,0] = 28.0`
- At index 5: expected `result[1,0,1] = 67.0`, but got `result[0,1,1] = 40.0`

The outer dimensions are swapped because the index calculation uses the wrong iteration order.

# Full context

The `contract` function is the core implementation for tensor contraction operations in the Einstein summation (einsum) feature. It's called by:

1. **Ein sum execution engine** (`ein_sum_execution.rs`): The einsum parser translates einsum notation (like `"ijk,kl->ijl"`) into AST nodes that call the `contract` function to perform the actual computation.

2. **High-level tensor API**: Users can invoke einsum operations through the `CausalTensor::ein_sum()` method, which internally uses this contract function for various operations including:
    - Matrix multiplication
    - Batched matrix multiplication
    - Dot products
    - Tensor reductions
    - General tensor contractions

The bug affects any einsum operation that:
- Contracts tensors with **2 or more free (non-contracted) axes** on either side
- This includes common operations like batch matrix multiplication when tensors have additional batch dimensions

Operations that work correctly (unaffected by this bug):
- Simple matrix multiplication (2D × 2D): Only 1 free axis per tensor
- Dot product (1D × 1D): No free axes
- Tensor with 1 free axis: Only a single dimension to iterate

The bug causes silent data corruption - the function returns a result with the correct shape and seemingly valid values, but elements are in wrong positions. This makes the bug particularly dangerous as it may not be immediately obvious, leading to incorrect results in downstream computations.

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Existing tests only cover simple cases**: The test suite includes tests for matrix multiplication (2D × 2D) and dot products (1D × 1D), both of which have at most 1 free axis per tensor. With only 1 free axis, there's no opportunity for the dimensions to be reordered, so the bug doesn't manifest.

2. **Symmetric test data**: Most existing tests use symmetric matrices or uniform values (e.g., `vec![1.0; 4]`), where reordering elements doesn't change the results. This masks the bug even if the wrong elements are accessed.

3. **The bug is order-dependent**: The incorrect ordering only becomes apparent with:
    - Multiple free axes (2 or more)
    - Non-uniform, distinct values
    - Asymmetric tensors

4. **Correct implementation of simpler operations**: The specialized `mat_mul_2d` function (which handles 2D × 2D matrix multiplication directly without using `contract`) is implemented correctly. This means the most common use case works fine, while the general `contract` function has the bug.

5. **No end-to-end einsum tests with complex operations**: While einsum functionality exists, there are no tests that specifically verify complex contractions with multiple batch dimensions or free axes.

# Recommended fix

Change the index calculation loops to iterate **backward** through the axes, consistent with row-major memory layout:

In the `contract` function at lines 250-256, change:
```rust
for (i, &ax) in lhs_free.iter().enumerate() {
```
to:
```rust
for (i, &ax) in lhs_free.iter().enumerate().rev() {
```

Similarly, make the same change at:
- Lines 269-275 (rhs free axes)
- Lines 258-264 (contracted axes for lhs)
- Lines 277-283 (contracted axes for rhs)

However, there's a subtlety: when iterating in reverse with `.enumerate().rev()`, the indices are still in forward order. The proper fix needs to iterate through the axes in reverse order. Here's the correct approach:

```rust
// Instead of forward iteration
for (i, &ax) in lhs_free.iter().enumerate() {
    let dim = lhs_free_sizes[i];
    // ...
}

// Use reverse iteration
for i in (0..lhs_free.len()).rev() {
    let ax = lhs_free[i];
    let dim = lhs_free_sizes[i];
    if dim > 0 {
        lhs_index[ax] = lhs_free_remaining % dim;
        lhs_free_remaining /= dim;
    }
}
```

This ensures that the **last** free axis is processed first (with modulo operations), making it the fastest-varying dimension, which matches row-major layout.

Apply this same fix to all four index-calculation loops in the `contract` function.


# Summary
- **Context**: The Cholesky decomposition implementation in `deep_causality_tensor/src/types/cpu_tensor/ops/tensor_svd/mod.rs` is a core linear algebra operation used for solving least squares problems and other numerical computations.
- **Bug**: The `cholesky_decomposition_impl` function does not verify that the input matrix is symmetric before performing the decomposition.
- **Actual vs. expected**: The function silently accepts non-symmetric matrices and produces incorrect results where L * L^T ≠ A, instead of returning an error as documented.
- **Impact**: Users may unknowingly pass non-symmetric matrices and receive mathematically invalid decompositions, leading to incorrect results in downstream computations like least squares solving.

# Code with bug
```rust
pub(in crate::types::cpu_tensor) fn cholesky_decomposition_impl(
    &self,
) -> Result<Self, CausalTensorError> {
    // Input validation: Must be a square matrix
    let ndim = self.ndim();
    if ndim != 2 {
        return Err(CausalTensorError::DimensionMismatch);
    }
    let n = self.shape()[0];
    if n != self.shape()[1] {
        return Err(CausalTensorError::ShapeMismatch);
    }

    // BUG 🔴: No verification that the matrix is symmetric
    // The algorithm assumes A[i,j] = A[j,i] but never checks this

    let l_data = vec![T::zero(); n * n];
    let mut l_matrix = InternalCpuTensor::from_vec_and_shape_unchecked(l_data, &[n, n]);

    for i in 0..n {
        for j in 0..i + 1 {
            // Iterate up to and including the diagonal
            let mut sum = T::zero();
            for k in 0..j {
                let l_ik = *l_matrix
                    .get(&[i, k])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?;
                let l_jk = *l_matrix
                    .get(&[j, k])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?;
                sum += l_ik * l_jk;
            }

            if i == j {
                // Diagonal elements
                let s_ii = *self
                    .get(&[i, i])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?;
                let val = s_ii - sum;
                if val <= T::zero() {
                    // Check for positive definiteness
                    return Err(CausalTensorError::SingularMatrix); // Not positive definite
                }
                let sqrt_val = val.sqrt(); // Need a sqrt method on T

                if let Some(l_ref) = l_matrix.get_mut(&[i, j]) {
                    *l_ref = sqrt_val;
                }
            } else {
                // Off-diagonal elements
                let s_ij = *self
                    .get(&[i, j])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?; // <-- BUG 🔴 Only reads lower triangle
                let l_jj = *l_matrix
                    .get(&[j, j])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?;
                let val = (s_ij - sum) / l_jj;

                if let Some(l_ref) = l_matrix.get_mut(&[i, j]) {
                    *l_ref = val;
                }
            }
        }
    }
    Ok(l_matrix)
}
```

# Evidence

## Example

Consider the non-symmetric matrix:
```
A = [[4.0, 1.0],
     [2.0, 4.0]]
```

Note that A[0,1] = 1.0 but A[1,0] = 2.0, so A is not symmetric.

**Step-by-step execution:**

1. **i=0, j=0** (diagonal):
    - Computes L[0,0] = sqrt(A[0,0]) = sqrt(4.0) = 2.0

2. **i=1, j=0** (off-diagonal):
    - Reads A[1,0] = 2.0 (from lower triangle)
    - Computes L[1,0] = A[1,0] / L[0,0] = 2.0 / 2.0 = 1.0

3. **i=1, j=1** (diagonal):
    - Computes L[1,1] = sqrt(A[1,1] - L[1,0]^2) = sqrt(4.0 - 1.0) = sqrt(3.0) ≈ 1.732

Result: L = [[2.0, 0.0], [1.0, 1.732]]

**Verification:** L * L^T = [[4.0, 2.0], [2.0, 4.0]]

This does NOT equal the input A = [[4.0, 1.0], [2.0, 4.0]] !

The algorithm effectively ignores the upper triangle and treats the matrix as if it were:
```
[[4.0, 2.0],  ← uses A[1,0]=2.0 instead of A[0,1]=1.0
 [2.0, 4.0]]
```

## Inconsistency with own spec / docstring / comment

### Reference spec

From `deep_causality_tensor/src/types/cpu_tensor/api/mod.rs:496-522`:

```rust
/// Computes the Cholesky decomposition of a symmetric, positive-definite matrix.
///
/// For a symmetric, positive-definite matrix $A$, its Cholesky decomposition is
/// $A = L L^T$, where $L$ is a lower triangular matrix with positive diagonal entries,
/// and $L^T$ is its transpose.
///
/// # Constraints
///
/// - The input `InternalCpuTensor` must represent a 2D square matrix.
/// - The matrix must be symmetric and positive-definite. If it is not positive-definite,
///   the decomposition will fail (e.g., attempt to take the square root of a negative number,
///   or encounter a zero on the diagonal).
///
/// # Errors
///
/// - `CausalTensorError::DimensionMismatch`: If the tensor is not 2-dimensional.
/// - `CausalTensorError::ShapeMismatch`: If the tensor is not a square matrix.
/// - `CausalTensorError::SingularMatrix`: If the matrix is not positive-definite (e.g., a diagonal
///   element becomes zero or negative during computation).
```

From `deep_causality_tensor/src/traits/tensor.rs:498-522`:

```rust
/// Computes the Cholesky decomposition of a symmetric, positive-definite matrix.
///
/// # Constraints
///
/// - The input `CausalTensor` must represent a 2D square matrix.
/// - The matrix must be symmetric and positive-definite. If it is not positive-definite,
///   the decomposition will fail (e.g., attempt to take the square root of a negative number,
///   or encounter a zero on the diagonal).
///
/// # Errors
///
/// - `CausalTensorError::DimensionMismatch`: If the tensor is not 2-dimensional.
/// - `CausalTensorError::ShapeMismatch`: If the tensor is not a square matrix.
/// - `CausalTensorError::SingularMatrix`: If the matrix is not positive-definite (e.g., a diagonal
///   element becomes zero or negative during computation).
```

### Current code

From `deep_causality_tensor/src/types/cpu_tensor/ops/tensor_svd/mod.rs:105-167`:

```rust
pub(in crate::types::cpu_tensor) fn cholesky_decomposition_impl(
    &self,
) -> Result<Self, CausalTensorError> {
    // Input validation: Must be a square matrix
    let ndim = self.ndim();
    if ndim != 2 {
        return Err(CausalTensorError::DimensionMismatch);
    }
    let n = self.shape()[0];
    if n != self.shape()[1] {
        return Err(CausalTensorError::ShapeMismatch);
    }

    // No symmetry check here!

    let l_data = vec![T::zero(); n * n];
    let mut l_matrix = InternalCpuTensor::from_vec_and_shape_unchecked(l_data, &[n, n]);

    for i in 0..n {
        for j in 0..i + 1 {
            // Only processes lower triangle, assumes symmetry
            // ...
```

### Contradiction

The documentation explicitly states the matrix "must be symmetric" and lists expected errors, but the implementation:
1. Does not verify symmetry before proceeding
2. Does not return an error for non-symmetric input
3. Silently produces incorrect results that violate the mathematical invariant L * L^T = A

## Failing test

### Test script

```rust
// Test demonstrating the bug: cholesky_decomposition does not verify symmetry
//
// This test shows that the Cholesky decomposition implementation silently
// accepts non-symmetric matrices and produces incorrect results.

use deep_causality_tensor::{CausalTensor, Tensor};

#[test]
fn test_cholesky_fails_on_non_symmetric_matrix() {
    // Create a non-symmetric matrix:
    // A = [[4.0, 1.0],
    //      [2.0, 4.0]]
    // Note: A[0,1] = 1.0 but A[1,0] = 2.0, so it's not symmetric
    let non_sym_data = vec![4.0, 1.0, 2.0, 4.0];
    let a = CausalTensor::new(non_sym_data, vec![2, 2]).unwrap();

    // Cholesky decomposition should only work on symmetric positive-definite matrices
    // This should either:
    // 1. Return an error indicating the matrix is not symmetric, OR
    // 2. Verify symmetry and return CausalTensorError::ShapeMismatch or similar
    //
    // Currently it succeeds but produces incorrect results
    let l = a.cholesky_decomposition().unwrap();

    // Compute L * L^T
    let mut result = vec![0.0; 4];
    for i in 0..2 {
        for j in 0..2 {
            let mut sum = 0.0;
            for k in 0..2 {
                sum += l.as_slice()[i * 2 + k] * l.as_slice()[j * 2 + k];
            }
            result[i * 2 + j] = sum;
        }
    }

    // For a correct Cholesky decomposition, L * L^T should equal A
    // But it doesn't because the algorithm only reads the lower triangle
    let epsilon = 1e-9;
    let mut decomposition_is_correct = true;
    for i in 0..4 {
        let diff: f64 = result[i] - a.as_slice()[i];
        if diff.abs() > epsilon {
            decomposition_is_correct = false;
            break;
        }
    }

    // BUG: This assertion fails because L * L^T != A
    // The decomposition produced L * L^T = [[4.0, 2.0], [2.0, 4.0]]
    // But the input was A = [[4.0, 1.0], [2.0, 4.0]]
    assert!(
        decomposition_is_correct,
        "Bug confirmed: Cholesky decomposition on non-symmetric matrix produces L*L^T != A. \
         Expected L*L^T to equal input A, but A[0,1]={} while (L*L^T)[0,1]={}",
        a.as_slice()[1],
        result[1]
    );
}

fn main() {
    test_cholesky_fails_on_non_symmetric_matrix();
}
```

### Test output

```
running 1 test
test test_cholesky_fails_on_non_symmetric_matrix ... FAILED

failures:

---- test_cholesky_fails_on_non_symmetric_matrix stdout ----

thread 'test_cholesky_fails_on_non_symmetric_matrix' (6989) panicked at test_cholesky_non_symmetric.rs:52:5:
Bug confirmed: Cholesky decomposition on non-symmetric matrix produces L*L^T != A. Expected L*L^T to equal input A, but A[0,1]=1 while (L*L^T)[0,1]=2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_cholesky_fails_on_non_symmetric_matrix

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

# Full context

The Cholesky decomposition is implemented in the `InternalCpuTensor` type, which is the low-level CPU backend for tensor operations in the deep_causality framework. This implementation is:

1. **Called by the public API**: The `Tensor::cholesky_decomposition()` method in `deep_causality_tensor/src/types/cpu_tensor/api/mod.rs` delegates to this implementation.

2. **Used by least squares solver**: The `solve_least_squares_cholsky_impl` function in the same file uses Cholesky decomposition to solve the normal equations A^T A x = A^T b. If the Cholesky decomposition is incorrect, the least squares solution will also be incorrect.

3. **Part of linear algebra backend**: This is exposed through the `LinearAlgebraBackend` trait in `deep_causality_tensor/src/traits/backend_linear_algebra.rs` and used in higher-level tensor operations.

4. **Production code**: The crate is used in the deep_causality framework for causal inference and is part of a larger production system, as indicated by the copyright notices and SPDX license headers.

The bug affects any code that:
- Directly calls `cholesky_decomposition()` on a non-symmetric matrix
- Calls `solve_least_squares_cholsky()` where the design matrix A has linearly dependent columns in a non-symmetric way
- Relies on the documented behavior that non-symmetric matrices should fail with an error

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **All existing tests use symmetric matrices**: Looking at `deep_causality_tensor/tests/types/causal_tensor/op_tensor_svd_tests.rs`, all test cases use properly symmetric matrices:
    - `test_cholesky_decomposition_success`: Uses a symmetric 3x3 matrix
    - `test_cholesky_decomposition_another_success`: Uses another symmetric 3x3 matrix
    - `test_cholesky_decomposition_1x1_success`: Uses a 1x1 matrix (trivially symmetric)
    - Tests for singular matrices also use symmetric matrices with the correct symmetry

2. **Correct usage in practice**: Users likely provide symmetric matrices in practice because:
    - The documentation clearly states symmetry is required
    - Many applications naturally produce symmetric matrices (e.g., A^T A in least squares)
    - The function is typically used in contexts where symmetry is guaranteed by construction

3. **Silent failure produces plausible output**: The function doesn't crash or produce obviously wrong results; it produces a valid lower triangular matrix L that satisfies L * L^T = A_sym, where A_sym is the "symmetrized" version using only the lower triangle. This appears reasonable unless you carefully verify that L * L^T equals the original input.

4. **Mathematical subtlety**: The Cholesky algorithm itself only reads the lower triangle of the matrix, so it's easy to overlook that the input should be verified as symmetric. The algorithm works correctly for the matrix it "thinks" it's processing (the lower triangle), but this doesn't match the actual input when it's non-symmetric.

5. **Recent refactoring**: According to git history, there was a fix commit (2139c629) on Dec 4, 2025 that "Fixed cholesky_decomposition_impl", but that commit focused on other issues (using permute_axes and handling singular matrices) and didn't add symmetry verification.

# Recommended fix

Add a symmetry check at the beginning of `cholesky_decomposition_impl`, before performing the decomposition:

```rust
pub(in crate::types::cpu_tensor) fn cholesky_decomposition_impl(
    &self,
) -> Result<Self, CausalTensorError> {
    // Input validation: Must be a square matrix
    let ndim = self.ndim();
    if ndim != 2 {
        return Err(CausalTensorError::DimensionMismatch);
    }
    let n = self.shape()[0];
    if n != self.shape()[1] {
        return Err(CausalTensorError::ShapeMismatch);
    }

    // FIX 🟢: Verify the matrix is symmetric
    let epsilon = T::from(1e-10).unwrap_or(T::zero()); // Tolerance for floating-point comparison
    for i in 0..n {
        for j in i + 1..n {
            let a_ij = *self
                .get(&[i, j])
                .ok_or(CausalTensorError::IndexOutOfBounds)?;
            let a_ji = *self
                .get(&[j, i])
                .ok_or(CausalTensorError::IndexOutOfBounds)?;
            let diff = if a_ij > a_ji { a_ij - a_ji } else { a_ji - a_ij };
            if diff > epsilon {
                return Err(CausalTensorError::InvalidOperation); // FIX 🟢: Or add a new error variant for non-symmetric
            }
        }
    }

    // Continue with existing decomposition logic...
    let l_data = vec![T::zero(); n * n];
    // ... rest of the function
```

Alternative approaches:
1. Add a new error variant `CausalTensorError::NonSymmetricMatrix` for clearer error reporting
2. Add a parameter to optionally skip the symmetry check for performance-critical code paths where symmetry is guaranteed
3. Document that the function only uses the lower triangle and users should ensure symmetry themselves (though this is less safe)

The first approach (adding the check) is recommended because:
- It aligns with the documented behavior
- It prevents subtle bugs from propagating
- The performance cost is O(n²), which is acceptable for a decomposition that's also O(n³)
- It fails fast with a clear error message


# Summary
- **Context**: `InternalCpuTensor` is the CPU backend implementation for tensor operations, using stride-based memory layout to support efficient multi-dimensional indexing through metadata-only view transformations like `permute_axes`.
- **Bug**: The `reshape()` and `ravel()` methods ignore the current stride-based logical view and instead just clone the raw physical data array, breaking the abstraction when used after `permute_axes()`.
- **Actual vs. expected**: After permuting axes, reshaping should produce elements in row-major order of the logical (permuted) view, but it produces elements in the order of the original physical memory layout instead.
- **Impact**: Data corruption in tensors that undergo `permute_axes()` followed by `reshape()` or `ravel()`, leading to incorrect numerical results in any computation using these reshaped tensors.

# Code with bug

In `deep_causality_tensor/src/types/cpu_tensor/ops/tensor_shape/mod.rs`:

```rust
pub(in crate::types::cpu_tensor) fn reshape_impl(
    &self,
    new_shape: &[usize],
) -> Result<Self, CausalTensorError> {
    let new_len: usize = new_shape.iter().product();
    if new_len != self.len() {
        return Err(CausalTensorError::ShapeMismatch);
    }
    // This is a metadata-only operation, so we clone the data but re-calculate strides.
    Ok(Self::from_vec_and_shape_unchecked(
        self.data.clone(),  // <-- BUG 🔴 Clones raw physical data, ignoring strides
        new_shape,
    ))
}
```

Similarly in `deep_causality_tensor/src/types/cpu_tensor/ops/tensor_shape/mod.rs`:

```rust
pub(in crate::types::cpu_tensor) fn ravel_impl(mut self) -> Self {
    let len = self.len();
    self.shape = vec![len];
    self.strides = vec![1];
    self  // <-- BUG 🔴 Reuses data in original physical order, ignoring strides
}
```

# Evidence

## Example

Consider a 2×3 tensor with data `[1, 2, 3, 4, 5, 6]` and shape `[2, 3]`:
```
Logical view (row-major):
[[1, 2, 3],
 [4, 5, 6]]
Physical data: [1, 2, 3, 4, 5, 6]
Strides: [3, 1]
```

After `permute_axes([1, 0])` to transpose:
```
Logical view (row-major):
[[1, 4],
 [2, 5],
 [3, 6]]
Physical data: [1, 2, 3, 4, 5, 6]  (unchanged - this is the bug's root cause)
Strides: [1, 3]  (changed to create the transposed view)
```

The strides `[1, 3]` correctly map logical indices to physical memory:
- Logical `[0, 0]` → physical `0*1 + 0*3 = 0` → value `1` ✓
- Logical `[0, 1]` → physical `0*1 + 1*3 = 3` → value `4` ✓
- Logical `[1, 0]` → physical `1*1 + 0*3 = 1` → value `2` ✓

When we call `reshape([6])` on the permuted tensor, we expect the result to be the logical view flattened in row-major order:
```
Expected: [1, 4, 2, 5, 3, 6]  (reading the logical view row by row)
```

But the actual implementation just clones the physical data:
```
Actual: [1, 2, 3, 4, 5, 6]  (original physical order, wrong!)
```

## Inconsistency with own spec / docstring

### Reference comment
From `deep_causality_tensor/src/types/cpu_tensor/mod.rs` lines 280-297:
```rust
/// Returns a new tensor with the same data but a different shape.
///
/// This is a metadata-only operation; it creates a new `InternalCpuTensor` with a cloned copy
/// of the original flat data. The underlying data is *not* physically reordered or reallocated.
/// Only the `shape` and `strides` are recomputed to reflect the new logical view.
/// The total number of elements implied by the `new_shape` must be equal to the total number of
/// elements in the original tensor (`self.len()`).
```

### Current code
The implementation in `tensor_shape/mod.rs`:
```rust
Ok(Self::from_vec_and_shape_unchecked(
    self.data.clone(),  // Clones physical data directly
    new_shape,
))
```

### Contradiction
The docstring claims reshape is a "metadata-only operation" that "only" modifies "shape and strides", but this is misleading. The real issue is that reshape should work on the **logical view** of the tensor (as defined by current shape and strides), not the physical memory layout.

When a tensor has non-contiguous strides (after `permute_axes`), the "original flat data" mentioned in the docstring is physically ordered according to the **original** tensor's layout, not the **current** logical view. The implementation violates the principle that all operations should respect the current logical view.

## Inconsistency within the codebase

### Reference code
In `deep_causality_tensor/src/types/cpu_tensor/ops/tensor_view/mod.rs`, the `slice_impl` method correctly respects the logical view:

```rust
pub(in crate::types::cpu_tensor) fn slice_impl(
    &self,
    axis: usize,
    index: usize,
) -> Result<InternalCpuTensor<T>, CausalTensorError> {
    // ...
    let mut current_index = vec![0; self.ndim()];
    for _ in 0..self.len() {
        if current_index[axis] == index {
            let flat_index = self.get_flat_index(&current_index).unwrap();
            new_data.push(self.as_slice()[flat_index].clone());  // Uses strides via get_flat_index
        }
        // Increment multi-dimensional index...
    }
    InternalCpuTensor::new(new_data, new_shape)
}
```

### Current code
In `deep_causality_tensor/src/types/cpu_tensor/ops/tensor_shape/mod.rs`:

```rust
pub(in crate::types::cpu_tensor) fn reshape_impl(
    &self,
    new_shape: &[usize],
) -> Result<Self, CausalTensorError> {
    // ...
    Ok(Self::from_vec_and_shape_unchecked(
        self.data.clone(),  // Directly clones physical data
        new_shape,
    ))
}
```

### Comparison
The `slice_impl` method iterates through multi-dimensional indices and uses `get_flat_index` to correctly access elements according to the current strides. This respects the logical view regardless of the underlying memory layout.

In contrast, `reshape_impl` directly clones `self.data`, which is always in the original physical memory order, completely ignoring the logical view created by the current strides. This inconsistency means `slice()` works correctly after `permute_axes()`, but `reshape()` does not.

## Failing test

### Test script
```rust
// Failing unit test demonstrating the reshape/ravel bug after permute_axes
use deep_causality_tensor::{InternalCpuTensor, Tensor};

#[test]
fn test_reshape_after_permute_axes() {
    // Create a 2x3 tensor:
    // [[1, 2, 3],
    //  [4, 5, 6]]
    let tensor = InternalCpuTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();

    // Permute axes to transpose: [1, 0]
    // Logical view becomes:
    // [[1, 4],
    //  [2, 5],
    //  [3, 6]]
    let permuted = tensor.permute_axes(&[1, 0]).unwrap();

    // Verify permuted view is correct
    assert_eq!(permuted.shape(), &[3, 2]);
    assert_eq!(permuted.get(&[0, 0]), Some(&1));
    assert_eq!(permuted.get(&[0, 1]), Some(&4));
    assert_eq!(permuted.get(&[1, 0]), Some(&2));
    assert_eq!(permuted.get(&[1, 1]), Some(&5));
    assert_eq!(permuted.get(&[2, 0]), Some(&3));
    assert_eq!(permuted.get(&[2, 1]), Some(&6));

    // Reshape to 1D vector
    // Expected: elements in row-major order of the LOGICAL view
    // [[1, 4], [2, 5], [3, 6]] -> [1, 4, 2, 5, 3, 6]
    let reshaped = permuted.reshape(&[6]).unwrap();

    // THIS ASSERTION FAILS - demonstrating the bug
    assert_eq!(
        reshaped.as_slice(),
        &[1, 4, 2, 5, 3, 6],
        "reshape() should respect the logical view created by permute_axes"
    );
}

#[test]
fn test_ravel_after_permute_axes() {
    // Test the same bug with ravel()
    let tensor = InternalCpuTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let permuted = tensor.permute_axes(&[1, 0]).unwrap();

    // ravel() should also respect the logical view
    let raveled = permuted.ravel();

    // THIS ASSERTION FAILS - demonstrating the bug
    assert_eq!(
        raveled.as_slice(),
        &[1, 4, 2, 5, 3, 6],
        "ravel() should respect the logical view created by permute_axes"
    );
}

#[test]
fn test_reshape_after_permute_3d() {
    // Test with 3D tensor to show bug isn't specific to 2D
    let tensor = InternalCpuTensor::new(vec![0, 1, 2, 3, 4, 5, 6, 7], vec![2, 2, 2]).unwrap();

    // Permute axes [2, 0, 1]
    let permuted = tensor.permute_axes(&[2, 0, 1]).unwrap();
    assert_eq!(permuted.shape(), &[2, 2, 2]);

    // Verify some elements in the permuted view
    assert_eq!(permuted.get(&[0, 0, 0]), Some(&0));
    assert_eq!(permuted.get(&[0, 1, 0]), Some(&4));
    assert_eq!(permuted.get(&[1, 0, 0]), Some(&1));

    // Reshape to 1D - should preserve logical order
    let reshaped = permuted.reshape(&[8]).unwrap();

    // Build expected by manually reading in row-major order from permuted view
    let mut expected = Vec::new();
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                expected.push(*permuted.get(&[i, j, k]).unwrap());
            }
        }
    }

    assert_eq!(
        reshaped.as_slice(),
        expected.as_slice(),
        "reshape() should respect logical view for 3D tensors too"
    );
}

fn main() {}
```

### Test output
```
thread 'test_reshape_after_permute_3d' (11525) panicked at test_failing_unit_test.rs:84:5:
assertion `left == right` failed: reshape() should respect logical view for 3D tensors too
  left: [0, 1, 2, 3, 4, 5, 6, 7]
 right: [0, 2, 4, 6, 1, 3, 5, 7]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'test_ravel_after_permute_axes' (11524) panicked at test_failing_unit_test.rs:50:5:
assertion `left == right` failed: ravel() should respect the logical view created by permute_axes
  left: [1, 2, 3, 4, 5, 6]
 right: [1, 4, 2, 5, 3, 6]

thread 'test_reshape_after_permute_axes' (11526) panicked at test_failing_unit_test.rs:33:5:
assertion `left == right` failed: reshape() should respect the logical view created by permute_axes
  left: [1, 2, 3, 4, 5, 6]
 right: [1, 4, 2, 5, 3, 6]


running 3 tests
test test_reshape_after_permute_3d ... FAILED
test test_ravel_after_permute_axes ... FAILED
test test_reshape_after_permute_axes ... FAILED

failures:

failures:
    test_ravel_after_permute_axes
    test_reshape_after_permute_3d
    test_reshape_after_permute_axes

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

# Full context

The `InternalCpuTensor` type is the core CPU backend implementation for the tensor abstraction in DeepCausality. It uses a stride-based memory layout where a single contiguous `Vec<T>` stores all elements, and separate `shape` and `strides` vectors define how to interpret multi-dimensional indices.

The `permute_axes()` operation creates a new logical view by reordering dimensions. According to the existing tests in `op_tensor_shape_tests.rs`, after permuting, the `get()` method correctly retrieves elements from the permuted view. This is because `get()` uses `get_flat_index()` which respects the current strides.

However, `reshape()` and `ravel()` are called after `permute_axes()` in real-world workflows:
1. In tensor manipulation pipelines where users permute axes to match desired dimension ordering
2. In serialization when flattening a tensor after reordering dimensions
3. In neural network operations that combine axis permutation with reshaping

The bug manifests in the `Tensor` trait implementation (`deep_causality_tensor/src/types/cpu_tensor/api/mod.rs`), which exposes these methods publicly. Users calling the trait methods will encounter incorrect results.

This affects:
- The `CausalTensor` type (the public API wrapper around `InternalCpuTensor`)
- The `BackendTensor` trait implementations that delegate to CPU backend
- Any computation using reshaped tensors in mathematical operations, as the data elements are in the wrong positions

The `slice()` operation (in `tensor_view/mod.rs`) correctly handles permuted tensors by iterating through logical indices and using `get_flat_index()`, demonstrating that the codebase has a pattern for correctly respecting strides. The `reshape()` and `ravel()` implementations simply don't follow this pattern.

# Why has this bug gone undetected?

1. **No combination tests**: The test suite in `op_tensor_shape_tests.rs` tests `permute_axes()` and `reshape()` separately, but never in combination. The permute tests verify element access, and the reshape tests use tensors with standard (contiguous) memory layout.

2. **Silent data corruption**: The bug doesn't cause panics or obvious errors. It silently produces wrong data that looks plausible—it's still the same numbers, just in the wrong order. This makes it hard to notice unless you specifically verify the element ordering.

3. **Uncommon operation sequence**: Users might not frequently permute axes and then reshape. More common patterns include:
    - Reshape then permute (works correctly because reshape starts with contiguous data)
    - Multiple permutes without reshape (works correctly)
    - Reshape without prior permute (works correctly)

4. **Recent implementation**: The CPU backend was completed in late December 2024 (commit e02b3281, December 29, 2025), so the code hasn't been in production long enough for users to encounter this edge case.

5. **Misleading documentation**: The docstring calls these "metadata-only operations", which might lead developers to assume they can't have bugs since they're not doing complex computations—just adjusting metadata. The actual bug is subtle: it's about which view of the data to use as input.

# Recommended fix

The `reshape_impl` and `ravel_impl` methods should iterate through the logical view (respecting strides) when copying data, similar to how `slice_impl` does it. Here's the approach:

```rust
pub(in crate::types::cpu_tensor) fn reshape_impl(
    &self,
    new_shape: &[usize],
) -> Result<Self, CausalTensorError> {
    let new_len: usize = new_shape.iter().product();
    if new_len != self.len() {
        return Err(CausalTensorError::ShapeMismatch);
    }

    // Collect elements in row-major order of the current logical view  // <-- FIX 🟢
    let mut new_data = Vec::with_capacity(new_len);
    let mut current_index = vec![0; self.ndim()];

    if self.ndim() == 0 {
        // Scalar case
        new_data.push(self.data[0].clone());
    } else {
        // Iterate through all elements in row-major order
        for _ in 0..new_len {
            let flat_index = self.get_flat_index(&current_index).unwrap();
            new_data.push(self.data[flat_index].clone());

            // Increment multi-dimensional index
            for i in (0..self.ndim()).rev() {
                current_index[i] += 1;
                if current_index[i] < self.shape[i] {
                    break;
                }
                current_index[i] = 0;
            }
        }
    }

    Self::new(new_data, new_shape.to_vec())
}
```

Similarly for `ravel_impl`, convert it to iterate through the logical view rather than reusing the physical data directly.

# Related bugs

The same issue affects `ravel_impl` in the same file, as demonstrated by the failing test `test_ravel_after_permute_axes`. Both methods need the same fix.
