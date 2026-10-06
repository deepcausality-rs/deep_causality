/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::RealField;
use deep_causality_context_store::{ContextoidId, ProjectionError};
use deep_causality_num::FromPrimitive;

/// `value`, a scalar of the record of node `id`, in the node's scalar type `R`. A non-finite
/// `value` lifts to the non-finite value of `R`.
///
/// # Errors
/// [`ProjectionError::Scalar`] naming `id` and `value` when `R` has no value for it: when
/// `R::from_f64(value)` is `None`, or when a finite `value` lies past the range of `R` and would
/// become an infinity, as it does for `f32` and `BFloat16` beyond about 3.4·10³⁸. The `Storable`
/// impl for `f32` refuses the same values.
pub(crate) fn lift_scalar<R: RealField + FromPrimitive>(
    id: ContextoidId,
    value: f64,
) -> Result<R, ProjectionError> {
    R::from_f64(value)
        .filter(|lifted| lifted.is_finite() || !value.is_finite())
        .ok_or(ProjectionError::Scalar(id, value))
}
