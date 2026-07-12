/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the QCM freeze-check example.

use deep_causality_num_complex::Complex;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. `f64` only ever appears at the display
/// boundary; the model is written against `FloatType`.
pub type FloatType = f64;

/// The complex scalar carried by every Choi–Jamiołkowski factor.
pub type C = Complex<FloatType>;
