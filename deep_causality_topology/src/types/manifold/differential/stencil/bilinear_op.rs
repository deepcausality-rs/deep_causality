/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The compiled bilinear stage (the wedge): per output cell, a list of
//! `(ia, ib, coeff)` triples evaluating `Σ coeff · a[ia] · b[ib]`. The
//! triples carry the cup-shuffle signs and the ½ antisymmetrization
//! factor, so the apply is a pure gather–multiply–accumulate.

use deep_causality_num::RealField;
use deep_causality_par::MaybeParallel;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[cfg(feature = "parallel")]
const PAR_BILINEAR_THRESHOLD: usize = 1 << 16;

/// A compiled bilinear operator over two input cochains.
#[derive(Debug, Clone)]
pub(crate) struct BilinearOp<R> {
    rows: usize,
    cols_a: usize,
    cols_b: usize,
    ptr: Vec<usize>,
    ia: Vec<u32>,
    ib: Vec<u32>,
    coeff: Vec<R>,
}

impl<R> BilinearOp<R>
where
    R: RealField + MaybeParallel,
{
    /// Build from per-row triple lists.
    pub(crate) fn from_rows(
        rows_entries: Vec<Vec<(usize, usize, R)>>,
        cols_a: usize,
        cols_b: usize,
    ) -> Self {
        let rows = rows_entries.len();
        let nnz: usize = rows_entries.iter().map(|r| r.len()).sum();
        let mut ptr = Vec::with_capacity(rows + 1);
        let mut ia = Vec::with_capacity(nnz);
        let mut ib = Vec::with_capacity(nnz);
        let mut coeff = Vec::with_capacity(nnz);
        ptr.push(0);
        for row in rows_entries {
            for (a, b, v) in row {
                debug_assert!(a < cols_a && b < cols_b);
                ia.push(a as u32);
                ib.push(b as u32);
                coeff.push(v);
            }
            ptr.push(ia.len());
        }
        Self {
            rows,
            cols_a,
            cols_b,
            ptr,
            ia,
            ib,
            coeff,
        }
    }

    pub(crate) fn rows(&self) -> usize {
        self.rows
    }

    pub(crate) fn cols_a(&self) -> usize {
        self.cols_a
    }

    pub(crate) fn cols_b(&self) -> usize {
        self.cols_b
    }

    /// `out[r] = Σ coeff · a[ia] · b[ib]` per row.
    pub(crate) fn apply(&self, a: &[R], b: &[R], out: &mut [R]) {
        debug_assert_eq!(a.len(), self.cols_a);
        debug_assert_eq!(b.len(), self.cols_b);
        debug_assert_eq!(out.len(), self.rows);

        #[cfg(feature = "parallel")]
        if self.coeff.len() >= PAR_BILINEAR_THRESHOLD {
            out.par_iter_mut().enumerate().for_each(|(r, o)| {
                let mut acc = R::zero();
                for e in self.ptr[r]..self.ptr[r + 1] {
                    acc += self.coeff[e] * a[self.ia[e] as usize] * b[self.ib[e] as usize];
                }
                *o = acc;
            });
            return;
        }

        for (r, o) in out.iter_mut().enumerate() {
            let mut acc = R::zero();
            for e in self.ptr[r]..self.ptr[r + 1] {
                acc += self.coeff[e] * a[self.ia[e] as usize] * b[self.ib[e] as usize];
            }
            *o = acc;
        }
    }
}
