/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rocket-performance kernels: propellant mass flow from specific impulse and
//! the Tsiolkovsky rocket equation. Pure pointwise relations; propellant
//! bookkeeping and throttle mapping live in the CFD-side stages.

use crate::constants::{EARTH_GRAVITY_ACCELERATION, real_from_f64};
use crate::{Force, Mass, MassFlowRate, PhysicsError, Speed};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Propellant mass flow from thrust and specific impulse
///
/// $$ \dot m = \frac{T}{I_{sp} \cdot g_0} $$
///
/// with $g_0 = 9.80665\,m/s^2$ the standard gravity that defines specific
/// impulse in seconds ([`EARTH_GRAVITY_ACCELERATION`], exact by convention).
///
/// # Arguments
/// * `thrust` — engine thrust `T` (N, ≥ 0).
/// * `isp_s` — specific impulse `Isp` (s, > 0).
///
/// # References
/// * Sutton, G. P., & Biblarz, O., "Rocket Propulsion Elements," 9th ed.,
///   Wiley (2017), Ch. 2 — the specific-impulse definition
///   `Isp = T / (ṁ·g0)`, solved for `ṁ`.
pub fn propellant_mass_flow_kernel<R>(
    thrust: Force<R>,
    isp_s: R,
) -> Result<MassFlowRate<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if !isp_s.is_finite() || isp_s <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Specific impulse must be positive".into(),
        ));
    }
    let t = thrust.value();
    if t < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Thrust cannot be negative".into(),
        ));
    }
    let g0: R = real_from_f64(EARTH_GRAVITY_ACCELERATION);
    MassFlowRate::new(t / (isp_s * g0))
}

/// Tsiolkovsky rocket equation
///
/// $$ \Delta v = I_{sp} \cdot g_0 \cdot \ln\!\frac{m_0}{m_1} $$
///
/// the ideal velocity increment of a burn from initial mass `m₀` to final
/// mass `m₁` — the relation that sizes a propellant reserve against a demanded
/// margin (the retropulsion descent's weather-table job downstream).
///
/// # Arguments
/// * `isp_s` — specific impulse `Isp` (s, > 0).
/// * `initial_mass` — mass before the burn `m₀` (kg).
/// * `final_mass` — mass after the burn `m₁` (kg, > 0, ≤ `m₀`).
///
/// # References
/// * Tsiolkovsky, K. E. (1903); Sutton, G. P., & Biblarz, O., "Rocket
///   Propulsion Elements," 9th ed., Wiley (2017), Ch. 4 (flight performance,
///   ideal burn).
pub fn tsiolkovsky_delta_v_kernel<R>(
    isp_s: R,
    initial_mass: Mass<R>,
    final_mass: Mass<R>,
) -> Result<Speed<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if !isp_s.is_finite() || isp_s <= R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Specific impulse must be positive".into(),
        ));
    }
    let m0 = initial_mass.value();
    let m1 = final_mass.value();
    if m1 <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Final mass must be positive".into(),
        ));
    }
    if m0 < m1 {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "A burn cannot end heavier than it began (m0 < m1)".into(),
        ));
    }
    let g0: R = real_from_f64(EARTH_GRAVITY_ACCELERATION);
    Speed::new(isp_s * g0 * (m0 / m1).ln())
}
