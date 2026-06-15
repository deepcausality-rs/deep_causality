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

use deep_causality_cfd::{DecNsSolver, DropoutVerbosity, UncertainInflowZone, march_inflow};
use deep_causality_haft::LogSize;
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
