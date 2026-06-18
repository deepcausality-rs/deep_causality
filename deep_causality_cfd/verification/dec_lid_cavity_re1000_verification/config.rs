/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration layer for the Re-1000 lid-driven cavity: the case parameters, the Ghia (1982)
//! reference tables, the refinement-trend gates, and the single `CfdFlow::march` case builder.
//!
//! The case is expressed entirely through the `deep_causality_cfd` **CfdFlow** DSL: an all-walls box
//! mesh at spacing `h`, the DEC incompressible solver at `ν = U/Re`, the y-max lid, and a rest seed.
//! `CfdFlow::march` lowers onto the same projected DEC step the hand-rolled solver used, so the
//! marched field is reproduced exactly. Precision is a parameter: exact `f64` specifications enter
//! once through [`ft`] and the compute runs at the working precision [`FloatType`]; the centerline /
//! vortex analysis downcasts to `f64` at the display boundary (in `print_utils`).

use crate::FloatType;
use deep_causality_cfd::{CfdConfigBuilder, MarchConfig, Mesh, PhysicsError, Seed};
use deep_causality_num::FromPrimitive;

/// Reynolds number of the benchmark (exact specification).
pub const RE: f64 = 1000.0;
/// Tangential lid speed `U` on the y-max face.
pub const LID_SPEED: f64 = 1.0;
/// CFL safety factor on the unit-spacing lattice: `dt = CFL · h`.
pub const CFL: f64 = 0.45;

/// Ghia et al. (1982), Re = 1000: u along the vertical centerline, (y, u).
pub const GHIA_U: [(f64, f64); 17] = [
    (1.0000, 1.00000),
    (0.9766, 0.65928),
    (0.9688, 0.57492),
    (0.9609, 0.51117),
    (0.9531, 0.46604),
    (0.8516, 0.33304),
    (0.7344, 0.18719),
    (0.6172, 0.05702),
    (0.5000, -0.06080),
    (0.4531, -0.10648),
    (0.2813, -0.27805),
    (0.1719, -0.38289),
    (0.1016, -0.29730),
    (0.0703, -0.22220),
    (0.0625, -0.20196),
    (0.0547, -0.18109),
    (0.0000, 0.00000),
];

/// Ghia et al. (1982), Re = 1000: v along the horizontal centerline, (x, v).
pub const GHIA_V: [(f64, f64); 17] = [
    (1.0000, 0.00000),
    (0.9688, -0.21388),
    (0.9609, -0.27669),
    (0.9531, -0.33714),
    (0.9453, -0.39188),
    (0.9063, -0.51500),
    (0.8594, -0.42665),
    (0.8047, -0.31966),
    (0.5000, 0.02526),
    (0.2344, 0.32235),
    (0.2266, 0.33075),
    (0.1563, 0.37095),
    (0.0938, 0.32627),
    (0.0781, 0.30353),
    (0.0703, 0.29012),
    (0.0625, 0.27485),
    (0.0000, 0.00000),
];

/// Ghia et al. (1982), Re = 1000 vortex centers (node-snapped to their 129² grid): (name, x, y).
pub const GHIA_VORTICES: [(&str, f64, f64); 3] = [
    ("primary", 0.5313, 0.5625),
    ("bottom-left", 0.0859, 0.0781),
    ("bottom-right", 0.8594, 0.1094),
];

// --- refinement-trend gates (used by `main::run_trend`) -----------------------------------------

/// Time-converged horizon for both trend grids.
pub const TREND_T_END: f64 = 60.0;
/// Trend grids (coarse, fine).
pub const TREND_GRIDS: [usize; 2] = [17, 33];
/// Pinned RMSE gate for the coarse (17²) grid.
pub const TREND_COARSE_GATE: f64 = 0.32;
/// Pinned RMSE gate for the fine (33²) grid.
pub const TREND_FINE_GATE: f64 = 0.20;
/// Required strict-decrease margin between the coarse and fine RMSE.
pub const TREND_MARGIN: f64 = 0.04;

/// Lift an exact `f64` specification into the working precision [`FloatType`] through
/// `FromPrimitive` (so the same lift serves `f32`, `f64`, and `Float106`).
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

/// Vertex spacing of an `n × n` unit-square lattice: `h = 1/(n−1)`.
pub fn grid_spacing(n: usize) -> f64 {
    1.0 / (n - 1) as f64
}

/// CFL-safe time step on the unit-spacing lattice: `dt = CFL · h`.
pub fn time_step(h: f64) -> f64 {
    CFL * h
}

/// Number of march steps to reach `t_end` at step `dt`.
pub fn step_count(t_end: f64, dt: f64) -> usize {
    (t_end / dt).ceil() as usize
}

/// The `MarchConfig` for the Re-1000 cavity: an all-walls box at spacing `h`, the DEC solver at
/// `ν = U/Re`, the y-max lid at `U`, marched `steps` steps from rest — built through
/// `CfdConfigBuilder`, run by the `CfdFlow` DSL.
///
/// # Errors
/// Any solver-config or container validation failure.
pub fn build_march_config(
    n: usize,
    h: f64,
    dt: f64,
    steps: usize,
) -> Result<MarchConfig<2, FloatType, (), ()>, PhysicsError> {
    CfdConfigBuilder::march::<2, FloatType>("cavity-re1000")
        .mesh(Mesh::box_domain([n, n]).spacing(ft(h)))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(ft(LID_SPEED / RE))
                .time_step(ft(dt))
                .build()
                .expect("cavity solver configuration"),
        )
        .lid([ft(LID_SPEED), ft(0.0)])
        .seed(Seed::Rest)
        .march_for(steps)
        .build()
}
