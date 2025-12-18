/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::universal::SPEED_OF_LIGHT;
use crate::{AmountOfSubstance, Energy, HalfLife, Mass, PhysicsError, Time};

// Kernels

/// Calculates the remaining amount of a radioactive substance: $N(t) = N_0 \cdot 2^{-t / t_{1/2}}$.
///
/// This kernel models the exponential decay of a quantity over time based on its half-life.
/// The decay follows the standard determining equation:
/// $$ N(t) = N_0 e^{-\lambda t} $$
/// where $\lambda = \frac{\ln(2)}{t_{1/2}}$.
///
/// # Arguments
/// * `n0` - Initial amount of substance $N_0$ (moles, particles, or activity).
/// * `half_life` - The time $t_{1/2}$ required for the quantity to reduce to half its initial value.
/// * `time` - The elapsed time interval $t$.
///
/// # Returns
/// * `Ok(AmountOfSubstance)` - The remaining amount of substance $N(t)$.
///
/// # Errors
/// * `Singularity` - If `half_life` is zero (infinite decay rate).
pub fn radioactive_decay_kernel(
    n0: &AmountOfSubstance,
    half_life: &HalfLife,
    time: &Time,
) -> Result<AmountOfSubstance, PhysicsError> {
    if half_life.value() == 0.0 {
        return Err(PhysicsError::Singularity(
            "Radioactive half-life cannot be zero".into(),
        ));
    }

    // Calculation: N(t) = N0 * 2^(-t / t_half)
    // We use base 2 for numerical stability with half-life calculations.
    let decay_ratio = time.value() / half_life.value();
    let remaining = n0.value() * 2.0_f64.powf(-decay_ratio);

    AmountOfSubstance::new(remaining)
}

/// Calculates nuclear binding energy (mass defect): $E = \Delta m c^2$.
///
/// # Arguments
/// * `mass_defect` - Mass difference $\Delta m$.
///
/// # Returns
/// * `Ok(Energy)` - Binding energy $E$.
pub fn binding_energy_kernel(mass_defect: &Mass) -> Result<Energy, PhysicsError> {
    // E = m c^2
    // Mass-Energy Equivalence
    let c = SPEED_OF_LIGHT;
    let e = mass_defect.value() * c * c;
    Energy::new(e)
}

/// Simulates phenomenological hadronization: $E_{\text{density}} \rightarrow \vec{p}_{\text{jet}}$.
///
/// This kernel implements a simplified string fragmentation model where local energy density
/// fluctuations exceeding a critical threshold $E_c$ condense into particle momentum vectors (jets).
///
/// Under the 'Local Parton-Hadron Duality' (LPHD) hypothesis, the energy density maps directly
/// to the momentum magnitude of the resulting jet, assuming a massless limit ($E \approx |p|c$).
/// The resulting momentum is assigned to the primary spatial component ($x$-axis/index 1) of the
/// CausalMultiVector, representing a collimated jet in the local coordinate frame.
///
/// # Arguments
/// * `energy_density` - A slice of scalar energy density fields (0-forms).
/// * `threshold` - The critical deconfinement energy density $E_c$.
/// * `dim` - The spatial dimension $d$ of the resulting manifold (requires $d > 0$).
///
/// # Returns
/// * `Result<Vec<PhysicalVector>, PhysicsError>` - A collection of generated momentum vectors.
///
/// # Errors
/// * `DimensionMismatch` - If `dim == 0`.
/// * `PhysicalInvariantBroken` - If `threshold < 0`.
/// * `NumericalInstability` - If vector creation fails.
pub fn hadronization_kernel(
    energy_density: &[crate::nuclear::quantities::EnergyDensity],
    threshold: f64,
    dim: usize,
) -> Result<Vec<crate::dynamics::kinematics::PhysicalVector>, PhysicsError> {
    if dim == 0 {
        return Err(PhysicsError::DimensionMismatch(
            "Spatial dimension must be > 0".into(),
        ));
    }

    if threshold < 0.0 {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Hadronization threshold cannot be negative".into(),
        ));
    }

    // Heuristic: Pre-allocate assuming a sparse hadronization fraction (e.g., 20%) to minimize re-allocations
    // without over-committing memory for large empty regions.
    let estimated_capacity = (energy_density.len() as f64 * 0.2).ceil() as usize;
    let mut particles = Vec::with_capacity(estimated_capacity);

    // Euclidean signature for the spatial part of the momentum 4-vector
    let metric = deep_causality_multivector::Metric::Euclidean(dim);
    // Grade 1 (Vector) components start at index 1 (Index 0 is Scalar)
    let vector_component_index = 1;
    let multivector_size = 1 << dim;

    for &density_wrapper in energy_density {
        let density = density_wrapper.value();

        if density > threshold {
            let mut components = vec![0.0; multivector_size];

            // Assign excess energy density to momentum magnitude.
            // In a full simulation, this vector direction would be determined by the
            // energy gradient $\nabla E$ or flow tensor. Here, we align with the primary axis.
            if multivector_size > vector_component_index {
                components[vector_component_index] = density;
            }

            // Note: Metric is Copy, so it's implicitly copied here.
            let p_vec = deep_causality_multivector::CausalMultiVector::new(components, metric)
                .map_err(|e| {
                    PhysicsError::NumericalInstability(format!(
                        "Failed to construct momentum vector: {}",
                        e
                    ))
                })?;

            particles.push(crate::dynamics::kinematics::PhysicalVector(p_vec));
        }
    }

    Ok(particles)
}
