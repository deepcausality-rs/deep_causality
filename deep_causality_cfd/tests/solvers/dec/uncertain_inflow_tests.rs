/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Group C — the uncertain-inflow zone (`uncertain-inflow-zone`).
//!
//! Fast, no-validation rungs (the heavy march lives in the cylinder example, per the
//! tests-fast / examples-verify split):
//!
//! - **no-dropout equivalence (C4)**: an all-present sensor stream reproduces the deterministic
//!   moving-wall control march to rounding, and logs nothing;
//! - **dropout intervention (C4)**: an intermittently-absent stream completes the march, falling
//!   back to the last-good value through a logged `intervene`;
//! - **memory confined to the patch (C4/D6)**: the marched state's heavy field stays plain `R` at
//!   the lattice's edge count — the `MaybeUncertain` carrier is only the per-step stream.

use deep_causality_cfd::{
    DecNsSolver, DropoutVerbosity, InflowContext, InflowMarchState, UncertainInflowZone,
    inflow_march_step, march_inflow,
};
use deep_causality_core::EffectValue;
use deep_causality_haft::LogSize;
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};
use deep_causality_uncertain::MaybeUncertain;

const N: usize = 5;
const NU: f64 = 0.1;
const DT: f64 = 0.02;
const U_IN: f64 = 0.2;
const STEPS: usize = 4;

/// An all-walls `N×N` unit-square lattice (so the moving-wall boundary is valid).
fn wall_manifold() -> Manifold<LatticeComplex<2, f64>, f64> {
    let h = 1.0 / (N - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([N, N], [false, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn base_solver(m: &Manifold<LatticeComplex<2, f64>, f64>) -> DecNsSolver<'_, 2, f64> {
    DecNsSolver::new(m, NU, DT, None).unwrap()
}

/// A zone driving the top wall (axis 1, far face) with a streamwise (axis 0) inflow, with cheap
/// SPRT/collapse budgets for a fast test.
fn fast_zone(default_inflow: f64) -> UncertainInflowZone<f64> {
    UncertainInflowZone::new(1, true, 0, default_inflow)
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(8)
}

#[test]
fn no_dropout_stream_matches_the_deterministic_control_to_rounding() {
    let m = wall_manifold();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();

    // Shared rest seed (lift-free), cloned for both marches so they start identically.
    let seed = base_solver(&m).seed_from_vertex_vectors(&rest).unwrap();

    // Deterministic control: a fixed moving wall at U_IN, marched by hand.
    let control_solver = base_solver(&m)
        .with_moving_wall(1, true, [U_IN, 0.0])
        .unwrap();
    let mut control = seed.clone();
    for _ in 0..STEPS {
        control = control_solver.step(&control).unwrap().into_state();
    }

    // Uncertain march: an all-present stream at exactly U_IN.
    let zone = fast_zone(U_IN);
    let stream = vec![MaybeUncertain::<f64>::from_value(U_IN); STEPS];
    let process = march_inflow(base_solver(&m), seed, zone, stream, STEPS).unwrap();

    assert!(process.error().is_none(), "no-dropout march must not error");
    assert_eq!(
        process.logs().len(),
        0,
        "an all-present stream records no dropouts"
    );

    let got = process.state().field().as_one_form().as_slice();
    let want = control.as_one_form().as_slice();
    assert_eq!(got.len(), want.len());
    for (g, w) in got.iter().zip(want.iter()) {
        assert!(
            (g - w).abs() <= 1e-9,
            "uncertain march diverged from the deterministic control: {g} vs {w}"
        );
    }
}

#[test]
fn dropout_stream_completes_via_a_logged_intervention() {
    let m = wall_manifold();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = base_solver(&m).seed_from_vertex_vectors(&rest).unwrap();

    // Present, absent, present, absent — dropouts at steps 1 and 3.
    let stream = vec![
        MaybeUncertain::<f64>::from_value(U_IN),
        MaybeUncertain::<f64>::always_none(),
        MaybeUncertain::<f64>::from_value(U_IN),
        MaybeUncertain::<f64>::always_none(),
    ];
    let zone = fast_zone(U_IN);
    let process = march_inflow(base_solver(&m), seed, zone, stream, STEPS).unwrap();

    assert!(
        process.error().is_none(),
        "the march survives dropouts via the fallback"
    );

    // Each of the two dropouts records a fallback entry plus the `intervene` value alternation.
    assert_eq!(
        process.logs().len(),
        4,
        "two dropouts × (fallback record + value alternation)"
    );
    let log = format!("{}", process.logs());
    assert!(
        log.contains("dropout"),
        "the fallback is recorded in the EffectLog: {log}"
    );
    assert!(
        log.contains("ValueAlternation"),
        "the fallback is applied through a Pearl do(...) intervention: {log}"
    );

    // The field stays finite, and the last-good value held through the dropouts.
    assert!(
        process
            .state()
            .field()
            .as_one_form()
            .as_slice()
            .iter()
            .all(|v| v.is_finite()),
        "the marched field stays finite under dropouts"
    );
    assert!(
        (process.state().last_good() - U_IN).abs() <= 1e-9,
        "the last present value is retained as the fallback (got {})",
        process.state().last_good()
    );
}

#[test]
fn transition_verbosity_logs_only_onset_and_recovery() {
    let m = wall_manifold();
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = base_solver(&m).seed_from_vertex_vectors(&rest).unwrap();

    // present, absent, absent, present: one onset (step 1) and one recovery (step 3).
    let stream = vec![
        MaybeUncertain::<f64>::from_value(U_IN),
        MaybeUncertain::<f64>::always_none(),
        MaybeUncertain::<f64>::always_none(),
        MaybeUncertain::<f64>::from_value(U_IN),
    ];
    let zone = fast_zone(U_IN).with_verbosity(DropoutVerbosity::Transitions);
    let process = march_inflow(base_solver(&m), seed, zone, stream, STEPS).unwrap();

    assert!(process.error().is_none());
    let log = format!("{}", process.logs());
    assert!(
        log.contains("ONSET"),
        "the first dropout logs an onset: {log}"
    );
    assert!(
        log.contains("RECOVERY"),
        "the return to a present sample logs a recovery: {log}"
    );
    // Two transition records, plus a value alternation for each of the two dropped steps.
    assert_eq!(process.logs().len(), 4);
}

/// A fully periodic `N×N` lattice — the prescribed moving wall is invalid on a periodic axis.
fn periodic_manifold() -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice = LatticeComplex::<2, f64>::new([N, N], [true, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(1.0);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn rest_seed(
    m: &Manifold<LatticeComplex<2, f64>, f64>,
) -> deep_causality_physics::SolenoidalField<f64> {
    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    base_solver(m).seed_from_vertex_vectors(&rest).unwrap()
}

#[test]
fn march_state_reports_step_count_and_dropout_flag() {
    let m = wall_manifold();

    // A clean stream: the final state reports the full step count and no dropout.
    let clean = vec![MaybeUncertain::<f64>::from_value(U_IN); STEPS];
    let process = march_inflow(
        base_solver(&m),
        rest_seed(&m),
        fast_zone(U_IN),
        clean,
        STEPS,
    )
    .unwrap();
    assert_eq!(process.state().step(), STEPS);
    assert!(!process.state().in_dropout());

    // A stream whose final sample is absent leaves the march flagged in dropout.
    let ends_absent = vec![
        MaybeUncertain::<f64>::from_value(U_IN),
        MaybeUncertain::<f64>::always_none(),
    ];
    let process = march_inflow(
        base_solver(&m),
        rest_seed(&m),
        fast_zone(U_IN),
        ends_absent,
        2,
    )
    .unwrap();
    assert!(process.state().in_dropout());
}

#[test]
fn march_inflow_rejects_out_of_range_flow_axis() {
    let m = wall_manifold();
    let zone = UncertainInflowZone::new(1, true, 5, U_IN); // flow axis 5 ≥ D = 2
    let stream = vec![MaybeUncertain::<f64>::from_value(U_IN); STEPS];
    let err = march_inflow(base_solver(&m), rest_seed(&m), zone, stream, STEPS).unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));
}

#[test]
fn march_inflow_rejects_a_stream_shorter_than_the_horizon() {
    let m = wall_manifold();
    let stream = vec![MaybeUncertain::<f64>::from_value(U_IN); STEPS - 1];
    let err = march_inflow(
        base_solver(&m),
        rest_seed(&m),
        fast_zone(U_IN),
        stream,
        STEPS,
    )
    .unwrap_err();
    assert!(matches!(err.0, PhysicsErrorEnum::DimensionMismatch(_)));
}

#[test]
fn inflow_step_without_context_short_circuits() {
    let m = wall_manifold();
    let state = InflowMarchState::new(base_solver(&m), rest_seed(&m), U_IN);
    let process = inflow_march_step::<2, f64>(EffectValue::Value(U_IN), state, None);
    assert!(
        process.error().is_some(),
        "a missing context short-circuits the step"
    );
}

#[test]
fn inflow_step_with_exhausted_stream_short_circuits() {
    let m = wall_manifold();
    let state = InflowMarchState::new(base_solver(&m), rest_seed(&m), U_IN);
    // Step 0 against an empty stream is past the horizon.
    let context = InflowContext::new(fast_zone(U_IN), Vec::new());
    let process = inflow_march_step(EffectValue::Value(U_IN), state, Some(context));
    assert!(
        process.error().is_some(),
        "an exhausted stream short-circuits the step"
    );
}

#[test]
fn inflow_step_rejects_an_invalid_wall_reconfiguration() {
    // A present sample resolves, but reconfiguring a moving wall on a periodic axis is rejected.
    let m = periodic_manifold();
    let state = InflowMarchState::new(base_solver(&m), rest_seed(&m), U_IN);
    let zone = UncertainInflowZone::new(0, true, 1, U_IN) // wall axis 0 is periodic
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(8);
    let context = InflowContext::new(zone, vec![MaybeUncertain::<f64>::from_value(U_IN)]);
    let process = inflow_march_step(EffectValue::Value(U_IN), state, Some(context));
    assert!(
        process.error().is_some(),
        "reconfiguring a moving wall on a periodic axis is rejected"
    );
}

#[test]
fn inflow_step_on_a_consumed_solver_short_circuits() {
    // A failed wall reconfiguration consumes the solver, leaving the returned state with
    // `solver: None`. Re-binding that state must short-circuit on the "solver consumed by a prior
    // failure" guard rather than panic.
    let m = periodic_manifold();
    let state = InflowMarchState::new(base_solver(&m), rest_seed(&m), U_IN);
    let zone = UncertainInflowZone::new(0, true, 1, U_IN) // wall axis 0 is periodic → reconfig fails
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(8);
    let context = InflowContext::new(zone, vec![MaybeUncertain::<f64>::from_value(U_IN); 2]);
    let failed = inflow_march_step(EffectValue::Value(U_IN), state, Some(context.clone()));
    assert!(
        failed.error().is_some(),
        "the reconfiguration must fail first"
    );

    // Re-bind the errored state (its solver was consumed).
    let again = inflow_march_step(
        EffectValue::Value(U_IN),
        failed.into_parts().1,
        Some(context),
    );
    let err = again.error().expect("the consumed-solver guard must fire");
    assert!(
        format!("{err:?}").contains("consumed"),
        "expected the consumed-solver guard: {err:?}"
    );
}

#[test]
fn inflow_step_short_circuits_when_sample_resolution_fails() {
    // A QMC collapse of a branch-divergent conditional sample fails (the tree is not statically
    // structured). The march step must short-circuit on the resolution error, keeping the solver
    // live in the returned state.
    use deep_causality_uncertain::Uncertain;

    let m = wall_manifold();
    let state = InflowMarchState::new(base_solver(&m), rest_seed(&m), U_IN);
    let zone = UncertainInflowZone::new(1, true, 0, U_IN)
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(8)
        .with_qmc_collapse(0x0BAD_5EED);

    let cond = Uncertain::<bool>::point(true);
    let dynamic = Uncertain::<f64>::conditional(
        cond,
        Uncertain::normal(0.2, 0.01),
        Uncertain::normal(0.9, 0.01),
    );
    let context = InflowContext::new(zone, vec![MaybeUncertain::<f64>::from_uncertain(dynamic)]);

    let process = inflow_march_step(EffectValue::Value(U_IN), state, Some(context));
    let err = process
        .error()
        .expect("a resolution failure short-circuits the step");
    assert!(
        format!("{err:?}").contains("resolution"),
        "expected the sample-resolution guard: {err:?}"
    );
    // The solver survives the resolution failure (it was never consumed), so the state is re-marchable.
    assert!(
        process
            .state()
            .field()
            .as_one_form()
            .as_slice()
            .iter()
            .all(|v| v.is_finite()),
        "the field is preserved through a resolution failure"
    );
}

#[test]
fn memory_cost_is_confined_to_the_tagged_patch() {
    let m = wall_manifold();
    let n0 = m.complex().num_cells(0);
    let n1 = m.complex().num_cells(1);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let seed = base_solver(&m).seed_from_vertex_vectors(&rest).unwrap();

    let stream = vec![MaybeUncertain::<f64>::from_value(U_IN); STEPS];
    let process = march_inflow(base_solver(&m), seed, fast_zone(U_IN), stream, STEPS).unwrap();

    // The heavy state — the velocity field — stays plain `R` at the lattice edge count; the
    // uncertain types never enter the march, so the only MaybeUncertain memory is the per-step
    // stream (the tagged inflow patch), not the O(edges) field.
    assert_eq!(
        process.state().field().as_one_form().len(),
        n1,
        "the marched field is the plain edge cochain, not widened by the zone"
    );
}
