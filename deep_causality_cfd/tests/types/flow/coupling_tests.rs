/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `.couple` multi-physics seam: static stage composition, error short-circuiting, and an
//! end-to-end `ν(T)` feedback through the march (coupled physics changes the flow dynamics).

use deep_causality_cfd::{
    Ambient, CoupledField, Coupling, DecNs, Flow, Mesh, Observe, PhysicsError, PhysicsStage, Seed,
    StepContext, ThermalRelax, ViscosityArrhenius,
};
use deep_causality_physics::{SolenoidalField, VelocityOneForm};
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
    assert!(temp[0] > 300.0, "T relaxes toward the hot wall: {}", temp[0]);
    // ν(T) = 0.01·exp(0.7·(300/T − 1)) < ν_ref once T > 300.
    let nu_after = *field.ambient().nu();
    assert!(
        nu_after < nu_before,
        "rising temperature lowers ν via Arrhenius: {nu_before} -> {nu_after}"
    );
}

#[test]
fn identity_coupling_is_a_no_op() {
    let (manifold, state) = empty_context();
    let ctx = StepContext::new(&manifold, &state, 0.1, 1);
    let mut field = CoupledField::new(Ambient::new(0.02, 0.0, None));
    // The empty coupling (`Coupling::between_steps().build()` is `()`) is the identity stage.
    PhysicsStage::apply(&Coupling::between_steps().build(), &ctx, &mut field)
        .expect("identity applies");
    assert_eq!(*field.ambient().nu(), 0.02, "the empty coupling changes nothing");
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
fn coupled_viscosity_feedback_changes_the_flow_dynamics() {
    // A 3D Taylor–Green vortex decay marched twice: once single-physics, once with a hot-wall
    // thermal relaxation feeding ν(T) back through the ambient. The coupled run dissipates
    // differently, proving the between-step coupling drives the fluid dynamics over time.
    let n = 8usize;
    let nu0 = 0.02_f64;

    let baseline = Flow::march::<3, f64>("tgv-baseline")
        .mesh(Mesh::periodic_cube(n))
        .solver(DecNs::config().viscosity(nu0).time_step(0.02).build().unwrap())
        .seed(Seed::TaylorGreenVortex)
        .march_for(10)
        .observe(Observe::default().kinetic_energy())
        .run()
        .expect("baseline runs");

    let coupled = Flow::march::<3, f64>("tgv-coupled")
        .mesh(Mesh::periodic_cube(n))
        .solver(DecNs::config().viscosity(nu0).time_step(0.02).build().unwrap())
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
        .run()
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
