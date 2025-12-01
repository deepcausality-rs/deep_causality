/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CsrMatrix;
use deep_causality_num::{AbelianGroup, Ring};
use std::ops::{Add, Mul, Neg, Sub};

// ============================================================================
// Add Implementations (4 variants for all ownership combinations)
// ============================================================================

// owned + owned
impl<T> Add for CsrMatrix<T>
where
    T: AbelianGroup + Copy + std::ops::Neg<Output = T> + Default + PartialEq, // Added Default + PartialEq
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        CsrMatrix::add(&self, &rhs)
    }
}

// ref + ref
impl<T> Add for &CsrMatrix<T>
where
    T: AbelianGroup + Copy + std::ops::Neg<Output = T> + Default + PartialEq, // Added Default + PartialEq
{
    type Output = CsrMatrix<T>;
    fn add(self, rhs: Self) -> CsrMatrix<T> {
        CsrMatrix::add(self, rhs)
    }
}

// owned + ref
impl<T> Add<&CsrMatrix<T>> for CsrMatrix<T>
where
    T: AbelianGroup + Copy + std::ops::Neg<Output = T> + Default + PartialEq, // Added Default + PartialEq
{
    type Output = Self;
    fn add(self, rhs: &Self) -> Self {
        CsrMatrix::add(&self, rhs)
    }
}

// ref + owned
impl<T> Add<CsrMatrix<T>> for &CsrMatrix<T>
where
    T: AbelianGroup + Copy + std::ops::Neg<Output = T> + Default + PartialEq, // Added Default + PartialEq
{
    type Output = CsrMatrix<T>;
    fn add(self, rhs: CsrMatrix<T>) -> CsrMatrix<T> {
        CsrMatrix::add(self, &rhs)
    }
}

// ============================================================================
// Sub Implementations (4 variants for all ownership combinations)
// ============================================================================

// owned + owned
impl<T> Sub for CsrMatrix<T>
where
    T: AbelianGroup
        + Copy
        + std::ops::Sub<Output = T>
        + std::ops::Neg<Output = T>
        + Default
        + PartialEq, // Added Default + PartialEq
{
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        CsrMatrix::sub(&self, &rhs)
    }
}

// ref + ref
impl<T> Sub for &CsrMatrix<T>
where
    T: AbelianGroup
        + Copy
        + std::ops::Sub<Output = T>
        + std::ops::Neg<Output = T>
        + Default
        + PartialEq, // Added Default + PartialEq
{
    type Output = CsrMatrix<T>;
    fn sub(self, rhs: Self) -> CsrMatrix<T> {
        CsrMatrix::sub(self, rhs)
    }
}

// owned + ref
impl<T> Sub<&CsrMatrix<T>> for CsrMatrix<T>
where
    T: AbelianGroup
        + Copy
        + std::ops::Sub<Output = T>
        + std::ops::Neg<Output = T>
        + Default
        + PartialEq, // Added Default + PartialEq
{
    type Output = Self;
    fn sub(self, rhs: &Self) -> Self {
        CsrMatrix::sub(&self, rhs)
    }
}

// ref + owned
impl<T> Sub<CsrMatrix<T>> for &CsrMatrix<T>
where
    T: AbelianGroup
        + Copy
        + std::ops::Sub<Output = T>
        + std::ops::Neg<Output = T>
        + Default
        + PartialEq, // Added Default + PartialEq
{
    type Output = CsrMatrix<T>;
    fn sub(self, rhs: CsrMatrix<T>) -> CsrMatrix<T> {
        CsrMatrix::sub(self, &rhs)
    }
}

// ============================================================================
// Neg Implementations (2 variants for owned and borrowed)
// ============================================================================

// owned
impl<T> Neg for CsrMatrix<T>
where
    T: AbelianGroup + Copy + std::ops::Neg<Output = T> + Default + PartialEq, // Added Default + PartialEq
{
    type Output = Self;
    fn neg(self) -> Self {
        CsrMatrix::neg(&self) // Calls the CsrMatrix::neg(&self) method
    }
}

// borrowed
impl<T> Neg for &CsrMatrix<T>
where
    T: AbelianGroup + Copy + std::ops::Neg<Output = T> + Default + PartialEq, // Added Default + PartialEq
{
    type Output = CsrMatrix<T>;
    fn neg(self) -> CsrMatrix<T> {
        CsrMatrix::neg(self) // Calls the CsrMatrix::neg(&self) method
    }
}

// ============================================================================
// Mul Implementations (4 variants for matrix multiplication)
// ============================================================================

// owned + owned
impl<T> Mul for CsrMatrix<T>
where
    T: Ring + Copy + Default + PartialEq, // Added Default + PartialEq
{
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        CsrMatrix::mul(&self, &rhs)
    }
}

// ref + ref
impl<T> Mul for &CsrMatrix<T>
where
    T: Ring + Copy + Default + PartialEq, // Added Default + PartialEq
{
    type Output = CsrMatrix<T>;
    fn mul(self, rhs: Self) -> CsrMatrix<T> {
        CsrMatrix::mul(self, rhs)
    }
}

// owned + ref
impl<T> Mul<&CsrMatrix<T>> for CsrMatrix<T>
where
    T: Ring + Copy + Default + PartialEq, // Added Default + PartialEq
{
    type Output = Self;
    fn mul(self, rhs: &Self) -> Self {
        CsrMatrix::mul(&self, rhs)
    }
}

// ref + owned
impl<T> Mul<CsrMatrix<T>> for &CsrMatrix<T>
where
    T: Ring + Copy + Default + PartialEq, // Added Default + PartialEq
{
    type Output = CsrMatrix<T>;
    fn mul(self, rhs: CsrMatrix<T>) -> CsrMatrix<T> {
        CsrMatrix::mul(self, &rhs)
    }
}
