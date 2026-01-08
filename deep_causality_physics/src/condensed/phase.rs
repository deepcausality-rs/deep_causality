/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ChemicalPotentialGradient, Concentration, Energy, Mobility, OrderParameter, PhysicsError,
    VectorPotential,
};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{Complex, DivisionAlgebra};
use deep_causality_tensor::CausalTensor;

/// Calculates the Ginzburg-Landau Free Energy density.
///
/// Implements the phenomenological free energy functional for a superconductor or superfluid near
/// the critical temperature $T_c$.
///
/// $$ F = \alpha |\psi|^2 + \frac{\beta}{2} |\psi|^4 + |(\nabla - i \mathbf{A})\psi|^2 $$
///
/// # Physical Model
/// *   **Order Parameter ($\psi$)**: Complex scalar field representing the superconducting condensate.
/// *   **Potential Term**: $\alpha |\psi|^2 + \frac{\beta}{2} |\psi|^4$. Describes the phase transition.
///     *   $\alpha < 0$ below $T_c$ (superconducting state).
///     *   $\alpha > 0$ above $T_c$ (normal state).
/// *   **Kinetic/Gauge Term**: $|(\nabla - i \mathbf{A})\psi|^2$. Describes the energy cost of
///     spatial variations and coupling to the magnetic vector potential $\mathbf{A}$.
///     *   Note: This implementation assumes natural units where effective charge/mass factors
///         (like $\frac{1}{2m^*}$) are absorbed into the input fields or coefficients.
///
/// # Arguments
/// *   `psi` - Order parameter $\psi$ (Complex).
/// *   `alpha` - Phenomenological coefficient $\alpha(T)$.
/// *   `beta` - Phenomenological coefficient $\beta$ (must be positive for stability).
/// *   `gradient_psi` - Gradient vector $\nabla \psi$ (Complex MultiVector).
/// *   `vector_potential` - Electromagnetic vector potential $\mathbf{A}$ (Real MultiVector).
///     If `None`, $\mathbf{A}=0$ is assumed.
///
/// # Returns
/// *   `Result<Energy, PhysicsError>` - Free energy density.
pub fn ginzburg_landau_free_energy_kernel(
    psi: OrderParameter,
    alpha: f64,
    beta: f64,
    gradient_psi: &CausalMultiVector<Complex<f64>>,
    vector_potential: Option<&VectorPotential>,
) -> Result<Energy, PhysicsError> {
    let val = psi.value();
    let mag_sq = psi.magnitude_squared();

    // Potential term
    let potential_term = alpha * mag_sq + (beta / 2.0) * mag_sq * mag_sq;

    // Kinetic term: |(grad - iA)psi|^2
    // Calculation iterates over vector components.
    let kinetic_norm_sq = if let Some(a_wrapper) = vector_potential {
        let a = a_wrapper.inner();
        if a.metric() != gradient_psi.metric() {
            return Err(PhysicsError::DimensionMismatch(
                "Metric mismatch between gradient and vector potential".into(),
            ));
        }

        let i_psi = Complex::new(0.0, 1.0) * val;

        let a_data = a.data();
        let grad_data = gradient_psi.data();

        if a_data.len() != grad_data.len() {
            return Err(PhysicsError::DimensionMismatch(
                "Vector size mismatch".into(),
            ));
        }

        gradient_psi
            .data()
            .iter()
            .zip(a.data().iter())
            .map(|(g, a_val)| {
                // Component: grad_k - i * A_k * psi
                let term_a = Complex::new(*a_val, 0.0) * i_psi;
                (*g - term_a).norm_sqr()
            })
            .sum::<f64>()
    } else {
        // A = 0 case
        gradient_psi
            .data()
            .iter()
            .map(|c| c.norm_sqr())
            .sum::<f64>()
    };

    let total = potential_term + kinetic_norm_sq;
    Energy::new(total)
}

/// Calculates the Cahn-Hilliard Flux with degenerate mobility (Type B).
///
/// $$ \mathbf{J} = -M(c) \nabla \mu $$
///
/// # Physical Model
/// Describes the flux of a conserved order parameter $c$ (concentration) driven by the chemical potential gradient.
/// This implementation uses a **degenerate mobility** model, where diffusion vanishes in the pure phases ($c=0, 1$).
///
/// $$ M(c) = M_0 c (1 - c) $$
///
/// This ensures the concentration remains bounded within $[0, 1]$ during evolution.
///
/// # Arguments
/// *   `concentration` - Local concentration field $c$ (Scalar Tensor).
/// *   `mobility` - Base mobility coefficient $M_0$.
/// *   `chem_potential_grad` - Gradient of the chemical potential $\nabla \mu$ (Vector Field Tensor).
///
/// # Implementation Details
/// *   **Clamping**: The concentration $c$ used in the mobility calculation is clamped to $[0, 1]$ to
///     prevent non-physical negative mobility if numerical noise causes $c$ to exceed bounds.
/// *   **Element-wise Operation**: The flux is computed by iterating over the tensor data slices
///     to handle the scalar-vector multiplication correctly.
///
/// # Returns
/// *   `Result<CausalTensor<f64>, PhysicsError>` - Flux vector field $\mathbf{J}$.
pub fn cahn_hilliard_flux_kernel(
    concentration: &Concentration,
    mobility: Mobility,
    chem_potential_grad: &ChemicalPotentialGradient,
) -> Result<CausalTensor<f64>, PhysicsError> {
    let grad_mu = chem_potential_grad.inner();
    let c_tensor = concentration.inner();
    let m0 = mobility.value();

    if c_tensor.shape() != grad_mu.shape() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Concentration shape {:?} != Gradient shape {:?}",
            c_tensor.shape(),
            grad_mu.shape()
        )));
    }

    // 1. Calculate degenerate mobility field M(c)
    // Create ones tensor for (1-c) term
    let ones: CausalTensor<f64> = CausalTensor::one(c_tensor.shape());
    let one_minus_c: CausalTensor<f64> = ones - c_tensor.clone();

    // M(c) = M0 * c * (1 - c)
    let c_factor = c_tensor.clone() * one_minus_c;
    let mobility_field: CausalTensor<f64> = c_factor * m0;

    // 2. Calculate Flux J = - M(c) * grad_mu
    let m_data = mobility_field.as_slice();
    let g_data = grad_mu.as_slice();

    if m_data.len() != g_data.len() {
        return Err(PhysicsError::DimensionMismatch(
            "Mobility field size does not match gradient field size".into(),
        ));
    }

    // Apply flux formula element-wise with stability clamping
    let flux_data: Vec<f64> = m_data
        .iter()
        .zip(g_data.iter())
        .map(|(&m_val, &g_val): (&f64, &f64)| {
            // Clamp mobility to be non-negative
            let m_clamped = m_val.max(0.0);
            -m_clamped * g_val
        })
        .collect();

    CausalTensor::new(flux_data, grad_mu.shape().to_vec()).map_err(PhysicsError::from)
}
