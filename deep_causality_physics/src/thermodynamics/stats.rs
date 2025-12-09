/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Efficiency;
use crate::Ratio;
use crate::constants::thermodynamics::BOLTZMANN_CONSTANT;
use crate::dynamics::quantities::Volume;
use crate::fluids::quantities::Pressure;
use crate::nuclear::quantities::AmountOfSubstance;
use crate::quantum::quantities::{Energy, Probability};
use crate::thermodynamics::quantities::Temperature;
// use crate::constants::thermodynamics::AVOGADRO_CONSTANT; // For R calculation if needed, or universal constant usage

use crate::error::PhysicsError;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_tensor::CausalTensor;

// Kernels

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

// Wrappers

pub fn ideal_gas_law(
    pressure: Pressure,
    volume: Volume,
    moles: AmountOfSubstance,
    temp: Temperature,
) -> PropagatingEffect<Ratio> {
    match ideal_gas_law_kernel(pressure, volume, moles, temp) {
        Ok(val) => match Ratio::new(val) {
            Ok(r) => PropagatingEffect::pure(r),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn carnot_efficiency(
    temp_hot: Temperature,
    temp_cold: Temperature,
) -> PropagatingEffect<Efficiency> {
    match carnot_efficiency_kernel(temp_hot, temp_cold) {
        Ok(val) => match Efficiency::new(val) {
            Ok(eff) => PropagatingEffect::pure(eff),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn boltzmann_factor(energy: Energy, temp: Temperature) -> PropagatingEffect<Probability> {
    // P = exp(-E / kT)  (unnormalized factor)
    // Actually usually return probability *if* normalized, but here it's likely the factor.
    // Spec says "Boltzmann Factor -> Probability".
    // Usually means exp(-beta E). If E=0, factor=1.
    // If Prob is constrained <= 1, then this assumes E >= 0 and Energy is relative to ground state.

    let e = energy.value();
    let t = temp.value();
    let k = BOLTZMANN_CONSTANT;

    if t == 0.0 {
        return PropagatingEffect::from_error(CausalityError::from(PhysicsError::new(
            crate::error::PhysicsErrorEnum::ZeroKelvinViolation,
        )));
    }

    let beta = 1.0 / (k * t);
    let factor = (-beta * e).exp();

    // Note: factor can be > 1 if E < 0. Assuming E is kinetic energy or excitation > 0.
    // Probability new() checks for [0, 1].

    match Probability::new(factor) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn shannon_entropy(probs: &CausalTensor<f64>) -> PropagatingEffect<f64> {
    // H = - Sum p_i log(p_i)
    // Using as_slice() assuming it gives access to underlying data

    let data = probs.as_slice();
    let entropy: f64 = data
        .iter()
        .filter(|&&p| p > 0.0) // lim x->0 x log x = 0. Exclude 0 and negative.
        .map(|&p| -p * p.ln())
        .sum();

    PropagatingEffect::pure(entropy)
}

pub fn heat_capacity(diff_energy: Energy, diff_temp: Temperature) -> PropagatingEffect<f64> {
    // C = dE / dT
    let de = diff_energy.value();
    let dt = diff_temp.value();

    if dt == 0.0 {
        return PropagatingEffect::from_error(CausalityError::from(PhysicsError::new(
            crate::error::PhysicsErrorEnum::PhysicalInvariantBroken(
                "Zero temperature difference in heat capacity".into(),
            ),
        )));
    }

    let c = de / dt;
    PropagatingEffect::pure(c)
}

pub fn partition_function(
    energies: &CausalTensor<f64>,
    temp: Temperature,
) -> PropagatingEffect<f64> {
    // Z = Sum exp(-E_i / kT)

    let t = temp.value();
    let k = BOLTZMANN_CONSTANT;

    // Check T=0 handled?
    if t == 0.0 {
        // If T=0, only ground state contributes? Or undefined?
        // Let's return error.
        return PropagatingEffect::from_error(CausalityError::from(PhysicsError::new(
            crate::error::PhysicsErrorEnum::ZeroKelvinViolation,
        )));
    }

    let beta = 1.0 / (k * t);
    let data = energies.as_slice();

    let z: f64 = data.iter().map(|&e| (-beta * e).exp()).sum();

    PropagatingEffect::pure(z)
}
