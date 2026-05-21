/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::universal::SPEED_OF_LIGHT;
use crate::{AmountOfSubstance, Energy, HalfLife, Mass, PhysicsError, Time};
use deep_causality_num::{FromPrimitive, RealField};

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
pub fn radioactive_decay_kernel<R>(
    n0: &AmountOfSubstance<R>,
    half_life: &HalfLife<R>,
    time: &Time<f64>,
) -> Result<AmountOfSubstance<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let zero = R::zero();
    if half_life.value() == zero {
        return Err(PhysicsError::Singularity(
            "Radioactive half-life cannot be zero".into(),
        ));
    }

    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let t = R::from_f64(time.value())
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(time) failed".into()))?;

    let decay_ratio = t / half_life.value();
    let remaining = n0.value() * two.powf(-decay_ratio);

    AmountOfSubstance::new(remaining)
}

/// Calculates nuclear binding energy (mass defect): $E = \Delta m c^2$.
///
/// # Arguments
/// * `mass_defect` - Mass difference $\Delta m$.
///
/// # Returns
/// * `Ok(Energy)` - Binding energy $E$.
pub fn binding_energy_kernel(mass_defect: &Mass) -> Result<Energy<f64>, PhysicsError> {
    // E = m c^2
    // Mass-Energy Equivalence
    let c = SPEED_OF_LIGHT;
    let e = mass_defect.value() * c * c;
    Energy::new(e)
}
