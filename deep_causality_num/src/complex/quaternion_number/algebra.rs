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
        self._inverse_impl()
    }
}

impl<T: RealField> DivisionAlgebra<T> for Quaternion<T> {
    /// Returns the conjugate of the quaternion.
    ///
    /// For a quaternion `q = w + xi + yj + zk`, its conjugate is `w - xi - yj - zk`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let conj_q : Quaternion<f64>  = q.conjugate();
    /// assert_eq!(conj_q, Quaternion::new(1.0, -2.0, -3.0, -4.0));
    /// ```
    fn conjugate(&self) -> Self {
        self._conjugate_impl()
    }

    /// Computes the squared norm (magnitude squared) of the quaternion.
    ///
    /// For a quaternion `q = w + xi + yj + zk`, the squared norm is `w^2 + x^2 + y^2 + z^2`.
    /// This method avoids the square root operation, making it more efficient
    /// when only relative magnitudes are needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// assert_eq!(q.norm_sqr(), 1.0*1.0 + 2.0*2.0 + 3.0*3.0 + 4.0*4.0);
    /// ```
    fn norm_sqr(&self) -> T {
        self._norm_sqr_impl()
    }

    /// Returns the inverse of the quaternion.
    ///
    /// For a non-zero quaternion `q`, its inverse `q^-1` is `q.conjugate() / q.norm_sqr()`.
    /// If the quaternion is a zero quaternion, it returns a quaternion with `NaN` components.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    ///
    /// let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let inv_q = q.inverse();
    /// // For a unit quaternion, inverse is its conjugate.
    /// // For a general quaternion, q * q.inverse() should be identity.
    /// let identity_q : Quaternion<f64>  = q * inv_q;
    /// assert!((identity_q.w - 1.0).abs() < 1e-9);
    /// assert!((identity_q.x - 0.0).abs() < 1e-9);
    /// assert!((identity_q.y - 0.0).abs() < 1e-9);
    /// assert!((identity_q.z - 0.0).abs() < 1e-9);
    ///
    /// let zero_q = Quaternion::<f64>::new(0.0, 0.0, 0.0, 0.0);
    /// let inv_zero_q = zero_q.inverse();
    /// assert!(inv_zero_q.w.is_nan());
    /// ```
    fn inverse(&self) -> Self {
        self._inverse_impl()
    }
}

impl<T: RealField> Quaternion<T> {
    pub(crate) fn _conjugate_impl(&self) -> Self {
        Quaternion {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    pub(crate) fn _norm_sqr_impl(&self) -> T {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub(crate) fn _inverse_impl(&self) -> Self {
        let n_sqr = self._norm_sqr_impl();
        if n_sqr.is_zero() {
            Quaternion::new(T::nan(), T::nan(), T::nan(), T::nan())
        } else {
            self._conjugate_impl() / n_sqr
        }
    }
}
