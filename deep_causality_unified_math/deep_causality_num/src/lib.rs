/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

mod alias;
mod cast;
mod combinatorics;
mod float;
mod float_106;
mod float_bfloat16;
mod float_option;
mod gf2;
mod identity;
mod integer;
pub mod lift;
mod num;

// Alias types
pub use crate::alias::{Matrix3, Vector3};

// Casts
pub use crate::cast::as_primitive::AsPrimitive;
pub use crate::cast::as_scalar::float_as_scalar_impl::FloatAsScalar;
pub use crate::cast::as_scalar::int_as_scalar_impl::IntAsScalar;
pub use crate::cast::from_primitive::FromPrimitive;
pub use crate::cast::num_cast::NumCast;
pub use crate::cast::to_float::{FloatFromInt, IntoFloat};
pub use crate::cast::to_primitive::ToPrimitive;

// Float number types
pub use crate::float::Float;
pub use crate::float_106::Float106;
pub use crate::float_bfloat16::BFloat16;
pub use crate::float_option::FloatOption;

// Finite field types
pub use crate::combinatorics::{stirling_first_unsigned, stirling_second};

pub use crate::gf2::Gf2;

// Identity types
pub use crate::identity::one::{ConstOne, One};
pub use crate::identity::zero::{ConstZero, Zero};

// Integer types
pub use crate::integer::{Integer, NaturalNumber, SignedInt, UnsignedInt};

// The precision-boundary crossings: every primitive float and integer into the working type,
// and the working type back out to f64, f32 or a count.
pub use crate::lift::{
    Lift, Lower, lift, lift_count, lift_f32, lift_f64, lift_i8, lift_i16, lift_i32, lift_i64,
    lift_i128, lift_isize, lift_u8, lift_u16, lift_u32, lift_u64, lift_u128, lift_usize, lower,
    lower_f32, to_count, try_lift, try_lift_count, try_lift_f32, try_lift_f64, try_lift_i8,
    try_lift_i16, try_lift_i32, try_lift_i64, try_lift_i128, try_lift_isize, try_lift_u8,
    try_lift_u16, try_lift_u32, try_lift_u64, try_lift_u128, try_lift_usize, try_lower,
    try_lower_f32,
};

// General numeric traits
pub use crate::num::Num;
pub use crate::num::num_ops::*;
