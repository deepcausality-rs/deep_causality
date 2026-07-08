/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `PropagatingEffect` wrappers for the hypersonic Park-2T kernels — each maps a
//! `Result<Quantity, PhysicsError>` into the causal effect monad.

use crate::kernels::hypersonic::{finite_rate, ionization, shock, thermochemistry};
use crate::{
    DissociationFraction, ElectronDensity, ElectronTemperature, EquilibriumConstant,
    IonizationFraction, ReactionRate, Temperature, VibrationalTemperature,
};
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;

pub fn vibrational_relaxation<R>(
    t_ve: VibrationalTemperature<R>,
    t_tr: Temperature<R>,
    pressure_atm: R,
    reduced_mass_amu: R,
    theta_vib: R,
    dt: R,
) -> PropagatingEffect<VibrationalTemperature<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match thermochemistry::vibrational_relaxation_kernel(
        t_ve,
        t_tr,
        pressure_atm,
        reduced_mass_amu,
        theta_vib,
        dt,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn arrhenius_rate<R>(
    temperature: Temperature<R>,
    prefactor: R,
    exponent: R,
    activation_temp: R,
) -> PropagatingEffect<ReactionRate<R>>
where
    R: RealField + MaybeParallel + Debug,
{
    match thermochemistry::arrhenius_rate_kernel(temperature, prefactor, exponent, activation_temp)
    {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn saha_ionization_fraction<R>(
    temperature: Temperature<R>,
    total_number_density: R,
    ionization_energy_ev: R,
    partition_ratio: R,
) -> PropagatingEffect<IonizationFraction<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match ionization::saha_ionization_fraction_kernel(
        temperature,
        total_number_density,
        ionization_energy_ev,
        partition_ratio,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn park2t_ionization_surrogate<R>(
    temperature: Temperature<R>,
    total_number_density: R,
) -> PropagatingEffect<IonizationFraction<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match ionization::park2t_ionization_surrogate_kernel(temperature, total_number_density) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn electron_density<R>(
    alpha: IonizationFraction<R>,
    total_number_density: R,
) -> PropagatingEffect<ElectronDensity<R>>
where
    R: RealField + MaybeParallel + Debug,
{
    match ionization::electron_density_kernel(alpha, total_number_density) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn rankine_hugoniot_temperature<R>(
    t_inf: Temperature<R>,
    mach: R,
    gamma: R,
) -> PropagatingEffect<Temperature<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match shock::rankine_hugoniot_temperature_kernel(t_inf, mach, gamma) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn recovery_temperature<R>(
    t_post: Temperature<R>,
    speed: R,
    c_p: R,
) -> PropagatingEffect<Temperature<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match shock::recovery_temperature_kernel(t_post, speed, c_p) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn no_dissociative_recombination_rate<R>(
    electron_temperature: ElectronTemperature<R>,
) -> PropagatingEffect<ReactionRate<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::no_dissociative_recombination_rate_kernel(electron_temperature) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn electron_impact_ionization_n_rate<R>(
    electron_temperature: ElectronTemperature<R>,
) -> PropagatingEffect<ReactionRate<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::electron_impact_ionization_n_rate_kernel(electron_temperature) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn electron_impact_ionization_o_rate<R>(
    electron_temperature: ElectronTemperature<R>,
) -> PropagatingEffect<ReactionRate<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::electron_impact_ionization_o_rate_kernel(electron_temperature) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn n2_dissociation_equilibrium<R>(
    temperature: Temperature<R>,
) -> PropagatingEffect<EquilibriumConstant<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::n2_dissociation_equilibrium_kernel(temperature) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn o2_dissociation_equilibrium<R>(
    temperature: Temperature<R>,
) -> PropagatingEffect<EquilibriumConstant<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::o2_dissociation_equilibrium_kernel(temperature) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn dissociation_equilibrium_fraction<R>(
    k_eq: EquilibriumConstant<R>,
    nuclei_density: R,
) -> PropagatingEffect<DissociationFraction<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::dissociation_equilibrium_fraction_kernel(k_eq, nuclei_density) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn finite_rate_ionization_fixed_point<R>(
    production: R,
    linear_coefficient: R,
    loss_coefficient: R,
) -> PropagatingEffect<ElectronDensity<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::finite_rate_ionization_fixed_point_kernel(
        production,
        linear_coefficient,
        loss_coefficient,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn no_associative_ionization_rate<R>(
    temperature: Temperature<R>,
) -> PropagatingEffect<ReactionRate<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::no_associative_ionization_rate_kernel(temperature) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn zeldovich_exchange_rate<R>(temperature: Temperature<R>) -> PropagatingEffect<ReactionRate<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::zeldovich_exchange_rate_kernel(temperature) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn park_controlling_temperature<R>(
    t_translational: Temperature<R>,
    t_vibrational: Temperature<R>,
    q: R,
) -> PropagatingEffect<Temperature<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match finite_rate::park_controlling_temperature_kernel(t_translational, t_vibrational, q) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
