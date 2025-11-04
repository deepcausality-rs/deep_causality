//! This module defines the Quaternion struct and its core implementations.

use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

pub use crate::float::Float;

mod arithmetic;
mod arithmetic_assign;
mod as_primitive;
mod constructors;
mod debug;
mod display;
mod from_primitives;
mod identity;
mod neg;
mod num;
mod num_cast;
mod part_ord;
mod quaternion_number;
mod rotation;
mod to_primitive;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct Quaternion<F>
where
    F: Float,
    Self: Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Neg<Output = Self>
        + Rem<Output = Self>
        + Sum
        + Product,
{
    pub w: F, // Scalar part
    pub x: F, // Vector part i
    pub y: F, // Vector part j
    pub z: F, // Vector part k
}
