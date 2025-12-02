use crate::Manifold;
use deep_causality_num::Field;
use deep_causality_tensor::CausalTensor;

impl<T> Manifold<T>
where
    T: Field + Copy + std::ops::Neg<Output = T>,
{
    /// Computes the Laplace-Beltrami operator on a scalar field (0-form).
    ///
    /// For discrete 0-forms on a simplicial complex, this is defined as `Δf = δ₁d₀f`,
    /// where `d₀` is the exterior derivative (coboundary) and `δ₁` is the codifferential (boundary).
    /// This corresponds to the graph Laplacian `L = B₁ᵀB₁` where B₁ is the incidence matrix.
    ///
    /// # Returns
    /// A new tensor representing `Δf` where f is the scalar field in `self.data`.
    pub fn laplacian(&self) -> CausalTensor<T> {
        // For k=0 (scalar fields on vertices), the discrete Laplacian is Δ₀ = δ₁d₀.
        // d₀ is the coboundary operator on 0-forms, which is `coboundary_operators[0]`.
        // δ₁ is the boundary operator on 1-forms, which is `boundary_operators[1]`.

        if self.complex.boundary_operators().len() < 2
            || self.complex.coboundary_operators().is_empty()
        {
            // Not enough operators to compute Laplacian (e.g., no edges).
            let vertex_count = self.complex.skeletons().get(0).map_or(0, |s| s.simplices.len());
            return CausalTensor::new(vec![T::zero(); vertex_count], vec![vertex_count])
                .expect("Failed to create zero Laplacian");
        }

        // 1. Apply d₀ to the 0-form f to get the 1-form df.
        // exterior_derivative(0) applies coboundary_operators[0].
        let df = self.exterior_derivative(0);

        // 2. Apply δ₁ to the 1-form df.
        // This corresponds to applying the boundary_operators[1] matrix.
        let boundary_1 = &self.complex.boundary_operators()[1];
        let laplacian_f = super::utils::apply_operator(boundary_1, df.as_slice());

        let vertex_count = self.complex.skeletons().get(0).map_or(0, |s| s.simplices.len());

        CausalTensor::new(laplacian_f, vec![vertex_count])
            .expect("Failed to create Laplacian tensor")
    }
}
