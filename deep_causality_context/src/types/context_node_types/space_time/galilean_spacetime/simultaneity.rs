/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::GalileanSpacetime;
use crate::utils::seconds::seconds;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

impl<R: RealField + FromPrimitive> GalileanSpacetime<R> {
    /// Whether `self` and `other` happen at the same instant.
    ///
    /// Two times in the same scale are compared exactly. Two times in different scales are
    /// compared in seconds: they are one instant when their seconds `a` and `b` satisfy
    /// `|a − b| ≤ ε (|a| + |b|)`, `ε` the machine epsilon of `R`. In `f32` and `f64`, where every
    /// conversion factor is exact, the bound covers a decimal rounded once to `R` and converted
    /// once, each rounding at most `ε / 2` of its magnitude: `0.1` months and `262 974.6` s are
    /// one instant, as are `1.11` h and `66.6` min. A time that carries more error than that, such
    /// as a `Float106` lifted from an `f64` literal, compares as another instant. Across scales
    /// the relation is not transitive: `262 974.6` s and the next `f64` above it are two
    /// instants, and each is `0.1` months.
    ///
    /// The result is `None` when the instants cannot be compared: a time that is not finite, or
    /// not finite once in seconds, names no instant, and a scale that names no duration
    /// (`NoScale`, `Steps`, `Symbolic`) compares only with itself.
    pub fn is_simultaneous_with(&self, other: &Self) -> Option<bool> {
        if !(self.t.is_finite() && other.t.is_finite()) {
            return None;
        }
        if self.time_scale == other.time_scale {
            return Some(self.t == other.t);
        }
        let a = seconds(self.t, self.time_scale)?;
        let b = seconds(other.t, other.time_scale)?;
        if !(a.is_finite() && b.is_finite()) {
            return None;
        }
        Some((a - b).abs() <= R::epsilon() * (a.abs() + b.abs()))
    }
}
