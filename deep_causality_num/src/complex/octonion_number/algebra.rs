/// Implements various algebraic traits for `Octonion<T>`.
///
/// | Type | `Distributive` | `Associative` | `Commutative` | Trait |
/// | :--- | :---: | :---: | :---: | :--- |
/// | **Octonion** | ✅ | ❌ | ❌ | `DivisionAlgebra` |
///
/// This file is responsible for binding the `Octonion` type to the algebraic
/// hierarchy defined in the `deep_causality_num/src/algebra/` module.
/// It explicitly states the algebraic properties of octonions, such as being
/// a distributive and abelian group, and a division algebra, while correctly
/// identifying their non-associative and non-commutative nature.
use crate::{AbelianGroup, Distributive, DivisionAlgebra, Octonion, RealField};

// Marker Traits
/// Implements the `Distributive` marker trait for `Octonion`.
///
/// This signifies that multiplication of `Octonion`s distributes over addition,
/// i.e., `a * (b + c) = (a * b) + (a * c)`. This property holds for octonions.
impl<T: RealField> Distributive for Octonion<T> {}

// DO NOT IMPLEMENT `Associative` as octonion multiplication is non-associative.
// DO NOT IMPLEMENT `Commutative` as octonion multiplication is non-commutative.

// Octonion addition is component-wise and commutative.
/// Implements the `AbelianGroup` trait for `Octonion`.
///
/// This signifies that `Octonion`s form an abelian (commutative) group under addition.
/// Addition is component-wise, ensuring commutativity and associativity.
impl<T: RealField> AbelianGroup for Octonion<T> {}

// Octonions are a Division Algebra, but NOT associative.
// The blanket impl for `Algebra<T>` will apply, since we can satisfy its
// bounds: `Module<T> + Mul<...> + MulAssign + One + Distributive`.
/// Implements the `DivisionAlgebra` trait for `Octonion`.
///
/// Octonions form a non-associative division algebra over the real numbers.
/// This trait provides methods for `conjugate`, `norm_sqr`, and `inverse`,
/// which are fundamental to division algebras.
impl<T: RealField> DivisionAlgebra<T> for Octonion<T> {
    /// Computes the conjugate of the octonion.
    ///
    /// The conjugate of an octonion `s + v` is `s - v`, where `s` is the scalar part
    /// and `v` is the vector part. All imaginary components are negated.
    ///
    /// # Returns
    /// A new octonion representing the conjugate of `self`.
    fn conjugate(&self) -> Self {
        self.conjugate()
    }

    /// Computes the squared norm (magnitude squared) of the octonion.
    ///
    /// The squared norm is the sum of the squares of all eight components:
    /// `s² + e₁² + e₂² + e₃² + e₄² + e₅² + e₆² + e₇²`.
    ///
    /// # Returns
    /// The scalar value `T` representing the squared norm.
    fn norm_sqr(&self) -> T {
        self.norm_sqr()
    }

    /// Computes the multiplicative inverse of the octonion.
    ///
    /// For a non-zero octonion `o`, its inverse `o⁻¹` is defined as
    /// `conjugate(o) / norm_sqr(o)`.
    ///
    /// # Returns
    /// A new octonion representing the inverse of `self`.
    fn inverse(&self) -> Self {
        self.inverse()
    }
}

// Octonions do not form a multiplicative group because multiplication is not associative.
// However, non-zero octonions do have multiplicative inverses.
// The `MulGroup` trait requires associativity, so it is not implemented.
