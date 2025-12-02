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
        // 1. Compute δ(d(ω)) - this always exists
        let d_omega = self.exterior_derivative(k); // (k+1)-form

        let delta_d_omega = {
            let mut data = vec![T::zero(); self.data().len()];
            let mut offset = 0;
            for i in 0..(k + 1) {
                offset += self.complex.skeletons()[i].simplices().len();
            }
            data[offset..offset + d_omega.len()].copy_from_slice(d_omega.as_slice());
            let m = Manifold::new(
                self.complex().clone(),
                CausalTensor::new(data, self.data().shape().to_vec()).unwrap(),
                0,
            )
            .unwrap();
            m.codifferential(k + 1)
        };

        // 2. Compute d(δ(ω)) - only if k > 0
        if k == 0 {
            // For 0-forms, Δ = δd only
            return delta_d_omega;
        }

        let delta_omega = self.codifferential(k); // (k-1)-form
        let d_delta_omega = {
            let mut data = vec![T::zero(); self.data().len()];
            let mut offset = 0;
            for i in 0..(k - 1) {
                offset += self.complex.skeletons()[i].simplices().len();
            }
            data[offset..offset + delta_omega.len()].copy_from_slice(delta_omega.as_slice());
            let m = Manifold::new(
                self.complex().clone(),
                CausalTensor::new(data, self.data().shape().to_vec()).unwrap(),
                0,
            )
            .unwrap();
            m.exterior_derivative(k - 1) // k-form
        };

        // 3. Sum the two k-forms
        let mut result_data = d_delta_omega.as_slice().to_vec();
        let delta_d_slice = delta_d_omega.as_slice();

        // Ensure both forms have the same length before summing
        if result_data.len() != delta_d_slice.len() {
            panic!(
                "Laplacian components have mismatched lengths: d_delta_omega_len={}, delta_d_omega_len={}",
                result_data.len(),
                delta_d_slice.len()
            );
        }

        for (r, &d) in result_data.iter_mut().zip(delta_d_slice.iter()) {
            *r = *r + d;
        }

        CausalTensor::new(
            result_data,
            vec![self.complex.skeletons()[k].simplices().len()],
        )
        .unwrap()
    }
}
