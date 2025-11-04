use crate::float::Float;
use crate::quaternion::Quaternion;
use crate::{NumCast, ToPrimitive};

// NumCast
impl<F: Float> NumCast for Quaternion<F> {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        F::from(n).map(|f| Quaternion::new(f, F::zero(), F::zero(), F::zero()))
    }
}
