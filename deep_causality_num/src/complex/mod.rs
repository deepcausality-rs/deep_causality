/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::float::Float;

mod arithmetic;
mod arithmetic_assign;
mod arithmetic_complex;
mod as_primitive;
mod constructors;
mod debug;
mod display;
mod float;
mod from_primitives;
mod identity;
mod neg;
mod num_cast;
mod part_ord;
mod to_primitive;

/// Represents a complex number with real and imaginary parts.
#[derive(Copy, Clone, PartialEq, Default)]
pub struct Complex<F>
where
    F: Float,
{
    pub re: F,
    pub im: F,
}
