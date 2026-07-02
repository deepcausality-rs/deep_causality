/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The resumable, forkable QTT march (Stage 4.7/4.8): `run_until` → [`MarchPause`], O(1) `fork`,
//! alternation on the fork (verbatim vocabulary, error channel never alternated), `continue_march`
//! with copy-on-write isolation between branches.

use deep_causality_cfd::MarchStop;
use deep_causality_cfd::{
    AeroBlackoutStub, Ambient, BlackoutTrigger, CfdFlow, CoupledField, CyberneticCorrect,
    QttMarchConfig, QttMarchConfigBuilder, QttObserve, SafetyEnvelope,
};
use deep_causality_core::{AlternatableContext, AlternatableState};
use deep_causality_haft::LogSize;
use deep_causality_tensor::{CausalTensor, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const N: usize = 16;
const L: usize = 4;

/// A named world: a Taylor–Green seed (so viscosity differences between worlds are visible),
/// fixed horizon, blackout observables on.
fn world(name: &str, nu: f64, steps: usize) -> QttMarchConfig<f64> {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_bond(4096).unwrap();
    QttMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(L, L, dx, dx)
        .solver(0.005, nu, trunc)
        .seed_fn(|x, y| (-(x.cos() * y.sin()), x.sin() * y.cos()))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(
            QttObserve::default()
                .electron_density()
                .plasma_frequency()
                .blackout_dwell(),
        )
        .build()
        .unwrap()
}

/// The ④ stub raising `n_e` to the blackout level over coupling steps `[2, 5)`.
fn stub() -> AeroBlackoutStub<f64> {
    AeroBlackoutStub::new(3.0_f64, 1.0e17, 1.0e20, 2, 5)
}

fn trigger() -> BlackoutTrigger<f64> {
    BlackoutTrigger::new(1.0e9)
}

fn initial() -> CoupledField<f64> {
    CoupledField::new(Ambient::new(0.01, 0.0, None))
}

/// The blackout-onset predicate: pause when the published `n_e` reaches the blackout level.
fn at_blackout_onset(field: &CoupledField<f64>, _step: usize) -> bool {
    field
        .scalar("n_e")
        .is_some_and(|ne| ne.iter().any(|&x| x >= 1.0e20))
}

#[test]
fn run_until_pauses_at_the_predicate() {
    let cfg = world("nominal_reentry", 0.05, 10);
    let pause = CfdFlow::qtt_march(&cfg)
        .run_until(stub(), initial(), trigger(), 0.01, at_blackout_onset)
        .unwrap();

    // The stub's window opens at coupling step 2: that is where the march pauses.
    assert_eq!(pause.step(), 2, "paused at blackout onset");
    assert!(pause.error().is_none());
    assert!(pause.field().scalar("n_e").is_some());
    assert!(
        format!("{}", pause.field().log()).contains("march paused at step 2"),
        "the pause is provenanced"
    );
}

#[test]
fn run_until_exhausts_the_horizon_when_the_predicate_never_fires() {
    let cfg = world("nominal_reentry", 0.05, 6);
    let pause = CfdFlow::qtt_march(&cfg)
        .run_until(stub(), initial(), trigger(), 0.01, |_, _| false)
        .unwrap();
    assert_eq!(
        pause.step(),
        6,
        "dwell exhausted: paused at the stop horizon"
    );
    assert!(pause.error().is_none());
}

#[test]
fn fork_is_o1_and_shares_the_onset_by_reference() {
    let cfg = world("nominal_reentry", 0.05, 10);
    let pause = CfdFlow::qtt_march(&cfg)
        .run_until(stub(), initial(), trigger(), 0.01, at_blackout_onset)
        .unwrap();

    let fork = pause.fork();
    assert!(fork.shares_fluid_with(&pause), "no tensor data copied");
    assert!(fork.shares_field_with(&pause), "no field data copied");
    assert!(
        fork.audit_log().is_empty(),
        "a fresh fork has no alternations"
    );
}

#[test]
fn continued_branches_are_isolated_and_deterministic() {
    let cfg = world("nominal_reentry", 0.05, 10);
    let pause = CfdFlow::qtt_march(&cfg)
        .run_until(stub(), initial(), trigger(), 0.01, at_blackout_onset)
        .unwrap();
    let onset_ne = pause.field().scalar("n_e").unwrap().to_vec();
    let onset_log_len = pause.field().log().len();

    // Two identical branches from the same shared onset.
    let a = pause.fork().continue_march(3).unwrap();
    let b = pause.fork().continue_march(3).unwrap();

    // Determinism: identical worlds → identical continued reports.
    assert_eq!(a.series("n_e").unwrap(), b.series("n_e").unwrap());
    assert_eq!(a.final_field().unwrap(), b.final_field().unwrap());
    assert_eq!(
        a.series("n_e").unwrap().len(),
        3,
        "the continued segment only"
    );

    // Copy-on-write isolation: the branches wrote their own clones; the pause is pristine.
    assert_eq!(pause.field().scalar("n_e").unwrap(), &onset_ne[..]);
    assert_eq!(pause.field().log().len(), onset_log_len);

    // The branch log carries the full history: the pause marker plus the resume marker.
    let log = format!("{}", a.effect_log().expect("branch log"));
    assert!(log.contains("march paused at step 2"), "history: {log}");
    assert!(
        log.contains("march resumed at step 2 for 3 steps"),
        "resume: {log}"
    );
}

#[test]
fn alternate_context_forks_into_a_different_world() {
    let nominal = world("nominal_reentry", 0.05, 10);
    let steep = world("steep_reentry", 0.09, 10);

    let pause = CfdFlow::qtt_march(&nominal)
        .run_until(stub(), initial(), trigger(), 0.01, at_blackout_onset)
        .unwrap();

    let baseline = pause.fork().continue_march(3).unwrap();
    let branch = pause
        .fork()
        .alternate_context(&steep)
        .continue_march(3)
        .unwrap();

    // The branch marched the steep world from the shared onset state.
    assert_eq!(branch.name(), "steep_reentry");
    assert_ne!(
        branch.final_field().unwrap(),
        baseline.final_field().unwrap(),
        "a different world diverges from the shared onset"
    );
    let log = format!("{}", branch.effect_log().unwrap());
    assert!(log.contains("!!ContextAlternation!!"), "marker: {log}");
    assert!(
        log.contains("nominal_reentry") && log.contains("steep_reentry"),
        "the entry names both worlds: {log}"
    );
}

#[test]
fn alternate_state_reseeds_the_branch_fluid() {
    let cfg = world("nominal_reentry", 0.05, 10);
    let pause = CfdFlow::qtt_march(&cfg)
        .run_until(stub(), initial(), trigger(), 0.01, at_blackout_onset)
        .unwrap();

    let ncells = N * N;
    let u = CausalTensor::new(vec![0.25_f64; ncells], vec![N, N]).unwrap();
    let v = CausalTensor::new(vec![0.0_f64; ncells], vec![N, N]).unwrap();

    let baseline = pause.fork().continue_march(2).unwrap();
    let reseeded = pause
        .fork()
        .alternate_state((u, v))
        .continue_march(2)
        .unwrap();

    assert_ne!(
        reseeded.final_field().unwrap(),
        baseline.final_field().unwrap(),
        "the alternated fluid state diverges"
    );
    let log = format!("{}", reseeded.effect_log().unwrap());
    assert!(log.contains("!!StateAlternation!!"), "marker: {log}");
}

#[test]
fn a_step_error_is_captured_into_the_pause() {
    // A breached envelope errors the coupling at step 1: the pause captures it instead of tearing
    // the march down, so the state and log remain inspectable.
    let cfg = world("nominal_reentry", 0.05, 10);
    let gate = CyberneticCorrect::new(SafetyEnvelope::new(1.0e6, 12.0, 0.5));
    let mut field = initial();
    field.set_scalar("heat_flux", vec![2.0e6]); // above the ceiling from the start

    let pause = CfdFlow::qtt_march(&cfg)
        .run_until(gate, field, trigger(), 0.01, |_, _| false)
        .unwrap();

    assert!(pause.error().is_some(), "the breach is captured");
    assert_eq!(pause.step(), 0, "no step completed");
    let log = format!("{}", pause.field().log());
    assert!(log.contains("march error captured at step 1"), "log: {log}");
}

#[test]
fn alternation_on_an_errored_fork_is_a_noop_with_only_the_audit_entry() {
    let nominal = world("nominal_reentry", 0.05, 10);
    let steep = world("steep_reentry", 0.09, 10);
    let gate = CyberneticCorrect::new(SafetyEnvelope::new(1.0e6, 12.0, 0.5));
    let mut field = initial();
    field.set_scalar("heat_flux", vec![2.0e6]);

    let pause = CfdFlow::qtt_march(&nominal)
        .run_until(gate, field, trigger(), 0.01, |_, _| false)
        .unwrap();
    assert!(pause.error().is_some());

    // The error channel is never alternated: the swap is refused, only the audit entry lands.
    let fork = pause.fork().alternate_context(&steep);
    assert_eq!(fork.audit_log().len(), 1);
    assert!(
        format!("{}", fork.audit_log()).contains("not applied"),
        "audit: {}",
        fork.audit_log()
    );

    // And the broken chain propagates its error on continue.
    assert!(fork.continue_march(2).is_err());
}
