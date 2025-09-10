[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# CausalTensor - A Flexible Tensor for Dynamic Data

The `CausalTensor` provides a flexible, multi-dimensional array (tensor) backed by a single, contiguous `Vec<T>`. It is designed for efficient numerical computations, featuring a stride-based memory layout that supports broadcasting for element-wise binary operations. Unlike `ArrayGrid`, its dimensions are determined at runtime, offering greater flexibility for dynamic data shapes at the cost of heap allocation and the loss of some compile-time optimizations.

## Usage

`CausalTensor` is straightforward to use. You create it from a flat vector of data and a vector defining its shape.

```rust
use deep_causality_data_structures::CausalTensor;

fn main() {
    // 1. Create a 2x3 tensor.
    let data = vec![1, 2, 3, 4, 5, 6];
    let shape = vec![2, 3];
    let tensor = CausalTensor::new(data, shape).unwrap();
    println!("Original Tensor: {}", tensor);

    // 2. Get an element
    let element = tensor.get(&[1, 2]).unwrap();
    assert_eq!(*element, 6);
    println!("Element at [1, 2]: {}", element);

    // 3. Reshape the tensor
    let reshaped = tensor.reshape(&[3, 2]).unwrap();
    assert_eq!(reshaped.shape(), &[3, 2]);
    println!("Reshaped to 3x2: {}", reshaped);

    // 4. Perform tensor-scalar addition
    let added = &tensor + 10;
    assert_eq!(added.as_slice(), &[11, 12, 13, 14, 15, 16]);
    println!("Tensor + 10: {}", added);

    // 5. Perform tensor-tensor addition with broadcasting
    let t1 = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    // A [1, 3] tensor...
    let t2 = CausalTensor::new(vec![10, 20, 30], vec![1, 3]).unwrap(); 
    // ...is broadcasted across the rows of the [2, 3] tensor.
    let result = (&t1 + &t2).unwrap();
    assert_eq!(result.as_slice(), &[11, 22, 33, 14, 25, 36]);
    println!("Tensor-Tensor Add with Broadcast: {}", result);

    // 6. Sum all elements in the tensor (full reduction)
    let sum = tensor.sum_axes(&[]).unwrap();
    assert_eq!(sum.as_slice(), &[21]);
    println!("Sum of all elements: {}", sum);
}
```

## Performance

The following benchmarks were run on a `CausalTensor` of size 100x100 (10,000 `f64` elements).

| Operation                     | Time      | Notes                                      |
|-------------------------------|-----------|--------------------------------------------|
| `tensor_get`                  | ~1.64 ns  | Accessing a single element.                |
| `tensor_reshape`              | ~786 ns   | Metadata only, but clones data in the test.|
| `tensor_scalar_add`           | ~2.69 µs  | Element-wise addition with a scalar.       |
| `tensor_tensor_add_broadcast` | ~37.7 µs  | Element-wise addition with broadcasting.   |
| `tensor_sum_full_reduction`   | ~6.86 µs  | Summing all 10,000 elements of the tensor. |

### Key Observations
1.  **Element Access (`get`):** Access is extremely fast, demonstrating the efficiency of the stride-based index calculation.
2.  **Shape Manipulation (`reshape`):** This operation is very fast as it only adjusts metadata (shape and strides) and clones the underlying data vector.
3.  **Arithmetic Operations:** Performance is excellent. The optimized `binary_op` function provides efficient broadcasting for tensor-tensor operations, avoiding allocations in hot loops.

### Technical Details
- Sample size: 10 measurements per benchmark
- All benchmarks were run with random access patterns to simulate real-world usage

### Hardware & OS
- Architecture: ARM64 (Apple Silicon, M3 Max)
- OS: macOS 15.1


## Problem

While `ArrayGrid` is extremely fast for low-dimensional data where the shape is known at compile time, many real-world scenarios require more flexibility. Data shapes might be determined dynamically at runtime, or the number of dimensions might be larger than what `ArrayGrid` comfortably supports. A naive approach using nested vectors (e.g., `Vec<Vec<T>>`) for multi-dimensional data is often inefficient due to multiple, non-contiguous memory allocations, leading to poor cache performance.

## Solution

`CausalTensor` solves this by using a single, heap-allocated `Vec<T>` to store all data contiguously. It calculates and stores a `strides` vector internally, which allows it to map multi-dimensional indices to a single flat index into the data vector.

This approach provides:
-   **Dynamic Shapes:** Tensor dimensions are defined at runtime and can be easily changed.
-   **Efficient Memory Layout:** A single contiguous block of memory leads to better cache performance and avoids the overhead of nested pointers.
-   **Broadcasting:** A powerful feature that allows for element-wise operations between tensors of different but compatible shapes, reducing the need for manual data tiling or expansion.

## Technical Implementation

### Strides
The core of `CausalTensor` is its stride-based memory layout. For a given shape (e.g., `[d1, d2, d3]`), the strides represent the number of elements to skip in the flat data vector to move one step along a particular dimension. For a row-major layout, the strides would be `[d2*d3, d3, 1]`. This allows the tensor to calculate the flat index for any multi-dimensional index `[i, j, k]` with a simple dot product: `i*strides[0] + j*strides[1] + k*strides[2]`.

### Broadcasting
Binary operations support broadcasting, which follows rules similar to those in libraries like NumPy. When operating on two tensors, `CausalTensor` compares their shapes dimension by dimension (from right to left). Two dimensions are compatible if:
1. They are equal.
2. One of them is 1.

The smaller tensor's data is conceptually "stretched" or repeated along the dimensions where its size is 1 to match the larger tensor's shape, without actually copying the data. The optimized `binary_op` implementation achieves this by manipulating how it calculates the flat index for each tensor inside the computation loop.

### API Overview
The `CausalTensor` API is designed to be comprehensive and intuitive:
-   **Constructor:** `CausalTensor::new(data: Vec<T>, shape: Vec<usize>)`
-   **Inspectors:** `shape()`, `num_dim()`, `len()`, `is_empty()`, `as_slice()`
-   **Indexing:** `get()`, `get_mut()`
-   **Shape Manipulation:** `reshape()`, `ravel()`
-   **Reduction Operations:** `sum_axes()`, `mean_axes()`, `arg_sort()`
-   **Arithmetic:** Overloaded `+`, `-`, `*`, `/` operators for both tensor-scalar and tensor-tensor operations.
