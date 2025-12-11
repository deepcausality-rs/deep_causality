/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::BOLTZMANN_CONSTANT;

use crate::{AmountOfSubstance, Energy, PhysicsError, Pressure, Probability, Temperature, Volume};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

/// Calculates the Heat Equation step: $\frac{\partial u}{\partial t} = \alpha \Delta u$.
///
/// Computes the change in temperature field $u$ due to diffusion.
/// Note: The Laplacian returns positive $\Delta u$. Conservation requires $\frac{du}{dt} = - \alpha \Delta u$.
///
/// # Arguments
/// * `temp_manifold` - Manifold containing the temperature 0-form field $u$ (on vertices).
/// * `diffusivity` - Thermal diffusivity $\alpha$.
///
/// # Returns
/// * `Result<CausalTensor<f64>, PhysicsError>` - Rate of change tensor $\frac{du}{dt}$.
pub fn heat_diffusion_kernel(
    temp_manifold: &Manifold<f64>,
    diffusivity: f64,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // Heat Eq: du/dt = - alpha * Laplacian(u)

    if diffusivity < 0.0 {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::PhysicalInvariantBroken(
                "Negative diffusivity violates second law of thermodynamics".into(),
            ),
        ));
    }

    // 1. Compute Laplacian
    let laplacian = temp_manifold.laplacian(0);

    // 2. Multiply by -alpha
    // CausalTensor * scalar.
    let diff_tensor = laplacian * (-diffusivity);

    Ok(diff_tensor)
}

/// Calculates the Ideal Gas Constant $R$ from state variables: $R = \frac{PV}{nT}$.
///
/// # Arguments
/// * `pressure` - Pressure $P$.
/// * `volume` - Volume $V$.
/// * `moles` - Amount of substance $n$.
/// * `temp` - Temperature $T$.
///
/// # Returns
/// * `Ok(f64)` - Calculated Gas Constant $R$.
pub fn ideal_gas_law_kernel(
    pressure: Pressure,
    volume: Volume,
    moles: AmountOfSubstance,
    temp: Temperature,
) -> Result<f64, PhysicsError> {
    // PV = nRT -> R = PV / nT
    // Calculates the Gas Constant R implied by the state variables.
    // If input variables are consistent with Ideal Gas, R should be close to 8.314

    let p = pressure.value();
    let v = volume.value();
    let n = moles.value();
    let t = temp.value();

    if n == 0.0 || t == 0.0 {
        // Singularity or invalid state for gas law derivation
        // Technically strict zero T is allowed only if P*V is 0
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::Singularity(
                "Zero moles or zero temp in ideal gas calculation".into(),
            ),
        ));
    }

    let r = (p * v) / (n * t);
    Ok(r)
}

/// Calculates Carnot Efficiency: $\eta = 1 - \frac{T_C}{T_H}$.
///
/// # Arguments
/// * `temp_hot` - Hot reservoir temperature $T_H$.
/// * `temp_cold` - Cold reservoir temperature $T_C$.
///
/// # Returns
/// * `Ok(f64)` - Efficiency $\eta$.
pub fn carnot_efficiency_kernel(
    temp_hot: Temperature,
    temp_cold: Temperature,
) -> Result<f64, PhysicsError> {
    let th = temp_hot.value();
    let tc = temp_cold.value();

    if th <= 0.0 || tc < 0.0 {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::ZeroKelvinViolation,
        ));
    }

    if tc >= th {
        // Not a heat engine if Tc >= Th (or strictly invalid for Carnot cycle)
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::PhysicalInvariantBroken(
                "Cold reservoir >= Hot reservoir".into(),
            ),
        ));
    }

    let eff = 1.0 - (tc / th);
    Ok(eff)
}

/// Calculates the unnormalized Boltzmann factor: $e^{-E/k_BT}$.
///
/// Returns a `Probability` which clamps the value to [0, 1].
/// Assumes $E \ge 0$ relative to ground state.
///
/// # Arguments
/// * `energy` - Energy state $E$.
/// * `temp` - Temperature $T$.
///
/// # Returns
/// * `Result<Probability, PhysicsError>` - Boltzmann factor.
pub fn boltzmann_factor_kernel(
    energy: Energy,
    temp: Temperature,
) -> Result<Probability, PhysicsError> {
    // P = exp(-E / kT)  (unnormalized factor)
    // Actually usually return probability *if* normalized, but here it's likely the factor.
    // Spec says "Boltzmann Factor -> Probability".
    // Usually means exp(-beta E). If E=0, factor=1.
    // If Prob is constrained <= 1, then this assumes E >= 0 and Energy is relative to ground state.

    let e = energy.value();
    let t = temp.value();
    let k = BOLTZMANN_CONSTANT;

    if t == 0.0 {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::ZeroKelvinViolation,
        ));
    }

    let beta = 1.0 / (k * t);
    let factor = (-beta * e).exp();

    // Note: factor can be > 1 if E < 0. Assuming E is kinetic energy or excitation > 0.
    // Probability new() checks for [0, 1].

    Probability::new(factor)
}

/// Calculates Shannon Entropy: $H = -\sum p_i \ln(p_i)$.
///
/// # Arguments
/// * `probs` - Probability distribution (Tensor).
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Entropy in nats.
pub fn shannon_entropy_kernel(probs: &CausalTensor<f64>) -> Result<f64, PhysicsError> {
    // H = - Sum p_i log(p_i)
    // Using as_slice() assuming it gives access to underlying data

    // Using as_slice() assuming it gives access to underlying data
    let data = probs.as_slice();

    if data.is_empty() {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::DimensionMismatch("Probability tensor is empty".into()),
        ));
    }

    // Validate sum of probabilities ~ 1? Or just values?
    // Shannon entropy requires p >= 0.
    // Check for negative probabilities?
    if data.iter().any(|&p| p < 0.0) {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::NormalizationError(
                "Negative probability in Shannon Entropy".into(),
            ),
        ));
    }

    let entropy: f64 = data
        .iter()
        .filter(|&&p| p > 0.0) // lim x->0 x log x = 0. Exclude 0 and negative.
        .map(|&p| -p * p.ln())
        .sum();

    Ok(entropy)
}

/// Calculates Heat Capacity: $C = \frac{dE}{dT}$.
///
/// # Arguments
/// * `diff_energy` - Change in energy $dE$.
/// * `diff_temp` - Change in temperature $dT$.
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Heat capacity.
pub fn heat_capacity_kernel(
    diff_energy: Energy,
    diff_temp: Temperature,
) -> Result<f64, PhysicsError> {
    // C = dE / dT
    let de = diff_energy.value();
    let dt = diff_temp.value();

    if dt == 0.0 {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::PhysicalInvariantBroken(
                "Zero temperature difference in heat capacity".into(),
            ),
        ));
    }

    let c = de / dt;
    Ok(c)
}

/// Calculates Partition Function: $Z = \sum e^{-E_i / k_B T}$.
///
/// # Arguments
/// * `energies` - List of energy states (Tensor).
/// * `temp` - Temperature $T$.
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Partition function $Z$.
pub fn partition_function_kernel(
    energies: &CausalTensor<f64>,
    temp: Temperature,
) -> Result<f64, PhysicsError> {
    // Z = Sum exp(-E_i / kT)

    let t = temp.value();
    let k = BOLTZMANN_CONSTANT;

    // Check T=0 handled?
    if t == 0.0 {
        // If T=0, only ground state contributes? Or undefined?
        // Let's return error.
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::ZeroKelvinViolation,
        ));
    }

    let beta = 1.0 / (k * t);
    if !beta.is_finite() {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::NumericalInstability(
                "Invalid beta in partition function".into(),
            ),
        ));
    }
    let data = energies.as_slice();

    // Prevent overflow in exp by clamping exponent to a safe range
    // f64::EXP_MAX ~ 709 for e^x; clamp to, e.g., [-700, 700]
    let z: f64 = data
        .iter()
        .map(|&e| {
            let x = (-beta * e).clamp(-700.0, 700.0);
            x.exp()
        })
        .sum();

    if !z.is_finite() {
        return Err(PhysicsError::new(
            crate::error::PhysicsErrorEnum::NumericalInstability(
                "Non-finite partition function value".into(),
            ),
        ));
    }

    Ok(z)
}
