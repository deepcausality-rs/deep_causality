/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the energy-budget diagnostic (dec-ns-stability): the budget
//! identity (`projected = convective + viscous + body_force` for a
//! divergence-free state on a torus, where the projector is M-orthogonal
//! and drops out of the inner product), sign sanity of the viscous term,
//! and fused-vs-generic agreement. The instability *localization* run is
//! the `#[ignore]`d probe at the bottom (release, explicit invocation).

use deep_causality_physics::{DecNsRate, DecNsSolver, VelocityOneForm, dec_kinetic_energy};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposeOptions, LatticeComplex, Manifold,
};

const RE: f64 = 1600.0;

fn torus3(n: usize) -> Manifold<LatticeComplex<3, f64>, f64> {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(n);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn tg_vertex_tensor(m: &Manifold<LatticeComplex<3, f64>, f64>, n: usize) -> CausalTensor<f64> {
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = m.complex().num_cells(0);
    let mut vertex = vec![0.0; 3 * n0];
    for (vi, v) in m.complex().iter_cells(0).enumerate() {
        let p = v.position();
        let (x, y, z) = (k * p[0] as f64, k * p[1] as f64, k * p[2] as f64);
        vertex[3 * vi] = x.sin() * y.cos() * z.cos();
        vertex[3 * vi + 1] = -x.cos() * y.sin() * z.cos();
    }
    CausalTensor::new(vertex, vec![3 * n0]).unwrap()
}

fn nu_for(n: usize) -> f64 {
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    1.0 / (k * RE)
}

/// Budget identity on a divergence-free state: the projector is
/// M-orthogonal, so it drops out of `⟨u, ·⟩_M` and the projected power
/// equals the unprojected sum.
#[test]
fn budget_identity_on_divergence_free_state() {
    let n = 8usize;
    let m = torus3(n);
    let solver = DecNsSolver::new(&m, nu_for(n), 0.1, None).unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&m, n))
        .unwrap();
    let u = VelocityOneForm::new(state.as_one_form().clone(), &m).unwrap();

    let rate = DecNsRate::new(&m, nu_for(n), None).unwrap();
    let b = rate
        .energy_budget(&u, &HodgeDecomposeOptions::default())
        .unwrap();

    assert!(
        (b.projected() - b.unprojected_sum()).abs() < 1e-10,
        "projected {} vs unprojected sum {}",
        b.projected(),
        b.unprojected_sum()
    );
    assert!(b.viscous() < 0.0, "viscous power {} not dissipative", b.viscous());
    assert_eq!(b.body_force(), 0.0);
}

/// The two assemblies must report the same budget to rounding.
#[test]
fn budget_agrees_between_fused_and_generic() {
    let n = 8usize;
    let m = torus3(n);
    let solver = DecNsSolver::new(&m, nu_for(n), 0.1, None).unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&m, n))
        .unwrap();
    let u = VelocityOneForm::new(state.as_one_form().clone(), &m).unwrap();
    let opts = HodgeDecomposeOptions::default();

    let fused = DecNsRate::new(&m, nu_for(n), None)
        .unwrap()
        .energy_budget(&u, &opts)
        .unwrap();
    let generic = DecNsRate::new(&m, nu_for(n), None)
        .unwrap()
        .with_generic_assembly()
        .energy_budget(&u, &opts)
        .unwrap();

    for (a, b, name) in [
        (fused.convective(), generic.convective(), "convective"),
        (fused.viscous(), generic.viscous(), "viscous"),
        (fused.projected(), generic.projected(), "projected"),
    ] {
        assert!((a - b).abs() < 1e-11, "{name}: fused {a} vs generic {b}");
    }
}

/// LOCALIZATION PROBE (task 1.3): march the destabilizing 32³ Re-1600
/// TGV and log the per-term budget — the term whose cumulative
/// contribution turns positive and tracks the energy growth is the
/// defect. Run explicitly:
///
/// ```text
/// cargo test -p deep_causality_physics --release --test mod \
///     budget_localizes -- --ignored --nocapture
/// ```
#[test]
#[ignore]
fn budget_localizes_the_instability() {
    let n = 32usize;
    let dt = 0.2;
    let m = torus3(n);
    let solver = DecNsSolver::new(&m, nu_for(n), dt, None).unwrap();
    let rate = DecNsRate::new(&m, nu_for(n), None).unwrap();
    let opts = HodgeDecomposeOptions::default();
    let k = 2.0 * std::f64::consts::PI / (n as f64);

    let mut state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&m, n))
        .unwrap();
    let t_star_max = 13.0;
    let steps = (t_star_max / (dt * k)).ceil() as usize;
    println!("t*,E,conv_power,visc_power,projected_power");
    for step in 0..steps {
        if step % 10 == 0 {
            let u = VelocityOneForm::new(state.as_one_form().clone(), &m).unwrap();
            let b = rate.energy_budget(&u, &opts).unwrap();
            let e = dec_kinetic_energy(&m, state.as_one_form()).unwrap();
            println!(
                "{:.3},{:.8},{:+.3e},{:+.3e},{:+.3e}",
                step as f64 * dt * k,
                e,
                b.convective(),
                b.viscous(),
                b.projected()
            );
        }
        match solver.step(&state) {
            Ok(out) => state = out.into_state(),
            Err(e) => {
                println!("# ABORT at step {step}: {e}");
                break;
            }
        }
    }
}
