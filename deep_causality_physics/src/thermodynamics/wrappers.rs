/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::thermodynamics::stats;
use crate::{AmountOfSubstance, Efficiency, Energy, Pressure, Ratio, Temperature, Volume};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_tensor::CausalTensor;

use crate::Probability;
use deep_causality_topology::Manifold;

/// Causal wrapper for [`stats::heat_diffusion_kernel`].
pub fn heat_diffusion(
    temp_manifold: &Manifold<f64, f64>,
    diffusivity: f64,
) -> PropagatingEffect<CausalTensor<f64>> {
    match stats::heat_diffusion_kernel(temp_manifold, diffusivity) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::ideal_gas_law_kernel`]. Returns result as `Ratio` (or scalar).
pub fn ideal_gas_law(
    pressure: Pressure,
    volume: Volume,
    moles: AmountOfSubstance,
    temp: Temperature,
) -> PropagatingEffect<Ratio> {
    match stats::ideal_gas_law_kernel(pressure, volume, moles, temp) {
        Ok(val) => match Ratio::new(val) {
            Ok(r) => PropagatingEffect::pure(r),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::carnot_efficiency_kernel`].
pub fn carnot_efficiency(
    temp_hot: Temperature,
    temp_cold: Temperature,
) -> PropagatingEffect<Efficiency> {
    match stats::carnot_efficiency_kernel(temp_hot, temp_cold) {
        Ok(val) => match Efficiency::new(val) {
            Ok(eff) => PropagatingEffect::pure(eff),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::boltzmann_factor_kernel`].
pub fn boltzmann_factor(energy: Energy, temp: Temperature) -> PropagatingEffect<Probability> {
    match stats::boltzmann_factor_kernel(energy, temp) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::shannon_entropy_kernel`].
pub fn shannon_entropy(probs: &CausalTensor<f64>) -> PropagatingEffect<f64> {
    match stats::shannon_entropy_kernel(probs) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::heat_capacity_kernel`].
pub fn heat_capacity(diff_energy: Energy, diff_temp: Temperature) -> PropagatingEffect<f64> {
    match stats::heat_capacity_kernel(diff_energy, diff_temp) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::partition_function_kernel`].
pub fn partition_function(
    energies: &CausalTensor<f64>,
    temp: Temperature,
) -> PropagatingEffect<f64> {
    match stats::partition_function_kernel(energies, temp) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
