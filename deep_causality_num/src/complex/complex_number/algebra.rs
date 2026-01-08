/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    AbelianGroup, Associative, Commutative, Complex, Distributive, DivisionAlgebra, RealField,
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

// Implement all methods for DivisionAlgebra, delegating to inherent methods.
impl<T: RealField> DivisionAlgebra<T> for Complex<T> {
    fn conjugate(&self) -> Self {
        self._conjugate_impl()
    }

    fn norm_sqr(&self) -> T {
        self._norm_sqr_impl()
    }

    fn inverse(&self) -> Self {
        self._inverse_impl()
    }
}
