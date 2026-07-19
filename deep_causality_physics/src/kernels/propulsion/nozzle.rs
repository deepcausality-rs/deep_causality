/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Nozzle exit-state kernels: the branch-selected inverse of the area–Mach
//! relation and the isentropic exit-state composition. Both build on the
//! existing Mach-parameterized isentropic kernels in `kernels/fluids/` — no
//! isentropic formula is restated here.

use crate::{
    Density, FlowBranch, NozzleExitState, PhysicsError, Pressure, Speed, Temperature,
    area_mach_ratio_kernel, isentropic_pressure_ratio_kernel, isentropic_temperature_ratio_kernel,
    speed_of_sound_ideal_gas_kernel,
};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Bisection iterations for the inverse area–Mach solve. The bracket halves
/// each step, so 200 iterations exhaust the mantissa of every supported real
/// field (f64 and Float106 alike) — a fixed count keeps the solve
/// deterministic across precisions.
const INVERSE_AREA_MACH_ITERATIONS: usize = 200;

/// Mach number from the area ratio — the branch-selected inverse of the
/// **area–Mach relation**
///
/// $$ \frac{A}{A^*} = \frac{1}{M}\left[\frac{2}{\gamma+1}
///    \left(1+\frac{\gamma-1}{2}M^2\right)\right]^{\frac{\gamma+1}{2(\gamma-1)}} $$
///
/// For every `A/A* > 1` the relation has one subsonic and one supersonic
/// root; the caller selects the branch. The forward relation is the existing
/// [`area_mach_ratio_kernel`] and is *called*, not restated: the inverse is a
/// deterministic bisection against it (monotone decreasing on the subsonic
/// branch, monotone increasing on the supersonic branch).
///
/// # Arguments
/// * `area_ratio` — `A/A*` (≥ 1).
/// * `gamma` — ratio of specific heats `γ` (> 1).
/// * `branch` — [`FlowBranch::Subsonic`] or [`FlowBranch::Supersonic`].
///
/// # References
/// * Anderson, J. D., "Modern Compressible Flow," 3rd ed., McGraw-Hill (2003),
///   Ch. 5 (quasi-one-dimensional flow, area–Mach relation) — the forward
///   kernel's own anchor.
pub fn inverse_area_mach_kernel<R>(
    area_ratio: R,
    gamma: R,
    branch: FlowBranch,
) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let one = R::one();
    if !area_ratio.is_finite() || area_ratio < one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Area ratio A/A* must be finite and >= 1".into(),
        ));
    }
    if gamma <= one {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Ratio of specific heats must be > 1".into(),
        ));
    }
    if area_ratio == one {
        return Ok(one);
    }

    // Bracket the root on the selected branch. Subsonic: A/A* decreases from
    // +inf (M -> 0) to 1 (M = 1). Supersonic: A/A* increases from 1 without
    // bound, so the upper bound doubles until it encloses the target.
    let (mut lo, mut hi) = match branch {
        FlowBranch::Subsonic => {
            let tiny = R::from_f64(1.0e-9).ok_or_else(|| {
                PhysicsError::NumericalInstability("R::from_f64(1.0e-9) failed".into())
            })?;
            // A/A* diverges as M -> 0, so the fixed lower bound encloses the
            // root only up to a finite area ratio. Reject beyond it rather than
            // silently converging to the bound (the "reject, never extrapolate"
            // discipline; a real subsonic nozzle never approaches this ratio).
            if area_mach_ratio_kernel(tiny, gamma)? < area_ratio {
                return Err(PhysicsError::NumericalInstability(
                    "inverse_area_mach_kernel: subsonic area ratio exceeds the solve bracket"
                        .into(),
                ));
            }
            (tiny, one)
        }
        FlowBranch::Supersonic => {
            let cap = R::from_f64(1.0e9).ok_or_else(|| {
                PhysicsError::NumericalInstability("R::from_f64(1.0e9) failed".into())
            })?;
            let two = R::from_f64(2.0).ok_or_else(|| {
                PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into())
            })?;
            let mut hi = two;
            while area_mach_ratio_kernel(hi, gamma)? < area_ratio {
                hi *= two;
                if hi > cap {
                    return Err(PhysicsError::NumericalInstability(
                        "inverse_area_mach_kernel: supersonic bracket exceeded cap".into(),
                    ));
                }
            }
            (one, hi)
        }
    };

    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    for _ in 0..INVERSE_AREA_MACH_ITERATIONS {
        let mid = (lo + hi) * half;
        if mid <= lo || mid >= hi {
            break; // interval exhausted at this precision
        }
        let f = area_mach_ratio_kernel(mid, gamma)?;
        // On the subsonic branch f decreases with M; on the supersonic branch
        // f increases with M. Keep the sub-interval that brackets the target.
        let root_is_above = match branch {
            FlowBranch::Subsonic => f > area_ratio,
            FlowBranch::Supersonic => f < area_ratio,
        };
        if root_is_above {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Ok((lo + hi) * half)
}

/// Isentropically expanded nozzle exit state from chamber conditions — the
/// unstated input that makes the SRP momentum-flux ratio computable from a
/// commanded throttle: exit Mach from the inverse area–Mach relation, static
/// exit pressure/temperature from the existing isentropic-ratio kernels, exit
/// density from the ideal-gas state relation `ρ = p/(R_s·T)` (this crate's
/// single authoritative use), and exit velocity `u_e = M_e · a(T_e)` through
/// the existing speed-of-sound kernel.
///
/// # Arguments
/// * `chamber_pressure` — stagnation (chamber) pressure `p₀` (Pa, > 0).
/// * `chamber_temperature` — stagnation (chamber) temperature `T₀` (K, > 0).
/// * `area_ratio` — nozzle expansion ratio `ε = A_e/A*` (≥ 1).
/// * `gamma` — exhaust ratio of specific heats `γ` (> 1).
/// * `r_specific` — exhaust specific gas constant `R_s` (J·kg⁻¹·K⁻¹, > 0).
///
/// # References
/// * Anderson, J. D., "Modern Compressible Flow," 3rd ed., McGraw-Hill (2003),
///   Ch. 5; Sutton, G. P., & Biblarz, O., "Rocket Propulsion Elements,"
///   9th ed., Wiley (2017), Ch. 3 (ideal-nozzle isentropic expansion).
pub fn nozzle_exit_state_kernel<R>(
    chamber_pressure: Pressure<R>,
    chamber_temperature: Temperature<R>,
    area_ratio: R,
    gamma: R,
    r_specific: R,
) -> Result<NozzleExitState<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    if chamber_pressure.value() <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Chamber pressure must be positive".into(),
        ));
    }
    if chamber_temperature.value() <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Chamber temperature must be positive".into(),
        ));
    }
    if r_specific <= R::zero() {
        return Err(PhysicsError::Singularity(
            "Specific gas constant must be positive".into(),
        ));
    }

    let mach_e = inverse_area_mach_kernel(area_ratio, gamma, FlowBranch::Supersonic)?;
    let p_ratio = isentropic_pressure_ratio_kernel(mach_e, gamma)?; // p0 / p_e
    let t_ratio = isentropic_temperature_ratio_kernel(mach_e, gamma)?; // T0 / T_e
    let p_e = Pressure::new(chamber_pressure.value() / p_ratio)?;
    let t_e = Temperature::new(chamber_temperature.value() / t_ratio)?;
    let rho_e = Density::new(p_e.value() / (r_specific * t_e.value()))?;
    let a_e = speed_of_sound_ideal_gas_kernel(gamma, r_specific, &t_e)?;
    let u_e = Speed::new(mach_e * a_e.value())?;
    Ok(NozzleExitState::new(mach_e, p_e, t_e, rho_e, u_e))
}
