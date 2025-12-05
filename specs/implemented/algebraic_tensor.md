# Algebraic Tensor Specification

## Goal
Implement algebraic traits for `CausalTensor` to enable seamless composition with other algebraic types (like `CausalMultiVector`, `Complex`, `Quaternion`, `Octonion`) and to support recursive stacked types (e.g., `CausalMultiVector<CausalTensor<Complex<f64>>>`).

## Background
The `deep_causality_num` crate defines a hierarchy of algebraic traits:
- **Group Theory:** `Magma`, `Semigroup`, `Monoid`, `Group`, `AbelianGroup`, `AddGroup`.
- **Ring Theory:** `Ring`, `AssociativeRing`, `CommutativeRing`, `Field`.
- **Module Theory:** `Module<S>`.

Due to HKT constraints requiring `T` to be unbound in the struct definition, `CausalMultiVector` implements these traits "indirectly". This means:
1.  **Methods:** Algebraic operations (`add`, `sub`, `scale`, etc.) are implemented as methods on the struct within `impl` blocks that have specific bounds (e.g., `impl<T> CausalMultiVector<T> where T: AddGroup`).
2.  **Standard Ops:** Standard Rust traits (`Add`, `Sub`, `Mul`, `Div`) are implemented by calling these methods.
3.  **Algebraic Traits:** The high-level algebraic traits (`AddGroup`, `Ring`, etc.) are then implemented (where possible) or the type is structured to satisfy the requirements of other types that expect these traits (via the methods and standard ops).

We will apply this same pattern to `CausalTensor`.

## Proposed Changes

We will implement the following traits for `CausalTensor<T>` in `deep_causality_tensor/src/types/causal_tensor/algebra/` and `deep_causality_tensor/src/types/causal_tensor/arithmetic/`.

### 1. Algebraic Methods (The "Indirect" Implementation)
**File:** `deep_causality_tensor/src/types/causal_tensor/algebra/mod.rs` (plit into files if large)

Implement methods directly on `CausalTensor<T>` with appropriate bounds.

-   **Group Operations:**
    -   `add(&self, rhs: &Self) -> Self` where `T: AddGroup + Copy`
    -   `sub(&self, rhs: &Self) -> Self` where `T: AddGroup + Copy`
    -   `zero(shape: &[usize]) -> Self` (Static constructor, as `Zero` trait might be hard to impl directly without shape context)
-   **Module Operations:**
    -   `scale<S>(&self, scalar: S) -> Self` where `T: Module<S> + Copy`
-   **Ring Operations:**
    -   `mul(&self, rhs: &Self) -> Self` (Hadamard product) where `T: Ring + Copy`
    -   `geometric_product(&self, rhs: &Self) -> Self` (Tensor product or specific tensor algebra product? *Clarification:* Tensor product usually increases rank. If we want a Ring structure, it's usually Hadamard or Matrix multiplication. Given `CausalMultiVector` uses `geometric_product`, we should check if `CausalTensor` has an equivalent or if `mul` is sufficient. For now, we assume Hadamard for `Ring` compatibility, or maybe `ein_sum` for more complex ops. *Decision:* Implement `mul` as element-wise (Hadamard) for Ring structure).

### 2. Standard Ops Implementation
**File:** `deep_causality_tensor/src/types/causal_tensor/arithmetic/mod.rs` (Update existing or add new)

Implement standard Rust traits calling the algebraic methods.

-   `impl<T> Add for CausalTensor<T> where T: AddGroup ...`
-   `impl<T> Sub for CausalTensor<T> where T: AddGroup ...`
-   `impl<T> Mul<S> for CausalTensor<T>` (Scalar mul)
-   `impl<T> Mul for CausalTensor<T>` (Element-wise mul)

### 3. Algebraic Trait Implementations
**File:** `deep_causality_tensor/src/types/causal_tensor/algebra/traits.rs` (New File)

Implement the `deep_causality_num` traits where possible.

-   `impl<T> AddGroup for CausalTensor<T> where T: AddGroup ...`
    -   *Note:* If `AddGroup` requires `zero() -> Self`, we might need to panic or return an empty tensor if shape is unknown, or rely on `Default`.
-   `impl<T> AbelianGroup for CausalTensor<T> where T: AbelianGroup ...` (Marker trait)
-   `impl<T> Ring for CausalTensor<T> where T: Ring ...`
-   `impl<T> Module<S> for CausalTensor<T> where T: Module<S> ...`

## Implementation Details

### Directory Structure
`deep_causality_tensor/src/types/causal_tensor/`
-   `algebra/`
    -   `mod.rs`: Module definition.
    -   `group.rs`: Group methods.
    -   `ring.rs`: Ring methods.
    -   `module.rs`: Module methods.
    -   `traits.rs`: Trait implementations.

### Dependencies
-   `deep_causality_num`: Import traits.

## Verification Plan

### Automated Tests
-   Create `deep_causality_tensor/src/types/causal_tensor/algebra/tests.rs`.
-   **Test Case 1: Complex Number Tensor**
    -   Create `CausalTensor<Complex<f64>>`.
    -   Perform `add`, `sub`, `mul` (element-wise).
    -   Verify results against manual calculation.
-   **Test Case 2: Recursive Tensor (if dependencies allow)**
    -   *Note:* `deep_causality_multivector` depends on `deep_causality_tensor`? If so, we can't test `Tensor<MultiVector>` here.
    -   Instead, we can test `CausalTensor<CausalTensor<f64>>` (Tensor of Tensors) to verify recursion support.
-   **Test Case 3: Algebraic Properties**
    -   Verify `a + b = b + a` (Commutativity).
    -   Verify `a * (b + c) = a * b + a * c` (Distributivity).

### Manual Verification
-   Review the API surface to ensure it matches `deep_causality_num` expectations.
