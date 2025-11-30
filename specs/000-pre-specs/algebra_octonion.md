# Octonion Algebra Specification

## Overview

This document specifies the algebraic structure and behavior of the `Octonion<F>` type within the `deep_causality_num` crate. Octonions are an 8-dimensional non-associative, non-commutative normed division algebra over the real numbers. They are constructed via the Cayley-Dickson process, extending quaternions.

### Key Characteristics:
- **Non-Associative Multiplication**: For octonions `a, b, c`, generally `(a * b) * c != a * (b * c)`. This is a fundamental departure from real, complex, and quaternion numbers.
- **Non-Commutative Multiplication**: For octonions `a, b`, generally `a * b != b * a`.
- **Normed Division Algebra**: Every non-zero octonion has a unique multiplicative inverse, and there is a well-defined norm.
- **Alternativity**: Octonion multiplication is alternative, meaning `(a * a) * b = a * (a * b)` and `(a * b) * b = a * (b * b)`. This weaker form of associativity is maintained.

## Algebraic Trait Implementations

The `Octonion<F>` type is designed to integrate with the algebraic trait hierarchy defined in `deep_causality_num::algebra`. The generic parameter `F` is constrained by the `Float` trait, which encapsulates the necessary real field properties for the components of the octonion.

### Implemented Traits:

-   **`Num`**: Implemented directly for `Octonion<F>`, indicating it behaves like a generic number.
-   **`Zero`**: Octonions have an additive identity (`0`).
-   **`One`**: Octonions have a multiplicative identity (`1`).
-   **`AddGroup`**: Octonions form a group under addition, which is component-wise.
-   **`AbelianGroup`**: Octonion addition is commutative, making it an Abelian group under addition.
-   **`AddMonoid`**: Octonions form a monoid under addition.
-   **`MulMonoid`**: Octonions technically form a monoid under multiplication in the Rust type system, as it requires the existence of a multiplicative identity (`One`) and a multiplication operator (`MulAssign`). However, mathematically, general octonion multiplication is **not associative**. This is a distinction between the formal Rust trait implementation due to blanket impls and the underlying mathematical properties.
-   **`Ring`**: Octonions technically satisfy the Rust `Ring` trait, which is blanket-implemented for types satisfying `AbelianGroup` and `MulMonoid`. However, it's crucial to note that mathematically, Octonions are a **non-associative ring** as their multiplication is not associative.
-   **`Algebra<F>`**: Octonions form an algebra over their scalar field `F` (where `F` is a `Float`, which implies `Ring` properties). Similar to `Ring` and `MulMonoid`, while technically satisfying the Rust trait bounds, the inherent non-associativity of octonion multiplication means it is a **non-associative algebra**.
-   **`DivisionAlgebra<F>`**: Every non-zero octonion has a multiplicative inverse. This is a core property of octonions, and the scalar field `F` (which is a `Float` and thus a `Field`) supports the required division operations.

### Traits NOT Implemented (due to Octonion properties):

-   **`AssociativeRing`**: Octonion multiplication is fundamentally non-associative. While a blanket implementation for `Ring` might technically apply this trait in Rust, mathematically this property is not held by octonions.
-   **`CommutativeRing`**: Octonion multiplication is not commutative.
-   **`Field`**: Due to non-commutativity and non-associativity of multiplication, octonions do not form a field.
-   **`AssociativeAlgebra<F>`**: As octonion multiplication is not associative, it cannot form an associative algebra. This trait specifically marks algebras with associative multiplication.
-   **`AssociativeDivisionAlgebra<F>`**: This trait combines `AssociativeAlgebra` and `DivisionAlgebra`. Since octonions are not associative, they cannot be an associative division algebra, even though they are a division algebra.

## Inherent Methods

The `Octonion<F>` struct provides the following inherent methods:

-   **`Octonion::new(s: F, e1: F, e2: F, e3: F, e4: F, e5: F, e6: F, e7: F) -> Self`**: Constructs a new Octonion from its 8 scalar components.
-   **`Octonion::identity() -> Self`**: Returns the multiplicative identity octonion (1 + 0e₁ + ...).
-   **`fn conjugate(&self) -> Self`**: Computes the conjugate of the octonion (negates imaginary parts).
-   **`fn norm_sqr(&self) -> F`**: Calculates the squared Euclidean norm of the octonion (`s^2 + e1^2 + ... + e7^2`).
-   **`fn norm(&self) -> F`**: Calculates the Euclidean norm (magnitude) of the octonion.
-   **`fn normalize(&self) -> Self`**: Returns a unit octonion (norm = 1), or the original if the norm is zero.
-   **`fn inverse(&self) -> Self`**: Computes the multiplicative inverse (`conjugate() / norm_sqr()`). Returns NaN components if `norm_sqr` is zero.
-   **`fn dot(&self, other: &Self) -> F`**: Computes the standard Euclidean dot product of two octonions.

## Operator Overloads

Standard arithmetic and assignment operators are overloaded for `Octonion<F>`.

-   **`Add` (`+`)**: Component-wise addition.
-   **`Sub` (`-`)**: Component-wise subtraction.
-   **`Mul` (`*`)**:
    -   `Octonion<F> * Octonion<F>`: Octonion multiplication (Cayley-Dickson product), which is non-commutative and non-associative.
    -   `Octonion<F> * F`: Scalar multiplication (each component by the scalar).
-   **`Div` (`/`)**:
    -   `Octonion<F> / Octonion<F>`: Right division (`self * other.inverse()`). Note that left division (`other.inverse() * self`) would yield a different result due to non-commutativity.
    -   `Octonion<F> / F`: Scalar division (each component by the scalar).
-   **`Neg` (`-` unary)**: Component-wise negation.
-   **`Rem` (`%`)**: Placeholder behavior (returns `self`).
-   **`AddAssign` (`+=`)**: In-place component-wise addition.
-   **`SubAssign` (`-=`)**: In-place component-wise subtraction.
-   **`MulAssign` (`*=`)**: In-place octonion multiplication.
-   **`DivAssign` (`/=`)**: In-place octonion right division.
-   **`RemAssign` (`%=`)**: Placeholder behavior (does nothing).

## Casting and Utility Traits

-   **`FromPrimitive`**: Allows conversion from primitive numeric types (e.g., `i332`, `f64`) to `Octonion<F>`, initializing the scalar part and setting imaginary parts to zero.
-   **`ToPrimitive`**: Allows conversion from the scalar part of `Octonion<F>` to primitive numeric types, returning `Option<T>`.
-   **`AsPrimitive`**: Converts the scalar part of `Octonion<F>` to another primitive type `U` directly (`self.s.as_()`).
-   **`NumCast`**: Provides a generic casting mechanism from `ToPrimitive` types to `Octonion<F>`, populating the scalar part.
-   **`Sum`**: Enables summing an iterator of `Octonion<F>` instances.
-   **`Product`**: Enables multiplying an iterator of `Octonion<F>` instances (sequential left-to-right multiplication). Note: due to non-associativity, the result can depend on the order of operations if the `Product` trait implementation isn't carefully considered to match a specific mathematical convention (e.g., left-to-right).
-   **`PartialEq`**: Implements component-wise equality comparison.
-   **`PartialOrd`**: Provides a lexicographical partial ordering. It's important to note this is for utility and does not imply a standard mathematically meaningful ordering for octonions.

## Migration Plan

The existing `Octonion` implementation correctly adheres to its mathematical properties, especially non-associativity and non-commutativity. The primary migration tasks revolve around clarifying the trait hierarchy to align with these mathematical realities and to provide robust integration with higher-level algebraic structures.

1.  **Review `OctonionNumber` Trait Definition:**
    -   **Current**: `pub trait OctonionNumber<F>: Num + Sized ...`
    -   **Recommendation**: Keep `Num` as a supertrait. The `Num` trait (`PartialEq + Zero + One + NumOps`) is correctly implemented by `Octonion<F>` and `F` (as `Float` implements `Num`), making `Octonion` a valid "number" within the system. The previous instruction was based on a misunderstanding that `Complex` does not implement `Num`, which it does.
    -   **Action**: No change needed for the `Num` supertrait on `OctonionNumber`.

2.  **Explicitly Handle Non-Associativity and Non-Commutativity in Traits:**
    -   The current Rust trait hierarchy for algebraic structures uses blanket implementations that implicitly apply `AssociativeRing`, `AssociativeAlgebra`, `CommutativeRing`, and `Field` based on simpler supertraits like `Ring` and `MulMonoid`.
    -   **Problem**: Octonions do not satisfy the mathematical axioms for associativity or commutativity of multiplication, despite `Octonion<F>` technically satisfying the Rust type bounds that lead to these blanket implementations. This creates a disconnect between the Rust type system and mathematical truth.
    -   **Recommendation**: While it's difficult to prevent blanket implementations from applying without fundamental changes to the trait hierarchy, the documentation (`algebra_octonion.md`) must clearly state this discrepancy.
    -   **Action**: Explicitly add `AssociativeRing`, `CommutativeRing`, `Field`, `AssociativeAlgebra<F>`, and `AssociativeDivisionAlgebra<F>` to the "Traits NOT Implemented" section, with a detailed explanation of why these are mathematically inappropriate for Octonions, even if Rust's blanket implementations might technically apply them. Update the descriptions of `MulMonoid`, `Ring`, and `Algebra<F>` in "Implemented Traits" to highlight their non-associative nature for Octonions.

3.  **Doc-Test and Example Updates**:
    -   **Recommendation**: Ensure all doc-tests and examples in the `octonion_number` module clearly demonstrate the non-commutative (`a * b != b * a`) and non-associative `((a * b) * c != a * (b * (c)))` properties of octonion multiplication where applicable.
    -   **Action**: Review existing examples and add new ones that explicitly illustrate these properties.

4.  **Review `CausalMultiVector<T>` Integration**:
    -   **Problem**: The `deep_causality_multivector` crate explicitly checks for `AssociativeRing` when performing `geometric_product_general` and notes that `Octonion<f64>` does *not* implement `AssociativeRing`. This suggests a proper mathematical understanding of Octonions, but it also conflicts with the blanket implementation applying `AssociativeRing` through `Ring`.
    -   **Recommendation**: This indicates a deeper structural problem where Rust's blanket implementations for algebraic traits might be overreaching for higher-order algebras. For future refactoring, consider ways to explicitly *opt-out* of blanket trait implementations or introduce more granular traits for associativity/commutativity. For now, the current blocking in `CausalMultiVector` for `Octonion` is appropriate and aligns with the mathematical properties.
    -   **Action**: Acknowledge this in the `algebra_octonion.md` and keep the existing `CausalMultiVector` behavior as correct given the mathematical constraints. No direct change to `Octonion` implementation related to `CausalMultiVector` is immediately necessary, but the documentation should reflect the implications.

This migration plan ensures that the `Octonion` type's algebraic properties are accurately represented and understood, bridging the gap between mathematical theory and Rust's trait system.
