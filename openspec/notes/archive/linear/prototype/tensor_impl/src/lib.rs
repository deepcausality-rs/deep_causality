//! ORPHAN-RULE PROBE.
//!
//! Both `linear_b::MatrixView` and `deep_causality_tensor::CausalTensor` are
//! foreign to THIS crate. Enabling `--features orphan_probe` must fail with
//! E0117, proving the impl can only live in `deep_causality_tensor` itself —
//! i.e. that the user's proposed dependency direction (tensor -> linear) is the
//! only one that works.

#[cfg(feature = "orphan_probe")]
mod probe {
    use deep_causality_tensor::CausalTensor;
    use linear_b::MatrixView;

    impl MatrixView for CausalTensor<f64> {
        type Scalar = f64;
        fn rows(&self) -> usize {
            self.shape()[0]
        }
        fn cols(&self) -> usize {
            self.shape()[1]
        }
        fn get(&self, r: usize, c: usize) -> f64 {
            *CausalTensor::get(self, &[r, c]).unwrap()
        }
    }
}

/// A NEWTYPE is the workaround available to a third crate: local type, foreign
/// trait, so the orphan rule permits it. It also costs a wrapper at every call
/// site and does not give `deep_causality_tensor` the methods.
pub struct TensorView<'a>(pub &'a deep_causality_tensor::CausalTensor<f64>);

impl linear_b::MatrixView for TensorView<'_> {
    type Scalar = f64;
    fn rows(&self) -> usize {
        self.0.shape()[0]
    }
    fn cols(&self) -> usize {
        self.0.shape()[1]
    }
    fn get(&self, r: usize, c: usize) -> f64 {
        *self.0.get(&[r, c]).unwrap()
    }
}

/// The read-only side is implementable for a CSR matrix too, but the MUTABLE
/// side is not: `swap_rows` is fine on CSR, `axpy_rows` is not — adding a
/// multiple of one sparse row to another changes that row's non-zero pattern,
/// which in CSR means reallocating every row after it. Sparse elimination is a
/// different algorithm (fill-reducing ordering, symbolic factorisation), not a
/// different implementation of this one.
pub struct CsrView<'a>(pub &'a deep_causality_sparse::CsrMatrix<f64>);

impl linear_b::MatrixView for CsrView<'_> {
    type Scalar = f64;
    fn rows(&self) -> usize {
        self.0.shape().0
    }
    fn cols(&self) -> usize {
        self.0.shape().1
    }
    fn get(&self, r: usize, c: usize) -> f64 {
        let start = self.0.row_indices()[r];
        let end = self.0.row_indices()[r + 1];
        let cols = self.0.col_indices();
        let vals = self.0.values();
        (start..end)
            .find(|&i| cols[i] == c)
            .map(|i| vals[i])
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deep_causality_tensor::CausalTensor;
    use linear_b::MatrixView;

    #[test]
    fn newtype_view_reads_a_real_causal_tensor() {
        let t = CausalTensor::new(vec![1.0f64, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
        let v = TensorView(&t);
        assert_eq!(v.rows(), 2);
        assert_eq!(v.cols(), 2);
        assert_eq!(v.get(1, 0), 3.0);
    }
}
