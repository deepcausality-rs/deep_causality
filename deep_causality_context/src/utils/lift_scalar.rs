/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context_store::{ContextoidId, ProjectionError};
use deep_causality_num::FromPrimitive;

/// `value`, a scalar of the record of node `id`, in the node's scalar type `R`.
///
/// # Errors
/// [`ProjectionError::Scalar`] naming `id` and `value` when `R::from_f64(value)` is `None`. The
/// float types `f32`, `f64`, `BFloat16` and `Float106` return `Some` for every `f64`: `f32` rounds
/// a finite value past its range to an infinity.
pub(crate) fn lift_scalar<R: FromPrimitive>(
    id: ContextoidId,
    value: f64,
) -> Result<R, ProjectionError> {
    R::from_f64(value).ok_or(ProjectionError::Scalar(id, value))
}
