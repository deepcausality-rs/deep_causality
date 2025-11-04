use crate::float::Float;
use crate::quaternion::Quaternion;
use crate::{AsPrimitive, NumCast};

// AsPrimitive
impl<F: Float, T> AsPrimitive<T> for Quaternion<F>
where
    F: AsPrimitive<T>,
    T: 'static + Copy + NumCast,
{
    fn as_(self) -> T {
        self.w.as_() // Only the scalar part is converted
    }
}
