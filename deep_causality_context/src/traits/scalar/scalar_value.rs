/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// Marker trait to identify numeric scalar types.
///
/// This trait is meant to distinguish primitive, copyable, and comparable
/// scalar types (like `f64`, `i64`, `u32`) from structured data types
/// (like `Data<f64>`, `Vec<T>`, or symbolic inputs).
///
/// # Purpose
/// Used to gate generic causal operations like `verify_single_cause`
/// or `reason_all_causes()` when working with raw scalar inputs.
///
/// # Trait Bounds
/// Scalar values must be:
/// - `Copy`: to avoid borrowing complexity
/// - `Clone`: for repeated safe use
/// - `PartialOrd`: for threshold logic
///
/// # Examples
/// ```rust
/// use deep_causality_context::ScalarValue;
///
/// fn process_value<T: ScalarValue>(val: T) -> bool {
///     val > T::default()
/// }
/// ```
pub trait ScalarValue: Copy + Clone + PartialOrd + Default {}

// One blanket impl rather than a list of primitives. A list decides for the caller which scalars
// exist, so a working type outside it — a narrower float on an embedded target, a wider one for a
// long integration — could not be projected without an entry being added here first.
impl<T: Copy + PartialOrd + Default> ScalarValue for T {}
