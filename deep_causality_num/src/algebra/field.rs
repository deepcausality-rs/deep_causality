/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{MulGroup, Ring};

/// A Field is a Ring where multiplication is commutative and every non-zero
/// element has a multiplicative inverse.
///
/// Examples: f64, Complex<f64>, Rational numbers.
/// Counter-examples: Integers (no inverse), Quaternions (non-commutative).
pub trait Field: Ring + MulGroup {}

// Blanket Implementation
impl<T> Field for T where T: Ring + MulGroup {}
