/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// A value bounded by a minimum and a maximum
///
///  If input is less than min then this returns min.
///  If input is greater than max then this returns max.
///  Otherwise this returns input.
///
/// **Panics** in debug mode if `!(min <= max)`.
#[inline]
pub fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

/// A value bounded by a minimum value
///
///  If input is less than min then this returns min.
///  Otherwise this returns input.
///  `clamp_min(std::f32::NAN, 1.0)` preserves `NAN` different from `f32::min(std::f32::NAN, 1.0)`.
///
/// **Panics** in debug mode if `!(min == min)`. (This occurs if `min` is `NAN`.)
#[inline]
#[allow(clippy::eq_op)]
pub fn clamp_min<T: PartialOrd>(input: T, min: T) -> T {
    debug_assert!(min == min, "min must not be NAN");
    if input < min { min } else { input }
}

/// A value bounded by a maximum value
///
///  If input is greater than max then this returns max.
///  Otherwise this returns input.
///  `clamp_max(std::f32::NAN, 1.0)` preserves `NAN` different from `f32::max(std::f32::NAN, 1.0)`.
///
/// **Panics** in debug mode if `!(max == max)`. (This occurs if `max` is `NAN`.)
#[inline]
#[allow(clippy::eq_op)]
pub fn clamp_max<T: PartialOrd>(input: T, max: T) -> T {
    debug_assert!(max == max, "max must not be NAN");
    if input > max { max } else { input }
}
