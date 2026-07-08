/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shock and recovery-temperature kernels: the Rankine–Hugoniot normal-shock
//! temperature jump (mandatory — isentropic recovery alone is too cold to
//! ionize) and the Tier-A recovery-temperature reconstruction.

use crate::{PhysicsError, Temperature};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Post-shock temperature from the **Rankine–Hugoniot normal-shock relations**
///
/// $$ \frac{T_2}{T_1} = \frac{\big(2\gamma M^2 - (\gamma-1)\big)\big((\gamma-1) M^2 + 2\big)}
///    {(\gamma+1)^2 M^2} $$
///
/// This jump is mandatory for the Tier-A slice: isentropic recovery off the
/// incompressible field tops out far below ionization temperatures, so without
/// it the slice silently produces no plasma.
///
/// # Arguments
/// * `t_inf` — freestream temperature `T_1` (K).
/// * `mach` — freestream Mach number `M` (≥ 1).
/// * `gamma` — ratio of specific heats `γ` (> 1).
///
/// # References
/// * Anderson, "Hypersonic and High-Temperature Gas Dynamics" (normal-shock
///   relations); Park, "Nonequilibrium Hypersonic Aerothermodynamics," Wiley (1990).
pub fn rankine_hugoniot_temperature_kernel<R>(
    t_inf: Temperature<R>,
    mach: R,
    gamma: R,
) -> Result<Temperature<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    if mach < one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Mach number must be >= 1 for a shock".into(),
        ));
    }
    if gamma <= one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Ratio of specific heats must be > 1".into(),
        ));
    }
    let t1 = t_inf.value();
    if t1 <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Freestream temperature must be positive".into(),
        ));
    }

    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let m2 = mach * mach;
    let num = (two * gamma * m2 - (gamma - one)) * ((gamma - one) * m2 + two);
    let den = (gamma + one) * (gamma + one) * m2;
    Temperature::new(t1 * num / den)
}

/// Recovery-temperature reconstruction `T_tr = T_post − ½|u|²/c_p` — the Tier-A
/// stand-in for the post-shock translational temperature, built from the
/// post-shock stagnation temperature and the local speed off the incompressible
/// velocity field. Labeled a reconstruction, not a true thermodynamic path.
///
/// # Arguments
/// * `t_post` — post-shock stagnation temperature `T_post` (K).
/// * `speed` — local speed magnitude `|u|` (m/s).
/// * `c_p` — (frozen-mixture) specific heat at constant pressure (J·kg⁻¹·K⁻¹).
pub fn recovery_temperature_kernel<R>(
    t_post: Temperature<R>,
    speed: R,
    c_p: R,
) -> Result<Temperature<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if c_p <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Specific heat c_p must be positive".into(),
        ));
    }
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    // Temperature::new rejects a negative result (over-cooled), surfacing it as
    // a ZeroKelvinViolation rather than producing an invalid quantity.
    Temperature::new(t_post.value() - half * speed * speed / c_p)
}
