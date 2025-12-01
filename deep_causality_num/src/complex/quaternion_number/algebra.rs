/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    AbelianGroup, Associative, Distributive, DivisionAlgebra, MulGroup, Quaternion, RealField,
};

// | Type | `Distributive` | `Associative` | `Commutative` | Trait |
// | :--- | :---: | :---: | :---: | :--- |
// | **Quaternion** | ✅ | ✅ | ❌ | `AssociativeRing` |

// Marker Traits
impl<T: RealField> Associative for Quaternion<T> {}
impl<T: RealField> Distributive for Quaternion<T> {}

impl<T: RealField> AbelianGroup for Quaternion<T> {}

// Required by Field -> CommutativeRing -> Ring -> MulMonoid -> MulGroup
impl<T: RealField> MulGroup for Quaternion<T> {
    fn inverse(&self) -> Self {
        self.inverse()
    }
}

impl<T: RealField> DivisionAlgebra<T> for Quaternion<T> {
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
