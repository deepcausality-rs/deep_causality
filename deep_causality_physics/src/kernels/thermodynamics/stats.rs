/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::BOLTZMANN_CONSTANT;
use crate::{AmountOfSubstance, Energy, PhysicsError, Pressure, Probability, Temperature, Volume};
use core::fmt::Debug;
use core::iter::Sum;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

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
pub fn heat_diffusion_kernel<R>(
    temp_manifold: &SimplicialManifold<R, R>,
    diffusivity: R,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    if diffusivity < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Negative diffusivity violates second law of thermodynamics".into(),
        ));
    }

    let laplacian = temp_manifold.laplacian(0);

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
pub fn ideal_gas_law_kernel<R>(
    pressure: Pressure<R>,
    volume: Volume<R>,
    moles: AmountOfSubstance<R>,
    temp: Temperature<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel,
{
    let p = pressure.value();
    let v = volume.value();
    let n = moles.value();
    let t = temp.value();

    if n == R::zero() || t == R::zero() {
        return Err(PhysicsError::Singularity(
            "Zero moles or zero temp in ideal gas calculation".into(),
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
pub fn carnot_efficiency_kernel<R>(
    temp_hot: Temperature<R>,
    temp_cold: Temperature<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel,
{
    let th = temp_hot.value();
    let tc = temp_cold.value();

    if th <= R::zero() || tc < R::zero() {
        return Err(PhysicsError::ZeroKelvinViolation());
    }

    if tc >= th {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Cold reservoir >= Hot reservoir".into(),
        ));
    }

    let eff = R::one() - (tc / th);
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
pub fn boltzmann_factor_kernel<R>(
    energy: Energy<R>,
    temp: Temperature<R>,
) -> Result<Probability<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
{
    if temp.value() == R::zero() {
        return Err(PhysicsError::ZeroKelvinViolation());
    }

    let e = energy.value();
    let t = temp.value();
    let k = R::from_f64(BOLTZMANN_CONSTANT).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(BOLTZMANN_CONSTANT) failed".into())
    })?;

    let beta = R::one() / (k * t);
    let factor = (-beta * e).exp();

    Probability::<R>::new(factor)
}

/// Calculates Shannon Entropy: $H = -\sum p_i \ln(p_i)$.
///
/// # Arguments
/// * `probs` - Probability distribution (Tensor).
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Entropy in nats.
pub fn shannon_entropy_kernel<R>(probs: &CausalTensor<R>) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel + Sum,
{
    let data = probs.as_slice();

    if data.is_empty() {
        return Err(PhysicsError::DimensionMismatch(
            "Probability tensor is empty".into(),
        ));
    }

    if data.iter().any(|&p| p < R::zero()) {
        return Err(PhysicsError::NormalizationError(
            "Negative probability in Shannon Entropy".into(),
        ));
    }

    let entropy: R = data
        .iter()
        .filter(|&&p| p > R::zero()) // lim x->0 x log x = 0. Exclude 0 and negative.
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
pub fn heat_capacity_kernel<R>(
    diff_energy: Energy<R>,
    diff_temp: Temperature<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel,
{
    if diff_temp.value() == R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Zero temperature difference in heat capacity".into(),
        ));
    }

    let de = diff_energy.value();
    let dt = diff_temp.value();

    Ok(de / dt)
}

/// Calculates Partition Function: $Z = \sum e^{-E_i / k_B T}$.
///
/// # Arguments
/// * `energies` - List of energy states (Tensor).
/// * `temp` - Temperature $T$.
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Partition function $Z$.
pub fn partition_function_kernel<R>(
    energies: &CausalTensor<R>,
    temp: Temperature<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + Sum,
{
    let t = temp.value();
    let k = R::from_f64(BOLTZMANN_CONSTANT).ok_or_else(|| {
        PhysicsError::NumericalInstability("R::from_f64(BOLTZMANN_CONSTANT) failed".into())
    })?;

    if t == R::zero() {
        return Err(PhysicsError::ZeroKelvinViolation());
    }

    let beta = R::one() / (k * t);
    if !beta.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Invalid beta in partition function".into(),
        ));
    }
    let data = energies.as_slice();

    // Prevent overflow in exp by clamping exponent to a safe range
    let lo = R::from_f64(-700.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(-700)".into()))?;
    let hi = R::from_f64(700.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(700)".into()))?;
    let z: R = data
        .iter()
        .map(|&e| {
            let x = (-beta * e).clamp(lo, hi);
            x.exp()
        })
        .sum();

    if !z.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Non-finite partition function value".into(),
        ));
    }

    Ok(z)
}
