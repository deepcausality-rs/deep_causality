/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::condensed::moire;
use crate::kernels::condensed::phase;
use crate::kernels::condensed::qgt;
use crate::{
    BandDrudeWeight, ChemicalPotentialGradient, Concentration, Displacement, Energy, Length,
    Mobility, Momentum, OrderParameter, QuantumEigenvector, QuantumMetric, QuantumVelocity, Ratio,
    Speed, Stiffness, TwistAngle, VectorPotential,
};
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;
use std::iter::Sum;

// ============================================================================
// QGT Wrappers
// ============================================================================

/// Wrapper for [`qgt::quantum_geometric_tensor_kernel`].
pub fn quantum_geometric_tensor<R>(
    eigenvalues: &CausalTensor<R>,
    eigenvectors: &QuantumEigenvector<R>,
    velocity_i: &QuantumVelocity<R>,
    velocity_j: &QuantumVelocity<R>,
    band_n: usize,
    regularization: R,
) -> PropagatingEffect<Complex<R>>
where
    R: RealField + Default + Debug,
{
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
pub fn quasi_qgt<R>(
    eigenvalues: &CausalTensor<R>,
    eigenvectors: &QuantumEigenvector<R>,
    velocity_i: &QuantumVelocity<R>,
    velocity_j: &QuantumVelocity<R>,
    band_n: usize,
    regularization: R,
) -> PropagatingEffect<Complex<R>>
where
    R: RealField + Default + Debug,
{
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
pub fn effective_band_drude_weight<R>(
    energy_n: Energy<R>,
    energy_0: Energy<R>,
    curvature_ii: R,
    quantum_metric: QuantumMetric<R>,
    lattice_const: Length<R>,
) -> PropagatingEffect<BandDrudeWeight<R>>
where
    R: RealField + Default + Debug,
{
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
pub fn bistritzer_macdonald<R>(
    twist_angle: TwistAngle<R>,
    interlayer_coupling: Energy<R>,
    fermi_velocity: Speed<R>,
    k_point: Momentum<R>,
    shell_cutoff: usize,
) -> PropagatingEffect<CausalTensor<Complex<R>>>
where
    R: RealField + FromPrimitive + Default + Debug,
{
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
pub fn foppl_von_karman_strain_simple<R>(
    displacement_u: &Displacement<R>,
    youngs_modulus: Stiffness<R>,
    poisson_ratio: Ratio<R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + FromPrimitive + Default + Debug,
{
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
pub fn foppl_von_karman_strain<R>(
    u_manifold: &SimplicialManifold<R, R>,
    w_manifold: &SimplicialManifold<R, R>,
    youngs_modulus: Stiffness<R>,
    poisson_ratio: Ratio<R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + FromPrimitive + MaybeParallel + Default + Debug,
{
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
pub fn ginzburg_landau_free_energy<R>(
    psi: OrderParameter<R>,
    alpha: R,
    beta: R,
    gradient_psi: &CausalMultiVector<Complex<R>>,
    vector_potential: Option<&VectorPotential<R>>,
) -> PropagatingEffect<Energy<R>>
where
    R: RealField + FromPrimitive + Default + Debug + Sum,
{
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
pub fn cahn_hilliard_flux<R>(
    concentration: &Concentration<R>,
    mobility: Mobility<R>,
    chem_potential_grad: &ChemicalPotentialGradient<R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + Default + Debug,
{
    match phase::cahn_hilliard_flux_kernel(concentration, mobility, chem_potential_grad) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
