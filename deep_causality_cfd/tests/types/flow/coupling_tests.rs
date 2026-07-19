/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `.couple` multi-physics seam: static stage composition, error short-circuiting, and an
//! end-to-end `ν(T)` feedback through the march (coupled physics changes the flow dynamics).

use deep_causality_cfd::{
    AeroBlackoutStub, AeroForceCoupling, Ambient, CfdConfigBuilder, CoupledField, Coupling,
    IonizationStage, Mesh, Observe, PhysicsError, PhysicsStage, RecoveryTemperatureStage, Seed,
    StepContext, ThermalRelax, ViscosityArrhenius,
};
use deep_causality_physics::{PhysicsErrorEnum, SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

/// A tiny periodic manifold + zero fluid state to drive stage unit tests through a real
/// `StepContext` (the shipped stages read only `dt`/scalars, but the context is genuine).
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

#[test]
fn thermal_relax_then_arrhenius_drives_viscosity() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 1);

    let coupling = Coupling::between_steps()
        .then(ThermalRelax::new(0.5, 400.0))
        .then(ViscosityArrhenius::new(0.01, 300.0, 0.7))
        .build();

    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("temperature", vec![300.0_f64; 9]); // 9 cells on a 4×4 periodic lattice

    let nu_before = *field.ambient().nu();
    coupling.apply(&ctx, &mut field).expect("coupling applies");

    // Relaxation pulls T toward the 400 K wall, so the mean temperature rises above 300 K.
    let temp = field.scalar("temperature").expect("temperature field");
    assert!(
        temp[0] > 300.0,
        "T relaxes toward the hot wall: {}",
        temp[0]
    );
    // ν(T) = 0.01·exp(0.7·(300/T − 1)) < ν_ref once T > 300.
    let nu_after = *field.ambient().nu();
    assert!(
        nu_after < nu_before,
        "rising temperature lowers ν via Arrhenius: {nu_before} -> {nu_after}"
    );
}

#[test]
fn step_context_exposes_manifold_velocity_and_step() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.05, 7);
    assert_eq!(ctx.step(), 7);
    assert_eq!(ctx.dt(), 0.05);
    let _ = ctx.manifold();
    let _ = ctx.velocity();
    // The zero state samples to zero velocity at any point.
    let v = ctx.sample_velocity(&[1.0, 1.0]).expect("velocity sample");
    assert!(v.iter().all(|c| c.abs() < 1e-12));
}

#[test]
fn dec_context_exposes_manifold_and_velocity_some() {
    // The DEC-backed context returns `Some(..)` from both the manifold and velocity accessors.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 2);
    assert!(ctx.manifold().is_some(), "DEC context carries a manifold");
    assert!(ctx.velocity().is_some(), "DEC context carries a velocity");
    // And it can sample the (zero) velocity field.
    let v = ctx.sample_velocity(&[1.0, 1.0]).expect("DEC sample");
    assert!(v.iter().all(|c| c.abs() < 1e-12));
}

#[test]
fn qtt_context_has_no_manifold_or_velocity_and_cannot_sample() {
    // The QTT-backed context carries neither manifold nor velocity, and `sample_velocity` errors.
    let ctx = StepContext::<2, f64>::qtt(0.1, 3);
    assert_eq!(ctx.dt(), 0.1);
    assert_eq!(ctx.step(), 3);
    assert!(ctx.manifold().is_none(), "QTT context has no manifold");
    assert!(ctx.velocity().is_none(), "QTT context has no velocity");
    let err = ctx
        .sample_velocity(&[0.0, 0.0])
        .expect_err("no manifold to sample on a QTT context");
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));
}

#[test]
fn coupled_field_scalar_set_replace_and_access() {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("temperature", vec![1.0, 2.0]);
    // Setting the same name again replaces the field in place (the `find` hit branch).
    field.set_scalar("temperature", vec![3.0, 4.0, 5.0]);
    assert_eq!(field.scalar("temperature"), Some(&[3.0, 4.0, 5.0][..]));
    assert!(field.scalar("missing").is_none());

    // Mutable access, present and absent.
    field.scalar_mut("temperature").expect("present")[0] = 9.0;
    assert_eq!(field.scalar("temperature").expect("present")[0], 9.0);
    assert!(field.scalar_mut("missing").is_none());
}

#[test]
fn coupled_field_take_scalar_consumes_the_field() {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("gnss_fix", vec![1.0, 2.0, 3.0]);
    field.set_scalar("temperature", vec![300.0]);

    // Taking removes the field and hands its data back; other fields are untouched.
    assert_eq!(field.take_scalar("gnss_fix"), Some(vec![1.0, 2.0, 3.0]));
    assert!(field.scalar("gnss_fix").is_none(), "consumed, not latched");
    assert_eq!(field.scalar("temperature"), Some(&[300.0][..]));

    // Taking again (or taking a name that never existed) yields nothing.
    assert!(field.take_scalar("gnss_fix").is_none());
    assert!(field.take_scalar("missing").is_none());
}

#[test]
fn coupling_is_itself_a_physics_stage() {
    // Passing the `Coupling` (not its `.build()` tuple) as a stage delegates through its wrapper.
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 1);
    let coupling = Coupling::between_steps().then(MarkRan);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    PhysicsStage::apply(&coupling, &ctx, &mut field).expect("Coupling delegates to its stages");
    assert_eq!(*field.ambient().freestream(), 1.0);
}

#[test]
fn thermal_relax_without_field_is_a_noop() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 1);
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    // No "temperature" field present → the relaxation skips the absent field.
    PhysicsStage::apply(&ThermalRelax::new(0.5, 400.0), &ctx, &mut field).expect("noop");
    assert!(field.scalar("temperature").is_none());
}

#[test]
fn arrhenius_edge_cases_leave_viscosity_unchanged() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 1);
    let stage = ViscosityArrhenius::new(0.01, 300.0, 0.7);

    // No temperature field → no-op.
    let mut field = CoupledField::new(Ambient::new(0.05_f64, 0.0, None));
    PhysicsStage::apply(&stage, &ctx, &mut field).expect("no field");
    assert_eq!(*field.ambient().nu(), 0.05);

    // Empty temperature field → no-op.
    field.set_scalar("temperature", Vec::<f64>::new());
    PhysicsStage::apply(&stage, &ctx, &mut field).expect("empty field");
    assert_eq!(*field.ambient().nu(), 0.05);

    // Non-positive reference temperature → no-op.
    field.set_scalar("temperature", vec![300.0, 320.0]);
    let stage_bad_tref = ViscosityArrhenius::new(0.01, 0.0, 0.7);
    PhysicsStage::apply(&stage_bad_tref, &ctx, &mut field).expect("t_ref <= 0");
    assert_eq!(*field.ambient().nu(), 0.05);
}

#[test]
fn arrhenius_nonpositive_mean_temperature_errors() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 1);
    let stage = ViscosityArrhenius::new(0.01, 300.0, 0.7);
    let mut field = CoupledField::new(Ambient::new(0.05_f64, 0.0, None));
    field.set_scalar("temperature", vec![-10.0, -20.0]);
    let err = PhysicsStage::apply(&stage, &ctx, &mut field)
        .expect_err("a non-positive mean temperature is rejected");
    assert!(matches!(
        err.0,
        PhysicsErrorEnum::PhysicalInvariantBroken(_)
    ));
}

#[test]
fn identity_coupling_is_a_no_op() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 1);
    let mut field = CoupledField::new(Ambient::new(0.02, 0.0, None));
    // The empty coupling (`Coupling::between_steps().build()` is `()`) is the identity stage.
    PhysicsStage::apply(&Coupling::between_steps().build(), &ctx, &mut field)
        .expect("identity applies");
    assert_eq!(
        *field.ambient().nu(),
        0.02,
        "the empty coupling changes nothing"
    );
}

/// A stage that always errors short-circuits the rest of the chain.
struct AlwaysErr;
impl<const D: usize, R: deep_causality_cfd::CfdScalar> PhysicsStage<D, R> for AlwaysErr {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        _field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        Err(PhysicsError::PhysicalInvariantBroken("boom".into()))
    }
}

/// A stage that records it ran (by bumping the ambient freestream), to detect short-circuiting.
struct MarkRan;
impl<const D: usize, R: deep_causality_cfd::CfdScalar> PhysicsStage<D, R> for MarkRan {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        field.ambient_mut().set_freestream(R::one());
        Ok(())
    }
}

#[test]
fn a_stage_error_short_circuits_the_coupling() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 1);

    let coupling = Coupling::between_steps()
        .then(AlwaysErr)
        .then(MarkRan)
        .build();
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));

    let result = coupling.apply(&ctx, &mut field);
    assert!(result.is_err(), "the failing stage propagates its error");
    assert_eq!(
        *field.ambient().freestream(),
        0.0,
        "the stage after the failure never ran (short-circuit)"
    );
}

#[test]
#[cfg_attr(miri, ignore)]
fn coupled_viscosity_feedback_changes_the_flow_dynamics() {
    // A 3D Taylor–Green vortex decay marched twice: once single-physics, once with a hot-wall
    // thermal relaxation feeding ν(T) back through the ambient. The coupled run dissipates
    // differently, proving the between-step coupling drives the fluid dynamics over time.
    let n = 8usize;
    let nu0 = 0.02_f64;

    let baseline = super::run_march(
        CfdConfigBuilder::march::<3, f64>("tgv-baseline")
            .mesh(Mesh::periodic_cube(n))
            .solver(
                CfdConfigBuilder::dec_ns()
                    .viscosity(nu0)
                    .time_step(0.02)
                    .build()
                    .unwrap(),
            )
            .seed(Seed::TaylorGreenVortex)
            .march_for(10)
            .observe(Observe::default().kinetic_energy())
            .build()
            .expect("config build"),
    )
    .expect("baseline runs");

    let coupled = super::run_march(
        CfdConfigBuilder::march::<3, f64>("tgv-coupled")
            .mesh(Mesh::periodic_cube(n))
            .solver(
                CfdConfigBuilder::dec_ns()
                    .viscosity(nu0)
                    .time_step(0.02)
                    .build()
                    .unwrap(),
            )
            .seed(Seed::TaylorGreenVortex)
            // Heat the bulk toward a hot wall and raise ν strongly with temperature.
            .couple(
                Coupling::between_steps()
                    .then(ThermalRelax::new(2.0, 800.0))
                    .then(ViscosityArrhenius::new(nu0, 300.0, -1.5))
                    .build(),
            )
            .coupled_scalar("temperature", 300.0)
            .march_for(10)
            .observe(Observe::default().kinetic_energy())
            .build()
            .expect("config build"),
    )
    .expect("coupled runs");

    let e_base = baseline.series("kinetic_energy").unwrap();
    let e_coup = coupled.series("kinetic_energy").unwrap();
    assert_eq!(e_base[0], e_coup[0], "identical seed energy");
    // The coupling raised ν above ν0 as the bulk heated, so the coupled run dissipates more.
    assert!(
        *e_coup.last().unwrap() < *e_base.last().unwrap(),
        "ν(T) feedback dissipates more energy: coupled {} vs baseline {}",
        e_coup.last().unwrap(),
        e_base.last().unwrap()
    );
}

#[test]
fn coupled_field_nav_channels_default_none_and_roundtrip() {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    // The two navigation channels start empty (existing couplings are unaffected).
    assert_eq!(field.aero_force(), None);
    assert_eq!(field.control_action(), None);

    field.set_aero_force([-2.0, 0.5, -0.25]);
    field.set_control_action(0.13);
    assert_eq!(field.aero_force(), Some([-2.0, 0.5, -0.25]));
    assert_eq!(field.control_action(), Some(0.13));
}

#[test]
fn aero_blackout_stub_publishes_force_and_windowed_ne() {
    // The stub satisfies the ④ contract on a QTT-backed context (no manifold needed).
    let stub = AeroBlackoutStub::new(3.0_f64, 1.0e17, 1.0e20, 2, 5);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));

    // Outside the window (step 1): mock drag published; n_e at ambient.
    let ctx_out = StepContext::<2, f64>::qtt(0.1, 1);
    stub.apply(&ctx_out, &mut field).expect("stub applies");
    assert_eq!(field.aero_force(), Some([-3.0, 0.0, 0.0]));
    assert_eq!(field.scalar("n_e"), Some([1.0e17].as_slice()));

    // Inside the window (step 3 ∈ [2,5)): n_e rises to the blackout level.
    let ctx_in = StepContext::<2, f64>::qtt(0.1, 3);
    stub.apply(&ctx_in, &mut field).expect("stub applies");
    assert_eq!(field.scalar("n_e"), Some([1.0e20].as_slice()));

    // Past the window (step 5): back to ambient.
    let ctx_past = StepContext::<2, f64>::qtt(0.1, 5);
    stub.apply(&ctx_past, &mut field).expect("stub applies");
    assert_eq!(field.scalar("n_e"), Some([1.0e17].as_slice()));
}

#[test]
fn aero_force_coupling_scales_with_dynamic_pressure() {
    // a = −(C_d·A/m)·½·ρ·U_max²; the force scales quadratically with the flow speed.
    let (rho, cda) = (0.01_f64, 2.0_f64);
    let stage = AeroForceCoupling::new(rho, cda);
    let ctx = StepContext::<2, f64>::qtt(0.1, 1);

    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("speed", vec![1000.0, 500.0, 800.0]); // U_max = 1000
    stage.apply(&ctx, &mut field).expect("aero applies");
    let expected = -(cda * 0.5 * rho * 1000.0 * 1000.0);
    let f = field.aero_force().expect("aero force set");
    assert!(
        (f[0] - expected).abs() < 1e-9,
        "aero accel {f:?} vs {expected}"
    );
    assert_eq!((f[1], f[2]), (0.0, 0.0));

    // Doubling the peak speed quadruples the drag.
    let mut field2 = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field2.set_scalar("speed", vec![2000.0]);
    stage.apply(&ctx, &mut field2).expect("aero applies");
    let f2 = field2.aero_force().expect("aero force set");
    assert!(
        (f2[0] / f[0] - 4.0).abs() < 1e-9,
        "quadratic in speed: {}",
        f2[0] / f[0]
    );
}

#[test]
fn aero_force_coupling_is_a_noop_without_speed() {
    let stage = AeroForceCoupling::new(0.01_f64, 2.0);
    let ctx = StepContext::<2, f64>::qtt(0.1, 1);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    stage.apply(&ctx, &mut field).expect("noop");
    assert_eq!(field.aero_force(), None, "no speed ⇒ no aero force");
}

#[test]
fn real_reacting_plus_aero_stack_fills_the_coupling_contract() {
    // The real ④ producer stack — RecoveryTemperature → Ionization → AeroForceCoupling — populates the
    // same channels the AeroBlackoutStub faked: a real transported n_e (from the reacting physics) and a
    // real flow-derived aero force. Swapping the stub for this stack changes no consumer.
    let stack = Coupling::between_steps()
        .then(RecoveryTemperatureStage::new(20.0_f64, 1.4, 250.0, 1004.5))
        .then(IonizationStage::new(1.0e22))
        .then(AeroForceCoupling::new(0.01, 2.0))
        .build();

    let ctx = StepContext::<2, f64>::qtt(1.0e-6, 1);
    let mut field = CoupledField::new(Ambient::new(0.01, 0.0, None));
    field.set_scalar("speed", vec![1000.0_f64, 1200.0, 900.0, 1100.0]); // the marcher's per-cell |u|

    PhysicsStage::apply(&stack, &ctx, &mut field).expect("real stack applies");

    // ④ aero-force channel populated (flow-derived, non-zero).
    let f = field
        .aero_force()
        .expect("aero force produced by the real stack");
    assert!(f[0] < 0.0, "drag opposes motion: {f:?}");
    // ④ electron density produced by the real reacting physics (not a scheduled constant).
    let ne = field
        .scalar("n_e")
        .expect("n_e produced by the reacting stage");
    assert!(
        ne.iter().copied().fold(0.0, f64::max) > 0.0,
        "reacting flow ionizes"
    );
}

// ── The powered-descent command/force channels (blackout-coupling-interface) ──

#[test]
fn throttle_channel_round_trips_independent_of_the_bank_channel() {
    let mut field = CoupledField::<f64>::new(Ambient::new(0.01, 0.0, None));
    // Default is absent, so every existing coupling is unaffected.
    assert_eq!(field.throttle_action(), None);
    // The two command axes are independent: a bank write does not disturb the throttle read.
    field.set_control_action(0.3);
    field.set_throttle_action(0.7);
    assert_eq!(field.throttle_action(), Some(0.7));
    assert_eq!(field.control_action(), Some(0.3));
}

#[test]
fn a_field_without_a_throttle_write_reports_none() {
    let mut field = CoupledField::<f64>::new(Ambient::new(0.01, 0.0, None));
    field.set_control_action(0.2);
    assert_eq!(field.throttle_action(), None);
}

#[test]
fn add_aero_force_sums_onto_the_channel_and_treats_none_as_zero() {
    let mut field = CoupledField::<f64>::new(Ambient::new(0.01, 0.0, None));
    // Over an unset channel the result is exactly the delta.
    field.add_aero_force([1.0, 2.0, 3.0]);
    assert_eq!(field.aero_force(), Some([1.0, 2.0, 3.0]));
    // Over a set force it composes component-wise — thrust adds to lift instead of clobbering it.
    field.set_aero_force([0.5, -0.5, 0.0]);
    field.add_aero_force([0.25, 0.25, -1.0]);
    assert_eq!(field.aero_force(), Some([0.75, -0.25, -1.0]));
}
