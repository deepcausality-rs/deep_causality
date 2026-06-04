/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::ops::Div;
use deep_causality_num::{FromPrimitive, Real};

/// The scalar a differentiable or integrable model is written against.
///
/// - [`Real`](deep_causality_num::Real) supplies ring arithmetic and the elementary
///   functions (the analytic axis), without requiring field division.
/// - `Div` lets `Dual` itself be a `Real`, so the tangent functor **nests**
///   (`Dual<Dual<…>>` gives higher derivatives).
/// - [`FromPrimitive`](deep_causality_num::FromPrimitive) is the precision-safe constant
///   lift: a model raises its literal constants into the working scalar at any precision
///   (`f32` / `f64` / `Float106`, and `Dual` over each via the blanket in
///   `deep_causality_num`). `From<f64>` is deliberately *not* used here, because `f32` does
///   not implement it.
///
/// This is distinct from multivector's `ScalarEval` (which abstracts a value's real
/// *modulus* over Real/Complex for norm work); `Scalar` is the differentiation *variable*.
///
/// Blanket-implemented, so every qualifying concrete number is a `Scalar` automatically.
pub trait Scalar: Real + Div<Output = Self> + FromPrimitive {}

impl<T: Real + Div<Output = T> + FromPrimitive> Scalar for T {}
