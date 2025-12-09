/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::universal::SPEED_OF_LIGHT;
use crate::{AmountOfSubstance, Energy, HalfLife, Mass, PhysicsError, Time};

// Kernels

/// Calculates radioactive decay: $N(t) = N_0 \left(\frac{1}{2}\right)^{t / t_{1/2}}$.
///
/// # Arguments
/// * `n0` - Initial amount of substance $N_0$.
/// * `half_life` - Half-life $t_{1/2}$.
/// * `time` - Elapsed time $t$.
///
/// # Returns
/// * `Ok(AmountOfSubstance)` - Remaining amount $N(t)$.
pub fn radioactive_decay_kernel(
    n0: &AmountOfSubstance,
    half_life: &HalfLife,
    time: &Time,
) -> Result<AmountOfSubstance, PhysicsError> {
    // N(t) = N0 * (1/2)^(t / t_half)
    // Decay Law using Half-Life

    if half_life.value() == 0.0 {
        // Avoid division by zero, though HalfLife check usually prevents negative. 0 might be valid "instant decay"?
        // Assume 0 half life means instant decay to 0?
        return AmountOfSubstance::new(0.0);
    }

    let ratio = time.value() / half_life.value();
    let n = n0.value() * 0.5_f64.powf(ratio);
    AmountOfSubstance::new(n)
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

// Wrappers
