use crate::float::Float;
use crate::quaternion::Quaternion;
use std::cmp::Ordering;

// PartialOrd
impl<F: Float> PartialOrd for Quaternion<F> {
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
