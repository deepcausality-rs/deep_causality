use crate::{AbelianGroup, Distributive, DivisionAlgebra, Octonion, RealField};

// Marker Traits
impl<T: RealField> Distributive for Octonion<T> {}

// DO NOT IMPLEMENT `Associative`
// DO NOT IMPLEMENT `Commutative`

// Octonion addition is component-wise and commutative.
impl<T: RealField> AbelianGroup for Octonion<T> {}

// Octonions are a Division Algebra, but NOT associative.
// The blanket impl for `Algebra<T>` will apply, since we can satisfy its
// bounds: `Module<T> + Mul<...> + MulAssign + One + Distributive`.
impl<T: RealField> DivisionAlgebra<T> for Octonion<T> {
    fn conjugate(&self) -> Self {
        self.conjugate()
    }

    fn norm_sqr(&self) -> T {
        self.norm_sqr()
    }

    fn inverse(&self) -> Self {
        self.inverse()
    }
}

// Octonions do not form a multiplicative group because multiplication is not associative.
// However, non-zero octonions do have multiplicative inverses.
// The `MulGroup` trait requires associativity, so it is not implemented.
