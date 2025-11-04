use crate::float::Float;
use crate::quaternion::Quaternion;
use crate::{NumCast, ToPrimitive};

// NumCast
impl<F: Float> NumCast for Quaternion<F> {
    /// Converts a primitive type `T` into a `Quaternion`.
    ///
    /// The primitive value `n` is used to set the scalar (`w`) component of the quaternion,
    /// while the vector components (`x`, `y`, `z`) are set to zero.
    /// Returns `None` if the conversion from `T` to `F` fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use deep_causality_num::NumCast;
    ///
    /// let q_f64 = <Quaternion<f64> as NumCast>::from(123.45).unwrap();
    /// assert_eq!(q_f64, Quaternion::new(123.45, 0.0, 0.0, 0.0));
    ///
    /// let q_i32 = <Quaternion<f64> as NumCast>::from(123).unwrap();
    /// assert_eq!(q_i32, Quaternion::new(123.0, 0.0, 0.0, 0.0));
    /// ```
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        F::from(n).map(|f| Quaternion::new(f, F::zero(), F::zero(), F::zero()))
    }
}
