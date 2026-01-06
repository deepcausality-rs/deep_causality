/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

mod algebra;
mod alias;
mod cast;
mod complex;
pub mod float;
mod float_option;
mod identity;
pub mod num;
mod ops;
pub mod utils_tests;
mod float_double;

// Alias types
pub use crate::alias::{Matrix3, Vector3};

// Algebra types
pub use crate::algebra::algebra_assoc::AssociativeAlgebra;
pub use crate::algebra::algebra_assoc_div::AssociativeDivisionAlgebra;
pub use crate::algebra::algebra_base::Algebra;
pub use crate::algebra::algebra_div::DivisionAlgebra;
pub use crate::algebra::algebra_properties::{Associative, Commutative, Distributive};
pub use crate::algebra::field::Field;
pub use crate::algebra::field_real::RealField;
pub use crate::algebra::group::Group;
pub use crate::algebra::group_abelian::AbelianGroup;
pub use crate::algebra::group_add::AddGroup;
pub use crate::algebra::group_div::DivGroup;
pub use crate::algebra::group_mul::MulGroup;
pub use crate::algebra::magma::{AddMagma, MulMagma};
pub use crate::algebra::module::Module;
pub use crate::algebra::monoid::{AddMonoid, InvMonoid, MulMonoid};
pub use crate::algebra::ring::Ring;
pub use crate::algebra::ring_associative::AssociativeRing;
pub use crate::algebra::ring_com::CommutativeRing;
pub use crate::algebra::rotation::Rotation;

// Casts
pub use crate::cast::as_primitive::AsPrimitive;
pub use crate::cast::as_scalar::float_as_scalar_impl::FloatAsScalar;
pub use crate::cast::as_scalar::int_as_scalar_impl::IntAsScalar;
pub use crate::cast::from_primitive::FromPrimitive;
pub use crate::cast::num_cast::NumCast;
pub use crate::cast::to_float::{FloatFromInt, IntoFloat};
pub use crate::cast::to_primitive::ToPrimitive;

//  Complex number types
pub use crate::complex::complex_number::{Complex, Complex32, Complex64};
pub use crate::complex::octonion_number::{Octonion, Octonion32, Octonion64};
pub use crate::complex::quaternion_number::{Quaternion, Quaternion32, Quaternion64};

// Float number types
pub use crate::float::Float;
pub use crate::float_option::FloatOption;

//  General numeric traits
pub use crate::identity::one::{ConstOne, One};
pub use crate::identity::zero::{ConstZero, Zero};
pub use crate::num::Num;
pub use crate::ops::num_ops::*;
