/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use std::cmp::Ordering;

/// Implements the `PartialEq` trait for `Octonion`.
///
/// Compares two `Octonion` instances for equality by comparing each of their corresponding components.
/// Two octonions are equal if and only if all their scalar and imaginary components are equal.
///
/// # Arguments
/// * `self` - The left-hand side `Octonion`.
/// * `other` - The right-hand side `Octonion`.
///
/// # Returns
/// `true` if all components are equal, `false` otherwise.
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
///
/// let o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
/// let o2 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
/// let o3 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0);
///
/// assert_eq!(o1, o2);
/// assert_ne!(o1, o3);
/// ```
impl<F: Float> PartialEq<Self> for Octonion<F> {
    fn eq(&self, other: &Self) -> bool {
        self.s == other.s
            && self.e1 == other.e1
            && self.e2 == other.e2
            && self.e3 == other.e3
            && self.e4 == other.e4
            && self.e5 == other.e5
            && self.e6 == other.e6
            && self.e7 == other.e7
    }
}

/// Implements the `PartialOrd` trait for `Octonion`.
///
/// Provides a partial ordering for `Octonion` instances based on a lexicographical comparison
/// of their components (s, e1, e2, ..., e7).
///
/// It's important to note that a total ordering is not generally defined for hypercomplex numbers
/// like octonions in a mathematically meaningful way. This implementation provides an ordering
/// primarily for utility purposes (e.g., sorting, unique collection insertion) and does not
/// imply a standard mathematical ordering.
///
/// # Arguments
/// * `self` - The left-hand side `Octonion`.
/// * `other` - The right-hand side `Octonion`.
///
/// # Returns
/// An `Option<Ordering>` which is:
/// - `Some(Ordering::Less)` if `self` is less than `other`.
/// - `Some(Ordering::Equal)` if `self` is equal to `other`.
/// - `Some(Ordering::Greater)` if `self` is greater than `other`.
/// - `None` if the values are incomparable (e.g., due to NaN components).
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
/// use std::cmp::Ordering;
///
/// let o1 = Octonion::new(1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
/// let o2 = Octonion::new(1.0, 3.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
/// let o3 = Octonion::new(2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
///
/// assert_eq!(o1.partial_cmp(&o2), Some(Ordering::Less));
/// assert_eq!(o2.partial_cmp(&o1), Some(Ordering::Greater));
/// assert_eq!(o1.partial_cmp(&o1), Some(Ordering::Equal));
/// assert_eq!(o1.partial_cmp(&o3), Some(Ordering::Less));
///
/// let o_nan = Octonion::new(f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
/// assert_eq!(o1.partial_cmp(&o_nan), None);
/// ```
impl<F: Float> PartialOrd for Octonion<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Lexicographical comparison, not mathematically standard for octonions.
        self.s
            .partial_cmp(&other.s)
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e1.partial_cmp(&other.e1)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e2.partial_cmp(&other.e2)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e3.partial_cmp(&other.e3)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e4.partial_cmp(&other.e4)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e5.partial_cmp(&other.e5)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e6.partial_cmp(&other.e6)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e7.partial_cmp(&other.e7)
                } else {
                    Some(ord)
                }
            })
    }
}
