/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The [`TrajectoryNav`] stage: KS predict off the aero-force channel, the ESKF fold, and the
//! consumed one-shot fixes.

use super::{ctx, field};
use deep_causality_cfd::{
    GoverningModel, InsErrorState, MachRegime, NavFilter, PhysicsStage, ReentryNavEngine,
    RegimeClass, ThrustState, TrajectoryNav,
};
use deep_causality_haft::LogSize;
use deep_causality_physics::EARTH_GM;

fn nav_engine() -> ReentryNavEngine<f64> {
    // The bound LEO-ish state the nav module's own tests use.
    let (r0, v0) = ([7.0e6, 1.0e6, 2.0e6], [-1.0e3, 6.5e3, 3.0e3]);
    let filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]).unwrap();
    ReentryNavEngine::new(r0, v0, EARTH_GM, filter)
}

fn nav_stage() -> TrajectoryNav<f64> {
    // Q diagonal, GNSS 5 m 1σ (25 m² variance), through-plasma optical 50 m 1σ.
    TrajectoryNav::new([1.0e-6; 17], 25.0, 2500.0)
}

fn denied_regime() -> RegimeClass<f64> {
    RegimeClass {
        model: GoverningModel::Continuum,
        knudsen: 1.0e-3,
        plasma_frequency: 1.0e10,
        gnss_denied: true,
        // A corridor-class regime carries the powered-descent axes neutral.
        mach_regime: MachRegime::Unknown,
        thrust_state: ThrustState::Unknown,
        touchdown: false,
    }
}

#[test]
fn trajectory_nav_is_a_noop_without_an_engine() {
    let mut f = field();
    nav_stage().apply(&ctx(1), &mut f).expect("applies");
    assert!(f.scalar("nav_position").is_none());
    assert!(f.log().is_empty());
}

#[test]
fn trajectory_nav_dead_reckons_and_publishes_witnesses() {
    let mut f = field();
    f.set_nav(nav_engine());

    nav_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.scalar("nav_position").unwrap().len(), 3);
    assert_eq!(f.scalar("nav_position_variance").unwrap().len(), 1);
    assert!(f.nav().is_some(), "the engine threads back into the field");
    assert_eq!(f.log().len(), 1, "the dead-reckoning transition is logged");

    // A second dead-reckoning step is not a transition: no new entry.
    nav_stage().apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(f.log().len(), 1);
}

#[test]
fn trajectory_nav_reads_the_aero_force_channel() {
    let stage = nav_stage();

    let mut coasting = field();
    coasting.set_nav(nav_engine());
    stage.apply(&ctx(1), &mut coasting).expect("applies");

    let mut dragged = field();
    dragged.set_nav(nav_engine());
    dragged.set_aero_force([-1.0e3, 0.0, 0.0]);
    stage.apply(&ctx(1), &mut dragged).expect("applies");

    let a = coasting.scalar("nav_position").unwrap();
    let b = dragged.scalar("nav_position").unwrap();
    assert!(
        (a[0] - b[0]).abs() > 1.0,
        "the ④ aero kick perturbs the predicted position: {} vs {}",
        a[0],
        b[0]
    );
}

#[test]
fn gnss_fix_is_folded_when_available() {
    let stage = nav_stage();

    // Twin runs: one dead-reckons, one folds a GNSS fix at the propagated position.
    let mut dead = field();
    dead.set_nav(nav_engine());
    stage.apply(&ctx(1), &mut dead).expect("applies");

    let mut aided = field();
    aided.set_nav(nav_engine());
    let predicted = dead.scalar("nav_position").unwrap().to_vec();
    aided.set_scalar("gnss_fix", predicted);
    stage.apply(&ctx(1), &mut aided).expect("applies");

    assert_eq!(aided.scalar("nav_mode").unwrap()[0], 1.0, "aided");
    assert_eq!(dead.scalar("nav_mode").unwrap()[0], 0.0, "dead reckoning");
    // The fix collapses the position variance below the predict-only twin.
    let v_aided = aided.scalar("nav_position_variance").unwrap()[0];
    let v_dead = dead.scalar("nav_position_variance").unwrap()[0];
    assert!(
        v_aided < v_dead,
        "fix collapses variance: {v_aided} vs {v_dead}"
    );
}

#[test]
fn gnss_fix_is_gated_by_the_denial_flag() {
    let mut f = field();
    f.set_nav(nav_engine());
    f.set_regime(denied_regime());
    f.set_scalar("gnss_fix", vec![7.0e6, 1.0e6, 2.0e6]);

    nav_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(
        f.scalar("nav_mode").unwrap()[0],
        0.0,
        "a denied link never folds the GNSS fix"
    );
    assert!(
        f.scalar("gnss_fix").is_none(),
        "the denied-step broadcast is consumed unread, not latched for reacquisition"
    );
}

#[test]
fn optical_fix_rides_through_the_blackout() {
    let mut f = field();
    f.set_nav(nav_engine());
    f.set_regime(denied_regime());
    f.set_scalar("optical_fix", vec![7.0e6, 1.0e6, 2.0e6]);

    nav_stage().apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(
        f.scalar("nav_mode").unwrap()[0],
        1.0,
        "the through-plasma optical fix folds even when GNSS is denied"
    );
    assert!(
        f.scalar("optical_fix").is_none(),
        "the folded optical fix is consumed"
    );
}

#[test]
fn a_fix_is_consumed_and_not_refolded_when_the_publisher_goes_quiet() {
    let stage = nav_stage();
    let mut f = field();
    f.set_nav(nav_engine());

    // Step 1: a published fix is folded (aided) and consumed off the field.
    f.set_scalar("gnss_fix", vec![7.0e6, 1.0e6, 2.0e6]);
    stage.apply(&ctx(1), &mut f).expect("applies");
    assert_eq!(f.scalar("nav_mode").unwrap()[0], 1.0, "aided");
    assert!(f.scalar("gnss_fix").is_none(), "the fix is consumed");
    let v_folded = f.scalar("nav_position_variance").unwrap()[0];

    // Step 2: the publisher goes quiet. The old fix must not be re-folded as fresh — the
    // filter drops to dead reckoning and the position variance grows instead of collapsing.
    stage.apply(&ctx(2), &mut f).expect("applies");
    assert_eq!(f.scalar("nav_mode").unwrap()[0], 0.0, "dead reckoning");
    let v_quiet = f.scalar("nav_position_variance").unwrap()[0];
    assert!(
        v_quiet > v_folded,
        "a quiet publisher grows the variance: {v_quiet} vs {v_folded}"
    );
}

#[test]
fn with_imu_senses_the_specific_force_through_the_bias() {
    use deep_causality_cfd::ImuModel;

    // Twin engines, same true aero force. The biased IMU's dead-reckoned position diverges from
    // the unbiased twin: the real INS drift mechanism.
    let aero = [-30.0_f64, 0.0, 0.0];
    let imu = ImuModel::new([0.5, -0.3, 0.2], [0.0; 3], [1.0e-4; 17]);

    let mut clean = field();
    clean.set_nav(nav_engine());
    clean.set_aero_force(aero);
    nav_stage().apply(&ctx(1), &mut clean).expect("applies");

    let mut biased = field();
    biased.set_nav(nav_engine());
    biased.set_aero_force(aero);
    TrajectoryNav::new([1.0e-6; 17], 25.0, 2500.0)
        .with_imu(imu)
        .apply(&ctx(1), &mut biased)
        .expect("applies");

    let a = clean.scalar("nav_position").unwrap();
    let b = biased.scalar("nav_position").unwrap();
    assert!(
        (a[0] - b[0]).abs() > 1e-6 || (a[1] - b[1]).abs() > 1e-6,
        "the accelerometer bias drifts the dead-reckoned position"
    );
}

#[test]
fn nav_predict_failure_short_circuits_but_threads_the_engine_back() {
    // An unbound (hyperbolic) state cannot re-lift onto the KS manifold: predict fails.
    let filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]).unwrap();
    let unbound = ReentryNavEngine::new([7.0e6, 0.0, 0.0], [1.0e5, 0.0, 0.0], EARTH_GM, filter);

    let mut f = field();
    f.set_nav(unbound);
    let result = nav_stage().apply(&ctx(1), &mut f);
    assert!(result.is_err(), "an unbound predict short-circuits");
    assert!(
        f.nav().is_some(),
        "the engine threads back so a pause captures a whole state"
    );
}
