/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Row reduction and everything read off it.
//!
//! # One core, two entry points
//!
//! Every function here runs the same private elimination over the
//! [`RowOps`](crate::RowOps) seam, naming no representation, no scalar and no word width. They come
//! in pairs because the pivot rule cannot be chosen by the representation alone:
//!
//! | suffix | pivot | admits |
//! |---|---|---|
//! | none | first non-zero at or below the row | any [`Field`] — 𝔽₂, ℚ, ℝ, ℂ |
//! | `_stable` | largest modulus at or below the row | any [`NormedScalar`] — ℝ, ℂ, `Float106` |
//!
//! The exact rule needs no ordering and no epsilon, which is what lets 𝔽₂ and ℚ through — neither
//! has an order and neither needs one. Over the floats it is correct but ill-conditioned, so a float
//! caller wants the `_stable` entry point. Both search; neither takes the diagonal on faith. A
//! Cayley-Menger matrix has `m[0][0] = 0` by construction, and an elimination that assumes the
//! diagonal returns zero for every simplex volume.

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_build::MatrixBuild;
use crate::traits::row_ops::RowOps;
use alloc::vec::Vec;
use deep_causality_algebra::{Field, Normed, NormedScalar, Real};
use deep_causality_num::{One, Zero};

/// What a row reduction learned, beyond the reduced matrix itself.
///
/// Rank and pivot columns come out of the same pass, and a caller that recomputes one from the other
/// is doing the elimination twice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reduced {
    rank: usize,
    pivot_columns: Vec<usize>,
}

impl Reduced {
    /// The rank, which is the number of pivots found.
    pub fn rank(&self) -> usize {
        self.rank
    }

    /// The columns a pivot was found in, ascending.
    ///
    /// The free columns are the complement, and they are what index a kernel basis.
    pub fn pivot_columns(&self) -> &[usize] {
        &self.pivot_columns
    }
}

/// The elimination every entry point here runs.
///
/// Names no representation, no scalar and no word width. `choose_pivot` is the only thing the
/// entry points differ in: the exact rule takes the first non-zero, the stable rule the largest
/// modulus. Both search the column at or below the current row; neither takes the diagonal.
fn reduce<M, P>(m: &mut M, mut choose_pivot: P) -> Result<Reduced, LinearError>
where
    M: RowOps,
    M::Scalar: Field,
    P: FnMut(&M, usize, usize) -> Option<usize>,
{
    let (rows, cols) = (m.rows(), m.cols());
    let mut pivot_columns = Vec::new();
    let mut row = 0usize;

    for col in 0..cols {
        if row >= rows {
            break;
        }
        let Some(p) = choose_pivot(m, col, row) else {
            continue;
        };
        m.swap_rows(row, p)?;

        let head = m.get(row, col)?;
        let inv = M::Scalar::one() / head;
        m.scale_row(row, &inv, col)?;

        for other in 0..rows {
            if other == row {
                continue;
            }
            let factor = m.get(other, col)?;
            if factor.is_zero() {
                continue;
            }
            let neg = M::Scalar::zero() - factor;
            m.axpy_rows(other, row, &neg, col)?;
        }

        pivot_columns.push(col);
        row += 1;
    }

    Ok(Reduced {
        rank: pivot_columns.len(),
        pivot_columns,
    })
}

/// The exact pivot rule: the first non-zero at or below `from_row`.
fn pivot_exact<M>(m: &M, col: usize, from_row: usize) -> Option<usize>
where
    M: RowOps,
    M::Scalar: Field,
{
    m.pivot_in_column(col, from_row)
}

/// The threshold below which a pivot candidate counts as zero, derived from the scalar's `epsilon`.
///
/// # Why a threshold at all
///
/// Elimination over the floats does not produce exact zeros. Reducing `[1 2 3; 4 5 6; 5 7 9]`,
/// whose third row is the sum of the first two, leaves a residue on the order of `1e-16` rather than
/// `0`. An exact `is_zero()` counts that residue as a pivot and reports rank 3 for a matrix of
/// rank 2.
///
/// # Why it is derived rather than written down
///
/// `linear-scalar-contract` requires that an operation whose result depends on a threshold take it
/// as an argument or derive it from `epsilon`, and never carry a literal in its body. A literal is
/// what `deep_causality_topology` has today — `1e-5`, applied to singular values of a matrix whose
/// entries are `{-1, 0, 1}` — and it is wrong at both ends: too coarse for a well-scaled problem and
/// too fine for a badly-scaled one.
///
/// This is the conventional relative bound: `eps * scale * max(rows, cols)`, where `scale` is the
/// largest modulus in the matrix. It moves with the data, so a matrix multiplied through by `1e6`
/// gets a threshold `1e6` times larger and reports the same rank.
///
/// The exact paths — [`rref`], [`rank`], and everything over 𝔽₂ and ℤ — use no threshold at all,
/// because they have no rounding to absorb.
fn negligible_below<M>(m: &M) -> <M::Scalar as Normed>::Real
where
    M: RowOps,
    M::Scalar: NormedScalar,
{
    let mut scale = <M::Scalar as Normed>::Real::zero();
    for r in 0..m.rows() {
        for c in 0..m.cols() {
            if let Ok(v) = m.get(r, c) {
                let mag = v.modulus_squared();
                if mag > scale {
                    scale = mag;
                }
            }
        }
    }
    // `scale` is a squared modulus, so the comparison below is squared too.
    let n = m.rows().max(m.cols()).max(1);
    let mut n_real = <M::Scalar as Normed>::Real::zero();
    for _ in 0..n {
        n_real += <M::Scalar as Normed>::Real::one();
    }
    let eps = <M::Scalar as Normed>::Real::epsilon();
    let rel = eps * n_real;
    scale * rel * rel
}

/// The stable pivot rule: the largest modulus at or below `from_row`, ignoring what is negligible.
///
/// `modulus_squared` lands in an ordered real, which is what lets this work over ℂ without the
/// scalar itself being ordered.
fn pivot_stable_with<M>(
    m: &M,
    col: usize,
    from_row: usize,
    floor: <M::Scalar as Normed>::Real,
) -> Option<usize>
where
    M: RowOps,
    M::Scalar: NormedScalar,
{
    let mut best: Option<(usize, <M::Scalar as Normed>::Real)> = None;
    for r in from_row..m.rows() {
        let Ok(v) = m.get(r, col) else { continue };
        let mag = v.modulus_squared();
        if mag <= floor {
            continue;
        }
        match &best {
            Some((_, b)) if *b >= mag => {}
            _ => best = Some((r, mag)),
        }
    }
    best.map(|(r, _)| r)
}

/// Reduces `m` to reduced row echelon form in place, pivoting on the first non-zero.
///
/// Bounded on [`Field`] because elimination divides by its pivot. That is the axiom ℤ lacks and the
/// reason the integer path is a separate algorithm rather than this one with a different scalar.
pub fn rref<M>(m: &mut M) -> Result<Reduced, LinearError>
where
    M: RowOps,
    M::Scalar: Field,
{
    reduce(m, pivot_exact)
}

/// Reduces `m` to reduced row echelon form in place, pivoting on the largest modulus.
///
/// Bounded on [`NormedScalar`], which supplies `modulus_squared` landing in an ordered real. That is
/// what lets the pivot be chosen by magnitude over ℂ without requiring the scalar itself to be
/// ordered — ℂ has no order, and comparing moduli does not need one.
pub fn rref_stable<M>(m: &mut M) -> Result<Reduced, LinearError>
where
    M: RowOps,
    M::Scalar: NormedScalar,
{
    let floor = negligible_below(m);
    reduce(m, move |mm, col, row| {
        pivot_stable_with(mm, col, row, floor)
    })
}

/// The rank, by exact elimination.
pub fn rank<M>(m: &M) -> Result<usize, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: Field,
{
    let mut work = m.clone();
    Ok(rref(&mut work)?.rank())
}

/// The rank, by elimination pivoting on magnitude.
///
/// This is the numerical rank and it is an approximation. Over ℝ it answers a different question
/// from the exact rank over ℤ or 𝔽₂, and the three are separate calls so that no caller reaches one
/// while meaning another.
pub fn rank_stable<M>(m: &M) -> Result<usize, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: NormedScalar,
{
    let mut work = m.clone();
    Ok(rref_stable(&mut work)?.rank())
}

/// A basis of the kernel, as the columns of the returned matrix.
///
/// Has `cols - rank` elements, one per free column, and every one of them is annihilated by `m`.
pub fn kernel_basis<M, B>(m: &M) -> Result<B, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: Field,
    B: MatrixBuild<Scalar = M::Scalar>,
{
    let cols = m.cols();
    let mut work = m.clone();
    let reduced = rref(&mut work)?;
    let pivots = reduced.pivot_columns().to_vec();
    let free: Vec<usize> = (0..cols).filter(|c| !pivots.contains(c)).collect();

    let mut basis = B::zeros(cols, free.len());
    for (k, &f) in free.iter().enumerate() {
        // The free variable is one; each pivot variable takes the negated reduced coefficient.
        basis.set(f, k, M::Scalar::one())?;
        for (row, &p) in pivots.iter().enumerate() {
            let coeff = work.get(row, f)?;
            if !coeff.is_zero() {
                basis.set(p, k, M::Scalar::zero() - coeff)?;
            }
        }
    }
    Ok(basis)
}

/// A basis of the image, as the columns of the returned matrix.
///
/// Has `rank` elements, and they are columns of `m` itself — the pivot columns — rather than
/// combinations of them, so that a caller reading them back gets vectors it recognises.
pub fn image_basis<M, B>(m: &M) -> Result<B, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: Field,
    B: MatrixBuild<Scalar = M::Scalar>,
{
    let rows = m.rows();
    let mut work = m.clone();
    let reduced = rref(&mut work)?;
    let pivots = reduced.pivot_columns().to_vec();

    // The pivot columns of the original, so a caller reads back vectors it recognises rather than
    // combinations of them.
    let mut basis = B::zeros(rows, pivots.len());
    for (k, &p) in pivots.iter().enumerate() {
        for i in 0..rows {
            basis.set(i, k, m.get(i, p)?)?;
        }
    }
    Ok(basis)
}

/// The determinant, pivoting on magnitude.
///
/// Closed forms at order three and below, elimination above. At small order a closed form is faster
/// and introduces no pivoting round-off at all, which is the same reason
/// `deep_causality_physics` keeps five fixed-size inverses of its own.
///
/// # Errors
///
/// [`LinearError::NotSquare`] if the matrix is not square. The determinant of the `0x0` matrix is
/// one, being the empty product.
pub fn determinant<M>(m: &M) -> Result<M::Scalar, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: NormedScalar,
{
    let (rows, cols) = (m.rows(), m.cols());
    if rows != cols {
        return Err(LinearError::NotSquare((rows, cols)));
    }
    // The empty product.
    if rows == 0 {
        return Ok(M::Scalar::one());
    }
    match rows {
        1 => return m.get(0, 0),
        2 => {
            let (a, b, c, d) = (m.get(0, 0)?, m.get(0, 1)?, m.get(1, 0)?, m.get(1, 1)?);
            return Ok(a * d - b * c);
        }
        3 => {
            let g = |i, j| m.get(i, j);
            let (a, b, c) = (g(0, 0)?, g(0, 1)?, g(0, 2)?);
            let (d, e, f) = (g(1, 0)?, g(1, 1)?, g(1, 2)?);
            let (h, i, j) = (g(2, 0)?, g(2, 1)?, g(2, 2)?);
            return Ok(a * (e * j - f * i) - b * (d * j - f * h) + c * (d * i - e * h));
        }
        _ => {}
    }

    // Forward elimination, pivoting by search. The product of the pivots, negated once per swap.
    let mut work = m.clone();
    let floor = negligible_below(&work);
    let mut det = M::Scalar::one();
    for col in 0..cols {
        let Some(p) = pivot_stable_with(&work, col, col, floor) else {
            return Ok(M::Scalar::zero());
        };
        if p != col {
            work.swap_rows(col, p)?;
            det = M::Scalar::zero() - det;
        }
        let head = work.get(col, col)?;
        det = det * head;
        let inv = M::Scalar::one() / head;
        for other in (col + 1)..rows {
            let factor = work.get(other, col)?;
            if factor.is_zero() {
                continue;
            }
            let neg = M::Scalar::zero() - factor * inv;
            work.axpy_rows(other, col, &neg, col)?;
        }
    }
    Ok(det)
}
