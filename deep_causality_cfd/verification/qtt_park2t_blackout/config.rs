/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration layer for the Tier-A Park-2T plasma-blackout verification: the flight condition, the
//! reacting-gas parameters, the blunt-body `QttMarchConfig`, and the LER coupling. `main.rs` runs the
//! coupled QTT march and self-verifies; `print_utils.rs` gates the six LER criteria.
//!
//! A blunt forebody is immersed in a periodic free-stream (Brinkman penalization). The mandatory
//! Rankine–Hugoniot normal-shock jump sets the post-shock temperature; the recovery-temperature
//! reconstruction drives ionization through the closed-form LER stage; the electron density feeds the
//! blackout trigger. Tier-A rides the **incompressible** rollout — `T_tr` is a recovery-temperature
//! reconstruction, not a true post-shock thermodynamic path.

use crate::FloatType;
use deep_causality_cfd::{
    BlackoutTrigger, Coupling, EosStage, IonizationStage, MarchStop, PhysicsError, PhysicsStage,
    QttMarchConfig, QttMarchConfigBuilder, QttObserve, RecoveryTemperatureStage, body_mask_2d,
};
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::{CausalTensorTrain, Truncation};

/// Kinematic viscosity of the incompressible carrier rollout.
pub const NU: f64 = 0.05;
/// Explicit-Euler time step.
pub const DT: f64 = 0.004;
/// Marched relaxation steps (the LER ionization dwell).
pub const STEPS: usize = 40;
/// Brinkman penalization parameter (small → hard wall).
pub const ETA: f64 = 0.016;
/// Free-stream speed of the incompressible carrier (the reconstruction rides this field).
pub const U_INF: f64 = 1.0;
/// Forebody radius as a fraction of the box length `2π`.
pub const RADIUS_FRAC: f64 = 0.18;
/// Mask smoothing width in cells.
pub const SMOOTH_CELLS: f64 = 2.0;

// ── Flight condition (RAM-C-like reentry) ────────────────────────────────
/// Free-stream Mach number (`M ≈ 25` reentry).
pub const MACH: f64 = 25.0;
/// Ratio of specific heats.
pub const GAMMA: f64 = 1.4;
/// Free-stream (ambient) temperature, K.
pub const T_INF: f64 = 250.0;
/// Frozen-mixture specific heat at constant pressure, J·kg⁻¹·K⁻¹.
pub const C_P: f64 = 1004.0;
/// Total heavy-particle number density (high-altitude reentry, m⁻³).
pub const NUMBER_DENSITY: f64 = 1.0e22;
/// Scalar (ionization-fraction) diffusivity used to transport the carried fraction.
pub const SCALAR_KAPPA: f64 = 0.05;
/// Comms band as an angular frequency (GPS L-band ≈ 1.5 GHz → ω ≈ 9.4e9 rad/s).
pub const COMMS_BAND_RAD_S: f64 = 9.4e9;

// ── Published reference cross-references (reported, with Tier-A disclaimers) ──
/// RAM-C II peak electron density band near the 71 km station, m⁻³ (order-of-magnitude anchor).
pub const RAMC_NE_REFERENCE: f64 = 1.0e19;

/// Lift an exact `f64` specification into the working precision.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

/// The grid spacing `Δx = 2π / 2^L`.
pub fn spacing(l: usize) -> FloatType {
    ft(2.0 * std::f64::consts::PI) / ft((1usize << l) as f64)
}

/// A round policy capping the bond dimension at `cap`.
pub fn trunc_bond(cap: usize) -> Truncation<FloatType> {
    Truncation::<FloatType>::by_bond(cap).expect("bond cap is valid")
}

/// The smoothed blunt-forebody mask, centered in the box.
///
/// # Errors
/// Propagates codec errors.
pub fn body_mask(
    l: usize,
    trunc: &Truncation<FloatType>,
) -> Result<CausalTensorTrain<FloatType>, PhysicsError> {
    let dx = spacing(l);
    let center = ft(std::f64::consts::PI);
    let radius = ft(RADIUS_FRAC * 2.0 * std::f64::consts::PI);
    let smoothing = ft(SMOOTH_CELLS) * dx;
    body_mask_2d::<FloatType>(l, l, dx, dx, center, center, radius, smoothing, trunc)
}

/// The `QttMarchConfig` for the blunt forebody, observing the blackout series.
///
/// # Errors
/// Propagates builder / codec errors.
pub fn build_config(l: usize, cap: usize) -> Result<QttMarchConfig<FloatType>, PhysicsError> {
    let dx = spacing(l);
    let trunc = trunc_bond(cap);
    let mask = body_mask(l, &trunc)?;
    let u_inf = ft(U_INF);
    QttMarchConfigBuilder::<FloatType>::new()
        .name("qtt-park2t-blackout")
        .grid(l, l, dx, dx)
        .solver(ft(DT), ft(NU), trunc)
        .seed_fn(|_, _| (u_inf, ft(0.0)))?
        .body(mask, ft(0.0), ft(0.0), ft(ETA), u_inf, dx)
        .stop(MarchStop::Fixed(STEPS))
        .observe(
            QttObserve::default()
                .electron_density()
                .plasma_frequency()
                .blackout_dwell(),
        )
        .build()
}

/// The Tier-A LER coupling: rebuild `T_tr` (RH jump + recovery), relax ionization, close the pressure.
/// A statically-composed cons-tuple of stages (no `dyn`).
pub fn coupling() -> impl PhysicsStage<2, FloatType> {
    Coupling::between_steps()
        .then(RecoveryTemperatureStage::new(
            ft(MACH),
            ft(GAMMA),
            ft(T_INF),
            ft(C_P),
        ))
        .then(IonizationStage::new(ft(NUMBER_DENSITY)))
        .then(EosStage::new(ft(NUMBER_DENSITY)))
        .build()
}

/// The blackout trigger at the configured comms band.
pub fn trigger() -> BlackoutTrigger<FloatType> {
    BlackoutTrigger::new(ft(COMMS_BAND_RAD_S))
}
