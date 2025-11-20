//! This module defines the Quaternion struct and its core implementations.

use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

pub use crate::float::Float;
use crate::{Matrix3, Num, Vector3};

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
mod part_ord;
mod quaternion_number_impl;
mod to_primitive;

pub trait QuaternionNumber<F>: Num + Sized
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
    fn to_axis_angle(&self) -> (Vector3<F>, F);
    fn to_rotation_matrix(&self) -> Matrix3<F>;
    fn slerp(&self, other: &Self, t: F) -> Self;
}

/// Represents a quaternion with a scalar part (`w`) and a vector part (`x`, `y`, `z`).
///
/// Quaternions are a number system that extends complex numbers and are commonly used
/// in 3D graphics and physics for representing rotations.
///
/// The `Quaternion` struct is generic over a float type `F`, allowing it to work
/// with different floating-point precisions (e.g., `f32` or `f64`).
///
/// # Fields
///
/// * `w`: The scalar component of the quaternion.
/// * `x`: The `i` component of the vector part.
/// * `y`: The `j` component of the vector part.
/// * `z`: The `k` component of the vector part.
///
/// # Examples
///
/// ```
/// use deep_causality_num::Quaternion;
///
/// let q1 = Quaternion::new(1.0, 2.0, 3.0, 4.0);
/// let q2 = Quaternion { w: 5.0, x: 6.0, y: 7.0, z: 8.0 };
///
/// assert_eq!(q1.w, 1.0);
/// assert_eq!(q2.x, 6.0);
/// ```
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

// Marker trait to ensure all Num requirements are implemented.
impl<F: Float> Num for Quaternion<F> {}
