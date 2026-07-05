/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain logic (the "how" of the physics): the march-and-reduce step, the closed-form
//! gas-dynamics references every gate compares against, and the one reduced-row struct.

use crate::FloatType;
use crate::constants::{
    AREA_MACH_BAND, CELLS, EXIT_AREA_M2, GAMMA, INLET_AREA_M2, LENGTH_M, NO_SHOCK_SENTINEL_M, T0_K,
    THROAT_AREA_M2,
};
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::{
    CaseRun, DuctConfig, FittedNormalShock, GateSeq, PhysicsError, StudyView, TableRow,
};
use deep_causality_physics::area_mach_ratio_kernel;

/// One swept back pressure's reduced result, all in the working precision.
#[derive(Clone)]
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

impl TableRow for MapRow {
    type Scalar = FloatType;
    const SCHEMA: &'static [(&'static str, &'static str)] = &[
        ("p_back_over_p0", "-"),
        ("mach_exit", "-"),
        ("shock_x", "m; -1 = none"),
        ("thrust_coefficient", "-"),
    ];
    fn cells(&self) -> Vec<FloatType> {
        vec![
            self.p_ratio,
            self.mach_exit,
            self.shock_x.unwrap_or(ft(NO_SHOCK_SENTINEL_M)),
            self.cf,
        ]
    }
}

/// Reduce one duct march to a [`MapRow`] (the grammar's `reduce` step): the swept ratio comes
/// from the case, the exit Mach / shock station / thrust coefficient from the report. The march
/// itself is the grammar's `.march()`; the case description is
/// [`model_config::duct_case`](crate::model_config::duct_case).
pub fn map_row(
    run: &CaseRun<'_, FloatType, DuctConfig<FloatType>, FloatType>,
) -> Result<MapRow, PhysicsError> {
    let p_ratio = *run.case();
    let report = run.report();

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

// ── The operating-map gating sequence ─────────────────────────────────────────────────────────

/// The nozzle's gating sequence: what the operating map must satisfy, each gate a closed-form
/// check the solver never sees. Schedule integrity is not gated — the sweep guarantees a row per
/// case by construction.
pub fn nozzle_gates() -> GateSeq<MapRow> {
    GateSeq::new("nozzle operating map")
        .gate("choking", gate_choking)
        .gate("shock position", gate_shock_position)
        .gate("shock-free profiles", gate_area_mach)
        .gate("physical thrust", gate_thrust)
}

/// Every choked row crosses Mach 1 within the stated band of the throat.
pub fn gate_choking(view: &StudyView<'_, MapRow>) -> (bool, String) {
    use crate::constants::SONIC_AT_THROAT_BAND_CELLS;
    let h = ft(LENGTH_M) / ft(CELLS as f64);
    let throat_x = ft(LENGTH_M) * ft(0.5);
    let first_critical = subsonic_exit_pressure_ratio();
    let band = ft(SONIC_AT_THROAT_BAND_CELLS) * h;
    for row in view.rows() {
        if row.p_ratio < first_critical {
            match row.sonic_x {
                Some(x) if (x - throat_x).abs() <= band => {}
                Some(x) => {
                    return (
                        false,
                        format!(
                            "p_ratio {:.2}: sonic crossing at x = {x:.4} m, throat {throat_x:.4} m, band {band:.4} m",
                            row.p_ratio
                        ),
                    );
                }
                None => {
                    return (
                        false,
                        format!(
                            "p_ratio {:.2}: choked row never reaches Mach 1",
                            row.p_ratio
                        ),
                    );
                }
            }
        }
    }
    (
        true,
        "every choked row crosses Mach 1 at the throat".to_string(),
    )
}

/// Every internal-shock row lands within the measured band of the closed-form shock position.
pub fn gate_shock_position(view: &StudyView<'_, MapRow>) -> (bool, String) {
    use crate::constants::SHOCK_BAND_CELLS;
    let h = ft(LENGTH_M) / ft(CELLS as f64);
    let first_critical = subsonic_exit_pressure_ratio();
    let shock_at_exit = exit_shock_back_pressure_ratio();
    let band = ft(SHOCK_BAND_CELLS) * h;
    for row in view.rows() {
        let has_analytic_shock = row.p_ratio > shock_at_exit && row.p_ratio < first_critical;
        if has_analytic_shock {
            let a = analytic_shock_position(row.p_ratio);
            match row.shock_x {
                Some(x) if (x - a).abs() <= band => {}
                Some(x) => {
                    return (
                        false,
                        format!(
                            "p_ratio {:.2}: shock at {x:.4} m, closed form {a:.4} m, band {band:.4} m",
                            row.p_ratio
                        ),
                    );
                }
                None => {
                    return (
                        false,
                        format!(
                            "p_ratio {:.2}: closed form places a shock at {a:.4} m, none reported",
                            row.p_ratio
                        ),
                    );
                }
            }
        }
    }
    (
        true,
        "every internal-shock row matches the closed form".to_string(),
    )
}

/// Shock-free rows track the area-Mach relation within the measured band.
pub fn gate_area_mach(view: &StudyView<'_, MapRow>) -> (bool, String) {
    for row in view.rows() {
        if row.shock_x.is_none()
            && let Some(dev) = row.area_mach_dev
            && dev > ft(AREA_MACH_BAND)
        {
            return (
                false,
                format!(
                    "p_ratio {:.2}: worst interior deviation {dev:.4} exceeds the {AREA_MACH_BAND} band",
                    row.p_ratio
                ),
            );
        }
    }
    (
        true,
        "every shock-free row tracks the area-Mach relation".to_string(),
    )
}

/// The thrust coefficient is finite and positive on every row.
pub fn gate_thrust(view: &StudyView<'_, MapRow>) -> (bool, String) {
    for row in view.rows() {
        if !(row.cf.is_finite() && row.cf > ft(0.0)) {
            return (
                false,
                format!("p_ratio {:.2}: Cf = {}", row.p_ratio, row.cf),
            );
        }
    }
    (
        true,
        "thrust coefficient finite and positive on every row".to_string(),
    )
}
