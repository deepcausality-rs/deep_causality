/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

// Private re-exports so the `rational` module keeps resolving these through `crate::` paths.
use deep_causality_algebra::{
    AbelianGroup, Associative, Commutative, Distributive, EuclideanDomain, Invertible,
};
use deep_causality_num::{One, Zero};

mod rational;

// The rational numbers ℚ — the field of fractions of an integral domain.
pub use crate::rational::rational_number::{Rational, RationalScalar};
