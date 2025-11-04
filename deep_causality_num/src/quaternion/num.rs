use crate::float::Float;
use crate::num::Num;
use crate::quaternion::Quaternion;

// Num
impl<F: Float> Num for Quaternion<F> {}
