//! Design B SUBSUMES Design C: `[Vec<F>]` is just one implementation of the
//! trait, and `linear` can ship it, because the trait is local here even
//! though `Vec` is not (the orphan rule only restricts foreign-trait impls).
//!
//! This is the shape `deep_causality_topology::regge_geometry::curvature`
//! already stores its Cayley-Menger matrix in (`det_recursive(m: &[Vec<R>])`,
//! `curvature.rs:275`), so that call site needs no conversion at all.

use crate::{MatrixView, RowOps};
use alloc::vec::Vec;
use deep_causality_algebra::Field;

impl<F: Field> MatrixView for [Vec<F>] {
    type Scalar = F;

    fn rows(&self) -> usize {
        self.len()
    }

    fn cols(&self) -> usize {
        self.first().map(|r| r.len()).unwrap_or(0)
    }

    fn get(&self, r: usize, c: usize) -> F {
        self[r][c].clone()
    }
}

impl<F: Field> RowOps for [Vec<F>] {
    fn swap_rows(&mut self, a: usize, b: usize) {
        self.swap(a, b); // swapping whole rows is a pointer swap here
    }

    fn scale_row(&mut self, r: usize, factor: F, from_col: usize) {
        for v in self[r][from_col..].iter_mut() {
            *v = v.clone() * factor.clone();
        }
    }

    fn axpy_rows(&mut self, dst: usize, src: usize, factor: F, from_col: usize) {
        let (lo, hi) = if dst < src { (dst, src) } else { (src, dst) };
        let (head, tail) = self.split_at_mut(hi);
        let (a, b) = (&mut head[lo], &mut tail[0]);
        let (d, s) = if dst < src { (a, &*b) } else { (b, &*a) };
        for (dv, sv) in d[from_col..].iter_mut().zip(s[from_col..].iter()) {
            *dv = dv.clone() + factor.clone() * sv.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{determinant, rank_in_place};
    use alloc::vec;

    #[test]
    fn rows_of_rows_is_just_another_impl() {
        let mut m = [
            vec![1.0f64, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![5.0, 7.0, 9.0],
        ];
        assert_eq!(rank_in_place(&mut m[..]), 2);
    }

    #[test]
    fn cayley_menger_shaped_determinant() {
        // The 5x5 shape from regge_geometry/curvature.rs:275, on a regular
        // tetrahedron of edge 1: det = 288 * Vol^2, Vol^2 = 1/72.
        let (z, o) = (0.0f64, 1.0f64);
        let d = 1.0f64; // squared edge length
        let mut m = [
            vec![z, o, o, o, o],
            vec![o, z, d, d, d],
            vec![o, d, z, d, d],
            vec![o, d, d, z, d],
            vec![o, d, d, d, z],
        ];
        let det = determinant(&mut m[..]).unwrap();
        assert!((det - 4.0).abs() < 1e-9, "det = {det}");
    }
}
