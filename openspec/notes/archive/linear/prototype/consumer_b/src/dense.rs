//! Stand-in for `CausalTensor<f64>`: a dense row-major buffer.
//!
//! LOC that this implementation owns is what the "code reuse" question turns on.

use deep_causality_algebra::{Field, RealField};
use linear_b::{MatrixBuild, MatrixView, RowOps};

#[derive(Clone, Debug, PartialEq)]
pub struct Dense<F> {
    rows: usize,
    cols: usize,
    data: Vec<F>,
}

impl<F: Field> Dense<F> {
    pub fn new(data: Vec<F>, rows: usize, cols: usize) -> Self {
        assert_eq!(data.len(), rows * cols);
        Self { rows, cols, data }
    }
}

impl<F: Field> MatrixView for Dense<F> {
    type Scalar = F;
    fn rows(&self) -> usize {
        self.rows
    }
    fn cols(&self) -> usize {
        self.cols
    }
    fn get(&self, r: usize, c: usize) -> F {
        self.data[r * self.cols + c].clone()
    }
}

impl<F: Field> RowOps for Dense<F> {
    fn swap_rows(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        for c in 0..self.cols {
            self.data.swap(a * self.cols + c, b * self.cols + c);
        }
    }

    fn scale_row(&mut self, r: usize, factor: F, from_col: usize) {
        let base = r * self.cols;
        for c in from_col..self.cols {
            self.data[base + c] = self.data[base + c].clone() * factor.clone();
        }
    }

    fn axpy_rows(&mut self, dst: usize, src: usize, factor: F, from_col: usize) {
        let (d, s) = (dst * self.cols, src * self.cols);
        for c in from_col..self.cols {
            let v = self.data[d + c].clone() + factor.clone() * self.data[s + c].clone();
            self.data[d + c] = v;
        }
    }
    // pivot_in_column: the default (first non-zero) is correct for an exact field.
}

impl<F: Field> MatrixBuild for Dense<F> {
    fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: core::iter::repeat_with(F::zero).take(rows * cols).collect(),
        }
    }
    fn set(&mut self, r: usize, c: usize, v: F) {
        self.data[r * self.cols + c] = v;
    }
}

/// A SECOND dense type, differing only in the pivot rule: partial pivoting by
/// magnitude, which needs `RealField` (order + `abs`) and is therefore NOT
/// expressible on `Field`. This is what a numerically serious `CausalTensor`
/// implementation would look like.
#[derive(Clone, Debug, PartialEq)]
pub struct DensePivoted<F> {
    inner: Dense<F>,
    /// Rank tolerance, captured from the ORIGINAL matrix before elimination.
    /// A tolerance computed per column during elimination is not enough: see
    /// the `numerical_rank` tests.
    tol: F,
}

impl<F: RealField> DensePivoted<F> {
    pub fn new(inner: Dense<F>) -> Self {
        let mut max = F::zero();
        for r in 0..inner.rows() {
            for c in 0..inner.cols() {
                let v = inner.get(r, c).abs();
                if v > max {
                    max = v;
                }
            }
        }
        let n = if inner.rows() > inner.cols() {
            inner.rows()
        } else {
            inner.cols()
        };
        let mut scale = F::zero();
        for _ in 0..n {
            scale += F::one();
        }
        Self {
            tol: F::epsilon() * max * scale,
            inner,
        }
    }

    /// The naive variant: tolerance relative to the current column only.
    pub fn new_column_relative(inner: Dense<F>) -> Self {
        Self {
            tol: F::epsilon(),
            inner,
        }
    }
}

impl<F: RealField> MatrixView for DensePivoted<F> {
    type Scalar = F;
    fn rows(&self) -> usize {
        self.inner.rows()
    }
    fn cols(&self) -> usize {
        self.inner.cols()
    }
    fn get(&self, r: usize, c: usize) -> F {
        self.inner.get(r, c)
    }
}

impl<F: RealField> RowOps for DensePivoted<F> {
    fn swap_rows(&mut self, a: usize, b: usize) {
        self.inner.swap_rows(a, b)
    }
    fn scale_row(&mut self, r: usize, factor: F, from_col: usize) {
        self.inner.scale_row(r, factor, from_col)
    }
    fn axpy_rows(&mut self, dst: usize, src: usize, factor: F, from_col: usize) {
        self.inner.axpy_rows(dst, src, factor, from_col)
    }

    /// The one override: largest magnitude wins, rejected below `self.tol`.
    fn pivot_in_column(&self, col: usize, from_row: usize) -> Option<usize> {
        let mut best: Option<(usize, F)> = None;
        for r in from_row..self.rows() {
            let v = self.get(r, col).abs();
            match &best {
                Some((_, b)) if *b >= v => {}
                _ => best = Some((r, v)),
            }
        }
        let (r, v) = best?;
        if v <= self.tol { None } else { Some(r) }
    }
}
