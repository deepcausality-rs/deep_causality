/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AbelianGroup, Field, Real};

/// An ordered `Field` that is also an analytic real scalar.
///
/// `RealField` is exactly a [`Real`] that is also a [`Field`]: it adds field
/// invertibility (a total multiplicative inverse / division) on top of the analytic
/// surface (ordering, `sqrt`/`exp`/`ln`/`sin`/…, constants) provided by [`Real`].
///
/// All analytic operations are declared on [`Real`] and inherited here via the
/// supertrait, so every existing `T: RealField` bound resolves the same method set
/// it always did. The split lets analytic-but-non-field types (for example dual
/// numbers, used for automatic differentiation) implement [`Real`] without falsely
/// claiming `Field`/`RealField`.
///
/// This trait abstracts over concrete floating-point types like `f32` and `f64`.
pub trait RealField: Real + Field {}

impl AbelianGroup for f32 {}
impl AbelianGroup for f64 {}

// CommutativeRing is derived automatically
impl RealField for f32 {}

impl RealField for f64 {}
