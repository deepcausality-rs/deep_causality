/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

// Private re-exports so the moved `complex` module keeps resolving these
// through `crate::` paths.
use deep_causality_algebra::{
    Annihilating, Associative, Commutative, ComplexField, Distributive, DivisionAlgebra, Field,
    Invertible, RealField, Rotation,
};
use deep_causality_num::{
    AsPrimitive, ConstOne, ConstZero, FromPrimitive, Matrix3, NumCast, One, ToPrimitive, Vector3,
    Zero,
};

mod complex;
mod extensions;
pub mod hom;
// Test fixtures, public because Bazel test targets cannot reach the `tests` tree, hidden
// because they are not API.
#[doc(hidden)]
pub mod utils_tests;

//  Complex number types
pub use crate::complex::complex_number::{Complex, Complex32, Complex64};
pub use crate::complex::octonion_number::{Octonion, Octonion32, Octonion64};
pub use crate::complex::quaternion_number::{Quaternion, Quaternion32, Quaternion64};
pub use crate::hom::{ComplexToReal, RealToComplex};

// HKT witnesses
pub use crate::extensions::{ComplexWitness, OctonionWitness, QuaternionWitness};
