/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    AbelianGroup, Associative, Commutative, Complex, Distributive, DivisionAlgebra, MulGroup,
    RealField,
};

// | Type | `Distributive` | `Associative` | `Commutative` | Trait |
// | :--- | :---: | :---: | :---: | :--- |
// | **Complex** | ✅ | ✅ | ✅ | `Field` |

// Marker Traits
impl<T: RealField> Associative for Complex<T> {}
impl<T: RealField> Commutative for Complex<T> {}
impl<T: RealField> Distributive for Complex<T> {}

impl<T: RealField> AbelianGroup for Complex<T> {}

// The blanket impls for AssociativeRing, Field, and AssociativeDivisionAlgebra
// will apply automatically as Complex<T> now satisfies their super-traits.

// Required by Field -> CommutativeRing -> Ring -> MulMonoid -> MulGroup
// This delegates to the safe, inherent `inverse` method via the `Div` trait.
impl<T: RealField> MulGroup for Complex<T> {
    fn inverse(&self) -> Self {
        self.inverse()
    }
}

// Implement all methods for DivisionAlgebra, delegating to inherent methods.
impl<T: RealField> DivisionAlgebra<T> for Complex<T> {
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
