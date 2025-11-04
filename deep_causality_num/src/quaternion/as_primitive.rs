use crate::float::Float;
use crate::quaternion::Quaternion;
use crate::{AsPrimitive, NumCast};

// AsPrimitive
impl<F: Float, T> AsPrimitive<T> for Quaternion<F>
where
    F: AsPrimitive<T>,
    T: 'static + Copy + NumCast,
{
    /// Converts the scalar part of the quaternion to a primitive type `T`.
    ///
    /// This conversion only considers the `w` (scalar) component of the quaternion.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::AsPrimitive;
    ///
    /// let q = Quaternion::new(123.45, 0.0, 0.0, 0.0);
    /// let val_f32: f32 = q.as_();
    /// assert_eq!(val_f32, 123.45f32);
    ///
    /// let val_i32: i32 = q.as_();
    /// assert_eq!(val_i32, 123);
    /// ```
    fn as_(self) -> T {
        self.w.as_() // Only the scalar part is converted
    }
}
