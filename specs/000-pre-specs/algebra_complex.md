# Migration Plan: Refactoring `Complex<T>` to Use Algebraic Traits

## 1. Objective

This document outlines the migration plan to refactor the `Complex<F>` number type in the `deep_causality_num` crate. The goal is to move from a generic implementation over a concrete `F: Float` trait to a more abstract and mathematically correct implementation over `T: RealField`, fully integrating with the new algebraic traits hierarchy.

## 2. Motivation

The current implementation of `Complex<F: Float>` and its associated `impl Float for Complex<F>` is problematic for several reasons:
-   **Incorrect Abstraction:** Complex numbers are not "floats." They form a field, but they lack properties like a total ordering, which the `Float` trait implies (e.g., via `min`, `max`). This leads to ambiguous or incorrect semantics.
-   **Mathematically Ill-defined Operations:** The current implementation includes a component-wise `Rem` (remainder) operation, which is not a standard operation for complex numbers. This will be removed.
-   **Ambiguous Power Function:** The current `powf` is ambiguous. A distinction must be made between raising a complex number to a real power (`z^x`) and a complex power (`z^w`), as they have different definitions.
-   **Lack of Generality:** Tying the implementation to the `Float` trait prevents it from being used with other potential real number types that might implement `RealField`.
-   **Code Fragmentation:** The implementation is spread across many small files, making it difficult to understand and maintain.

This migration will address these issues by improving correctness, enhancing abstraction, and consolidating the code.

## 3. Proposed File Structure

The contents of the `deep_causality_num/src/complex/complex_number/` directory will be refactored. The directory name **will remain the same**, but its internal structure will be simplified as follows:

```
deep_causality_num/src/complex/
└── complex_number/
    ├── mod.rs           # Core `Complex<T>` struct definition, type aliases, and module declarations.
    ├── complex_impl.rs  # Inherent methods (`new`, `norm`, `conj`, `sqrt`, `exp`, etc.).
    ├── algebra.rs       # Implementations of algebraic traits (`Field`, `DivisionAlgebra`, etc.).
    ├── ops.rs           # Implementations for `Add`, `Sub`, `Mul`, `Div`, `Neg` and `*Assign` traits.
    ├── cast.rs          # Implementations for `ToPrimitive`, `FromPrimitive`, `NumCast`.
    ├── identity.rs      # Implementations for `One` and `Zero`.
    └── fmt.rs           # Implementations for `Debug` and `Display`.
```

## 4. Migration Steps

### Phase 1: Source Code Restructuring

1.  **Delete Obsolete Files:**
    -   Within `deep_causality_num/src/complex/complex_number/`: Delete all `.rs` files except for `mod.rs` and `identity.rs`. This includes `float.rs`, `part_ord.rs`, `complex_number_impl.rs`, `arithmetic.rs`, etc.

2.  **Create New Files:**
    -   Within `deep_causality_num/src/complex/complex_number/`, create the new files: `complex_impl.rs`, `algebra.rs`, `ops.rs`, `cast.rs`, and `fmt.rs`.

3.  **Update `mod.rs`:**
    -   Remove the `ComplexNumber` trait definition.
    -   Update module declarations to match the new file structure.
    -   **Important:** The `impl<F> Num for Complex<F>` will be removed, as `Complex` will no longer support the `Rem` operation required by the `Num` trait.

### Phase 2: Core `Complex<T>` Refactoring

1.  **Update Struct Definition (`mod.rs`):**
    -   Modify the `Complex<T>` struct to use the `RealField` generic bound. Ensure `PartialOrd` is not derived.
    ```rust
    // In deep_causality_num/src/complex/complex_number/mod.rs
    use crate::RealField;

    #[derive(Copy, Clone, PartialEq, Default, Debug)]
    pub struct Complex<T: RealField> {
        pub re: T,
        pub im: T,
    }

    pub type Complex32 = Complex<f32>;
    pub type Complex64 = Complex<f64>;
    ```

2.  **Consolidate Inherent Methods (`complex_impl.rs`):**
    -   Create a single `impl<T: RealField> Complex<T>` block.
    -   **Move Core Logic:** Consolidate methods from old files (`constructors.rs`, `complex_number_impl.rs`).
    -   **Implement Safe `inverse()`:** Add a robust `inverse()` method that handles the zero case gracefully, returning `NaN` components to prevent a panic, consistent with IEEE 754 behavior.
        ```rust
        pub fn inverse(&self) -> Self {
            if self.is_zero() {
                return Self::new(T::nan(), T::nan());
            }
            let norm_sq = self.norm_sqr();
            let inv_norm_sq = norm_sq.inverse(); // Uses RealField -> Field -> MulGroup -> inverse()
            Self {
                re: self.re * inv_norm_sq,
                im: -self.im * inv_norm_sq,
            }
        }
        ```
    -   **Implement Power Functions:** Create two distinct, correctly defined power functions. The existing `powi` can be retained for integer exponents.
        ```rust
        // Raises to a REAL power
        pub fn powf(&self, n: T) -> Self {
            // (r * (cos(t) + i*sin(t)))^n = r^n * (cos(n*t) + i*sin(n*t))
            let r_pow_n = self.norm().powf(n);
            let n_theta = self.arg() * n;
            Self::new(r_pow_n * n_theta.cos(), r_pow_n * n_theta.sin())
        }

        // Raises to a COMPLEX power
        pub fn powc(&self, n: Self) -> Self {
            // z^w = exp(w * ln(z))
            (n * self.ln()).exp()
        }
        ```
    -   **Move Other Math Functions:** Move all other methods from the old `float.rs` (`sqrt`, `exp`, `ln`, `sin`, etc.) into this `impl` block, updating their logic to use methods from `T: RealField`.

### Phase 3: Operator and Standard Trait Implementation

1.  **Implement Operators (`ops.rs`):**
    -   Consolidate implementations for `Add`, `Sub`, `Mul`, `Div`, and `Neg` (and their `*Assign` variants) into this file.
    -   **Crucially, DO NOT implement `Rem` or `RemAssign`.**
    -   Update the generic bound from `<F: Float>` to `<T: RealField>`.
    -   Add implementations for scalar multiplication: `impl<T: RealField> Mul<T> for Complex<T>` and `MulAssign<T>`.

2.  **Implement Other Traits:**
    -   **`cast.rs`:** Consolidate `ToPrimitive`, `FromPrimitive`, and `NumCast` implementations.
    -   **`identity.rs`:** Update `One` and `Zero` trait implementations.
    -   **`fmt.rs`:** Consolidate `Debug` and `Display` trait implementations.

### Phase 4: Algebraic Trait Implementation

1.  **Implement Algebraic Traits (`algebra.rs`):**
    -   This file will connect `Complex<T>` to the algebraic hierarchy.

    ```rust
    // In deep_causality_num/src/complex/complex_number/algebra.rs
    use crate::{
        AbelianGroup, AssociativeDivisionAlgebra, AssociativeRing, Complex, DivisionAlgebra, Field, MulGroup, RealField
    };

    impl<T: RealField> AbelianGroup for Complex<T> {}
    impl<T: RealField> AssociativeRing for Complex<T> {}
    impl<T: RealField> Field for Complex<T> {}
    impl<T: RealField> AssociativeDivisionAlgebra<T> for Complex<T> {}

    // The inverse() method on the MulGroup trait must delegate to our safe, inherent implementation.
    impl<T: RealField> MulGroup for Complex<T> {
        fn inverse(&self) -> Self {
            self.inverse()
        }
    }

    impl<T: RealField> DivisionAlgebra<T> for Complex<T> {
        fn inverse(&self) -> Self {
            self.inverse()
        }
    }
    ```

### Phase 5: Test Code Migration

1.  **Delete Obsolete Tests:**
    -   In `deep_causality_num/tests/complex/complex_number/`:
        -   Delete `part_ord_tests.rs`.
        -   Delete `float_tests.rs`.

2.  **Update Existing Tests:**
    -   Create `complex_impl_tests.rs` and move relevant tests for math functions (e.g., `sqrt`, `exp`, power functions) into it.
    -   Go through `arithmetic_tests.rs` and `arithmetic_assign_tests.rs` and **delete all tests related to `Rem` (%)**.
    -   Add a new test case to verify that `Complex::zero().inverse()` returns `NaN` components as expected.
    -   Update all remaining test files to remove `use deep_causality_num::ComplexNumber;` and ensure they compile and pass against the new inherent methods.

## 6. Validation

-   **Compilation:** The primary validation will be a successful compilation of the `deep_causality_num` crate.
-   **Existing Tests:** All updated tests should pass, ensuring no regressions in functionality.
-   **New Tests:** New tests for the `inverse` of zero and the distinct `powf`/`powc` methods will confirm the correctness of the new logic.