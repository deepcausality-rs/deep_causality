/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::thermodynamics::stats;
use crate::{AmountOfSubstance, Efficiency, Energy, Pressure, Ratio, Temperature, Volume};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_tensor::CausalTensor;

use crate::Probability;
use core::fmt::Debug;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_topology::SimplicialManifold;

/// Causal wrapper for [`stats::heat_diffusion_kernel`].
pub fn heat_diffusion<R>(
    temp_manifold: &SimplicialManifold<R, R>,
    diffusivity: R,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + FromPrimitive + Default + PartialEq + Debug,
{
    match stats::heat_diffusion_kernel(temp_manifold, diffusivity) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::ideal_gas_law_kernel`]. Returns result as `Ratio` (or scalar).
pub fn ideal_gas_law<R>(
    pressure: Pressure<R>,
    volume: Volume<R>,
    moles: AmountOfSubstance<R>,
    temp: Temperature<R>,
) -> PropagatingEffect<Ratio<R>>
where
    R: RealField + Debug,
{
    match stats::ideal_gas_law_kernel(pressure, volume, moles, temp) {
        Ok(val) => match Ratio::<R>::new(val) {
            Ok(r) => PropagatingEffect::pure(r),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::carnot_efficiency_kernel`].
pub fn carnot_efficiency<R>(
    temp_hot: Temperature<R>,
    temp_cold: Temperature<R>,
) -> PropagatingEffect<Efficiency<R>>
where
    R: RealField + Debug,
{
    match stats::carnot_efficiency_kernel(temp_hot, temp_cold) {
        Ok(val) => match Efficiency::<R>::new(val) {
            Ok(eff) => PropagatingEffect::pure(eff),
            Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::boltzmann_factor_kernel`].
pub fn boltzmann_factor<R>(
    energy: Energy<R>,
    temp: Temperature<R>,
) -> PropagatingEffect<Probability<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match stats::boltzmann_factor_kernel(energy, temp) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::shannon_entropy_kernel`].
pub fn shannon_entropy<R>(probs: &CausalTensor<R>) -> PropagatingEffect<R>
where
    R: RealField + core::iter::Sum + Default + Debug,
{
    match stats::shannon_entropy_kernel(probs) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::heat_capacity_kernel`].
pub fn heat_capacity<R>(diff_energy: Energy<R>, diff_temp: Temperature<R>) -> PropagatingEffect<R>
where
    R: RealField + Default + Debug,
{
    match stats::heat_capacity_kernel(diff_energy, diff_temp) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`stats::partition_function_kernel`].
pub fn partition_function<R>(
    energies: &CausalTensor<R>,
    temp: Temperature<R>,
) -> PropagatingEffect<R>
where
    R: RealField + FromPrimitive + core::iter::Sum + Default + Debug,
{
    match stats::partition_function_kernel(energies, temp) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
