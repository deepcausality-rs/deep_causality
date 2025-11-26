use core::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

use crate::complex::quaternion_number::Quaternion;
use crate::float::Float;

// AddAssign
impl<F: Float + AddAssign> AddAssign for Quaternion<F> {
    /// Performs in-place quaternion addition.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use core::ops::AddAssign;
    ///
    /// let mut q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let q2 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    /// q1.add_assign(q2);
    /// assert_eq!(q1, Quaternion::new(6.0, 8.0, 10.0, 12.0));
    /// ```
    fn add_assign(&mut self, other: Self) {
        self.w += other.w;
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

// SubAssign
impl<F: Float + SubAssign> SubAssign for Quaternion<F> {
    /// Performs in-place quaternion subtraction.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use core::ops::SubAssign;
    ///
    /// let mut q1 = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    /// let q2 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// q1.sub_assign(q2);
    /// assert_eq!(q1, Quaternion::new(4.0, 4.0, 4.0, 4.0));
    /// ```
    fn sub_assign(&mut self, other: Self) {
        self.w -= other.w;
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

// MulAssign
impl<F: Float + MulAssign> MulAssign for Quaternion<F> {
    /// Performs in-place quaternion multiplication.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use core::ops::MulAssign;
    ///
    /// let mut q1 = Quaternion::new(1.0, 0.0, 0.0, 0.0); // Identity
    /// let q2 = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    /// q1.mul_assign(q2);
    /// assert_eq!(q1, Quaternion::new(0.0, 1.0, 0.0, 0.0)); // 1 * i = i
    /// ```
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

// DivAssign
impl<F: Float + DivAssign> DivAssign for Quaternion<F> {
    /// Performs in-place quaternion division.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use core::ops::DivAssign;
    ///
    /// let mut q1 = Quaternion::new(2.0, 4.0, 6.0, 8.0);
    /// let q2 = Quaternion::new(2.0, 0.0, 0.0, 0.0);
    /// q1.div_assign(q2);
    /// assert_eq!(q1, Quaternion::new(1.0, 2.0, 3.0, 4.0));
    /// ```
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}
