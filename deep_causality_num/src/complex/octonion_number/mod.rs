/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines the Octonion struct and its core implementations.

use crate::{Float, Num};
use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

mod arithmetic;
mod arithmetic_assign;
mod as_primitive;
mod constructors;
mod debug;
mod display;
mod from_primitives;
mod identity;
mod neg;
mod num_cast;
mod octonion_number_impl;
mod part_ord;
mod to_primitive;

pub trait OctonionNumber<F>: Num + Sized
where
    F: Float,
    Self: Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Rem<Output = Self>
        + Neg<Output = Self>
        + Sum
        + Product
        + PartialEq
        + Copy
        + Clone,
{
    fn conjugate(&self) -> Self;
    fn norm_sqr(&self) -> F;
    fn norm(&self) -> F;
    fn normalize(&self) -> Self;
    fn inverse(&self) -> Self;
    fn dot(&self, other: &Self) -> F;
}

#[derive(Copy, Clone, Default)]
pub struct Octonion<F>
where
    F: Float,
{
    pub s: F,  // Scalar part
    pub e1: F, // Vector part 1
    pub e2: F, // Vector part 2
    pub e3: F, // Vector part 3
    pub e4: F, // Vector part 4
    pub e5: F, // Vector part 5
    pub e6: F, // Vector part 6
    pub e7: F, // Vector part 7
}

// Marker trait to ensure all Num requirements are implemented.
impl<F: Float> Num for Octonion<F> {}
