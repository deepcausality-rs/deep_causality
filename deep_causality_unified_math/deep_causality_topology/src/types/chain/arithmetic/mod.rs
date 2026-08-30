/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::chain::Chain;
use core::ops::{Add, Mul, Neg, Sub};
use deep_causality_algebra::{AbelianGroup, Module, Ring};

// ============================================================================
// Add
// ============================================================================

impl<R, G> Add for Chain<R, G>
where
    G: AbelianGroup + Copy + PartialEq + Default + Neg<Output = G>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Chain::add(&self, &rhs)
    }
}

impl<R, G> Add for &Chain<R, G>
where
    G: AbelianGroup + Copy + PartialEq + Default + Neg<Output = G>,
{
    type Output = Chain<R, G>;

    fn add(self, rhs: Self) -> Chain<R, G> {
        Chain::add(self, rhs)
    }
}

// ============================================================================
// Sub
// ============================================================================

impl<R, G> Sub for Chain<R, G>
where
    G: AbelianGroup + Copy + PartialEq + Default + Neg<Output = G>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Chain::sub(&self, &rhs)
    }
}

impl<R, G> Sub for &Chain<R, G>
where
    G: AbelianGroup + Copy + PartialEq + Default + Neg<Output = G>,
{
    type Output = Chain<R, G>;

    fn sub(self, rhs: Self) -> Chain<R, G> {
        Chain::sub(self, rhs)
    }
}

// ============================================================================
// Neg
// ============================================================================

impl<R, G> Neg for Chain<R, G>
where
    G: AbelianGroup + Copy + PartialEq + Default + Neg<Output = G>,
{
    type Output = Self;

    fn neg(self) -> Self {
        Chain::neg(&self)
    }
}

impl<R, G> Neg for &Chain<R, G>
where
    G: AbelianGroup + Copy + PartialEq + Default + Neg<Output = G>,
{
    type Output = Chain<R, G>;

    fn neg(self) -> Chain<R, G> {
        Chain::neg(self)
    }
}

// ============================================================================
// Mul (Scalar)
// ============================================================================

impl<R, G, S> Mul<S> for Chain<R, G>
where
    G: Module<S> + Copy,
    S: Ring + Copy,
{
    type Output = Self;

    fn mul(self, scalar: S) -> Self {
        self.scale(scalar)
    }
}

impl<R, G, S> Mul<S> for &Chain<R, G>
where
    G: Module<S> + Copy,
    S: Ring + Copy,
{
    type Output = Chain<R, G>;

    fn mul(self, scalar: S) -> Chain<R, G> {
        self.scale(scalar)
    }
}
