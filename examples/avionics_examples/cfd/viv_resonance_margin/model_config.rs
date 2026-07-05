/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration construction (the "what"): one nondimensional validated cylinder-wake case per
//! swept airspeed. Execution stays in `model`; tuned values stay in `constants`.

use crate::FloatType;
use crate::constants::{
    CELLS_PER_D, CENTER_OFFSET_D, CENTER_X_D, CFL, CG_TOL, DIAMETER_M, LX_D, LY_D, MERGE_FLOOR,
    NU_AIR_M2_S, STEPS,
};
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::{
    Body, CfdConfigBuilder, CfdFlow, Inflow, MarchConfig, Marchable, Mesh, Observe, Outflow,
    PhysicsError, Report, Seed, SlipWall,
};
use deep_causality_topology::HodgeDecomposeOptions;

/// The wake-zone tuple of the validated isolated-cylinder setup: west inflow, east outflow,
/// far-field slip walls (dec_cylinder_verification), composed as the static zone tuple.
pub type WakeZones = (
    Inflow<2, FloatType>,
    (Outflow<2>, (SlipWall<2>, SlipWall<2>)),
);

/// One swept airspeed's wake case: the solver config and its time step, packaged as a single
/// marchable value. The grammar's `.case()` produces one config, and `.march()` needs
/// `C: Marchable`; the wake reduction additionally needs `dt` (the probe series is sampled every
/// `dt`), so the case carries both and exposes the step to the reduction.
pub struct WakeCase {
    config: MarchConfig<2, FloatType, WakeZones, ()>,
    dt: FloatType,
}

impl WakeCase {
    /// The time step the wake was marched at; the Strouhal reduction reads the probe series at
    /// this sampling interval.
    pub fn dt(&self) -> FloatType {
        self.dt
    }
}

/// One-shot geometry: each swept case owns a fresh grid, so `run_owned` materializes it
/// internally and drops it with the run. This is the campaign's one-case-one-report seam.
impl Marchable<FloatType> for WakeCase {
    fn march(&self) -> Result<Report<FloatType>, PhysicsError> {
        CfdFlow::march(&self.config).run_owned()
    }
}

/// The nondimensional case at `airspeed`: D = 1, U = 1, nu = 1/Re with Re = V·D/nu_air, dt =
/// CFL·h, the validated zone set, and the wake probe 1.5 D downstream on the centerline. Takes
/// the airspeed by reference so it plugs directly into the grammar's `.case(model_config::wake_case)`.
/// Counts and grid sizing are exact `f64` specifications; everything the march computes with lifts
/// into `FloatType` once, through `ft`.
pub fn wake_case(airspeed: &FloatType) -> Result<WakeCase, PhysicsError> {
    // Fail fast on a non-physical airspeed row: `Re` is a divisor below (`nu = 1/Re`), so a zero,
    // negative, or non-finite airspeed would feed infinite/NaN viscosity into the solver instead
    // of a clear setup error.
    let v: f64 = Into::into(*airspeed);
    if !v.is_finite() || v <= 0.0 {
        return Err(PhysicsError::CalculationError(format!(
            "airspeed must be finite and positive, got {v}"
        )));
    }
    let reynolds = *airspeed * ft(DIAMETER_M) / ft(NU_AIR_M2_S);

    let h_spec = 1.0 / CELLS_PER_D as f64;
    let nx = (LX_D / h_spec).round() as usize;
    let ny = (LY_D / h_spec).round() as usize;
    let center = [ft(CENTER_X_D), ft(LY_D * 0.5 + CENTER_OFFSET_D)];
    let nu = ft(1.0) / reynolds;
    let dt = ft(CFL * h_spec);

    let zones = (
        Inflow::<2, FloatType>::new(0, false, ft(1.0))?,
        (
            Outflow::<2>::new(0, true)?,
            (SlipWall::<2>::new(1, false)?, SlipWall::<2>::new(1, true)?),
        ),
    );

    let solver = CfdConfigBuilder::dec_ns()
        .viscosity(nu)
        .time_step(dt)
        .cg_options(HodgeDecomposeOptions {
            tolerance: Some(ft(CG_TOL)),
            // The verification harness's grid-scaled CG budget; the library default starves
            // finer grids.
            max_iterations: Some(30 * (nx + ny)),
        })
        .warm_start()
        .build()?;

    let config = CfdConfigBuilder::march::<2, FloatType>("viv-wake")
        .mesh(
            Mesh::box_domain([nx, ny])
                .spacing(ft(h_spec))
                .immersed(Body::disk(center, ft(0.5)).merge_floor(ft(MERGE_FLOOR))),
        )
        .solver(solver)
        .zones(zones)
        .seed(Seed::UniformX { speed: 1.0 })
        .march_for(STEPS)
        // The wake probe: transverse velocity 1.5 D downstream of the body, on its centerline.
        .observe(Observe::default().probe([center[0] + ft(1.5), center[1]]))
        .build()?;

    Ok(WakeCase { config, dt })
}
