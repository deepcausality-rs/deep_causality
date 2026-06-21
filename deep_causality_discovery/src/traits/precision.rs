/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{FromPrimitive, RealField, ToPrimitive};

/// Precision bound for the CDL compute pipeline.
///
/// Any real-field floating-point type (`f32`, `f64`, `Float106`, …) satisfies this via the
/// blanket implementation below. It bundles the trait bounds the discovery pipeline and the
/// underlying algorithms (SURD) require, so generic pipeline code can write `T: Precision`
/// instead of repeating the full bound list.
///
/// `ToPrimitive` is listed explicitly because the `RealField` bound does **not**
/// imply it: `RealField` is implied *by* `Float` (`impl<T: Float> RealField for T`),
/// and `RealField`'s own supertrait chain (`Real + Field`) never reaches
/// `ToPrimitive`. The concrete precisions (`f32`, `f64`, `Float106`) satisfy it
/// because they impl `Float`, and `Float: NumCast: ToPrimitive`. Folding it into
/// `Precision` lets consumers that downcast a score to a concrete primitive rely on
/// `T: Precision` alone, without repeating `+ ToPrimitive` at every call site.
pub trait Precision: RealField + FromPrimitive + ToPrimitive + Default + Send + Sync {}

impl<T> Precision for T where T: RealField + FromPrimitive + ToPrimitive + Default + Send + Sync {}
