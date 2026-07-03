/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The duct march against gas-dynamics closed forms: the shock-free profile against the
//! area-Mach relation, the shocked case against the analytic shock position, and the
//! loud convergence-failure path.

use deep_causality_cfd::{
    CfdFlow, DuctAreaProfile, DuctConfig, DuctInlet, DuctStop, FittedNormalShock,
};
use deep_causality_physics::area_mach_ratio_kernel;

const GAMMA: f64 = 1.4;

/// A converging-diverging test nozzle: throat at the strict minimum, exit twice the throat.
fn nozzle(back_pressure_ratio: f64, cells: usize, max_steps: usize) -> DuctConfig<f64> {
    let p0 = 101_325.0;
    DuctConfig::new(
        DuctAreaProfile::ConvergingDiverging {
            inlet_area: 2.0,
            throat_area: 1.0,
            exit_area: 2.0,
            length: 1.0,
        },
        DuctInlet { p0, t0: 300.0 },
        GAMMA,
        p0 * back_pressure_ratio,
        cells,
        DuctStop {
            max_steps,
            residual_tol: 1.0e-10,
        },
    )
    .expect("valid nozzle config")
}

/// Invert the area-Mach relation on one branch by bisection.
fn mach_from_area_ratio(area_ratio: f64, supersonic: bool) -> f64 {
    let (mut lo, mut hi) = if supersonic {
        (1.0, 10.0)
    } else {
        (1.0e-4, 1.0)
    };
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let r = area_mach_ratio_kernel(mid, GAMMA).expect("kernel");
        // A/A* decreases toward M = 1 on the subsonic branch and increases past it.
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
    0.5 * (lo + hi)
}

#[test]
fn a_shock_free_expansion_matches_the_area_mach_relation() {
    // Deep expansion: supersonic exit, no shock anywhere in the duct.
    let config = nozzle(0.02, 128, 200_000);
    let report = CfdFlow::duct_march(&config).run().expect("marches");
    assert!(
        report.series("shock_position").is_none(),
        "a shock-free run omits the shock series"
    );

    let x = report.series("x").expect("x");
    let mach = report.series("mach_profile").expect("mach");
    // Compare at interior stations away from the boundaries and the sonic throat, where the
    // first-order scheme is cleanest. Band measured at 128 cells; see the probe note below.
    let throat_x = 0.5;
    let mut worst: f64 = 0.0;
    for (xi, mi) in x.iter().zip(mach) {
        let s = (xi - throat_x).abs();
        if s < 0.1 || *xi < 0.05 || *xi > 0.95 {
            continue;
        }
        let area = area_of(*xi);
        let analytic = mach_from_area_ratio(area, *xi > throat_x);
        worst = worst.max((mi - analytic).abs() / analytic);
    }
    // Measured at 128 cells: worst interior relative deviation printed by the probe.
    assert!(
        worst < 0.05,
        "shock-free Mach profile within 5 percent of the area-Mach relation, got {worst:.4}"
    );
}

/// The test nozzle's area at x, mirroring the ConvergingDiverging profile exactly:
/// parabolic on each side with a smooth throat (A'(x_t) = 0), per `DuctAreaProfile::area_at`.
fn area_of(x: f64) -> f64 {
    let (inlet, throat, exit, len) = (2.0_f64, 1.0, 2.0, 1.0);
    let x_t = 0.5 * len;
    if x <= x_t {
        let s = (x_t - x) / x_t;
        throat + (inlet - throat) * s * s
    } else {
        let s = (x - x_t) / (len - x_t);
        throat + (exit - throat) * s * s
    }
}

#[test]
fn a_shocked_case_places_the_shock_near_the_analytic_position() {
    // Moderate back pressure: normal shock in the diverging section.
    let config = nozzle(0.7, 128, 200_000);
    let report = CfdFlow::duct_march(&config).run().expect("marches");
    let shock = report.series("shock_position").expect("shocked run")[0];

    let analytic = analytic_shock_position(0.7);
    let h = 1.0 / 128.0;
    // Band: the scheme smears the shock over a few cells and the first-order profile shifts
    // it slightly; measured at 128 cells, the placement lands within a few cell widths.
    assert!(
        (shock - analytic).abs() < 12.0 * h,
        "shock at {shock:.4}, analytic {analytic:.4}, band {:.4}",
        12.0 * h
    );
}

/// Closed-form shock position: bisect the shock station until the exit static pressure
/// matches the back pressure (isentropic to the shock, Rankine-Hugoniot across it,
/// subsonic isentropic to the exit).
fn analytic_shock_position(back_pressure_ratio: f64) -> f64 {
    let shock = FittedNormalShock::<f64>::new(GAMMA).expect("shock model");
    let exit_pressure_for = |xs: f64| -> f64 {
        let a_shock = area_of(xs);
        let m1 = mach_from_area_ratio(a_shock, true);
        let post = shock
            .post_shock(300.0, 1.0e25, m1)
            .expect("post-shock state");
        // M2 from the jump ratios: M2 = M1 * (u2/u1) / sqrt(T2/T1), T2/T1 = (p2/p1)/(rho2/rho1).
        let t2_over_t1 = post.p_ratio / post.rho_ratio;
        let m2 = m1 * post.u_ratio / t2_over_t1.sqrt();
        // Stagnation pressure downstream of the shock, relative to upstream p0:
        // p02/p01 = (p2/p1) * (p02/p2) / (p01/p1) with both isentropic ratios at m1, m2.
        let isen = |m: f64| (1.0 + (GAMMA - 1.0) / 2.0 * m * m).powf(GAMMA / (GAMMA - 1.0));
        let p02_over_p01 = post.p_ratio * isen(m2) / isen(m1);
        // New sonic-area reference downstream: A2* = A_shock / (A/A*)(M2).
        let a_star2 = a_shock / area_mach_ratio_kernel(m2, GAMMA).expect("kernel");
        let exit_ratio = area_of(1.0) / a_star2;
        let m_exit = mach_from_area_ratio(exit_ratio, false);
        // Exit static over upstream stagnation.
        p02_over_p01 / isen(m_exit)
    };
    let (mut lo, mut hi) = (0.55, 0.99);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        // A downstream shock is stronger (larger M1), loses more stagnation pressure, and
        // recovers LESS exit static pressure: exit pressure decreases monotonically with the
        // shock station, so a too-high recovery means the shock sits further downstream.
        if exit_pressure_for(mid) > back_pressure_ratio {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

#[test]
fn an_expired_step_budget_is_a_loud_error() {
    let config = nozzle(0.7, 64, 3);
    let err = CfdFlow::duct_march(&config)
        .run()
        .expect_err("three steps cannot converge");
    let msg = err.to_string();
    assert!(
        msg.contains('3') && (msg.contains("residual") || msg.contains("budget")),
        "the error names the budget and the residual: {msg}"
    );
}
