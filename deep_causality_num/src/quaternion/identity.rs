use crate::float::Float;
use crate::identity::one::{ConstOne, One};
use crate::identity::zero::{ConstZero, Zero};
use crate::quaternion::Quaternion;

// Zero
impl<F: Float> Zero for Quaternion<F> {
    fn zero() -> Self {
        Quaternion::new(F::zero(), F::zero(), F::zero(), F::zero())
    }

    fn is_zero(&self) -> bool {
        self.w.is_zero() && self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}

// ConstZero
impl<F: Float + ConstZero> ConstZero for Quaternion<F> {
    const ZERO: Self = Quaternion {
        w: F::ZERO,
        x: F::ZERO,
        y: F::ZERO,
        z: F::ZERO,
    };
}

// One
impl<F: Float> One for Quaternion<F> {
    fn one() -> Self {
        Quaternion::new(F::one(), F::zero(), F::zero(), F::zero())
    }

    fn is_one(&self) -> bool {
        self.w.is_one() && self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}

// ConstOne
impl<F: Float + ConstOne + ConstZero> ConstOne for Quaternion<F> {
    const ONE: Self = Quaternion {
        w: F::ONE,
        x: F::ZERO,
        y: F::ZERO,
        z: F::ZERO,
    };
}
