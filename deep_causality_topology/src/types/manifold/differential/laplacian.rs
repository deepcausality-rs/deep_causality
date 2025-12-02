use crate::Manifold;
use deep_causality_num::{Field, FromPrimitive, Zero};
use deep_causality_tensor::CausalTensor;

impl<T> Manifold<T>
where
    T: Field
        + Copy
        + FromPrimitive
        + core::ops::Mul<f64, Output = T>
        + core::ops::Neg<Output = T>
        + Default
        + PartialEq
        + Zero
        + std::fmt::Debug,
{
    /// Computes the Hodge-Laplacian operator `Δ` on a k-form.
    ///
    /// The Laplacian is defined as: Δ = dδ + δd
    /// where `d` is the exterior derivative and `δ` is the codifferential.
    /// It maps k-forms to k-forms.
    pub fn laplacian(&self, k: usize) -> CausalTensor<T> {
        let n = self.complex.max_simplex_dimension();
        let current_dim_size = self.complex.skeletons()[k].simplices().len();

        // 1. Term A: d(δ(ω))
        let term_a = if k > 0 {
            // Compute delta (k -> k-1)
            let delta = self.codifferential(k);

            // We must wrap this result in a temporary Manifold to apply 'd'.
            // Construct a manifold with data only in the (k-1) slot.
            let temp_manifold = self.create_temp_manifold(k - 1, delta);

            // Compute d (k-1 -> k)
            temp_manifold.exterior_derivative(k - 1)
        } else {
            // If k=0, delta is 0, so d(delta) is 0
            CausalTensor::new(vec![T::zero(); current_dim_size], vec![current_dim_size]).unwrap()
        };

        // 2. Term B: δ(d(ω))
        let term_b = if k < n {
            // Compute d (k -> k+1)
            let d = self.exterior_derivative(k);

            // Wrap in temp manifold
            let temp_manifold = self.create_temp_manifold(k + 1, d);

            // Compute delta (k+1 -> k)
            temp_manifold.codifferential(k + 1)
        } else {
            CausalTensor::new(vec![T::zero(); current_dim_size], vec![current_dim_size]).unwrap()
        };

        // 3. Sum: A + B
        // Note: Standard Laplacian definition is often - (d delta + delta d) depending on convention.
        // We return the positive operator (d delta + delta d). The user should subtract it (Heat Eq: dt = -Laplacian).

        let mut result_data = Vec::with_capacity(current_dim_size);
        let slice_a = term_a.as_slice();
        let slice_b = term_b.as_slice();

        // Safety padding if tensors differ in size (should not happen in valid complex)
        let len = slice_a.len().max(slice_b.len());

        for i in 0..len {
            let a = slice_a.get(i).copied().unwrap_or(T::zero());
            let b = slice_b.get(i).copied().unwrap_or(T::zero());
            result_data.push(a + b);
        }

        CausalTensor::new(result_data, vec![current_dim_size]).unwrap()
    }
}
