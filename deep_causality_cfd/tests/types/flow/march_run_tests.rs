/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `MarchRun` DSL surface: the no-op readability stages (`solver`/`seed`/`march`/
//! `observe`), the per-run counterfactual overrides (`*_with*`), the `run_with` per-step hook and
//! the `StepView` it exposes, the steady-state stop, and the centerline diagnostic. The marching
//! *cases* drive `.run()` directly, so these compositional surfaces are covered here.

use deep_causality_cfd::{CfdConfigBuilder, CfdFlow, MarchConfig, MarchStop, Mesh, Observe, Seed};
use deep_causality_physics::PhysicsErrorEnum;

#[test]
fn test_spectral_diffusion_on_a_nonperiodic_mesh_errors() {
    // Spectral diffusion is periodic-only; materializing it on a walled box domain is rejected.
    let config = CfdConfigBuilder::march::<2, f64>("spectral-nonperiodic")
        .mesh(Mesh::box_domain([8, 8]))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.005)
                .spectral_diffusion()
                .build()
                .expect("config builds; the periodic check is at materialization"),
        )
        .seed(Seed::Rest)
        .march_for(1)
        .build()
        .expect("march config builds");
    let manifold = config.materialize().expect("geometry");
    let err = CfdFlow::march(&config)
        .on(&manifold)
        .run()
        .expect_err("spectral diffusion on a non-periodic mesh is rejected");
    assert!(matches!(err.0, PhysicsErrorEnum::TopologyError(_)));
}

/// A small at-rest cavity that stays at rest (no forcing) — cheap and deterministic.
fn cavity(steps: usize, observe: Observe<2, f64>) -> MarchConfig<2, f64, (), ()> {
    CfdConfigBuilder::march::<2, f64>("dsl")
        .mesh(Mesh::box_domain([8, 8]))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(0.05)
                .time_step(0.005)
                .build()
                .expect("valid solver config"),
        )
        .seed(Seed::Rest)
        .march_for(steps)
        .observe(observe)
        .build()
        .expect("valid march config")
}

#[test]
fn test_noop_stages_compose_and_run() {
    let config = cavity(
        3,
        Observe::default().kinetic_energy().divergence().max_speed(),
    );
    let manifold = config.materialize().expect("geometry");
    let report = CfdFlow::march(&config)
        .on(&manifold)
        .solver()
        .seed()
        .march()
        .observe()
        .run()
        .expect("march runs");

    // Seed sample + one per step.
    assert_eq!(report.series("kinetic_energy").expect("energy").len(), 4);
    assert!(report.series("divergence").is_some());
    assert!(report.series("max_speed").is_some());
    // A DEC march records the final edge cochain.
    assert!(report.final_field().is_some());
}

#[test]
fn test_per_run_overrides() {
    let config = cavity(5, Observe::default().kinetic_energy());
    let manifold = config.materialize().expect("geometry");
    let alt_solver = CfdConfigBuilder::dec_ns()
        .viscosity(0.02)
        .time_step(0.004)
        .build()
        .expect("alt solver");
    let report = CfdFlow::march(&config)
        .on(&manifold)
        .solver_with_config(alt_solver)
        .seed_with_config(Seed::Rest)
        .march_with(MarchStop::Fixed(2))
        .observe_with_config(Observe::default().kinetic_energy())
        .run()
        .expect("march runs");
    // The march-stop override wins: seed sample + 2 steps.
    assert_eq!(report.series("kinetic_energy").expect("energy").len(), 3);
}

#[test]
fn test_run_with_hook_exposes_step_view() {
    let config = cavity(2, Observe::default().kinetic_energy());
    let manifold = config.materialize().expect("geometry");
    let mut steps_seen = 0usize;
    let report = CfdFlow::march(&config)
        .on(&manifold)
        .run_with(|view| {
            steps_seen += 1;
            assert_eq!(view.step(), steps_seen);
            assert!(view.dt() > 0.0);
            assert!((view.time() - view.dt() * view.step() as f64).abs() < 1e-12);
            // The raw state / cochain / manifold are reachable for bespoke probes.
            let _ = view.state();
            assert!(!view.one_form().as_slice().is_empty());
            let _ = view.manifold();
            assert!(view.kinetic_energy().expect("energy").is_finite());
            assert!(view.max_speed().expect("speed").is_finite());
            assert!(view.divergence().expect("divergence").is_finite());
        })
        .expect("march runs");
    assert_eq!(steps_seen, 2);
    assert!(report.series("kinetic_energy").is_some());
}

#[test]
fn test_steady_stop_breaks_on_convergence() {
    // At rest with no forcing, the energy change is zero, so the steady stop breaks immediately.
    let config = cavity(0, Observe::default().kinetic_energy());
    let manifold = config.materialize().expect("geometry");
    let report = CfdFlow::march(&config)
        .on(&manifold)
        .march_with(MarchStop::Steady {
            tol: 1e-9,
            max_steps: 50,
        })
        .run()
        .expect("march runs");
    let energy = report.series("kinetic_energy").expect("energy");
    // Converged well before max_steps (seed + a couple of steps at most).
    assert!(energy.len() < 50);
}

#[test]
fn test_centerline_profile_is_recorded() {
    let config = cavity(1, Observe::default().centerline(0));
    let manifold = config.materialize().expect("geometry");
    let report = CfdFlow::march(&config)
        .on(&manifold)
        .run()
        .expect("march runs");
    assert!(report.series("centerline").is_some());
}

#[test]
fn test_centerline_axis_out_of_range_errors() {
    let config = cavity(1, Observe::default().centerline(5));
    let manifold = config.materialize().expect("geometry");
    let err = CfdFlow::march(&config)
        .on(&manifold)
        .run()
        .expect_err("axis 5 is out of range for D = 2");
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));
}
