/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-A Park-2T plasma-blackout coupling: the LER closed-form mechanism, the
//! recovery-temperature reconstruction, ionization lag, and static stage composition.

use deep_causality_cfd::{
    Ambient, BlackoutTrigger, CoupledField, Coupling, EosStage, IonizationStage, PhysicsStage,
    RecoveryTemperatureStage, StepContext, ler_relax_scalar, ler_step,
};
use deep_causality_physics::ElectronDensity;
use deep_causality_physics::{
    SolenoidalField, Temperature, VelocityOneForm, park2t_ionization_surrogate_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

fn empty_context() -> (Manifold<LatticeComplex<2, f64>, f64>, SolenoidalField<f64>) {
    let n = 4;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [true, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(1.0);
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);
    let n1 = manifold.complex().num_cells(1);
    let zero = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let velocity = VelocityOneForm::from_raw(zero);
    let (state, _) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();
    (manifold, state)
}

// ── LER closed-form helper ───────────────────────────────────────────────

#[test]
fn test_ler_step_exact_on_linear_relaxation() {
    let (x, x_eq, tau, dt) = (300.0_f64, 7000.0_f64, 0.01_f64, 0.003_f64);
    let expected = x_eq - (x_eq - x) * (-(dt / tau)).exp();
    assert_eq!(ler_step(x, x_eq, tau, dt), expected);
}

#[test]
fn test_ler_step_equilibrium_limit() {
    // τ → 0 (and the degenerate τ ≤ 0) jumps exactly to the target.
    assert_eq!(ler_step(300.0, 7000.0, 0.0, 0.1), 7000.0);
    assert_eq!(ler_step(300.0, 7000.0, -1.0, 0.1), 7000.0);
}

#[test]
fn test_ler_step_zero_dt_is_identity() {
    assert_eq!(ler_step(300.0, 7000.0, 0.01, 0.0), 300.0);
}

#[test]
fn test_ler_step_stable_at_stiffness_where_euler_diverges() {
    // τ = dt/1000: the LER update stays bounded in [x0, x_eq] and monotone,
    // where an explicit-Euler rate step massively overshoots.
    let (dt, x_eq) = (1.0_f64, 7000.0);
    let tau = dt / 1000.0;
    let mut x = 300.0_f64;
    for _ in 0..50 {
        let nx = ler_step(x, x_eq, tau, dt);
        assert!(nx >= x - 1e-6 && nx <= x_eq + 1e-6, "overshoot: {nx}");
        x = nx;
    }
    assert!((x - x_eq).abs() < 1.0);

    // Explicit Euler with the same stiffness ratio dt/τ = 1000 diverges.
    let euler = 300.0 + (dt / tau) * (x_eq - 300.0);
    assert!(euler > x_eq * 100.0);
}

#[test]
fn test_ler_relax_scalar_relaxes_named_field() {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("alpha", vec![0.0, 0.0]);
    ler_relax_scalar(&mut field, "alpha", 1.0, &[1.0, 0.5], &[0.0, 0.0]).unwrap();
    // τ = 0 ⇒ each cell jumps to its target.
    assert_eq!(field.scalar("alpha"), Some(&[1.0, 0.5][..]));
}

#[test]
fn test_ler_relax_scalar_absent_is_noop_and_mismatch_errors() {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    // Absent field is a no-op.
    ler_relax_scalar(&mut field, "alpha", 1.0, &[], &[]).unwrap();
    // Length mismatch is an error.
    field.set_scalar("alpha", vec![0.0, 0.0]);
    assert!(ler_relax_scalar(&mut field, "alpha", 1.0, &[1.0], &[0.0]).is_err());
}

// ── Recovery-temperature reconstruction ──────────────────────────────────

#[test]
fn test_recovery_temperature_reaches_ionization_band() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-6, 1);
    let stage = RecoveryTemperatureStage::new(25.0, 1.4, 200.0, 1004.0);

    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("speed", vec![0.0, 1000.0, 2000.0]);
    stage.apply(&ctx, &mut field).unwrap();

    let t_tr = field.scalar("T_tr").expect("T_tr built");
    // The stagnation cell (u = 0) carries the full post-shock jump, in the ~10⁴ K band.
    assert!(t_tr[0] > 1.0e4, "T_post = {}", t_tr[0]);
    // Higher speed ⇒ more kinetic enthalpy removed ⇒ cooler.
    assert!(t_tr[1] > t_tr[2]);
}

// ── Ionization lag + electron production ─────────────────────────────────

#[test]
fn test_ionization_lags_then_catches_up() {
    let (manifold, state) = empty_context();
    let n_tot = 1.0e22_f64;
    let t_hot = 8000.0_f64;
    let alpha_eq = park2t_ionization_surrogate_kernel(Temperature::new(t_hot).unwrap(), n_tot)
        .unwrap()
        .value();
    let stage = IonizationStage::new(n_tot);

    // Small dt ≪ τ_ion: α lags well below equilibrium.
    let ctx_fast = StepContext::new(&manifold, &state, 1.0e-5, 1);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("T_tr", vec![t_hot]);
    stage.apply(&ctx_fast, &mut field).unwrap();
    let alpha_lagged = field.scalar("alpha").unwrap()[0];
    assert!(alpha_lagged < alpha_eq, "{alpha_lagged} vs {alpha_eq}");
    assert!(field.scalar("n_e").unwrap()[0] > 0.0);

    // A long dwell drives α toward equilibrium (gap closes).
    let ctx_slow = StepContext::new(&manifold, &state, 10.0, 2);
    stage.apply(&ctx_slow, &mut field).unwrap();
    let alpha_settled = field.scalar("alpha").unwrap()[0];
    assert!(alpha_settled > alpha_lagged);
    assert!((alpha_settled - alpha_eq).abs() < alpha_eq * 0.05);
}

#[test]
fn test_eos_stage_writes_pressure() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-6, 1);
    let stage = EosStage::new(1.0e22_f64);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("T_tr", vec![7000.0]);
    stage.apply(&ctx, &mut field).unwrap();
    let p = field.scalar("pressure").expect("pressure written");
    assert!(p[0] > 0.0);
}

// ── Static composition (no dyn) ──────────────────────────────────────────

#[test]
fn test_static_composition_of_tier_a_stages() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 1.0e-3, 1);

    // Recovery → ionization → EOS composes statically by cons-tuple.
    let coupling = Coupling::between_steps()
        .then(RecoveryTemperatureStage::new(25.0, 1.4, 200.0, 1004.0))
        .then(IonizationStage::new(1.0e22))
        .then(EosStage::new(1.0e22))
        .build();

    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("speed", vec![0.0, 500.0]);
    coupling.apply(&ctx, &mut field).expect("pipeline applies");

    assert!(field.scalar("T_tr").is_some());
    assert!(field.scalar("n_e").is_some());
    assert!(field.scalar("pressure").is_some());
}

// ── Blackout trigger ─────────────────────────────────────────────────────

#[test]
fn test_blackout_trigger_no_plasma_keeps_link() {
    // GPS L-band ≈ 1.5 GHz → ω ≈ 9.4e9 rad/s.
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    let state = trigger
        .evaluate(ElectronDensity::<f64>::new(0.0).unwrap())
        .unwrap();
    assert_eq!(state.plasma_frequency, 0.0);
    assert!(!state.denied);
}

#[test]
fn test_blackout_trigger_above_band_denies() {
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    // n_e = 1e18 ⇒ ω_p ≈ 5.6e10 rad/s > band ⇒ denied.
    let state = trigger
        .evaluate(ElectronDensity::<f64>::new(1.0e18).unwrap())
        .unwrap();
    assert!(state.plasma_frequency > 9.4e9);
    assert!(state.denied);
}

#[test]
fn test_blackout_trigger_below_band_keeps_link() {
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    // n_e = 1e15 ⇒ ω_p ≈ 1.8e9 rad/s < band ⇒ link available.
    let state = trigger
        .evaluate(ElectronDensity::<f64>::new(1.0e15).unwrap())
        .unwrap();
    assert!(state.plasma_frequency < 9.4e9);
    assert!(!state.denied);
}

#[test]
fn test_blackout_trigger_classify_is_pure_effect() {
    use deep_causality_core::EffectValue;
    let trigger = BlackoutTrigger::new(9.4e9_f64);
    let effect = trigger.classify(ElectronDensity::<f64>::new(1.0e18).unwrap());
    assert!(effect.is_ok());
    if let EffectValue::Value(state) = effect.value() {
        assert!(state.denied);
    } else {
        panic!("expected Value");
    }
}
