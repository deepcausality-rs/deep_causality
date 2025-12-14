/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::condensed::moire;
use crate::condensed::phase;
use crate::condensed::qgt;
use crate::{
    BandDrudeWeight, ChemicalPotentialGradient, Concentration, Displacement, Energy, Length,
    Mobility, Momentum, OrderParameter, QuantumEigenvector, QuantumMetric, QuantumVelocity, Ratio,
    Speed, Stiffness, TwistAngle, VectorPotential,
};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

// ============================================================================
// QGT Wrappers
// ============================================================================

/// Wrapper for [`qgt::quantum_geometric_tensor_kernel`].
pub fn quantum_geometric_tensor(
    eigenvalues: &CausalTensor<f64>,
    eigenvectors: &QuantumEigenvector,
    velocity_i: &QuantumVelocity,
    velocity_j: &QuantumVelocity,
    band_n: usize,
    regularization: f64,
) -> PropagatingEffect<Complex<f64>> {
    match qgt::quantum_geometric_tensor_kernel(
        eigenvalues,
        eigenvectors,
        velocity_i,
        velocity_j,
        band_n,
        regularization,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Wrapper for [`qgt::quasi_qgt_kernel`].
pub fn quasi_qgt(
    eigenvalues: &CausalTensor<f64>,
    eigenvectors: &QuantumEigenvector,
    velocity_i: &QuantumVelocity,
    velocity_j: &QuantumVelocity,
    band_n: usize,
    regularization: f64,
) -> PropagatingEffect<Complex<f64>> {
    match qgt::quasi_qgt_kernel(
        eigenvalues,
        eigenvectors,
        velocity_i,
        velocity_j,
        band_n,
        regularization,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Wrapper for [`qgt::effective_band_drude_weight_kernel`].
pub fn effective_band_drude_weight(
    energy_n: Energy,
    energy_0: Energy,
    curvature_ii: f64,
    quantum_metric: QuantumMetric,
    lattice_const: Length,
) -> PropagatingEffect<BandDrudeWeight> {
    match qgt::effective_band_drude_weight_kernel(
        energy_n,
        energy_0,
        curvature_ii,
        quantum_metric,
        lattice_const,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Moiré Wrappers
// ============================================================================

/// Wrapper for [`moire::bistritzer_macdonald_kernel`].
pub fn bistritzer_macdonald(
    twist_angle: TwistAngle,
    interlayer_coupling: Energy,
    fermi_velocity: Speed,
    k_point: Momentum,
    shell_cutoff: usize,
) -> PropagatingEffect<CausalTensor<Complex<f64>>> {
    match moire::bistritzer_macdonald_kernel(
        twist_angle,
        interlayer_coupling,
        fermi_velocity,
        k_point,
        shell_cutoff,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Wrapper for [`moire::foppl_von_karman_strain_simple_kernel`].
pub fn foppl_von_karman_strain_simple(
    displacement_u: &Displacement,
    youngs_modulus: Stiffness,
    poisson_ratio: Ratio,
) -> PropagatingEffect<CausalTensor<f64>> {
    match moire::foppl_von_karman_strain_simple_kernel(
        displacement_u,
        youngs_modulus,
        poisson_ratio,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Wrapper for [`moire::foppl_von_karman_strain_kernel`].
pub fn foppl_von_karman_strain(
    u_manifold: &Manifold<f64>,
    w_manifold: &Manifold<f64>,
    youngs_modulus: Stiffness,
    poisson_ratio: Ratio,
) -> PropagatingEffect<CausalTensor<f64>> {
    match moire::foppl_von_karman_strain_kernel(
        u_manifold,
        w_manifold,
        youngs_modulus,
        poisson_ratio,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Phase Wrappers
// ============================================================================

/// Wrapper for [`phase::ginzburg_landau_free_energy_kernel`].
pub fn ginzburg_landau_free_energy(
    psi: OrderParameter,
    alpha: f64,
    beta: f64,
    gradient_psi: &CausalMultiVector<Complex<f64>>,
    vector_potential: Option<&VectorPotential>,
) -> PropagatingEffect<Energy> {
    match phase::ginzburg_landau_free_energy_kernel(
        psi,
        alpha,
        beta,
        gradient_psi,
        vector_potential,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Wrapper for [`phase::cahn_hilliard_flux_kernel`].
pub fn cahn_hilliard_flux(
    concentration: &Concentration,
    mobility: Mobility,
    chem_potential_grad: &ChemicalPotentialGradient,
) -> PropagatingEffect<CausalTensor<f64>> {
    match phase::cahn_hilliard_flux_kernel(concentration, mobility, chem_potential_grad) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
