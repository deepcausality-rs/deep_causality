/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::chain::Chain;
use deep_causality_num::{AbelianGroup, Module, Ring};
use std::ops::{Add, Mul, Neg, Sub};

// ============================================================================
// Add
// ============================================================================

impl<T> Add for Chain<T>
where
    T: AbelianGroup + Copy + PartialEq + Default + Neg<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        (&self).add(&rhs)
    }
}

impl<T> Add for &Chain<T>
where
    T: AbelianGroup + Copy + PartialEq + Default + Neg<Output = T>,
{
    type Output = Chain<T>;

    fn add(self, rhs: Self) -> Chain<T> {
        self.add(rhs)
    }
}

// ============================================================================
// Sub
// ============================================================================

impl<T> Sub for Chain<T>
where
    T: AbelianGroup + Copy + PartialEq + Default + Neg<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        (&self).sub(&rhs)
    }
}

impl<T> Sub for &Chain<T>
where
    T: AbelianGroup + Copy + PartialEq + Default + Neg<Output = T>,
{
    type Output = Chain<T>;

    fn sub(self, rhs: Self) -> Chain<T> {
        self.sub(rhs)
    }
}

// ============================================================================
// Neg
// ============================================================================

impl<T> Neg for Chain<T>
where
    T: AbelianGroup + Copy + PartialEq + Default + Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self {
        (&self).neg()
    }
}

impl<T> Neg for &Chain<T>
where
    T: AbelianGroup + Copy + PartialEq + Default + Neg<Output = T>,
{
    type Output = Chain<T>;

    fn neg(self) -> Chain<T> {
        self.neg()
    }
}

// ============================================================================
// Mul (Scalar)
// ============================================================================

impl<T, S> Mul<S> for Chain<T>
where
    T: Module<S> + Copy,
    S: Ring + Copy,
{
    type Output = Self;

    fn mul(self, scalar: S) -> Self {
        self.scale(scalar)
    }
}

impl<T, S> Mul<S> for &Chain<T>
where
    T: Module<S> + Copy,
    S: Ring + Copy,
{
    type Output = Chain<T>;

    fn mul(self, scalar: S) -> Chain<T> {
        self.scale(scalar)
    }
}
