/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{FromPrimitive, RealField};

/// Precision bound for the CDL compute pipeline.
///
/// Any real-field floating-point type (`f32`, `f64`, `Float106`, …) satisfies this via the
/// blanket implementation below. It bundles the trait bounds the discovery pipeline and the
/// underlying algorithms (SURD) require, so generic pipeline code can write `T: Precision`
/// instead of repeating the full bound list.
pub trait Precision: RealField + FromPrimitive + Default + Send + Sync {}

impl<T> Precision for T where T: RealField + FromPrimitive + Default + Send + Sync {}
