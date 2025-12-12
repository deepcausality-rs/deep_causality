# DeepCausality Tensor: The Numerical Backbone

`deep_causality_tensor` provides the core multidimensional array primitive (`CausalTensor`) used throughout the ecosystem. It balances performance, safety, and integration with the project's monadic and functional structures (HKT).

---

## 🏗️ The Causal Tensor

The `CausalTensor<T>` is a dense, n-dimensional array backed by a contiguous `Vec<T>`.

### Key Characteristics
*   **Memory Layout**: Row-major (C-style) contiguous logic.
*   **Indexing**: Stride-based indexing for efficient element access.
*   **Ownership**: It is an owned type (owns its data `Vec<T>`), ensuring thread safety and clear ownership semantics, though it lacks the "view" capabilities of `ndarray` to prioritize safety over zero-copy slicing.
*   **Dimensionality**: Optimized for low-to-medium dimensionality (typically 2D-5D for causal models).

### Storage & Performance
Operations fall into two categories:
1.  **Metadata Ops** (`reshape`, `ravel`): Create a new tensor with cloned data but modified shape/strides. This avoids reordering elements in memory.
2.  **Compute Ops** (`add`, `matmul`, `slice`): Allocate new memory for the result.

---

## 🔢 Einstein Summation (EinSum)

A flagship feature is the robust support for **Einstein Summation**, which allows expressing complex tensor contractions in a concise, mathematical string format.

```rust
// Contract dimensions i and j
let result = tensor.ein_sum("ijk, kl -> il", &other_tensor)?;
```

This is critical for:
*   Physics simulations (metric tensor contractions).
*   Causal graph adjacency matrix operations.
*   Geometric Algebra operations.

---

## 🧩 Integration with DeepCausality

Unlike generic tensor libraries (like `ndarray` or `nalgebra`), `CausalTensor` is built to fit into the **Higher-Kinded Type (HKT)** system defined in `deep_causality_haft`.

### HKT Witness
It implements the `CausalTensorWitness` which allows tensors to be treated as generic functors and monads.

```rust
// HKT Magic: Mapping over a tensor generically
let new_tensor = tensor.fmap(|x| x * 2.0);
```

This allows causal algorithms to write generic logic that works on `Option<T>`, `Vec<T>`, or `CausalTensor<T>` interchangeably.

---

## 🛠 Features

*   **Typed**: Strongly typed for `Num + Copy` types (f64, i32, Complex, etc.).
*   **Basic Algebra**: Element-wise arithmetic (`+`, `-`, `*`, `/`).
*   **Linear Algebra**: Matrix multiplication, transposition, dot products.
*   **Stacking**: Concatenation of tensors along axes.
*   **Safety**: Stride calculations are validated at construction to prevent out-of-bounds access.

---

## Summary

`deep_causality_tensor` is not trying to replace `ndarray` or `torch`. It provides a specialized, HKT-compatible tensor primitive that serves as the numerical data carrier for the causal discovery, topology, and physics engines.
