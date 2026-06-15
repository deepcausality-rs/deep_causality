/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt::{Debug, Display};
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_par::MaybeParallel;

/// Scalar bound for every CFD theory and solver: precision as a parameter (`f32`,
/// `f64`, `Float106`), plus the `MaybeParallel` thread-safety marker that lets the
/// inner topology operator loops fan out under `--features parallel`. Blanket-
/// implemented for every qualifying type, so it is invisible to serial consumers and
/// is exactly the Rayon requirement under the `parallel` feature.
pub trait CfdScalar:
    RealField + FromPrimitive + Default + PartialEq + Debug + Display + MaybeParallel
{
}

impl<R: RealField + FromPrimitive + Default + PartialEq + Debug + Display + MaybeParallel> CfdScalar
    for R
{
}
