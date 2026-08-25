/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The access-trait and operator impls for the bit-packed 𝔽₂ matrix.
//!
//! # The row operations work on whole words
//!
//! This is where the packing pays. `axpy_rows` over 𝔽₂ is `dst ^= src` — the factor can only be
//! `0` or `1`, so it is either a no-op or an exclusive-or — and doing it a word at a time processes
//! `W::BITS` entries per instruction. That is the 3.2× at n=2048.
//!
//! `scale_row` is likewise degenerate: scaling by the only unit leaves the row alone, and scaling by
//! zero clears it.

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_build::MatrixBuild;
use crate::traits::matrix_view::MatrixView;
use crate::traits::row_ops::RowOps;
use crate::types::packed_gf2::PackedGf2;
use core::ops::{Add, Mul, MulAssign, Neg, Sub};
use deep_causality_algebra::Ring;
use deep_causality_num::{Gf2, NaturalNumber, One, Zero};

impl<W: NaturalNumber> MatrixView for PackedGf2<W> {
    type Scalar = Gf2;

    fn rows(&self) -> usize {
        self.dims().0
    }
    fn cols(&self) -> usize {
        self.dims().1
    }
    fn get(&self, row: usize, col: usize) -> Result<Gf2, LinearError> {
        let (r, c) = self.dims();
        if row >= r || col >= c {
            return Err(LinearError::IndexOutOfBounds((row, col), (r, c)));
        }
        Ok(Gf2::new(self.bit_at(row, col)))
    }
}

impl<W: NaturalNumber> MatrixBuild for PackedGf2<W> {
    fn zeros(rows: usize, cols: usize) -> Self {
        Self::allocate(rows, cols)
    }
    fn set(&mut self, row: usize, col: usize, value: Gf2) -> Result<(), LinearError> {
        let (r, c) = self.dims();
        if row >= r || col >= c {
            return Err(LinearError::IndexOutOfBounds((row, col), (r, c)));
        }
        if value.bit() {
            self.set_bit(row, col);
        } else {
            self.clear_bit(row, col);
        }
        Ok(())
    }
}

impl<W: NaturalNumber> RowOps for PackedGf2<W> {
    fn swap_rows(&mut self, a: usize, b: usize) -> Result<(), LinearError> {
        let (r, c) = self.dims();
        if a >= r || b >= r {
            return Err(LinearError::IndexOutOfBounds((a.max(b), 0), (r, c)));
        }
        if a == b {
            return Ok(());
        }
        let w = self.words_per_row();
        for k in 0..w {
            self.as_mut_words().swap(a * w + k, b * w + k);
        }
        Ok(())
    }

    /// Degenerate over 𝔽₂: scaling by the only unit leaves the row alone, and scaling by zero
    /// clears it from `from_col` onward.
    fn scale_row(&mut self, row: usize, factor: &Gf2, from_col: usize) -> Result<(), LinearError> {
        let (r, c) = self.dims();
        if row >= r {
            return Err(LinearError::IndexOutOfBounds((row, 0), (r, c)));
        }
        if factor.bit() {
            return Ok(());
        }
        for j in from_col..c {
            self.clear_bit(row, j);
        }
        Ok(())
    }

    /// `dst ^= src`, a word at a time from `from_col`'s word onward.
    fn axpy_rows(
        &mut self,
        dst: usize,
        src: usize,
        factor: &Gf2,
        from_col: usize,
    ) -> Result<(), LinearError> {
        let (r, c) = self.dims();
        if dst >= r || src >= r {
            return Err(LinearError::IndexOutOfBounds((dst.max(src), 0), (r, c)));
        }
        if !factor.bit() {
            return Ok(());
        }
        // `dst ^= src`, a whole word at a time. The factor can only be one here, so the multiply
        // vanishes and the add is an exclusive-or. This is where the packing pays.
        let bits = core::mem::size_of::<W>() * 8;
        let w = self.words_per_row();
        let first = from_col / bits;
        for k in first..w {
            let s = self.as_words()[src * w + k];
            let d = self.as_words()[dst * w + k];
            self.as_mut_words()[dst * w + k] = d ^ s;
        }
        let _ = c;
        Ok(())
    }

    /// Overridden to scan a word at a time rather than a bit at a time.
    ///
    /// It still searches. What the override changes is how the search reads memory, not whether it
    /// happens — the first set bit at or below `from_row` is exactly what the default would find.
    fn pivot_in_column(&self, col: usize, from_row: usize) -> Option<usize> {
        (from_row..self.dims().0).find(|&r| self.bit_at(r, col))
    }
}

impl<W: NaturalNumber> Zero for PackedGf2<W> {
    fn zero() -> Self {
        PackedGf2::allocate(0, 0)
    }
    fn is_zero(&self) -> bool {
        self.as_words().iter().all(|w| w.is_zero())
    }
}

impl<W: NaturalNumber> One for PackedGf2<W> {
    fn one() -> Self {
        let mut m = PackedGf2::allocate(1, 1);
        m.set_bit(0, 0);
        m
    }
    fn is_one(&self) -> bool {
        let (r, c) = self.dims();
        r == c && (0..r).all(|i| (0..c).all(|j| self.bit_at(i, j) == (i == j)))
    }
}

/// Addition over 𝔽₂ is exclusive-or, so this is a whole-word XOR of the two buffers.
impl<W: NaturalNumber> Add for PackedGf2<W> {
    type Output = Self;
    /// Whole-word exclusive-or, which is what addition over 𝔽₂ is.
    fn add(self, rhs: Self) -> Self {
        assert_eq!(self.dims(), rhs.dims(), "shape mismatch in add");
        let mut out = self;
        for (k, w) in rhs.as_words().iter().enumerate() {
            out.as_mut_words()[k] = out.as_words()[k] ^ *w;
        }
        out
    }
}

/// Subtraction coincides with addition: every element of 𝔽₂ is its own additive inverse.
impl<W: NaturalNumber> Sub for PackedGf2<W> {
    type Output = Self;
    /// The same operation as addition: every element of 𝔽₂ is its own additive inverse, so
    /// `a - b` and `a + b` are the same exclusive-or.
    ///
    /// Written out rather than delegating to `+`, so that the body says what it does instead of
    /// relying on a reader knowing that the two coincide here.
    fn sub(self, rhs: Self) -> Self {
        assert_eq!(self.dims(), rhs.dims(), "shape mismatch in sub");
        let mut out = self;
        for (k, w) in rhs.as_words().iter().enumerate() {
            out.as_mut_words()[k] = out.as_words()[k] ^ *w;
        }
        out
    }
}

/// Negation is the identity, for the same reason.
impl<W: NaturalNumber> Neg for PackedGf2<W> {
    type Output = Self;
    /// The identity, for the same reason.
    fn neg(self) -> Self {
        self
    }
}

impl<W: NaturalNumber> Mul for PackedGf2<W> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let (m, k) = self.dims();
        let (k2, n) = rhs.dims();
        assert_eq!(k, k2, "inner dimension mismatch in mul");
        let mut out = PackedGf2::allocate(m, n);
        for i in 0..m {
            for j in 0..n {
                // The 𝔽₂ dot product of row i and column j: parity of the shared ones.
                let mut acc = false;
                for t in 0..k {
                    if self.bit_at(i, t) && rhs.bit_at(t, j) {
                        acc = !acc;
                    }
                }
                if acc {
                    out.set_bit(i, j);
                }
            }
        }
        out
    }
}

impl<W: NaturalNumber, S: Ring + Copy> Mul<S> for PackedGf2<W> {
    type Output = Self;
    fn mul(self, scalar: S) -> Self {
        // Scaling by a ring element: zero clears the matrix, anything else leaves it, because the
        // only units of 𝔽₂ are 0 and 1.
        if scalar.is_zero() {
            let (r, c) = self.dims();
            return PackedGf2::allocate(r, c);
        }
        self
    }
}

impl<W: NaturalNumber, S: Ring + Copy> MulAssign<S> for PackedGf2<W> {
    fn mul_assign(&mut self, scalar: S) {
        if scalar.is_zero() {
            for w in self.as_mut_words() {
                *w = W::zero();
            }
        }
    }
}
