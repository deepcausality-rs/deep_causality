/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `PropagatingEffect` wrappers for the propulsion kernels — each maps a
//! `Result<Quantity, PhysicsError>` into the causal effect monad, following
//! the house convention (kernel name minus the `_kernel` suffix).

use crate::kernels::propulsion::{descent, nozzle, performance, plume, srp};
use crate::{
    Acceleration, Area, Density, FlowBranch, Force, Length, Mass, MassFlowRate, NozzleExitState,
    PlumeGeometry, Pressure, Speed, Temperature,
};
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;

pub fn propellant_mass_flow<R>(thrust: Force<R>, isp_s: R) -> PropagatingEffect<MassFlowRate<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match performance::propellant_mass_flow_kernel(thrust, isp_s) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn tsiolkovsky_delta_v<R>(
    isp_s: R,
    initial_mass: Mass<R>,
    final_mass: Mass<R>,
) -> PropagatingEffect<Speed<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match performance::tsiolkovsky_delta_v_kernel(isp_s, initial_mass, final_mass) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn inverse_area_mach<R>(area_ratio: R, gamma: R, branch: FlowBranch) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match nozzle::inverse_area_mach_kernel(area_ratio, gamma, branch) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn nozzle_exit_state<R>(
    chamber_pressure: Pressure<R>,
    chamber_temperature: Temperature<R>,
    area_ratio: R,
    gamma: R,
    r_specific: R,
) -> PropagatingEffect<NozzleExitState<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match nozzle::nozzle_exit_state_kernel(
        chamber_pressure,
        chamber_temperature,
        area_ratio,
        gamma,
        r_specific,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn stopping_distance<R>(
    speed: Speed<R>,
    net_deceleration: Acceleration<R>,
) -> PropagatingEffect<Length<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match descent::stopping_distance_kernel(speed, net_deceleration) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn ignition_altitude<R>(
    speed: Speed<R>,
    thrust_acceleration: Acceleration<R>,
    gravity: Acceleration<R>,
    margin: Length<R>,
) -> PropagatingEffect<Length<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match descent::ignition_altitude_kernel(speed, thrust_acceleration, gravity, margin) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn suicide_burn_deceleration<R>(
    speed: Speed<R>,
    altitude: Length<R>,
    gravity: Acceleration<R>,
) -> PropagatingEffect<Acceleration<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match descent::suicide_burn_deceleration_kernel(speed, altitude, gravity) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn srp_thrust_coefficient<R>(
    thrust: Force<R>,
    q_inf: Pressure<R>,
    s_ref: Area<R>,
) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match srp::srp_thrust_coefficient_kernel(thrust, q_inf, s_ref) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn momentum_flux_ratio<R>(
    rho_jet: Density<R>,
    u_jet: Speed<R>,
    rho_inf: Density<R>,
    u_inf: Speed<R>,
) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match srp::momentum_flux_ratio_kernel(rho_jet, u_jet, rho_inf, u_inf) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn srp_preserved_drag_fraction<R>(c_t: R) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match srp::srp_preserved_drag_fraction_kernel(c_t) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn jarvinen_adams_baseline_axial_coefficient<R>(mach: R) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match srp::jarvinen_adams_baseline_axial_coefficient_kernel(mach) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn srp_total_axial_force_coefficient<R>(c_t: R, mach: R) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match srp::srp_total_axial_force_coefficient_kernel(c_t, mach) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn srp_flow_regime_margin<R>(c_t: R, transition_c_t: R) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match srp::srp_flow_regime_margin_kernel(c_t, transition_c_t) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn prandtl_meyer<R>(mach: R, gamma: R) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match plume::prandtl_meyer_kernel(mach, gamma) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn choked_mass_flow<R>(
    throat_area: Area<R>,
    chamber_pressure: Pressure<R>,
    chamber_temperature: Temperature<R>,
    gamma: R,
    r_specific: R,
) -> PropagatingEffect<MassFlowRate<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match plume::choked_mass_flow_kernel(
        throat_area,
        chamber_pressure,
        chamber_temperature,
        gamma,
        r_specific,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn srp_post_bow_shock_total_pressure<R>(
    p_inf: Pressure<R>,
    mach_inf: R,
    gamma_inf: R,
) -> PropagatingEffect<Pressure<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match plume::srp_post_bow_shock_total_pressure_kernel(p_inf, mach_inf, gamma_inf) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn srp_terminal_shock_mach<R>(
    chamber_pressure: Pressure<R>,
    post_bow_shock_total_pressure: Pressure<R>,
    gamma_jet: R,
) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match plume::srp_terminal_shock_mach_kernel(
        chamber_pressure,
        post_bow_shock_total_pressure,
        gamma_jet,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn srp_jet_edge_mach<R>(
    exit_mach: R,
    exit_pressure: Pressure<R>,
    post_bow_shock_total_pressure: Pressure<R>,
    gamma_jet: R,
) -> PropagatingEffect<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug + Default,
{
    match plume::srp_jet_edge_mach_kernel(
        exit_mach,
        exit_pressure,
        post_bow_shock_total_pressure,
        gamma_jet,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cordell_braun_plume_boundary<R>(
    chamber_pressure: Pressure<R>,
    chamber_temperature: Temperature<R>,
    r_specific: R,
    gamma_jet: R,
    exit_mach: R,
    nozzle_half_angle_rad: R,
    throat_diameter: Length<R>,
    exit_radius: Length<R>,
    cone_length: Length<R>,
    p_inf: Pressure<R>,
    mach_inf: R,
    gamma_inf: R,
) -> PropagatingEffect<PlumeGeometry<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match plume::cordell_braun_plume_boundary_kernel(
        chamber_pressure,
        chamber_temperature,
        r_specific,
        gamma_jet,
        exit_mach,
        nozzle_half_angle_rad,
        throat_diameter,
        exit_radius,
        cone_length,
        p_inf,
        mach_inf,
        gamma_inf,
    ) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
