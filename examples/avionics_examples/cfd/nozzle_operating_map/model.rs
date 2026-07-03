/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain logic (the "how" of the physics): the march-and-reduce step, the closed-form
//! gas-dynamics references every gate compares against, and the one reduced-row struct.

use crate::FloatType;
use crate::constants::{EXIT_AREA_M2, GAMMA, INLET_AREA_M2, LENGTH_M, T0_K, THROAT_AREA_M2};
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::{CfdFlow, FittedNormalShock, PhysicsError};
use deep_causality_physics::area_mach_ratio_kernel;

/// One swept back pressure's reduced result, all in the working precision.
pub struct MapRow {
    /// The swept ratio `p_back / p0`.
    pub p_ratio: FloatType,
    /// Exit-plane Mach number.
    pub mach_exit: FloatType,
    /// Sonic-crossing station, when the profile reaches Mach 1.
    pub sonic_x: Option<FloatType>,
    /// Shock station, when the run reported one.
    pub shock_x: Option<FloatType>,
    /// Thrust coefficient.
    pub cf: FloatType,
    /// Worst interior deviation from the area-Mach relation (shock-free rows only).
    pub area_mach_dev: Option<FloatType>,
}

/// March one back pressure and reduce the report to a [`MapRow`] (execution; the case
/// description comes from [`model_config`](crate::model_config)).
pub fn map_row(p_ratio: FloatType) -> Result<MapRow, PhysicsError> {
    let config = crate::model_config::duct_case(p_ratio)?;
    let report = CfdFlow::duct_march(&config).run()?;

    let x = report
        .series("x")
        .ok_or_else(|| PhysicsError::CalculationError("duct report missing x".into()))?;
    let mach = report
        .series("mach_profile")
        .ok_or_else(|| PhysicsError::CalculationError("duct report missing mach".into()))?;
    let cf = report
        .series("thrust_coefficient")
        .and_then(|s| s.first().copied())
        .ok_or_else(|| PhysicsError::CalculationError("duct report missing Cf".into()))?;
    let shock_x = report.series("shock_position").map(|s| s[0]);

    // The first station where the profile crosses Mach 1 from below.
    let sonic_x = x
        .iter()
        .zip(mach)
        .find(|(_, m)| **m >= ft(1.0))
        .map(|(xi, _)| *xi);

    // Shock-free rows: worst interior deviation from the area-Mach relation, same exclusions
    // as the duct-march verification (throat +-0.1 L, 0.05 L off each end).
    let area_mach_dev = if shock_x.is_none() {
        let throat_x = ft(LENGTH_M) * ft(0.5);
        let mut worst = ft(0.0);
        for (xi, mi) in x.iter().zip(mach) {
            if (xi - throat_x).abs() < ft(0.1) * ft(LENGTH_M)
                || *xi < ft(0.05) * ft(LENGTH_M)
                || *xi > ft(0.95) * ft(LENGTH_M)
            {
                continue;
            }
            let ratio = area_at(*xi) / ft(THROAT_AREA_M2);
            let analytic = mach_from_area_ratio(ratio, *xi > throat_x);
            let dev = (mi - analytic).abs() / analytic;
            if dev > worst {
                worst = dev;
            }
        }
        Some(worst)
    } else {
        None
    };

    Ok(MapRow {
        p_ratio,
        mach_exit: *mach.last().expect("nonempty profile"),
        sonic_x,
        shock_x,
        cf,
        area_mach_dev,
    })
}

/// The duct's area at x, mirroring `DuctAreaProfile::ConvergingDiverging` exactly (parabolic
/// on each side, smooth throat).
pub fn area_at(x: FloatType) -> FloatType {
    let x_t = ft(LENGTH_M) * ft(0.5);
    if x <= x_t {
        let s = (x_t - x) / x_t;
        ft(THROAT_AREA_M2) + (ft(INLET_AREA_M2) - ft(THROAT_AREA_M2)) * s * s
    } else {
        let s = (x - x_t) / (ft(LENGTH_M) - x_t);
        ft(THROAT_AREA_M2) + (ft(EXIT_AREA_M2) - ft(THROAT_AREA_M2)) * s * s
    }
}

/// Invert the area-Mach relation on one branch by bisection.
pub fn mach_from_area_ratio(area_ratio: FloatType, supersonic: bool) -> FloatType {
    let (mut lo, mut hi) = if supersonic {
        (ft(1.0), ft(10.0))
    } else {
        (ft(1.0e-4), ft(1.0))
    };
    for _ in 0..200 {
        let mid = (lo + hi) * ft(0.5);
        let r = area_mach_ratio_kernel(mid, ft(GAMMA)).expect("area-Mach kernel");
        let too_low = if supersonic {
            r < area_ratio
        } else {
            r > area_ratio
        };
        if too_low {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) * ft(0.5)
}

/// Isentropic `p0/p` at Mach `m`.
pub fn isen(m: FloatType) -> FloatType {
    let g = ft(GAMMA);
    (ft(1.0) + (g - ft(1.0)) / ft(2.0) * m * m).powf(g / (g - ft(1.0)))
}

/// The first critical back-pressure ratio: the subsonic solution at the exit area ratio.
pub fn subsonic_exit_pressure_ratio() -> FloatType {
    let m_exit = mach_from_area_ratio(ft(EXIT_AREA_M2) / ft(THROAT_AREA_M2), false);
    ft(1.0) / isen(m_exit)
}

/// The back-pressure ratio that parks the normal shock exactly at the exit plane.
pub fn exit_shock_back_pressure_ratio() -> FloatType {
    let shock = FittedNormalShock::<FloatType>::new(ft(GAMMA)).expect("shock model");
    let m_design = mach_from_area_ratio(ft(EXIT_AREA_M2) / ft(THROAT_AREA_M2), true);
    let post = shock
        .post_shock(ft(T0_K), ft(1.0e25), m_design)
        .expect("post-shock state");
    post.p_ratio / isen(m_design)
}

/// Closed-form internal-shock position for a back-pressure ratio: isentropic to the shock,
/// Rankine-Hugoniot across it, subsonic isentropic recovery to the exit; bisected on the
/// monotone shock-station-to-exit-pressure relation.
pub fn analytic_shock_position(back_pressure_ratio: FloatType) -> FloatType {
    let shock = FittedNormalShock::<FloatType>::new(ft(GAMMA)).expect("shock model");
    let exit_pressure_for = |xs: FloatType| -> FloatType {
        let a_shock = area_at(xs);
        let m1 = mach_from_area_ratio(a_shock / ft(THROAT_AREA_M2), true);
        let post = shock
            .post_shock(ft(T0_K), ft(1.0e25), m1)
            .expect("post-shock state");
        let t2_over_t1 = post.p_ratio / post.rho_ratio;
        let m2 = m1 * post.u_ratio / t2_over_t1.sqrt();
        let p02_over_p01 = post.p_ratio * isen(m2) / isen(m1);
        let a_star2 = a_shock / area_mach_ratio_kernel(m2, ft(GAMMA)).expect("kernel");
        let m_exit = mach_from_area_ratio(ft(EXIT_AREA_M2) / a_star2, false);
        p02_over_p01 / isen(m_exit)
    };
    let (mut lo, mut hi) = (ft(LENGTH_M) * ft(0.55), ft(LENGTH_M) * ft(0.99));
    for _ in 0..200 {
        let mid = (lo + hi) * ft(0.5);
        // A downstream shock loses more stagnation pressure and recovers less exit pressure.
        if exit_pressure_for(mid) > back_pressure_ratio {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) * ft(0.5)
}
