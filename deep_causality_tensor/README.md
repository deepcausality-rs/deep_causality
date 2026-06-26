[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# CausalTensor - A Flexible Tensor for Dynamic Data

The CausalTensor provides a flexible, multi-dimensional array (tensor) backed by a single, contiguous `Vec<T>`. It is designed for efficient numerical computations, featuring a stride-based memory layout that supports broadcasting for
element-wise binary operations. It offers a comprehensive API for shape manipulation, element access, and common reduction operations like `sum` and `mean`, making it a versatile tool for causal modeling and other data-intensive
tasks.

## 📚 Docs

* [Design & Details](../deep_causality_tensor/README.md)
* [Benchmark](benches/benchmarks/causal_tensor_type)
* [Examples](../deep_causality_tensor/examples)
* [Test](../deep_causality_tensor/tests)

## Examples

To run the examples, use `cargo run --example <example_name>`.

*   **Applicative Causal Tensor**
    ```bash
    cargo run --example applicative_causal_tensor
    ```
*   **Basic Causal Tensor**
    ```bash
    cargo run --example causal_tensor
    ```
*   **Effect System Causal Tensor**
    ```bash
    cargo run --example effect_system_causal_tensor
    ```
*   **Einstein Summation Causal Tensor**
    ```bash
    cargo run --example ein_sum_causal_tensor
    ```
*   **Functor Causal Tensor**
    ```bash
    cargo run --example functor_causal_tensor
    ```

## Usage

`CausalTensor` is straightforward to use. You create it from a flat vector of data and a vector defining its shape.

```rust
use deep_causality_tensor::CausalTensor;

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

## Einstein Sum (ein_sum)

The `ein_sum` function provides a powerful and flexible way to perform various tensor operations, including matrix multiplication, dot products, and more, by constructing an Abstract Syntax Tree (AST) of operations.

```rust
use deep_causality_tensor::CausalTensor;
use deep_causality_tensor::types::causal_tensor::op_tensor_ein_sum::EinSumOp;

fn main() {
    // Example: Matrix Multiplication using ein_sum
    let lhs_data = vec![1.0, 2.0, 3.0, 4.0];
    let lhs_tensor = CausalTensor::new(lhs_data, vec![2, 2]).unwrap();

    let rhs_data = vec![5.0, 6.0, 7.0, 8.0];
    let rhs_tensor = CausalTensor::new(rhs_data, vec![2, 2]).unwrap();

    // Construct the AST for matrix multiplication
    let mat_mul_ast = EinSumOp::mat_mul(lhs_tensor, rhs_tensor);

    // Execute the Einstein summation
    let result = CausalTensor::ein_sum(&mat_mul_ast).unwrap();

    println!("Result of Matrix Multiplication:\n{:?}", result);
    // Expected: CausalTensor { data: [19.0, 22.0, 43.0, 50.0], shape: [2, 2], strides: [2, 1] }
    
     // Example: Dot Product
    let vec1_data = vec![1.0, 2.0, 3.0];
    let vec1_shape = vec![3];
    let vec1_tensor = CausalTensor::new(vec1_data, vec1_shape).unwrap();

    let vec2_data = vec![4.0, 5.0, 6.0];
    let vec2_shape = vec![3];
    let vec2_tensor = CausalTensor::new(vec2_data, vec2_shape).unwrap();
    
    // Execute the Einstein summation for dot product 
    let result_dot_prod = CausalTensor::ein_sum(&EinSumOp::dot_prod(vec1_tensor, vec2_tensor)).unwrap();
    println!("Result of Dot Product:\n{:?}", result_dot_prod);
}
```


## Functional Composition 

Causal Tensor implements a Higher Kinded Type via the `deep_causality_haft` crate as Witness Type. When imported, the CausalTensorWitness type allows monadic composition and abstract type programming. For example, one can write generic functions that uniformly process tensors and other types:

```rust
use deep_causality_haft::{Functor, HKT, OptionWitness, ResultWitness};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

fn triple_value<F>(m_a: F::Type<i32>) -> F::Type<i32>
where
    F: Functor<F> + HKT,
{
    F::fmap(m_a, |x| x * 3)
}

fn main() {
    println!("--- Functor Example: Tripling values in different containers ---");

    // Using triple_value with Option
    let opt = Some(5);
    println!("Original Option: {:?}", opt);
    let proc_opt = triple_value::<OptionWitness>(opt);
    println!("Doubled Option: {:?}", proc_opt);
    assert_eq!(proc_opt, Some(15));

    // Using triple_value with Result
    let res = Ok(5);
    println!("Original Result: {:?}", res);
    let proc_res = triple_value::<ResultWitness<i32>>(res);
    println!("Doubled Result: {:?}", proc_res);
    assert_eq!(proc_res, Ok(15));

    // Using triple_value with CausalTensor
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    println!("Original CausalTensor: {:?}", tensor);
    let proc_tensor = triple_value::<CausalTensorWitness>(tensor);
    println!("Doubled CausalTensor: {:?}", proc_tensor);
    assert_eq!(proc_tensor.data(), &[3, 6, 9]);
}
```
Functional composition of HKS tensors works best via an effect system that captures side effects and provides detailed errors and logs for each processing step. In the example below, Tensors are composed and the container MyMonadEffect3 capture the final tensor value, optional errors, and detailed logs from each processing step. 

```rust
    // ... Truncated  
      
    // 4. Chain Operations using Monad::bind
    println!("Processing steps...");
    let final_effect = MyMonadEffect3::bind(initial_effect, step1);
    let final_effect = MyMonadEffect3::bind(final_effect, step2);
    let final_effect = MyMonadEffect3::bind(final_effect, step3);

    println!();
    println!("--- Final Result ---");
    println!("Final CausalTensor: {:?}", final_effect.value);
    println!("Error: {:?}", final_effect.error);
    println!("Logs: {:?}", final_effect.logs);
```

For complex data processing pipelines, these information are invaluable for debugging and optimization. Also, in case more detailed information are required i.e. processing time for each step, then an Effect Monad of arity 4 or 5 can be used to capture additional fields at each step.


## Performance

The following benchmarks were run on a `CausalTensor` of size 100x100 (10,000 `f64` elements).

| Operation                     | Time       | Notes                                      |
|-------------------------------|------------|--------------------------------------------|
| `tensor_get`                  | ~2.31 ns   | Accessing a single element.                |
| `tensor_reshape`              | ~2.46 µs   | Metadata only, but clones data in the test.|
| `tensor_scalar_add`           | ~4.95 µs   | Element-wise addition with a scalar.       |
| `tensor_tensor_add_broadcast` | ~46.67 µs  | Element-wise addition with broadcasting.   |
| `tensor_sum_full_reduction`   | ~10.56 µs  | Summing all 10,000 elements of the tensor. |

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

## References

The tensor-network (tensor-train / MPS–MPO) layer implements the following algorithms. Each is also
cited in the doc comment of the source file where it is implemented.

**Decomposition & numerics**
- I. V. Oseledets, "Tensor-train decomposition," *SIAM J. Sci. Comput.* 33(5), 2295–2317 (2011).
  [doi:10.1137/090752286](https://doi.org/10.1137/090752286) — TT format and sequential-SVD construction (`from_dense`).
- J. Demmel and K. Veselić, "Jacobi's method is more accurate than QR," *SIAM J. Matrix Anal. Appl.*
  13(4), 1204–1245 (1992). [doi:10.1137/0613074](https://doi.org/10.1137/0613074) — high-relative-accuracy one-sided Jacobi SVD.
- G. H. Golub and C. F. Van Loan, *Matrix Computations*, 4th ed. (Johns Hopkins Univ. Press, 2013),
  §5.2 — Householder QR factorization (canonicalization).

**Cross approximation**
- I. V. Oseledets and E. E. Tyrtyshnikov, "TT-cross approximation for multidimensional arrays,"
  *Linear Algebra Appl.* 432(1), 70–88 (2010). [doi:10.1016/j.laa.2009.07.024](https://doi.org/10.1016/j.laa.2009.07.024) — black-box TT-cross (`cross`, `apply_nonlinear`).
- S. A. Goreinov, I. V. Oseledets, D. V. Savostyanov, E. E. Tyrtyshnikov, and N. L. Zamarashkin,
  "How to find a good submatrix," in *Matrix Methods: Theory, Algorithms and Applications* (World
  Scientific, 2010), pp. 247–256 — the maxvol index-selection principle.

**Solvers**
- S. Holtz, T. Rohwedder, and R. Schneider, "The alternating linear scheme for tensor optimization in
  the tensor train format," *SIAM J. Sci. Comput.* 34(2), A683–A713 (2012).
  [doi:10.1137/100818893](https://doi.org/10.1137/100818893) — one-site ALS (`solve::fit`).
- L. Grasedyck, M. Kluge, and S. Krämer, "Variants of alternating least squares tensor completion in
  the tensor train format," *SIAM J. Sci. Comput.* 37(5), A2424–A2450 (2015).
  [doi:10.1137/130942401](https://doi.org/10.1137/130942401) — ALS tensor completion from samples (`solve::fit`).
- S. V. Dolgov and D. V. Savostyanov, "Alternating minimal energy methods for linear systems in higher
  dimensions," *SIAM J. Sci. Comput.* 36(5), A2248–A2271 (2014).
  [doi:10.1137/140953289](https://doi.org/10.1137/140953289) (arXiv:1301.6068) — AMEn rank-adaptive linear solver (`solve::linear`).

**Roadmap (Stage 3+)**
- C. Hubig, I. P. McCulloch, U. Schollwöck, and F. A. Wolf, "Strictly single-site DMRG algorithm with
  subspace expansion," *Phys. Rev. B* 91, 155115 (2015).
  [doi:10.1103/PhysRevB.91.155115](https://doi.org/10.1103/PhysRevB.91.155115) (arXiv:1501.05504) — DMRG3S eigensolver.
- J. Gleis, J.-W. Li, and J. von Delft, "Controlled bond expansion for density matrix renormalization
  group ground state search at single-site costs," *Phys. Rev. Lett.* 130, 246402 (2023)
  (arXiv:2207.14712); and CBE-TDVP, *Phys. Rev. Lett.* 133, 026401 (2024) — controlled bond expansion.
- S. Paeckel, T. Köhler, A. Swoboda, S. R. Manmana, U. Schollwöck, and C. Hubig, "Time-evolution
  methods for matrix-product states," *Ann. Phys.* 411, 167998 (2019).
  [doi:10.1016/j.aop.2019.167998](https://doi.org/10.1016/j.aop.2019.167998) (arXiv:1901.05824) — TDVP and time-evolution review.

## Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).
