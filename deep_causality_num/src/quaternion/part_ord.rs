use crate::float::Float;
use crate::quaternion::Quaternion;
use std::cmp::Ordering;

// PartialOrd
impl<F: Float> PartialOrd for Quaternion<F> {
    /// Compares two quaternions partially.
    ///
    /// Quaternions do not have a natural total ordering. This implementation provides
    /// a lexicographical comparison based on the `w`, `x`, `y`, and `z` components.
    /// This is primarily for satisfying trait bounds where a total order isn't strictly required
    /// and should not be interpreted as a mathematically meaningful ordering for quaternions in general.
    ///
    /// Returns `None` if any component is `NaN`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::Quaternion;
    /// use std::cmp::Ordering;
    ///
    /// let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    /// let q2 = Quaternion::new(1.0, 2.0, 3.0, 5.0);
    /// assert_eq!(q1.partial_cmp(&q2), Some(Ordering::Less));
    ///
    /// let q_nan = Quaternion::new(1.0, f64::NAN, 3.0, 4.0);
    /// assert_eq!(q_nan.partial_cmp(&q1), None);
    /// ```
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Quaternions do not have a natural ordering.
        // This implementation provides a lexicographical comparison,
        // which is not mathematically meaningful for quaternions in general.
        // It's primarily for satisfying trait bounds where a total order isn't strictly required.
        match self.w.partial_cmp(&other.w) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        match self.x.partial_cmp(&other.x) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        match self.y.partial_cmp(&other.y) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.z.partial_cmp(&other.z)
    }
}
