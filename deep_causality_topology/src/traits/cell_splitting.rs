/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cell splitting: the shared operation underneath the cup product.
//!
//! The cup product of a `p`-cochain and a `q`-cochain evaluates, on each
//! `(p+q)`-cell, a signed sum of products over the ways that cell decomposes
//! into a pair of lower cells. The decomposition rule differs by family:
//!
//! - **Simplicial** (Alexander–Whitney): one term, the leading `p+1` vertices
//!   paired with the trailing `q+1` vertices, sign `+1`.
//! - **Cubical**: `C(k, p)` terms, one per choice of `p` of the cell's active
//!   axes, each carrying a shuffle sign.
//!
//! Both are the *same* question asked of a cell, which is why splitting rather
//! than vertex listing is the abstraction: a simplex's vertices are `usize`
//! indices while a lattice cell's are `[usize; D]` positions, so no common
//! vertex type is workable while a common splitting is.
//!
//! This is a trait separate from [`Cell`] rather than a method on it. `Cell` is
//! public and is used as a bound in well over a hundred places; a required
//! method would break every external implementor. A complex family opts in.
//!
//! Reference: Chen, Y.-A. & Tata, S., *Higher cup products on hypercubic
//! lattices*, arXiv:2106.05274, J. Math. Phys. **64**, 091902 (2023), Eq. (5)
//! for the simplicial case and Fig. 1 for the cubical one.

use crate::traits::cell::Cell;

/// The ambient lattice layout a splitting may need, as
/// [`CellularComplex::uniform_lattice_layout`](crate::CellularComplex::uniform_lattice_layout)
/// returns it: per-axis extent and per-axis periodicity.
///
/// A simplicial splitting ignores it. A cubical splitting needs it to wrap the
/// paired cell's position on periodic axes, since a cell at the far edge of a
/// torus pairs with one that has wrapped around.
pub type CellLayout = (Vec<usize>, Vec<bool>);

/// One term of a cell splitting: the pair of cells the two cochain factors are
/// evaluated on, and the sign the ordering induces.
///
/// The two cells are named for their **algebraic role**, not a geometric one.
/// Alexander–Whitney's left cell is the leading vertices of the simplex, while
/// the cubical left cell sits at the cell's base position and the right cell is
/// offset from it. A geometric name such as "front face" is true simplicially
/// and false cubically, and would mislead every implementor.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellSplit<C> {
    left: C,
    right: C,
    sign: i8,
}

impl<C> CellSplit<C> {
    /// A split term pairing `left` with `right` with sign `+1`.
    pub fn positive(left: C, right: C) -> Self {
        Self {
            left,
            right,
            sign: 1,
        }
    }

    /// A split term pairing `left` with `right` with sign `-1`.
    pub fn negative(left: C, right: C) -> Self {
        Self {
            left,
            right,
            sign: -1,
        }
    }

    /// A split term whose sign is the parity of `inversions`: `+1` when even,
    /// `-1` when odd.
    ///
    /// This is the shuffle sign a cubical split carries, expressed so the caller
    /// hands over the inversion count and cannot get the mapping wrong.
    pub fn from_parity(left: C, right: C, inversions: usize) -> Self {
        if inversions.is_multiple_of(2) {
            Self::positive(left, right)
        } else {
            Self::negative(left, right)
        }
    }

    /// The cell the left-hand cochain factor is evaluated on.
    pub fn left(&self) -> &C {
        &self.left
    }

    /// The cell the right-hand cochain factor is evaluated on.
    pub fn right(&self) -> &C {
        &self.right
    }

    /// The sign this term contributes, always exactly `+1` or `-1`.
    ///
    /// There is deliberately no constructor taking an arbitrary `i8`. A cup
    /// product multiplies by this sign, so a stray `0` would silently annihilate
    /// a term and any other magnitude would be silently read as a unit; the
    /// constructors above make both unrepresentable.
    pub fn sign(&self) -> i8 {
        self.sign
    }

    /// Consumes the term, yielding `(left, right, sign)`.
    pub fn into_parts(self) -> (C, C, i8) {
        (self.left, self.right, self.sign)
    }
}

/// A cell that can be split into the pairs a cup product sums over.
///
/// Implemented for `Simplex` and `LatticeCell<D>`. A type may implement [`Cell`]
/// without implementing this, in which case it is simply not eligible for the
/// cup product.
pub trait SplittableCell: Cell + Sized {
    /// Every way this cell decomposes into a left cell of dimension `left_dim`
    /// paired with a right cell of the complementary dimension, with signs.
    ///
    /// Returns an empty vector when `left_dim` exceeds the cell's own dimension,
    /// so a cup product in a degree the cell cannot carry contributes zero
    /// rather than failing.
    ///
    /// `layout` supplies the ambient extent and periodicity for families whose
    /// splitting depends on it; simplicial implementations ignore it.
    fn split(&self, left_dim: usize, layout: Option<&CellLayout>) -> Vec<CellSplit<Self>>;
}
