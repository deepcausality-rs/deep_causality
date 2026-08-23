/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[cfg(feature = "alloc")]
use alloc::string::String;
/// The unique identifier for a Cause or Context in the Causality Graph
pub type IdentificationValue = u64;

pub type TeloidTag = &'static str;
pub type TeloidID = u64;
pub type ContextId = u64;
pub type ContextoidId = u64;
pub type CausaloidId = u64;

#[cfg(feature = "alloc")]
/// A string value that provides a human-readable description of a Cause or Context
pub type DescriptionValue = String;
/// A floating point value that represents a numerical measure
pub type NumericalValue = f64;

/// A type alias for unsigned integers, used for numerical counting and indexing
pub type NumberType = u64;
/// A type alias for floating point numbers, used for numerical calculations
pub type FloatType = f64;

/// A type alias for the integers `ℤ`, used for exact integer arithmetic.
///
/// This is the integer counterpart of [`FloatType`]: the single alias a model changes to move
/// its whole integer computation to another width. Generic code should be written against the
/// algebraic bound (`CommutativeRing`, or `EuclideanDomain` where division with remainder or
/// `gcd` is needed) and should mention this alias only where a concrete type is unavoidable,
/// exactly as float-generic code is written against `RealField` and names `FloatType` only at
/// the boundary.
///
/// # How this differs from `FloatType`
///
/// The two aliases are not the same kind of parameter, and the difference matters.
///
/// `FloatType` selects **precision**. `f32`, `f64`, and `Float106` all approximate the same
/// set `ℝ`, and the choice trades accuracy against cost. The failure mode is rounding: a graded
/// error, bounded by `epsilon()`, which the tolerance machinery can carry and reason about.
///
/// `IntType` selects **range**. Every signed width represents a finite window of `ℤ` *exactly*;
/// there is no rounding and no analogue of `epsilon()`. The failure mode is overflow, which is
/// not a graded error but a hard wrongness — the computed value is not an approximation of the
/// true one. So widening `IntType` does not buy accuracy, it buys headroom, and integer code
/// carries an explicit overflow discipline (checked, saturating, or wrapping) instead of a
/// tolerance.
///
/// # Why signed
///
/// `ℤ` requires additive inverses. The unsigned types have none, so they are a commutative
/// semiring rather than a ring, and they do not implement `AbelianGroup`, `Ring`,
/// `CommutativeRing`, or `EuclideanDomain`. Use [`NumberType`] for counting and indexing, which
/// is what `ℕ` is for; use this alias for ring arithmetic.
pub type IntType = i64;
