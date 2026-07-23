/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration layer for the QTT immersed-cylinder verification: every case parameter and the
//! `QttMarchConfig` case builder. `main.rs` runs the CfdFlow march; `print_utils.rs` measures + verifies.
//!
//! A cylinder is immersed in a periodic free-stream by **Brinkman volume penalization** (a smoothed
//! mask drives the velocity to zero inside the body). The drag falls out as a tensor-train contraction
//! of the mask with the velocity deficit. The box is `[0, 2π]²`; precision enters once through [`ft`].

use crate::FloatType;
use deep_causality_cfd::{
    MarchStop, PhysicsError, QttMarchConfig, QttMarchConfigBuilder, QttObserve, body_mask_2d,
};
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::{CausalTensorTrain, Truncation};

/// Kinematic viscosity.
pub const NU: f64 = 0.05;
/// Explicit-Euler time step. Chosen from the numerical envelope at `L = 8`, **not** from a `dt/η`
/// ratio (`close-qtt-solver-envelope`). At `L = 8`, `dx = 2π/256 = 0.02454`, so the diffusive
/// explicit-stability limit `dt ≤ dx²/(4ν) = 3.01e-3` binds — the previous `dt = 0.004` is now
/// **refused** by `QttImmersed2d::new`. `dt = 0.0025` sits inside both that limit and the
/// penalization limit `dt ≤ 2η = 0.024`.
pub const DT: f64 = 0.0025;
/// Marched steps. Raised from 40 so the physical horizon `steps·dt = 0.16` is unchanged after the
/// `dt` reduction above (`0.16 / 0.0025 = 64`). A transient measurement — a periodic box has no
/// momentum source to hold the free-stream, so drag is read at a fixed horizon, not a steady state.
pub const STEPS: usize = 64;
/// Brinkman penalization parameter, chosen from a **wall-error target** and the resolution constraint
/// (`close-qtt-solver-envelope` item 10), not from an explicit-stability ratio. The penalization
/// layer thickness is `√(ην)`; resolving it needs `η ≥ dx²/ν`. At `L = 8`, `dx²/ν = 0.01205`, and
/// `η = 0.012` sits at that resolution floor, giving a layer `√(ην) = 0.0245 ≈ dx` and a slip error
/// `√(ην)/U = 2.5 %` — the L=8 row of the design's wall-error table. The previous `η = 0.016` was
/// pinned by `dt/η = 0.25` and left the layer 7× thinner than a cell at the old `L = 5`.
pub const ETA: f64 = 0.012;
/// Free-stream speed (the seed and the drag reference speed).
pub const U_INF: f64 = 1.0;
/// Cylinder radius as a fraction of the box length `2π`.
pub const RADIUS_FRAC: f64 = 0.15;
/// Mask smoothing width in cells.
pub const SMOOTH_CELLS: f64 = 2.0;

/// Committed DEC isolated-cylinder drag at Re 100 (`dec_cylinder_verification`) — the **cross-reference**
/// (disclaimed: the periodic penalized box is not the DEC inflow/outflow/far-field configuration).
pub const DEC_CD_REF: f64 = 1.345;

/// Lift an exact `f64` specification into the working precision.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

/// The grid spacing `Δx = 2π / 2^L`.
pub fn spacing(l: usize) -> FloatType {
    ft(2.0 * std::f64::consts::PI) / ft((1usize << l) as f64)
}

/// The cylinder diameter `2·RADIUS_FRAC·2π` (the drag reference length), at the working precision.
pub fn diameter() -> FloatType {
    ft(2.0 * RADIUS_FRAC * 2.0 * std::f64::consts::PI)
}

/// A round policy capping the bond dimension at `cap` — the accuracy-vs-bond knob.
pub fn trunc_bond(cap: usize) -> Truncation<FloatType> {
    Truncation::<FloatType>::by_bond(cap).expect("bond cap is valid")
}

/// The smoothed cylinder mask, centered in the box.
///
/// # Errors
/// Propagates codec errors.
pub fn cyl_mask(
    l: usize,
    trunc: &Truncation<FloatType>,
) -> Result<CausalTensorTrain<FloatType>, PhysicsError> {
    cyl_mask_smoothed(l, trunc, SMOOTH_CELLS)
}

/// The smoothed cylinder mask at an explicit smoothing width, in cells — the parameter the
/// smoothing ladder sweeps. [`cyl_mask`] is this at the default [`SMOOTH_CELLS`].
///
/// # Errors
/// Propagates codec errors.
pub fn cyl_mask_smoothed(
    l: usize,
    trunc: &Truncation<FloatType>,
    smooth_cells: f64,
) -> Result<CausalTensorTrain<FloatType>, PhysicsError> {
    let dx = spacing(l);
    let center = ft(std::f64::consts::PI); // 2π/2
    let radius = ft(RADIUS_FRAC * 2.0 * std::f64::consts::PI);
    let smoothing = ft(smooth_cells) * dx;
    body_mask_2d::<FloatType>(l, l, dx, dx, center, center, radius, smoothing, trunc)
}

/// The penalization-parameter ladder. Brinkman convergence to the no-slip solution is an `η → 0`
/// limit (Angot, Bruneau & Fabrie 1999, `O(η^{3/4})`); a bond ladder alone cannot see it, so the
/// harness sweeps `η` as a first-class check on whether the reported drag has a limit at all.
pub const ETA_LADDER: [f64; 5] = [0.128, 0.064, 0.032, 0.016, 0.008];

/// The mask smoothing-width ladder, in cells. The reported drag is an integral over the mask, so a
/// numerically-chosen skirt width sets its magnitude; sweeping it exposes that dependence instead
/// of leaving it in a disclaimer.
pub const SMOOTH_LADDER: [f64; 5] = [0.5, 1.0, 2.0, 3.0, 4.0];

/// The `QttMarchConfig` for an immersed cylinder in a uniform free-stream, marched `STEPS` steps at the
/// bond cap `cap`, observing drag/lift and divergence — built through the configuration layer, to be run
/// by `CfdFlow::march` in `main`.
///
/// # Errors
/// Propagates builder / codec errors.
pub fn build_config(l: usize, cap: usize) -> Result<QttMarchConfig<FloatType>, PhysicsError> {
    build_config_with(l, cap, ETA, SMOOTH_CELLS)
}

/// The `QttMarchConfig` at an explicit penalization parameter and mask smoothing width — the two
/// axes the η and smoothing ladders sweep. [`build_config`] is this at the defaults.
///
/// # Errors
/// Propagates builder / codec errors.
pub fn build_config_with(
    l: usize,
    cap: usize,
    eta: f64,
    smooth_cells: f64,
) -> Result<QttMarchConfig<FloatType>, PhysicsError> {
    let dx = spacing(l);
    let trunc = trunc_bond(cap);
    let mask = cyl_mask_smoothed(l, &trunc, smooth_cells)?;
    let u_inf = ft(U_INF);
    QttMarchConfigBuilder::<FloatType>::new()
        .name("qtt-cylinder")
        .grid(l, l, dx, dx)
        .solver(ft(DT), ft(NU), trunc)
        .seed_fn(|_, _| (u_inf, ft(0.0)))?
        .body(mask, ft(0.0), ft(0.0), ft(eta), u_inf, diameter())
        .stop(MarchStop::Fixed(STEPS))
        .observe(QttObserve::default().drag().divergence())
        .build()
}
