# Plan for Algebraic Trait Refactoring

This document outlines the plan to define new algebraic traits (`DivisionAlgebra`, `AssociativeRing`, `AssociativeAlgebra`) and refactor existing number types (`Complex`, `Quaternion`, `Octonion`) in the `deep_causality_num` crate to implement these and other relevant algebraic traits.

## High-Level Goals

*   Define new algebraic traits for `AssociativeRing`, `AssociativeAlgebra`, and `DivisionAlgebra`.
*   Ensure `Complex` numbers correctly implement the `Field` trait.
*   Ensure `Quaternion` numbers correctly implement the `AssociativeRing` and `AssociativeAlgebra` traits.
*   Ensure `Octonion` numbers correctly implement the `DivisionAlgebra` trait.
*   Update `Octonion` struct fields for consistency (`c0` to `c7`).
*   Integrate and verify all changes.

## Detailed Plan

### Phase 1: Trait Definitions

1.  **Define `AssociativeRing` Trait**
    *   **File:** `deep_causality_num/src/algebra/ring_associative.rs`
    *   **Purpose:** To define properties of a ring where multiplication is associative.
    *   **Dependencies:** Will likely extend the existing `Ring` trait (if one exists, otherwise define `Ring` first) and `core::ops::{Add, Mul, Sub, AddAssign, MulAssign, SubAssign}`.
    *   **Key Methods/Bounds:**
        *   Inherit `Add`, `Mul`, `Sub` (from `Ring`).
        *   Require `AddAssign`, `MulAssign`, `SubAssign`.
        *   Potentially add `one()` and `zero()` for identity elements.
        *   Formalize associativity for multiplication: `(a * b) * c == a * (b * c)`.

2.  **Define `AssociativeAlgebra` Trait**
    *   **File:** `deep_causality_num/src/algebra/algebra_associative.rs`
    *   **Purpose:** To define properties of an associative algebra over a `Field` (scalar multiplication).
    *   **Dependencies:** Will extend `AssociativeRing` and potentially `Module` (if an existing module trait is suitable, otherwise define one).
    *   **Key Methods/Bounds:**
        *   Inherit `AssociativeRing` properties.
        *   Require scalar multiplication (e.g., `Mul<Scalar, Output = Self>`).
        *   Define bounds for the `Scalar` type (e.g., `Field` or `RealField`).

3.  **Define `DivisionAlgebra` Trait**
    *   **File:** `deep_causality_num/src/algebra/algebra_division.rs`
    *   **Purpose:** To define properties of an algebra where every non-zero element has a multiplicative inverse.
    *   **Dependencies:** Will extend `AssociativeAlgebra` and `core::ops::Div`.
    *   **Key Methods/Bounds:**
        *   Inherit `AssociativeAlgebra` properties.
        *   Require an `inverse()` method that returns `Option<Self>` or `Self` (handling zero division).
        *   Formalize division property: `a * a.inverse() == One`.
        *   The scalar field for the algebra must itself be a `Field`.

### Phase 2: Refactor Octonion for `DivisionAlgebra`

1.  **Modify `Octonion` Struct Definition:**
    *   **File:** `deep_causality_num/src/complex/octonion_number/mod.rs`
    *   **Change:** Rename fields from `s`, `e1`, `e2`, ..., `e7` to `c0`, `c1`, `c2`, ..., `c7` respectively, to match the user's suggestion.
    *   **Impact:** Update all associated `impl` blocks (`new`, arithmetic, debug, display, etc.) to use the new field names.

2.  **Implement `DivisionAlgebra` for `Octonion<F>`:**
    *   **File:** `deep_causality_num/src/complex/octonion_number/octonion_algebra_impl.rs` (new file)
    *   **Implementation:** Implement the `DivisionAlgebra` trait for `Octonion<F>`. This will largely involve leveraging existing `conjugate`, `norm_sqr`, `inverse`, and arithmetic operations already defined.
    *   **Bounds:** Ensure `F` implements `RealField` (or `Field` if `RealField` implies it).

### Phase 3: Refactor Quaternion for `AssociativeRing` / `AssociativeAlgebra`

1.  **Implement `AssociativeRing` for `Quaternion<F>`:**
    *   **File:** `deep_causality_num/src/complex/quaternion_number/quaternion_algebra_impl.rs` (new file)
    *   **Implementation:** Implement the `AssociativeRing` trait. Quaternions' multiplication is already associative, so this will mostly involve declaring the implementation and ensuring `Mul` is correctly handled.
    *   **Bounds:** Ensure `F` implements `RealField`.

2.  **Implement `AssociativeAlgebra` for `Quaternion<F>`:**
    *   **File:** `deep_causality_num/src/complex/quaternion_number/quaternion_algebra_impl.rs`
    *   **Implementation:** Implement the `AssociativeAlgebra` trait, leveraging the `AssociativeRing` and scalar multiplication.
    *   **Bounds:** Ensure `F` implements `RealField`.

### Phase 4: Refactor Complex for `Field`

1.  **Implement `Field` for `Complex<F>`:**
    *   **File:** `deep_causality_num/src/complex/complex_number/complex_algebra_impl.rs` (new file)
    *   **Implementation:** Ensure all methods required by the `Field` trait (from `deep_causality_num/src/algebra/field.rs`) are properly implemented or delegated. This includes `zero`, `one`, `add`, `sub`, `mul`, `div`, and `inverse`. Complex numbers already have these operations.
    *   **Bounds:** Ensure `F` implements `RealField`.

### Phase 5: Integrate and Verify

1.  **Update `deep_causality_num/src/algebra/mod.rs`:**
    *   Add `pub mod ring_associative;`, `pub mod algebra_associative;`, `pub mod algebra_division;`.
2.  **Update `deep_causality_num/src/complex/octonion_number/mod.rs`, `quaternion_number/mod.rs`, `complex_number/mod.rs`**:
    *   Add `use` statements for the new algebraic traits.
    *   Add new `mod` statements for the `*_algebra_impl.rs` files.
3.  **Run `cargo check -p deep_causality_num`:** Check for compilation errors across the crate.
4.  **Run `cargo test -p deep_causality_num`:** Verify existing functionality and ensure new trait implementations are sound.
5.  **Add Unit Tests (if necessary):** Create new tests for the specific algebraic properties (associativity, inverse existence) in the respective `tests` directories.

---
This plan addresses the introduction of new algebraic traits and their implementation across `Complex`, `Quaternion`, and `Octonion` types, ensuring consistency with mathematical definitions.