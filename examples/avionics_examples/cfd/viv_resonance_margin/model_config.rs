/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration construction (the "what"): the nondimensional validated cylinder-wake case
//! at one Reynolds number. Execution stays in `model`; tuned values stay in `constants`.

use crate::FloatType;
use crate::constants::{
    CELLS_PER_D, CENTER_OFFSET_D, CENTER_X_D, CFL, CG_TOL, LX_D, LY_D, MERGE_FLOOR, STEPS,
};
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::{
    Body, CfdConfigBuilder, Inflow, MarchConfig, Mesh, Observe, Outflow, Seed, SlipWall,
};
use deep_causality_physics::PhysicsError;
use deep_causality_topology::HodgeDecomposeOptions;

/// The wake-zone tuple of the validated isolated-cylinder setup: west inflow, east outflow,
/// far-field slip walls (dec_cylinder_verification), composed as the static zone tuple.
pub type WakeZones = (
    Inflow<2, FloatType>,
    (Outflow<2>, (SlipWall<2>, SlipWall<2>)),
);

/// The nondimensional case at `reynolds`: D = 1, U = 1, nu = 1/Re, dt = CFL * h, the
/// validated zone set, and the wake probe 1.5 D downstream on the centerline. Returns the
/// case and its time step (the Strouhal reduction needs `dt`). Counts and grid sizing are
/// exact `f64` specifications; everything the march computes with lifts into `FloatType`
/// once, through `ft`.
pub fn wake_case(
    reynolds: FloatType,
) -> Result<(MarchConfig<2, FloatType, WakeZones, ()>, FloatType), PhysicsError> {
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

    let case = CfdConfigBuilder::march::<2, FloatType>("viv-wake")
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

    Ok((case, dt))
}
