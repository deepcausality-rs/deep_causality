/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The compressible coupled host, split by what is under test: [`carrier_tests`] covers the evolved
//! projections, the shock-fitted inflow strip, and the rebuild machinery; [`fork_tests`] the pause/fork
//! state sharing; [`study_tests`] the campaign grammar lowered onto it; [`audit_tests`] the save_log sink; and
//! [`forcing_tests`] the forcing-region seam with [`imprint_tests`] the plume-imprint channel riding on it.
//!
//! The world builders every submodule marches live here — one definition of the scheduled descent
//! world, so a change to the grid or the reference scales lands in one place.

mod audit_tests;
mod carrier_tests;
mod forcing_tests;
mod fork_tests;
mod imprint_tests;
mod study_tests;

use deep_causality_cfd::{
    Ambient, AtmosphereRow, CompressibleMarchConfig, CompressibleMarchConfigBuilder, CoupledField,
    DescentSchedule, MarchStop, QttObserve, ReferenceScales,
};
use deep_causality_physics::EARTH_RADIUS;
use deep_causality_tensor::Truncation;

pub const GAMMA_EFF: f64 = 1.1;

fn rows() -> Vec<AtmosphereRow<f64>> {
    vec![
        AtmosphereRow {
            altitude_m: 30_000.0,
            n_tot: 8.0e23,
            temperature: 226.0,
            sound_speed: 301.0,
        },
        AtmosphereRow {
            altitude_m: 61_000.0,
            n_tot: 1.3e21,
            temperature: 250.0,
            sound_speed: 317.0,
        },
        AtmosphereRow {
            altitude_m: 90_000.0,
            n_tot: 7.0e19,
            temperature: 187.0,
            sound_speed: 274.0,
        },
    ]
}

fn reference() -> ReferenceScales<f64> {
    ReferenceScales {
        t_ref: 8_044.0,
        n_ref: 2.645e22,
        u_ref: 376.0,
    }
}

/// A small scheduled descent world: post-shock-like uniform seed, `s_ref` roomy enough that no
/// rebuild triggers.
fn world(name: &str, s_ref: f64, steps: usize) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, s_ref, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(
            QttObserve::default()
                .electron_density()
                .plasma_frequency()
                .blackout_dwell(),
        )
        .schedule(DescentSchedule::new(rows(), GAMMA_EFF).unwrap())
        .reference(reference())
        .build()
        .unwrap()
}

/// A carried field with the truth vehicle at 61 km, flying Mach ~24 tangentially.
fn field_at_61km() -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01, 0.0, None));
    let r = EARTH_RADIUS + 61_000.0;
    f.set_scalar("truth_state", vec![r, 0.0, 0.0, 0.0, 7_650.0, 0.0]);
    f
}

/// A bare field carrying a flight velocity, so the plume stage can resolve a direction for its
/// along-velocity drag decrement.
fn imprint_field() -> CoupledField<f64> {
    let mut f = CoupledField::new(Ambient::new(0.01, 0.0, None));
    f.set_scalar(
        "truth_state",
        vec![EARTH_RADIUS + 30_000.0, 0.0, 0.0, -400.0, 0.0, 0.0],
    );
    f
}

/// The `world()` helper with a rebuild budget attached to its schedule.
fn budgeted_world(
    name: &str,
    s_ref: f64,
    steps: usize,
    budget: usize,
) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, s_ref, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .schedule(
            DescentSchedule::new(rows(), GAMMA_EFF)
                .unwrap()
                .with_rebuild_budget(budget),
        )
        .reference(reference())
        .build()
        .unwrap()
}

/// The reduced row the ensemble campaigns share: whether the case's world was alternated, and how
/// many draws its ensemble flew.
pub struct EnsRow {
    pub marked: bool,
    pub draws: usize,
}
