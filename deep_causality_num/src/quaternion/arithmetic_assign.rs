use std::ops::{AddAssign, DivAssign, MulAssign, SubAssign};

use crate::float::Float;
use crate::quaternion::Quaternion;

// AddAssign
impl<F: Float + AddAssign> AddAssign for Quaternion<F> {
    fn add_assign(&mut self, other: Self) {
        self.w += other.w;
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

// SubAssign
impl<F: Float + SubAssign> SubAssign for Quaternion<F> {
    fn sub_assign(&mut self, other: Self) {
        self.w -= other.w;
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

// MulAssign
impl<F: Float + MulAssign> MulAssign for Quaternion<F> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

// DivAssign
impl<F: Float + DivAssign> DivAssign for Quaternion<F> {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}
